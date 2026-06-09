use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::helpers::sanitize_id;
use super::types::*;

pub fn analyze_impact<I, S>(repo_graph: &RepoInspection, changed_files: I) -> ImpactReport
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let changed_files = changed_files
        .into_iter()
        .map(|file| normalize_changed_file(file.as_ref()))
        .filter(|file| !file.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut impacted_components = BTreeMap::<String, ImpactedComponent>::new();
    let mut impacted_workspaces = BTreeMap::<String, ImpactedWorkspace>::new();
    let mut warnings = repo_graph.warnings.clone();
    push_impact_warning(
        &mut warnings,
        DetectionIssue {
            id: "impact-warning-repo-graph-only".to_string(),
            severity: DetectionSeverity::Info,
            category: DetectionCategory::RepoGraphOnly,
            message: "Symbol-level impact is not implemented; impact is based on repository structure and paths only.".to_string(),
            path: None,
            evidence_id: None,
        },
    );

    let mut broad_change_detected = false;

    for changed_file in &changed_files {
        if is_broad_repo_change(repo_graph, changed_file) {
            broad_change_detected = true;
            for component in &repo_graph.components {
                insert_impacted_component(
                    &mut impacted_components,
                    component,
                    ImpactKind::Broad,
                    None,
                    "manifest_or_build_file_changed",
                    vec![component.evidence_id.clone()],
                );
            }
            for workspace in &repo_graph.workspaces {
                insert_impacted_workspace(
                    &mut impacted_workspaces,
                    workspace,
                    ImpactKind::Broad,
                    None,
                    "manifest_or_build_file_changed",
                    vec![workspace.evidence_id.clone()],
                );
            }
            continue;
        }

        let mut matched_any = false;
        for component in repo_graph
            .components
            .iter()
            .filter(|component| component_matches_changed_file(component, changed_file))
        {
            matched_any = true;
            insert_impacted_component(
                &mut impacted_components,
                component,
                ImpactKind::Direct,
                Some(0),
                if is_test_path(changed_file) {
                    "test_path_matched_component_scope"
                } else {
                    "path_matched_component_scope"
                },
                vec![component.evidence_id.clone()],
            );
        }

        if !matched_any {
            push_impact_warning(
                &mut warnings,
                DetectionIssue {
                    id: format!("impact-warning-unmapped-{}", sanitize_id(changed_file)),
                    severity: DetectionSeverity::Warning,
                    category: DetectionCategory::UnmappedChange,
                    message: "Changed file could not be mapped to a RepoGraph component."
                        .to_string(),
                    path: Some(changed_file.clone()),
                    evidence_id: None,
                },
            );
        }
    }

    let dependency_edges = repo_graph
        .relationships
        .iter()
        .filter(|relationship| relationship.kind == RelationshipKind::DependsOn)
        .collect::<Vec<_>>();

    if dependency_edges.is_empty() {
        push_impact_warning(
            &mut warnings,
            DetectionIssue {
                id: "impact-warning-no-dependency-edges".to_string(),
                severity: DetectionSeverity::Info,
                category: DetectionCategory::PartialSupport,
                message: "No depends_on relationships were available; transitive dependency impact was not computed.".to_string(),
                path: None,
                evidence_id: None,
            },
        );
    } else {
        add_reverse_dependency_impacts(repo_graph, &dependency_edges, &mut impacted_components);
    }

    for workspace in workspaces_for_components(repo_graph, impacted_components.keys()) {
        insert_impacted_workspace(
            &mut impacted_workspaces,
            workspace,
            if broad_change_detected {
                ImpactKind::Broad
            } else {
                ImpactKind::Direct
            },
            if broad_change_detected { None } else { Some(0) },
            if broad_change_detected {
                "manifest_or_build_file_changed"
            } else {
                "component_belongs_to_workspace"
            },
            vec![workspace.evidence_id.clone()],
        );
    }

    let impacted_components = impacted_components.into_values().collect::<Vec<_>>();
    let impacted_workspaces = impacted_workspaces.into_values().collect::<Vec<_>>();
    let recommended_tests = recommend_tests(
        repo_graph,
        &changed_files,
        &impacted_components,
        broad_change_detected,
    );
    let recommended_commands = recommend_commands(
        repo_graph,
        &impacted_components,
        !recommended_tests.is_empty(),
        broad_change_detected,
    );

    if !impacted_components.is_empty() && recommended_tests.is_empty() {
        push_impact_warning(
            &mut warnings,
            DetectionIssue {
                id: "impact-warning-no-test-command".to_string(),
                severity: DetectionSeverity::Warning,
                category: DetectionCategory::MissingCommand,
                message: "Impacted components were found, but no test target was available."
                    .to_string(),
                path: None,
                evidence_id: None,
            },
        );
    }

    warnings.sort_by(|a, b| a.id.cmp(&b.id));

    let status = if impacted_components.is_empty()
        && impacted_workspaces.is_empty()
        && recommended_commands.is_empty()
        && recommended_tests.is_empty()
    {
        ImpactStatus::InsufficientEvidence
    } else {
        ImpactStatus::Partial
    };
    let impact_scope = impact_scope(&impacted_components, broad_change_detected);
    let confidence = impact_confidence(
        status.clone(),
        &impacted_components,
        &recommended_commands,
        &recommended_tests,
    );

    let mut limitations = vec![
        "RepoGraph-only impact analysis; no symbols, imports, definitions, references, or call graph are used.".to_string(),
        "Recommendations are conservative and based on path containment, manifests, command scopes, test scopes, and explicit RepoGraph dependency edges.".to_string(),
    ];

    if dependency_edges.is_empty() {
        limitations.push(
            "No depends_on relationships were available; no transitive dependency closure was computed."
                .to_string(),
        );
    }

    ImpactReport {
        contract_version: IMPACT_CONTRACT_VERSION.to_string(),
        status,
        impact_scope,
        confidence,
        changed_files,
        impacted_components,
        impacted_workspaces,
        recommended_commands,
        recommended_tests,
        evidence: repo_graph.evidence.clone(),
        warnings,
        limitations,
    }
}

