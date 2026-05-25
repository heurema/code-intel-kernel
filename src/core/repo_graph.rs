use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

pub const INSPECT_CONTRACT_VERSION: &str = "0.2";
pub const IMPACT_CONTRACT_VERSION: &str = "0.2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoCommandKind {
    Test,
    Lint,
    Build,
    Check,
    Format,
    Typecheck,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManagerKind {
    Cargo,
    Npm,
    Pnpm,
    Yarn,
    Uv,
    Poetry,
    Pip,
    Go,
    Make,
    Just,
    Docker,
    GitHubActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectedFileKind {
    Manifest,
    Lockfile,
    WorkspaceConfig,
    BuildConfig,
    TestConfig,
    Workflow,
    ContainerConfig,
    SourceHint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionCategory {
    AmbiguousDetection,
    IgnoredPath,
    MalformedManifest,
    MissingCommand,
    NoSupportedManifests,
    PartialSupport,
    RepoGraphOnly,
    UnmappedChange,
    UnreadableManifest,
    UnsupportedPattern,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    Contains,
    BelongsToWorkspace,
    DefinesComponent,
    HasCommand,
    HasTest,
    Tests,
    DependsOn,
    UsesPackageManager,
    EvidenceFor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactStatus {
    Ok,
    Partial,
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactKind {
    Direct,
    Transitive,
    Broad,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactScope {
    Targeted,
    Broad,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactConfidence {
    High,
    Medium,
    Low,
    Insufficient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoInspection {
    pub contract_version: String,
    pub repo: RepoInfo,
    pub detected_files: Vec<DetectedFile>,
    pub package_managers: Vec<PackageManager>,
    pub workspaces: Vec<Workspace>,
    pub components: Vec<Component>,
    pub commands: Vec<RepoCommand>,
    pub tests: Vec<TestTarget>,
    pub relationships: Vec<Relationship>,
    pub evidence: Vec<Evidence>,
    pub warnings: Vec<DetectionIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoInfo {
    pub root: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedFile {
    pub id: String,
    pub path: String,
    pub kind: DetectedFileKind,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManager {
    pub id: String,
    pub kind: PackageManagerKind,
    pub name: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub file_patterns: Vec<String>,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoCommand {
    pub id: String,
    pub kind: RepoCommandKind,
    pub command: String,
    pub scope: String,
    pub scope_ref: Option<String>,
    pub confidence: f32,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestTarget {
    pub id: String,
    pub name: String,
    pub command: String,
    pub scope: String,
    pub scope_ref: Option<String>,
    pub confidence: f32,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub kind: RelationshipKind,
    pub src_id: String,
    pub dst_id: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub contract_version: String,
    pub status: ImpactStatus,
    pub impact_scope: ImpactScope,
    pub confidence: ImpactConfidence,
    pub changed_files: Vec<String>,
    pub impacted_components: Vec<ImpactedComponent>,
    pub impacted_workspaces: Vec<ImpactedWorkspace>,
    pub recommended_commands: Vec<RecommendedCommand>,
    pub recommended_tests: Vec<RecommendedTest>,
    pub evidence: Vec<Evidence>,
    pub warnings: Vec<DetectionIssue>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactedComponent {
    pub component_id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub impact_kind: ImpactKind,
    pub distance: Option<u32>,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactedWorkspace {
    pub workspace_id: String,
    pub name: String,
    pub impact_kind: ImpactKind,
    pub distance: Option<u32>,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendedCommand {
    pub command_id: String,
    pub command: String,
    pub kind: RepoCommandKind,
    pub scope_ref: Option<String>,
    pub rank: u32,
    pub reason: String,
    pub confidence: ImpactConfidence,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendedTest {
    pub test_id: String,
    pub command: String,
    pub scope_ref: Option<String>,
    pub rank: u32,
    pub reason: String,
    pub confidence: ImpactConfidence,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub field: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectionIssue {
    pub id: String,
    pub severity: DetectionSeverity,
    pub category: DetectionCategory,
    pub message: String,
    pub path: Option<String>,
    pub evidence_id: Option<String>,
}

pub fn inspect_repo(repo_path: impl AsRef<Path>) -> RepoInspection {
    let root_path = repo_path.as_ref();
    let root = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let mut builder = RepoGraphBuilder::new(display_path(&root));

    detect_rust(&root, &mut builder);
    detect_node(&root, &mut builder);
    detect_python(&root, &mut builder);
    detect_go(&root, &mut builder);
    detect_generic(&root, &mut builder);
    detect_ignored_paths(&root, &mut builder);

    if builder.detected_files.is_empty() {
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::NoSupportedManifests,
            "No supported repository manifests were detected.",
            None,
            None,
        );
    }

    builder.finish()
}

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
            message: "SymbolGraph is not implemented; impact is based on repository structure and paths only.".to_string(),
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

struct CargoWorkspaceMember {
    relative_manifest: PathBuf,
    package_name: String,
    component_id: String,
    manifest: TomlValue,
}

struct CargoDependency {
    name: String,
    field: String,
    path_dependency: bool,
}

struct CommandFileTargets {
    targets: Vec<String>,
    ambiguous_lines: usize,
}

struct RepoGraphBuilder {
    repo_root: String,
    next_evidence: usize,
    next_warning: usize,
    detected_files: Vec<DetectedFile>,
    package_managers: Vec<PackageManager>,
    workspaces: Vec<Workspace>,
    components: Vec<Component>,
    commands: Vec<RepoCommand>,
    tests: Vec<TestTarget>,
    relationships: Vec<Relationship>,
    evidence: Vec<Evidence>,
    warnings: Vec<DetectionIssue>,
}

impl RepoGraphBuilder {
    fn new(repo_root: String) -> Self {
        Self {
            repo_root,
            next_evidence: 1,
            next_warning: 1,
            detected_files: Vec::new(),
            package_managers: Vec::new(),
            workspaces: Vec::new(),
            components: Vec::new(),
            commands: Vec::new(),
            tests: Vec::new(),
            relationships: Vec::new(),
            evidence: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn finish(mut self) -> RepoInspection {
        self.detected_files.sort_by(|a, b| a.id.cmp(&b.id));
        self.package_managers.sort_by(|a, b| a.id.cmp(&b.id));
        self.workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        self.components.sort_by(|a, b| a.id.cmp(&b.id));
        self.commands.sort_by(|a, b| a.id.cmp(&b.id));
        self.tests.sort_by(|a, b| a.id.cmp(&b.id));
        self.relationships.sort_by(|a, b| a.id.cmp(&b.id));
        self.warnings.sort_by(|a, b| a.id.cmp(&b.id));

        RepoInspection {
            contract_version: INSPECT_CONTRACT_VERSION.to_string(),
            repo: RepoInfo {
                root: self.repo_root,
                read_only: true,
            },
            detected_files: self.detected_files,
            package_managers: self.package_managers,
            workspaces: self.workspaces,
            components: self.components,
            commands: self.commands,
            tests: self.tests,
            relationships: self.relationships,
            evidence: self.evidence,
            warnings: self.warnings,
        }
    }

    fn add_detected_file(
        &mut self,
        path: &Path,
        kind: DetectedFileKind,
        evidence_kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let evidence_id = self.add_evidence(path, evidence_kind, field, reason);
        let file_id = stable_id("file", &normalize_path(path));
        self.detected_files.push(DetectedFile {
            id: file_id.clone(),
            path: normalize_path(path),
            kind,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            &file_id,
            evidence_id.clone(),
        );
        evidence_id
    }

    fn add_evidence(
        &mut self,
        path: &Path,
        kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let id = format!("evidence-{}", self.next_evidence);
        self.next_evidence += 1;
        self.evidence.push(Evidence {
            id: id.clone(),
            path: normalize_path(path),
            kind: kind.to_string(),
            field: field.map(str::to_string),
            reason: reason.to_string(),
        });
        id
    }

    fn add_warning(
        &mut self,
        severity: DetectionSeverity,
        category: DetectionCategory,
        message: &str,
        path: Option<&Path>,
        evidence_id: Option<String>,
    ) {
        let id = format!("warning-{}", self.next_warning);
        self.next_warning += 1;
        self.warnings.push(DetectionIssue {
            id,
            severity,
            category,
            message: message.to_string(),
            path: path.map(normalize_path),
            evidence_id,
        });
    }

    fn add_package_manager(&mut self, kind: PackageManagerKind, name: &str, evidence_id: String) {
        let id = stable_id("package-manager", name);
        if self
            .package_managers
            .iter()
            .any(|manager| manager.kind == kind && manager.name == name)
        {
            return;
        }

        self.package_managers.push(PackageManager {
            id: id.clone(),
            kind,
            name: name.to_string(),
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            &id,
            evidence_id.clone(),
        );
    }

    fn add_workspace(&mut self, id: &str, name: &str, members: Vec<String>, evidence_id: String) {
        self.workspaces.push(Workspace {
            id: id.to_string(),
            name: name.to_string(),
            members,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    fn add_component(
        &mut self,
        id: &str,
        name: &str,
        kind: &str,
        path: &str,
        file_patterns: Vec<String>,
        evidence_id: String,
    ) {
        if self.components.iter().any(|existing| existing.id == id) {
            return;
        }

        self.components.push(Component {
            id: id.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            path: path.to_string(),
            file_patterns,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::DefinesComponent,
            "repo",
            id,
            evidence_id.clone(),
        );
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    fn add_command(
        &mut self,
        id: &str,
        kind: RepoCommandKind,
        command: &str,
        scope_ref: Option<&str>,
        confidence: f32,
        evidence_id: String,
    ) {
        if self
            .commands
            .iter()
            .any(|existing| existing.command == command)
        {
            return;
        }

        self.commands.push(RepoCommand {
            id: id.to_string(),
            kind,
            command: command.to_string(),
            scope: ".".to_string(),
            scope_ref: scope_ref.map(str::to_string),
            confidence,
            evidence_id: evidence_id.clone(),
        });
        if let Some(scope_ref) = scope_ref {
            self.add_relationship(
                RelationshipKind::HasCommand,
                scope_ref,
                id,
                evidence_id.clone(),
            );
        }
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    fn add_test(
        &mut self,
        id: &str,
        name: &str,
        command: &str,
        scope_ref: Option<&str>,
        confidence: f32,
        evidence_id: String,
    ) {
        if self
            .tests
            .iter()
            .any(|existing| existing.command == command)
        {
            return;
        }

        self.tests.push(TestTarget {
            id: id.to_string(),
            name: name.to_string(),
            command: command.to_string(),
            scope: ".".to_string(),
            scope_ref: scope_ref.map(str::to_string),
            confidence,
            evidence_id: evidence_id.clone(),
        });
        if let Some(scope_ref) = scope_ref {
            self.add_relationship(
                RelationshipKind::HasTest,
                scope_ref,
                id,
                evidence_id.clone(),
            );
            self.add_relationship(RelationshipKind::Tests, id, scope_ref, evidence_id.clone());
        }
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    fn add_relationship(
        &mut self,
        kind: RelationshipKind,
        src_id: &str,
        dst_id: &str,
        evidence_id: String,
    ) {
        let id = stable_relationship_id(&kind, src_id, dst_id);
        if self.relationships.iter().any(|existing| existing.id == id) {
            return;
        }

        self.relationships.push(Relationship {
            id,
            kind,
            src_id: src_id.to_string(),
            dst_id: dst_id.to_string(),
            evidence_id,
        });
    }
}

fn detect_rust(root: &Path, builder: &mut RepoGraphBuilder) {
    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        let manifest_evidence = builder.add_detected_file(
            Path::new("Cargo.toml"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Rust Cargo manifest detected.",
        );
        builder.add_package_manager(
            PackageManagerKind::Cargo,
            "cargo",
            manifest_evidence.clone(),
        );

        match read_toml(&cargo_toml) {
            Ok(manifest) => {
                add_cargo_commands(builder, manifest_evidence.clone());

                let package_name = manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
                    .and_then(TomlValue::as_str);

                if let Some(name) = package_name {
                    let evidence_id = builder.add_evidence(
                        Path::new("Cargo.toml"),
                        "manifest",
                        Some("package.name"),
                        "Cargo package name.",
                    );
                    builder.add_component(
                        "component-rust-package",
                        name,
                        "rust_crate",
                        ".",
                        vec![
                            "Cargo.toml".to_string(),
                            "Cargo.lock".to_string(),
                            "src/**".to_string(),
                            "tests/**".to_string(),
                        ],
                        evidence_id,
                    );
                    builder.add_relationship(
                        RelationshipKind::UsesPackageManager,
                        "component-rust-package",
                        "package-manager-cargo",
                        manifest_evidence.clone(),
                    );
                }

                if let Some(members) = manifest
                    .get("workspace")
                    .and_then(|workspace| workspace.get("members"))
                    .and_then(TomlValue::as_array)
                {
                    let workspace_members = members
                        .iter()
                        .filter_map(TomlValue::as_str)
                        .map(str::to_string)
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>();

                    if !workspace_members.is_empty() {
                        let evidence_id = builder.add_evidence(
                            Path::new("Cargo.toml"),
                            "manifest",
                            Some("workspace.members"),
                            "Cargo workspace members.",
                        );
                        builder.add_workspace(
                            "workspace-cargo",
                            "cargo-workspace",
                            workspace_members.clone(),
                            evidence_id.clone(),
                        );
                        detect_cargo_workspace_members(
                            root,
                            &workspace_members,
                            &evidence_id,
                            builder,
                        );
                    }
                }

                detect_cargo_targets(root, &manifest, package_name, builder);
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(Path::new("Cargo.toml")),
                Some(manifest_evidence),
            ),
        }
    }

    let cargo_lock = root.join("Cargo.lock");
    if cargo_lock.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("Cargo.lock"),
            DetectedFileKind::Lockfile,
            "lockfile",
            None,
            "Cargo lockfile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Cargo, "cargo", evidence_id);
    }
}

fn add_cargo_commands(builder: &mut RepoGraphBuilder, manifest_evidence: String) {
    builder.add_command(
        "cmd-cargo-check",
        RepoCommandKind::Check,
        "cargo check",
        Some("repo"),
        0.95,
        manifest_evidence.clone(),
    );
    builder.add_command(
        "cmd-cargo-test",
        RepoCommandKind::Test,
        "cargo test",
        Some("repo"),
        0.95,
        manifest_evidence.clone(),
    );
    builder.add_command(
        "cmd-cargo-build",
        RepoCommandKind::Build,
        "cargo build",
        Some("repo"),
        0.9,
        manifest_evidence.clone(),
    );
    builder.add_command(
        "cmd-cargo-clippy",
        RepoCommandKind::Lint,
        "cargo clippy -- -D warnings",
        Some("repo"),
        0.8,
        manifest_evidence.clone(),
    );
    builder.add_command(
        "cmd-cargo-fmt",
        RepoCommandKind::Format,
        "cargo fmt --check",
        Some("repo"),
        0.8,
        manifest_evidence.clone(),
    );
    builder.add_test(
        "test-cargo-test",
        "cargo test",
        "cargo test",
        Some("repo"),
        0.95,
        manifest_evidence,
    );
}

fn detect_cargo_workspace_members(
    root: &Path,
    workspace_members: &[String],
    workspace_evidence_id: &str,
    builder: &mut RepoGraphBuilder,
) {
    let mut members = Vec::<CargoWorkspaceMember>::new();

    for member in workspace_members {
        let relative_manifest = PathBuf::from(member).join("Cargo.toml");
        let absolute_manifest = root.join(&relative_manifest);

        if !absolute_manifest.exists() {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::UnsupportedPattern,
                "Cargo workspace member was listed, but its Cargo.toml was not found.",
                Some(&relative_manifest),
                Some(workspace_evidence_id.to_string()),
            );
            continue;
        }

        let manifest_evidence = builder.add_detected_file(
            &relative_manifest,
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Cargo workspace member manifest detected.",
        );

        match read_toml(&absolute_manifest) {
            Ok(manifest) => {
                let Some(package_name) = manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
                    .and_then(TomlValue::as_str)
                    .map(str::to_string)
                else {
                    builder.add_warning(
                        DetectionSeverity::Warning,
                        DetectionCategory::UnsupportedPattern,
                        "Cargo workspace member manifest did not define package.name.",
                        Some(&relative_manifest),
                        Some(manifest_evidence.clone()),
                    );
                    continue;
                };

                let component_id = stable_id("component-rust-crate", &package_name);
                let package_name_field = format!("{}/package.name", member);
                let component_evidence = builder.add_evidence(
                    &relative_manifest,
                    "manifest",
                    Some(&package_name_field),
                    "Cargo workspace member package name.",
                );
                builder.add_component(
                    &component_id,
                    &package_name,
                    "rust_crate",
                    member,
                    vec![
                        normalize_path(&relative_manifest),
                        format!("{member}/src/**"),
                        format!("{member}/tests/**"),
                    ],
                    component_evidence.clone(),
                );
                builder.add_relationship(
                    RelationshipKind::BelongsToWorkspace,
                    &component_id,
                    "workspace-cargo",
                    component_evidence.clone(),
                );
                builder.add_relationship(
                    RelationshipKind::UsesPackageManager,
                    &component_id,
                    "package-manager-cargo",
                    manifest_evidence,
                );

                members.push(CargoWorkspaceMember {
                    relative_manifest,
                    package_name,
                    component_id,
                    manifest,
                });
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(&relative_manifest),
                Some(manifest_evidence),
            ),
        }
    }

    let component_by_name = members
        .iter()
        .map(|member| (member.package_name.as_str(), member.component_id.as_str()))
        .collect::<BTreeMap<_, _>>();

    for member in &members {
        for dependency in cargo_dependencies(&member.manifest) {
            if !dependency.path_dependency {
                continue;
            }
            let Some(dependency_component_id) = component_by_name.get(dependency.name.as_str())
            else {
                continue;
            };

            let evidence_id = builder.add_evidence(
                &member.relative_manifest,
                "manifest",
                Some(&dependency.field),
                "Cargo workspace path dependency.",
            );
            builder.add_relationship(
                RelationshipKind::DependsOn,
                &member.component_id,
                dependency_component_id,
                evidence_id,
            );
        }
    }
}

fn detect_cargo_targets(
    root: &Path,
    manifest: &TomlValue,
    package_name: Option<&str>,
    builder: &mut RepoGraphBuilder,
) {
    if manifest.get("lib").is_some() {
        let evidence_id = builder.add_evidence(
            Path::new("Cargo.toml"),
            "manifest",
            Some("lib"),
            "Cargo library target.",
        );
        builder.add_component(
            "component-rust-lib",
            "lib",
            "rust_lib_target",
            ".",
            cargo_lib_patterns(root, manifest),
            evidence_id,
        );
    } else if root.join("src/lib.rs").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("src/lib.rs"),
            DetectedFileKind::SourceHint,
            "source_hint",
            None,
            "Cargo default library target source detected.",
        );
        builder.add_component(
            "component-rust-lib",
            "lib",
            "rust_lib_target",
            ".",
            vec!["src/lib.rs".to_string()],
            evidence_id,
        );
    }

    let mut explicit_bin_detected = false;
    if let Some(bin_targets) = manifest.get("bin").and_then(TomlValue::as_array) {
        for (index, bin) in bin_targets.iter().enumerate() {
            explicit_bin_detected = true;
            let name = bin.get("name").and_then(TomlValue::as_str).unwrap_or("bin");
            let evidence_id = builder.add_evidence(
                Path::new("Cargo.toml"),
                "manifest",
                Some("bin"),
                "Cargo binary target.",
            );
            builder.add_component(
                &format!("component-rust-bin-{index}"),
                name,
                "rust_bin_target",
                ".",
                cargo_bin_patterns(root, bin),
                evidence_id,
            );
        }
    }

    if !explicit_bin_detected && root.join("src/main.rs").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("src/main.rs"),
            DetectedFileKind::SourceHint,
            "source_hint",
            None,
            "Cargo default binary target source detected.",
        );
        builder.add_component(
            "component-rust-bin-0",
            package_name.unwrap_or("bin"),
            "rust_bin_target",
            ".",
            vec!["src/main.rs".to_string()],
            evidence_id,
        );
    }
}

fn detect_node(root: &Path, builder: &mut RepoGraphBuilder) {
    let package_json = root.join("package.json");
    if package_json.exists() {
        let manifest_evidence = builder.add_detected_file(
            Path::new("package.json"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Node package manifest detected.",
        );
        builder.add_package_manager(PackageManagerKind::Npm, "npm", manifest_evidence.clone());
        if !root.join("package-lock.json").exists()
            && !root.join("pnpm-lock.yaml").exists()
            && !root.join("yarn.lock").exists()
        {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::AmbiguousDetection,
                "package.json was found without a lockfile; npm is treated as the default package manager hint.",
                Some(Path::new("package.json")),
                Some(manifest_evidence.clone()),
            );
        }

        match read_json(&package_json) {
            Ok(manifest) => {
                if let Some(name) = manifest.get("name").and_then(JsonValue::as_str) {
                    let evidence_id = builder.add_evidence(
                        Path::new("package.json"),
                        "manifest",
                        Some("name"),
                        "Node package name.",
                    );
                    builder.add_component(
                        "component-node-package",
                        name,
                        "node_package",
                        ".",
                        vec![
                            "package.json".to_string(),
                            "src/**".to_string(),
                            "test/**".to_string(),
                            "tests/**".to_string(),
                        ],
                        evidence_id,
                    );
                    builder.add_relationship(
                        RelationshipKind::UsesPackageManager,
                        "component-node-package",
                        "package-manager-npm",
                        manifest_evidence.clone(),
                    );
                }

                if manifest.get("workspaces").is_some() {
                    if let Some(workspaces) = extract_package_json_workspaces(&manifest) {
                        let evidence_id = builder.add_evidence(
                            Path::new("package.json"),
                            "manifest",
                            Some("workspaces"),
                            "Node workspace members.",
                        );
                        builder.add_workspace(
                            "workspace-node",
                            "node-workspace",
                            workspaces,
                            evidence_id,
                        );
                    } else {
                        builder.add_warning(
                            DetectionSeverity::Warning,
                            DetectionCategory::UnsupportedPattern,
                            "package.json workspaces field is present but not in a supported array/packages shape.",
                            Some(Path::new("package.json")),
                            Some(manifest_evidence.clone()),
                        );
                    }
                }

                if let Some(scripts) = manifest.get("scripts").and_then(JsonValue::as_object) {
                    let has_test_script =
                        add_node_script(builder, scripts, "test", RepoCommandKind::Test, 0.9);
                    add_node_script(builder, scripts, "build", RepoCommandKind::Build, 0.85);
                    add_node_script(builder, scripts, "lint", RepoCommandKind::Lint, 0.85);
                    add_node_script(builder, scripts, "check", RepoCommandKind::Check, 0.8);
                    add_node_script(
                        builder,
                        scripts,
                        "typecheck",
                        RepoCommandKind::Typecheck,
                        0.8,
                    );
                    if !has_test_script {
                        builder.add_warning(
                            DetectionSeverity::Info,
                            DetectionCategory::MissingCommand,
                            "package.json does not define scripts.test; no Node test target was inferred.",
                            Some(Path::new("package.json")),
                            Some(manifest_evidence.clone()),
                        );
                    }
                } else {
                    builder.add_warning(
                        DetectionSeverity::Info,
                        DetectionCategory::MissingCommand,
                        "package.json does not define scripts; no Node commands were inferred.",
                        Some(Path::new("package.json")),
                        Some(manifest_evidence.clone()),
                    );
                }
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(Path::new("package.json")),
                Some(manifest_evidence),
            ),
        }
    }

    detect_lockfile(
        builder,
        root,
        "package-lock.json",
        PackageManagerKind::Npm,
        "npm",
    );
    detect_lockfile(
        builder,
        root,
        "pnpm-lock.yaml",
        PackageManagerKind::Pnpm,
        "pnpm",
    );
    detect_lockfile(builder, root, "yarn.lock", PackageManagerKind::Yarn, "yarn");

    let pnpm_workspace = root.join("pnpm-workspace.yaml");
    if pnpm_workspace.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("pnpm-workspace.yaml"),
            DetectedFileKind::WorkspaceConfig,
            "workspace_config",
            None,
            "pnpm workspace config detected.",
        );
        builder.add_package_manager(PackageManagerKind::Pnpm, "pnpm", evidence_id.clone());
        let members = parse_simple_yaml_packages(&pnpm_workspace);
        if !members.is_empty() {
            builder.add_workspace("workspace-pnpm", "pnpm-workspace", members, evidence_id);
        } else {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::UnsupportedPattern,
                "pnpm-workspace.yaml was detected but no supported packages list was parsed.",
                Some(Path::new("pnpm-workspace.yaml")),
                Some(evidence_id),
            );
        }
    }
}

fn detect_python(root: &Path, builder: &mut RepoGraphBuilder) {
    let mut python_project_detected = false;
    let mut python_project_evidence = None;
    let mut pytest_evidence = None;

    let pyproject = root.join("pyproject.toml");
    if pyproject.exists() {
        python_project_detected = true;
        let manifest_evidence = builder.add_detected_file(
            Path::new("pyproject.toml"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Python project manifest detected.",
        );
        python_project_evidence = Some(manifest_evidence.clone());

        match read_toml(&pyproject) {
            Ok(manifest) => {
                let name = manifest
                    .get("project")
                    .and_then(|project| project.get("name"))
                    .and_then(TomlValue::as_str)
                    .or_else(|| {
                        manifest
                            .get("tool")
                            .and_then(|tool| tool.get("poetry"))
                            .and_then(|poetry| poetry.get("name"))
                            .and_then(TomlValue::as_str)
                    });

                if let Some(name) = name {
                    let evidence_id = builder.add_evidence(
                        Path::new("pyproject.toml"),
                        "manifest",
                        Some("project.name"),
                        "Python project name.",
                    );
                    builder.add_component(
                        "component-python-project",
                        name,
                        "python_project",
                        ".",
                        vec![
                            "pyproject.toml".to_string(),
                            "requirements.txt".to_string(),
                            "src/**".to_string(),
                            "tests/**".to_string(),
                        ],
                        evidence_id,
                    );
                }

                if manifest
                    .get("tool")
                    .and_then(|tool| tool.get("poetry"))
                    .is_some()
                {
                    builder.add_package_manager(
                        PackageManagerKind::Poetry,
                        "poetry",
                        manifest_evidence.clone(),
                    );
                }

                if let Some(field) = pyproject_pytest_field(&manifest) {
                    let evidence_id = builder.add_evidence(
                        Path::new("pyproject.toml"),
                        "test_config",
                        Some(&field),
                        "pytest evidence detected in pyproject.toml.",
                    );
                    pytest_evidence = Some(evidence_id);
                }
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(Path::new("pyproject.toml")),
                Some(manifest_evidence),
            ),
        }
    }

    python_project_detected |=
        detect_python_lockfile(builder, root, "uv.lock", PackageManagerKind::Uv, "uv");
    python_project_detected |= detect_python_lockfile(
        builder,
        root,
        "poetry.lock",
        PackageManagerKind::Poetry,
        "poetry",
    );

    let requirements = root.join("requirements.txt");
    if requirements.exists() {
        python_project_detected = true;
        let evidence_id = builder.add_detected_file(
            Path::new("requirements.txt"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Python requirements file detected.",
        );
        python_project_evidence = Some(evidence_id.clone());
        builder.add_package_manager(PackageManagerKind::Pip, "pip", evidence_id.clone());
        if requirements_mentions_pytest(&requirements) {
            let pytest_requirement_evidence = builder.add_evidence(
                Path::new("requirements.txt"),
                "test_config",
                Some("requirements.pytest"),
                "pytest dependency detected in requirements.txt.",
            );
            pytest_evidence = Some(pytest_requirement_evidence);
        }
    }

    let pytest_ini = root.join("pytest.ini");
    if pytest_ini.exists() {
        python_project_detected = true;
        let evidence_id = builder.add_detected_file(
            Path::new("pytest.ini"),
            DetectedFileKind::TestConfig,
            "test_config",
            None,
            "pytest.ini detected.",
        );
        pytest_evidence = Some(evidence_id);
    }

    let tests_dir_evidence = if python_project_detected && root.join("tests").is_dir() {
        Some(builder.add_detected_file(
            Path::new("tests"),
            DetectedFileKind::TestConfig,
            "directory",
            None,
            "tests directory detected.",
        ))
    } else {
        None
    };

    if let Some(evidence_id) = pytest_evidence {
        add_pytest(builder, evidence_id);
    } else if let Some(evidence_id) = tests_dir_evidence {
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::AmbiguousDetection,
            "Python tests directory detected, but no pytest evidence was found; no test command was inferred.",
            Some(Path::new("tests")),
            Some(evidence_id),
        );
    } else if python_project_detected {
        let warning_path = if pyproject.exists() {
            Some(Path::new("pyproject.toml"))
        } else if requirements.exists() {
            Some(Path::new("requirements.txt"))
        } else {
            None
        };
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::MissingCommand,
            "Python project detected but no pytest configuration or tests directory was found.",
            warning_path,
            python_project_evidence,
        );
    }
}

fn detect_go(root: &Path, builder: &mut RepoGraphBuilder) {
    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        let manifest_evidence = builder.add_detected_file(
            Path::new("go.mod"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Go module manifest detected.",
        );
        builder.add_package_manager(PackageManagerKind::Go, "go", manifest_evidence.clone());
        if let Some(module_name) = read_go_module_name(&go_mod) {
            let test_file_evidence = first_go_test_file(root).map(|path| {
                builder.add_detected_file(
                    &path,
                    DetectedFileKind::TestConfig,
                    "source_hint",
                    None,
                    "Go test file detected.",
                )
            });
            let test_confidence = if test_file_evidence.is_some() {
                0.95
            } else {
                0.85
            };

            builder.add_component(
                "component-go-module",
                &module_name,
                "go_module",
                ".",
                vec![
                    "go.mod".to_string(),
                    "go.work".to_string(),
                    "*.go".to_string(),
                    "**/*.go".to_string(),
                ],
                manifest_evidence.clone(),
            );
            builder.add_command(
                "cmd-go-test",
                RepoCommandKind::Test,
                "go test ./...",
                Some("component-go-module"),
                test_confidence,
                test_file_evidence
                    .clone()
                    .unwrap_or_else(|| manifest_evidence.clone()),
            );
            builder.add_command(
                "cmd-go-build",
                RepoCommandKind::Build,
                "go build ./...",
                Some("component-go-module"),
                0.85,
                manifest_evidence.clone(),
            );
            builder.add_test(
                "test-go-test",
                "go test",
                "go test ./...",
                Some("component-go-module"),
                test_confidence,
                test_file_evidence.unwrap_or(manifest_evidence),
            );
        } else {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::MalformedManifest,
                "go.mod was detected but no module declaration was parsed; Go commands were not inferred.",
                Some(Path::new("go.mod")),
                Some(manifest_evidence),
            );
        }
    }

    let go_work = root.join("go.work");
    if go_work.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("go.work"),
            DetectedFileKind::WorkspaceConfig,
            "workspace_config",
            None,
            "Go workspace detected.",
        );
        let members = read_go_work_members(&go_work);
        builder.add_workspace(
            "workspace-go",
            "go-workspace",
            members.clone(),
            evidence_id.clone(),
        );
        if members.is_empty() {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::PartialSupport,
                "go.work was detected but no simple use members were parsed.",
                Some(Path::new("go.work")),
                Some(evidence_id),
            );
        }
    }
}

fn detect_generic(root: &Path, builder: &mut RepoGraphBuilder) {
    let makefile = root.join("Makefile");
    if makefile.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("Makefile"),
            DetectedFileKind::BuildConfig,
            "build_config",
            None,
            "Makefile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Make, "make", evidence_id.clone());
        builder.add_component(
            "component-make-project",
            "make-project",
            "generic_make_project",
            ".",
            vec!["Makefile".to_string()],
            evidence_id.clone(),
        );
        let parsed_targets = read_command_file_targets(&makefile);
        add_command_file_targets(
            builder,
            "Makefile",
            "make",
            "component-make-project",
            &parsed_targets.targets,
        );

        if !parsed_targets.targets.iter().any(|target| target == "test") {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "Makefile detected but no test target was parsed.",
                Some(Path::new("Makefile")),
                Some(evidence_id.clone()),
            );
        }

        if parsed_targets.ambiguous_lines > 0 {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::PartialSupport,
                "Makefile contains target-like lines that were not parsed conservatively.",
                Some(Path::new("Makefile")),
                Some(evidence_id),
            );
        }
    }

    let justfile = root.join("justfile");
    if justfile.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("justfile"),
            DetectedFileKind::BuildConfig,
            "build_config",
            None,
            "justfile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Just, "just", evidence_id.clone());
        builder.add_component(
            "component-just-project",
            "just-project",
            "generic_just_project",
            ".",
            vec!["justfile".to_string()],
            evidence_id.clone(),
        );
        let parsed_targets = read_command_file_targets(&justfile);
        add_command_file_targets(
            builder,
            "justfile",
            "just",
            "component-just-project",
            &parsed_targets.targets,
        );

        if !parsed_targets.targets.iter().any(|target| target == "test") {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "justfile detected but no test recipe was parsed.",
                Some(Path::new("justfile")),
                Some(evidence_id.clone()),
            );
        }

        if parsed_targets.ambiguous_lines > 0 {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::PartialSupport,
                "justfile contains recipe-like lines that were not parsed conservatively.",
                Some(Path::new("justfile")),
                Some(evidence_id),
            );
        }
    }

    if root.join("Dockerfile").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("Dockerfile"),
            DetectedFileKind::ContainerConfig,
            "container_config",
            None,
            "Dockerfile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Docker, "docker", evidence_id);
    }

    for compose_file in [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        let path = root.join(compose_file);
        if path.exists() {
            let evidence_id = builder.add_detected_file(
                Path::new(compose_file),
                DetectedFileKind::ContainerConfig,
                "container_config",
                None,
                "Docker Compose file detected.",
            );
            builder.add_package_manager(PackageManagerKind::Docker, "docker", evidence_id);
        }
    }

    let workflows_dir = root.join(".github").join("workflows");
    if let Ok(entries) = fs::read_dir(workflows_dir) {
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
                continue;
            };

            if matches!(extension, "yml" | "yaml") {
                let relative = PathBuf::from(".github")
                    .join("workflows")
                    .join(entry.file_name());
                let evidence_id = builder.add_detected_file(
                    &relative,
                    DetectedFileKind::Workflow,
                    "workflow",
                    None,
                    "GitHub Actions workflow detected.",
                );
                builder.add_package_manager(
                    PackageManagerKind::GitHubActions,
                    "github_actions",
                    evidence_id,
                );
            }
        }
    }
}

fn add_node_script(
    builder: &mut RepoGraphBuilder,
    scripts: &serde_json::Map<String, JsonValue>,
    script_name: &str,
    kind: RepoCommandKind,
    confidence: f32,
) -> bool {
    if scripts
        .get(script_name)
        .and_then(JsonValue::as_str)
        .is_some()
    {
        let evidence_id = builder.add_evidence(
            Path::new("package.json"),
            "manifest",
            Some(&format!("scripts.{script_name}")),
            "Node package script detected.",
        );
        let command = format!("npm run {script_name}");
        builder.add_command(
            &format!("cmd-npm-{script_name}"),
            kind.clone(),
            &command,
            Some("component-node-package"),
            confidence,
            evidence_id.clone(),
        );
        if matches!(kind, RepoCommandKind::Test) {
            builder.add_test(
                &format!("test-npm-{script_name}"),
                script_name,
                &command,
                Some("component-node-package"),
                confidence,
                evidence_id,
            );
        }
        true
    } else {
        false
    }
}

fn detect_lockfile(
    builder: &mut RepoGraphBuilder,
    root: &Path,
    file_name: &str,
    kind: PackageManagerKind,
    name: &str,
) {
    if root.join(file_name).exists() {
        let evidence_id = builder.add_detected_file(
            Path::new(file_name),
            DetectedFileKind::Lockfile,
            "lockfile",
            None,
            &format!("{name} lockfile detected."),
        );
        builder.add_package_manager(kind, name, evidence_id);
    }
}

fn detect_python_lockfile(
    builder: &mut RepoGraphBuilder,
    root: &Path,
    file_name: &str,
    kind: PackageManagerKind,
    name: &str,
) -> bool {
    if root.join(file_name).exists() {
        let evidence_id = builder.add_detected_file(
            Path::new(file_name),
            DetectedFileKind::Lockfile,
            "lockfile",
            None,
            &format!("{name} lockfile detected."),
        );
        builder.add_package_manager(kind, name, evidence_id);
        true
    } else {
        false
    }
}

fn add_pytest(builder: &mut RepoGraphBuilder, evidence_id: String) {
    builder.add_command(
        "cmd-python-pytest",
        RepoCommandKind::Test,
        "pytest",
        Some("component-python-project"),
        0.75,
        evidence_id.clone(),
    );
    builder.add_test(
        "test-python-pytest",
        "pytest",
        "pytest",
        Some("component-python-project"),
        0.75,
        evidence_id,
    );
}

fn pyproject_pytest_field(manifest: &TomlValue) -> Option<String> {
    if manifest
        .get("tool")
        .and_then(|tool| tool.get("pytest"))
        .is_some()
    {
        return Some("tool.pytest".to_string());
    }

    if toml_array_contains_package(
        manifest
            .get("project")
            .and_then(|project| project.get("dependencies")),
        "pytest",
    ) {
        return Some("project.dependencies.pytest".to_string());
    }

    if let Some(optional_dependencies) = manifest
        .get("project")
        .and_then(|project| project.get("optional-dependencies"))
        .and_then(TomlValue::as_table)
    {
        for (group, dependencies) in optional_dependencies {
            if toml_array_contains_package(Some(dependencies), "pytest") {
                return Some(format!("project.optional-dependencies.{group}.pytest"));
            }
        }
    }

    if manifest
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dev-dependencies"))
        .and_then(|dependencies| dependencies.get("pytest"))
        .is_some()
    {
        return Some("tool.poetry.dev-dependencies.pytest".to_string());
    }

    if manifest
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("group"))
        .and_then(|group| group.get("dev"))
        .and_then(|dev| dev.get("dependencies"))
        .and_then(|dependencies| dependencies.get("pytest"))
        .is_some()
    {
        return Some("tool.poetry.group.dev.dependencies.pytest".to_string());
    }

    None
}

fn toml_array_contains_package(value: Option<&TomlValue>, package_name: &str) -> bool {
    value.and_then(TomlValue::as_array).is_some_and(|items| {
        items
            .iter()
            .any(|item| toml_dependency_matches(item, package_name))
    })
}

fn toml_dependency_matches(value: &TomlValue, package_name: &str) -> bool {
    value
        .as_str()
        .is_some_and(|dependency| dependency_name_matches(dependency, package_name))
}

fn requirements_mentions_pytest(path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };

    contents.lines().any(|line| {
        let line = line.split('#').next().unwrap_or("").trim();
        dependency_name_matches(line, "pytest")
    })
}

fn dependency_name_matches(dependency: &str, package_name: &str) -> bool {
    let dependency = dependency.trim().to_ascii_lowercase();
    dependency == package_name
        || dependency
            .strip_prefix(package_name)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|character| matches!(character, '=' | '<' | '>' | '~' | '[' | '!' | ' '))
}

fn add_command_file_targets(
    builder: &mut RepoGraphBuilder,
    file_name: &str,
    tool_name: &str,
    scope_ref: &str,
    targets: &[String],
) {
    for target in targets {
        let Some(kind) = command_kind_for_target(target) else {
            continue;
        };

        let evidence_id = builder.add_evidence(
            Path::new(file_name),
            "build_target",
            Some(&format!("target.{target}")),
            "Build file target detected.",
        );
        let command = format!("{tool_name} {target}");
        builder.add_command(
            &stable_id(&format!("cmd-{tool_name}"), target),
            kind.clone(),
            &command,
            Some(scope_ref),
            0.75,
            evidence_id.clone(),
        );

        if kind == RepoCommandKind::Test {
            builder.add_test(
                &stable_id(&format!("test-{tool_name}"), target),
                &command,
                &command,
                Some(scope_ref),
                0.75,
                evidence_id,
            );
        }
    }
}

fn command_kind_for_target(target: &str) -> Option<RepoCommandKind> {
    match target {
        "test" => Some(RepoCommandKind::Test),
        "check" => Some(RepoCommandKind::Check),
        "build" => Some(RepoCommandKind::Build),
        "lint" => Some(RepoCommandKind::Lint),
        "fmt" | "format" => Some(RepoCommandKind::Format),
        _ => None,
    }
}

fn extract_package_json_workspaces(manifest: &JsonValue) -> Option<Vec<String>> {
    let workspaces = manifest.get("workspaces")?;

    if let Some(items) = workspaces.as_array() {
        let members = items
            .iter()
            .filter_map(JsonValue::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        return (!members.is_empty()).then_some(members);
    }

    let packages = workspaces.get("packages")?.as_array()?;
    let members = packages
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!members.is_empty()).then_some(members)
}

fn parse_simple_yaml_packages(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut in_packages = false;
    let mut members = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "packages:" {
            in_packages = true;
            continue;
        }

        if in_packages && trimmed.starts_with('-') {
            let member = trimmed
                .trim_start_matches('-')
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            if !member.is_empty() {
                members.push(member.to_string());
            }
        } else if in_packages && !trimmed.is_empty() && !line.starts_with(' ') {
            break;
        }
    }

    members
}

fn detect_ignored_paths(root: &Path, builder: &mut RepoGraphBuilder) {
    for (ignored_path, emit_warning) in [
        (".git", false),
        ("node_modules", true),
        ("target", true),
        ("dist", true),
        ("build", true),
        (".cache", true),
        (".venv", true),
        ("__pycache__", true),
        ("coverage", true),
    ] {
        if emit_warning && root.join(ignored_path).exists() {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::IgnoredPath,
                "Generated, dependency, or cache directory was ignored by RepoGraph inspection.",
                Some(Path::new(ignored_path)),
                None,
            );
        }
    }
}

fn read_command_file_targets(path: &Path) -> CommandFileTargets {
    let Ok(contents) = fs::read_to_string(path) else {
        return CommandFileTargets {
            targets: Vec::new(),
            ambiguous_lines: 0,
        };
    };

    let mut targets = BTreeSet::new();
    let mut ambiguous_lines = 0;

    for line in contents.lines() {
        let trimmed = line.trim_start();
        if line.starts_with(char::is_whitespace)
            || trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with(".PHONY:")
            || trimmed.contains(":=")
            || trimmed.contains("?=")
            || trimmed.contains("+=")
        {
            continue;
        }

        let Some((target, _)) = trimmed.split_once(':') else {
            continue;
        };
        let target = target.trim();

        if is_simple_command_target(target) {
            targets.insert(target.to_string());
        } else if is_ambiguous_target_syntax(target) {
            ambiguous_lines += 1;
        }
    }

    CommandFileTargets {
        targets: targets.into_iter().collect(),
        ambiguous_lines,
    }
}

fn is_simple_command_target(target: &str) -> bool {
    command_kind_for_target(target).is_some()
}

fn is_ambiguous_target_syntax(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('.')
        && (target.chars().any(char::is_whitespace)
            || target.contains('%')
            || target.contains('$')
            || target.contains('/'))
}