fn push_impact_warning(warnings: &mut Vec<DetectionIssue>, warning: DetectionIssue) {
    if !warnings.iter().any(|existing| existing.id == warning.id) {
        warnings.push(warning);
    }
}

fn insert_impacted_component(
    target: &mut BTreeMap<String, ImpactedComponent>,
    component: &Component,
    impact_kind: ImpactKind,
    distance: Option<u32>,
    reason: &str,
    evidence_ids: Vec<String>,
) {
    let candidate = ImpactedComponent {
        component_id: component.id.clone(),
        name: component.name.clone(),
        kind: component.kind.clone(),
        path: component.path.clone(),
        impact_kind,
        distance,
        reason: reason.to_string(),
        evidence_ids: stable_evidence_ids(evidence_ids),
    };

    match target.get(&candidate.component_id) {
        Some(existing) if impact_priority(existing) <= impact_priority(&candidate) => {}
        _ => {
            target.insert(candidate.component_id.clone(), candidate);
        }
    }
}

fn insert_impacted_workspace(
    target: &mut BTreeMap<String, ImpactedWorkspace>,
    workspace: &Workspace,
    impact_kind: ImpactKind,
    distance: Option<u32>,
    reason: &str,
    evidence_ids: Vec<String>,
) {
    let candidate = ImpactedWorkspace {
        workspace_id: workspace.id.clone(),
        name: workspace.name.clone(),
        impact_kind,
        distance,
        reason: reason.to_string(),
        evidence_ids: stable_evidence_ids(evidence_ids),
    };

    match target.get(&candidate.workspace_id) {
        Some(existing)
            if workspace_impact_priority(existing) <= workspace_impact_priority(&candidate) => {}
        _ => {
            target.insert(candidate.workspace_id.clone(), candidate);
        }
    }
}