fn read_go_module_name(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("module ")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
    })
}

fn read_go_work_members(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut members = BTreeSet::new();
    let mut in_use_block = false;

    for line in contents.lines() {
        let trimmed = line.split("//").next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "use (" {
            in_use_block = true;
            continue;
        }

        if in_use_block && trimmed == ")" {
            in_use_block = false;
            continue;
        }

        if in_use_block {
            if is_simple_go_work_member(trimmed) {
                members.insert(trimmed.to_string());
            }
            continue;
        }

        if let Some(member) = trimmed.strip_prefix("use ").map(str::trim) {
            if is_simple_go_work_member(member) {
                members.insert(member.to_string());
            }
        }
    }

    members.into_iter().collect()
}

fn is_simple_go_work_member(member: &str) -> bool {
    !member.is_empty()
        && !member.contains('"')
        && !member.contains(' ')
        && (member == "." || member.starts_with("./") || member.starts_with("../"))
}

fn first_go_test_file(root: &Path) -> Option<PathBuf> {
    first_matching_file(root, root, &|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_test.go"))
    })
}

fn first_matching_file(
    root: &Path,
    current: &Path,
    predicate: &impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let mut entries = fs::read_dir(current).ok()?.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if path.is_dir() {
            if is_ignored_dir_name(&file_name) {
                continue;
            }
            if let Some(found) = first_matching_file(root, &path, predicate) {
                return Some(found);
            }
        } else if predicate(&path) {
            return path
                .strip_prefix(root)
                .ok()
                .map(Path::to_path_buf)
                .or(Some(path));
        }
    }

    None
}

fn is_ignored_dir_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | "build" | ".cache" | ".venv" | "__pycache__"
    )
}

fn cargo_lib_patterns(root: &Path, manifest: &TomlValue) -> Vec<String> {
    if let Some(path) = manifest
        .get("lib")
        .and_then(|lib| lib.get("path"))
        .and_then(TomlValue::as_str)
    {
        return vec![path.to_string()];
    }

    if root.join("src/lib.rs").exists() {
        vec!["src/lib.rs".to_string()]
    } else {
        Vec::new()
    }
}

fn cargo_bin_patterns(root: &Path, bin: &TomlValue) -> Vec<String> {
    if let Some(path) = bin.get("path").and_then(TomlValue::as_str) {
        return vec![path.to_string()];
    }

    if root.join("src/main.rs").exists() {
        vec!["src/main.rs".to_string()]
    } else {
        Vec::new()
    }
}

fn cargo_dependencies(manifest: &TomlValue) -> Vec<CargoDependency> {
    let mut dependencies = Vec::new();
    for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(section) = manifest.get(section_name).and_then(TomlValue::as_table) else {
            continue;
        };

        for (name, value) in section {
            dependencies.push(CargoDependency {
                name: name.to_string(),
                field: format!("{section_name}.{name}"),
                path_dependency: value
                    .as_table()
                    .is_some_and(|dependency| dependency.contains_key("path")),
            });
        }
    }

    dependencies.sort_by(|a, b| a.field.cmp(&b.field));
    dependencies
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

fn stable_id(prefix: &str, value: &str) -> String {
    format!("{prefix}-{}", sanitize_id(value))
}

fn stable_relationship_id(kind: &RelationshipKind, src_id: &str, dst_id: &str) -> String {
    format!(
        "relationship-{}-{}-{}",
        sanitize_id(&format!("{kind:?}")),
        sanitize_id(src_id),
        sanitize_id(dst_id)
    )
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn manifest_warning_category(message: &str) -> DetectionCategory {
    if message.starts_with("Failed to read") {
        DetectionCategory::UnreadableManifest
    } else {
        DetectionCategory::MalformedManifest
    }
}

fn read_toml(path: &Path) -> Result<TomlValue, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {error}", normalize_path(path)))?
        .parse::<TomlValue>()
        .map_err(|error| format!("Failed to parse {}: {error}", normalize_path(path)))
}

fn read_json(path: &Path) -> Result<JsonValue, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {error}", normalize_path(path)))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Failed to parse {}: {error}", normalize_path(path)))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_path(path: &Path) -> String {
    normalize_path(path)
}