fn stable_evidence_ids(evidence_ids: Vec<String>) -> Vec<String> {
    evidence_ids
        .into_iter()
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn impact_priority(component: &ImpactedComponent) -> (u8, u32, &str) {
    (
        impact_kind_rank(&component.impact_kind),
        component.distance.unwrap_or(u32::MAX),
        component.component_id.as_str(),
    )
}

fn workspace_impact_priority(workspace: &ImpactedWorkspace) -> (u8, u32, &str) {
    (
        impact_kind_rank(&workspace.impact_kind),
        workspace.distance.unwrap_or(u32::MAX),
        workspace.workspace_id.as_str(),
    )
}

fn impact_kind_rank(kind: &ImpactKind) -> u8 {
    match kind {
        ImpactKind::Direct => 0,
        ImpactKind::Broad => 1,
        ImpactKind::Transitive => 2,
        ImpactKind::Uncertain => 3,
    }
}

fn add_reverse_dependency_impacts(
    repo_graph: &RepoInspection,
    dependency_edges: &[&Relationship],
    impacted_components: &mut BTreeMap<String, ImpactedComponent>,
) {
    let components_by_id = repo_graph
        .components
        .iter()
        .map(|component| (component.id.as_str(), component))
        .collect::<BTreeMap<_, _>>();
    let mut reverse_edges = BTreeMap::<&str, Vec<&Relationship>>::new();

    for edge in dependency_edges {
        reverse_edges
            .entry(edge.dst_id.as_str())
            .or_default()
            .push(*edge);
    }

    for edges in reverse_edges.values_mut() {
        edges.sort_by(|a, b| a.src_id.cmp(&b.src_id));
    }

    let roots = impacted_components
        .values()
        .filter(|component| {
            matches!(
                component.impact_kind,
                ImpactKind::Direct | ImpactKind::Broad
            )
        })
        .map(|component| {
            (
                component.component_id.clone(),
                component.distance.unwrap_or(0),
            )
        })
        .collect::<Vec<_>>();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();
    let mut seen = impacted_components
        .keys()
        .cloned()
        .collect::<BTreeSet<String>>();

    while let Some((component_id, distance)) = queue.pop_front() {
        let Some(edges) = reverse_edges.get(component_id.as_str()) else {
            continue;
        };

        for edge in edges {
            if !seen.insert(edge.src_id.clone()) {
                continue;
            }
            let Some(component) = components_by_id.get(edge.src_id.as_str()) else {
                continue;
            };
            let next_distance = distance.saturating_add(1);
            insert_impacted_component(
                impacted_components,
                component,
                ImpactKind::Transitive,
                Some(next_distance),
                "reverse_dependency",
                vec![component.evidence_id.clone(), edge.evidence_id.clone()],
            );
            queue.push_back((edge.src_id.clone(), next_distance));
        }
    }
}

fn workspaces_for_components<'a>(
    repo_graph: &'a RepoInspection,
    component_ids: impl Iterator<Item = &'a String>,
) -> Vec<&'a Workspace> {
    let impacted_component_ids = component_ids.collect::<BTreeSet<_>>();
    let workspace_ids = repo_graph
        .relationships
        .iter()
        .filter(|relationship| relationship.kind == RelationshipKind::BelongsToWorkspace)
        .filter(|relationship| impacted_component_ids.contains(&relationship.src_id))
        .map(|relationship| relationship.dst_id.as_str())
        .collect::<BTreeSet<_>>();

    repo_graph
        .workspaces
        .iter()
        .filter(|workspace| workspace_ids.contains(workspace.id.as_str()))
        .collect()
}

fn recommend_tests(
    repo_graph: &RepoInspection,
    changed_files: &[String],
    impacted_components: &[ImpactedComponent],
    broad_change_detected: bool,
) -> Vec<RecommendedTest> {
    let mut tests = repo_graph
        .tests
        .iter()
        .filter(|test| {
            broad_change_detected
                || test_applies_to_impacted_components(test, impacted_components)
                || changed_files.iter().any(|file| is_test_path(file))
        })
        .map(|test| {
            let scoped = test_scope_matches_impacted_component(test, impacted_components);
            RecommendedTest {
                test_id: test.id.clone(),
                command: test.command.clone(),
                scope_ref: test.scope_ref.clone(),
                rank: test_rank(test, broad_change_detected, scoped),
                reason: if broad_change_detected {
                    "broad_change_requires_test_command".to_string()
                } else if scoped {
                    "test_scope_matches_impacted_component".to_string()
                } else if changed_files.iter().any(|file| is_test_path(file)) {
                    "changed_file_is_test_path".to_string()
                } else {
                    "generic_test_command_for_impacted_component".to_string()
                },
                confidence: if scoped {
                    ImpactConfidence::High
                } else if impacted_components.is_empty() {
                    ImpactConfidence::Low
                } else {
                    ImpactConfidence::Medium
                },
                evidence_ids: stable_evidence_ids(vec![test.evidence_id.clone()]),
            }
        })
        .collect::<Vec<_>>();

    tests.sort_by(|a, b| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| a.test_id.cmp(&b.test_id))
            .then_with(|| a.command.cmp(&b.command))
    });
    tests
}

fn recommend_commands(
    repo_graph: &RepoInspection,
    impacted_components: &[ImpactedComponent],
    has_recommended_tests: bool,
    broad_change_detected: bool,
) -> Vec<RecommendedCommand> {
    let mut commands = repo_graph
        .commands
        .iter()
        .filter(|command| {
            broad_change_detected
                || command_applies_to_impacted_components(command, impacted_components)
        })
        .map(|command| {
            let scoped = command_scope_matches_impacted_component(command, impacted_components);
            RecommendedCommand {
                command_id: command.id.clone(),
                command: command.command.clone(),
                kind: command.kind.clone(),
                scope_ref: command.scope_ref.clone(),
                rank: command_rank(
                    command,
                    broad_change_detected,
                    scoped,
                    has_recommended_tests,
                ),
                reason: command_reason(command, broad_change_detected, scoped),
                confidence: if scoped {
                    ImpactConfidence::High
                } else if impacted_components.is_empty() {
                    ImpactConfidence::Low
                } else {
                    ImpactConfidence::Medium
                },
                evidence_ids: stable_evidence_ids(vec![command.evidence_id.clone()]),
            }
        })
        .collect::<Vec<_>>();

    commands.sort_by(|a, b| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| a.command_id.cmp(&b.command_id))
            .then_with(|| a.command.cmp(&b.command))
    });
    commands
}

fn test_rank(test: &TestTarget, broad_change_detected: bool, scoped: bool) -> u32 {
    if broad_change_detected {
        return 10;
    }
    if scoped {
        return 10;
    }
    match test.scope_ref.as_deref() {
        Some("repo") | None => 20,
        Some(_) => 30,
    }
}

fn command_rank(
    command: &RepoCommand,
    broad_change_detected: bool,
    scoped: bool,
    has_recommended_tests: bool,
) -> u32 {
    let base = if broad_change_detected {
        match command.kind {
            RepoCommandKind::Check => 10,
            RepoCommandKind::Test => 20,
            RepoCommandKind::Build => 30,
            RepoCommandKind::Lint | RepoCommandKind::Typecheck => 40,
            RepoCommandKind::Format => 50,
            RepoCommandKind::Other => 90,
        }
    } else {
        match command.kind {
            RepoCommandKind::Test if !has_recommended_tests => 10,
            RepoCommandKind::Check => 20,
            RepoCommandKind::Test => 30,
            RepoCommandKind::Lint | RepoCommandKind::Typecheck => 40,
            RepoCommandKind::Build => 50,
            RepoCommandKind::Format => 60,
            RepoCommandKind::Other => 90,
        }
    };

    if scoped {
        base
    } else {
        base + 5
    }
}

fn command_reason(command: &RepoCommand, broad_change_detected: bool, scoped: bool) -> String {
    if broad_change_detected {
        return match command.kind {
            RepoCommandKind::Check => "manifest_change_may_affect_compile_graph".to_string(),
            RepoCommandKind::Test => "manifest_change_may_affect_test_graph".to_string(),
            RepoCommandKind::Build => "manifest_change_may_affect_build_graph".to_string(),
            RepoCommandKind::Lint | RepoCommandKind::Typecheck => {
                "manifest_change_may_affect_static_analysis".to_string()
            }
            RepoCommandKind::Format | RepoCommandKind::Other => {
                "broad_change_matches_command_scope".to_string()
            }
        };
    }

    if scoped {
        "command_scope_matches_impacted_component".to_string()
    } else {
        "generic_command_for_impacted_component".to_string()
    }
}

fn impact_scope(
    impacted_components: &[ImpactedComponent],
    broad_change_detected: bool,
) -> ImpactScope {
    if broad_change_detected {
        return ImpactScope::Broad;
    }

    if impacted_components.is_empty() {
        return ImpactScope::Unknown;
    }

    if impacted_components
        .iter()
        .any(|component| component.impact_kind == ImpactKind::Transitive)
    {
        ImpactScope::Mixed
    } else {
        ImpactScope::Targeted
    }
}

fn impact_confidence(
    status: ImpactStatus,
    impacted_components: &[ImpactedComponent],
    recommended_commands: &[RecommendedCommand],
    recommended_tests: &[RecommendedTest],
) -> ImpactConfidence {
    if status == ImpactStatus::InsufficientEvidence {
        return ImpactConfidence::Insufficient;
    }

    if recommended_commands
        .iter()
        .any(|command| command.confidence == ImpactConfidence::High)
        || recommended_tests
            .iter()
            .any(|test| test.confidence == ImpactConfidence::High)
    {
        return ImpactConfidence::High;
    }

    if impacted_components.iter().any(|component| {
        matches!(
            component.impact_kind,
            ImpactKind::Direct | ImpactKind::Transitive
        )
    }) {
        return ImpactConfidence::Medium;
    }

    ImpactConfidence::Low
}

fn command_applies_to_impacted_components(
    command: &RepoCommand,
    components: &[ImpactedComponent],
) -> bool {
    match command.scope_ref.as_deref() {
        Some("repo") | None => !components.is_empty(),
        Some(scope_ref) => components
            .iter()
            .any(|component| component.component_id == scope_ref),
    }
}

fn test_applies_to_impacted_components(
    test: &TestTarget,
    components: &[ImpactedComponent],
) -> bool {
    match test.scope_ref.as_deref() {
        Some("repo") | None => !components.is_empty(),
        Some(scope_ref) => components
            .iter()
            .any(|component| component.component_id == scope_ref),
    }
}

fn command_scope_matches_impacted_component(
    command: &RepoCommand,
    components: &[ImpactedComponent],
) -> bool {
    command.scope_ref.as_deref().is_some_and(|scope_ref| {
        scope_ref != "repo"
            && components
                .iter()
                .any(|component| component.component_id == scope_ref)
    })
}

fn test_scope_matches_impacted_component(
    test: &TestTarget,
    components: &[ImpactedComponent],
) -> bool {
    test.scope_ref.as_deref().is_some_and(|scope_ref| {
        scope_ref != "repo"
            && components
                .iter()
                .any(|component| component.component_id == scope_ref)
    })
}

fn normalize_changed_file(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace('\\', "/")
        .to_string()
}

fn is_broad_repo_change(repo_graph: &RepoInspection, changed_file: &str) -> bool {
    repo_graph.detected_files.iter().any(|file| {
        file.path == changed_file
            && matches!(
                file.kind,
                DetectedFileKind::Manifest
                    | DetectedFileKind::Lockfile
                    | DetectedFileKind::WorkspaceConfig
                    | DetectedFileKind::BuildConfig
            )
    })
}

fn component_matches_changed_file(component: &Component, changed_file: &str) -> bool {
    component
        .file_patterns
        .iter()
        .any(|pattern| path_matches_pattern(changed_file, pattern))
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == path {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{suffix}"));
    }

    if let Some(suffix) = pattern.strip_prefix("**/*.") {
        return path.ends_with(&format!(".{suffix}"));
    }

    false
}

fn is_test_path(path: &str) -> bool {
    path.starts_with("tests/")
        || path.starts_with("test/")
        || path.ends_with("_test.go")
        || path.contains(".test.")
        || path.contains(".spec.")
}
