use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

pub const INSPECT_CONTRACT_VERSION: &str = "0.2";
pub const IMPACT_CONTRACT_VERSION: &str = "0.1";

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
    pub changed_files: Vec<String>,
    pub impacted_components: Vec<Component>,
    pub impacted_workspaces: Vec<Workspace>,
    pub recommended_commands: Vec<RepoCommand>,
    pub recommended_tests: Vec<TestTarget>,
    pub evidence: Vec<Evidence>,
    pub warnings: Vec<DetectionIssue>,
    pub limitations: Vec<String>,
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
        .collect::<Vec<_>>();

    let mut impacted_components = Vec::<Component>::new();
    let mut impacted_workspaces = Vec::<Workspace>::new();
    let mut recommended_commands = Vec::<RepoCommand>::new();
    let mut recommended_tests = Vec::<TestTarget>::new();
    let mut warnings = repo_graph.warnings.clone();
    warnings.push(DetectionIssue {
        id: "impact-warning-1".to_string(),
        severity: DetectionSeverity::Info,
        category: DetectionCategory::RepoGraphOnly,
        message: "SymbolGraph is not implemented; impact is based on repository structure and paths only.".to_string(),
        path: None,
        evidence_id: None,
    });

    for changed_file in &changed_files {
        if is_broad_repo_change(repo_graph, changed_file) {
            push_all_components(repo_graph, &mut impacted_components);
            push_all_workspaces(repo_graph, &mut impacted_workspaces);
            push_all_commands(repo_graph, &mut recommended_commands);
            push_all_tests(repo_graph, &mut recommended_tests);
            continue;
        }

        let matching_components = repo_graph
            .components
            .iter()
            .filter(|component| component_matches_changed_file(component, changed_file))
            .cloned()
            .collect::<Vec<_>>();

        if matching_components.is_empty() {
            warnings.push(DetectionIssue {
                id: format!("impact-warning-unmapped-{}", warnings.len() + 1),
                severity: DetectionSeverity::Warning,
                category: DetectionCategory::UnmappedChange,
                message: "Changed file could not be mapped to a RepoGraph component.".to_string(),
                path: Some(changed_file.clone()),
                evidence_id: None,
            });
            continue;
        }

        for component in matching_components {
            push_component(&mut impacted_components, component);
        }
    }

    if !impacted_components.is_empty() {
        for command in &repo_graph.commands {
            if command_applies_to_components(command, &impacted_components) {
                push_command(&mut recommended_commands, command.clone());
            }
        }

        for test in &repo_graph.tests {
            if test_applies_to_components(test, &impacted_components)
                || changed_files.iter().any(|file| is_test_path(file))
            {
                push_test(&mut recommended_tests, test.clone());
            }
        }
    }

    let status = if impacted_components.is_empty()
        && impacted_workspaces.is_empty()
        && recommended_commands.is_empty()
        && recommended_tests.is_empty()
    {
        ImpactStatus::InsufficientEvidence
    } else {
        ImpactStatus::Partial
    };

    ImpactReport {
        contract_version: IMPACT_CONTRACT_VERSION.to_string(),
        status,
        changed_files,
        impacted_components,
        impacted_workspaces,
        recommended_commands,
        recommended_tests,
        evidence: repo_graph.evidence.clone(),
        warnings,
        limitations: vec![
            "RepoGraph-only impact analysis; no symbols, imports, definitions, references, or call graph are used.".to_string(),
            "Recommendations are conservative and based on path containment, manifests, command scopes, and test scopes.".to_string(),
        ],
    }
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

                if let Some(name) = manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
                    .and_then(TomlValue::as_str)
                {
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
                            workspace_members,
                            evidence_id,
                        );
                    }
                }

                detect_cargo_targets(root, &manifest, builder);
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

fn detect_cargo_targets(root: &Path, manifest: &TomlValue, builder: &mut RepoGraphBuilder) {
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
    }

    if let Some(bin_targets) = manifest.get("bin").and_then(TomlValue::as_array) {
        for (index, bin) in bin_targets.iter().enumerate() {
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
    let mut pytest_config_evidence = None;

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

                if manifest
                    .get("tool")
                    .and_then(|tool| tool.get("pytest"))
                    .is_some()
                {
                    let evidence_id = builder.add_evidence(
                        Path::new("pyproject.toml"),
                        "test_config",
                        Some("tool.pytest"),
                        "pytest configuration detected.",
                    );
                    pytest_config_evidence = Some(evidence_id);
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
        builder.add_package_manager(PackageManagerKind::Pip, "pip", evidence_id);
    }

    if let Some(evidence_id) = pytest_config_evidence {
        add_pytest(builder, evidence_id);
    } else if python_project_detected && root.join("tests").is_dir() {
        let evidence_id = builder.add_detected_file(
            Path::new("tests"),
            DetectedFileKind::TestConfig,
            "directory",
            None,
            "tests directory detected.",
        );
        add_pytest(builder, evidence_id);
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
        let evidence_id = builder.add_detected_file(
            Path::new("go.mod"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Go module manifest detected.",
        );
        builder.add_package_manager(PackageManagerKind::Go, "go", evidence_id.clone());
        let module_name = read_go_module_name(&go_mod).unwrap_or_else(|| "go-module".to_string());
        if module_name == "go-module" {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::UnsupportedPattern,
                "go.mod was detected but no module declaration was parsed.",
                Some(Path::new("go.mod")),
                Some(evidence_id.clone()),
            );
        }
        builder.add_component(
            "component-go-module",
            &module_name,
            "go_module",
            ".",
            vec![
                "go.mod".to_string(),
                "go.work".to_string(),
                "*.go".to_string(),
            ],
            evidence_id.clone(),
        );
        builder.add_command(
            "cmd-go-test",
            RepoCommandKind::Test,
            "go test ./...",
            Some("component-go-module"),
            0.9,
            evidence_id.clone(),
        );
        builder.add_test(
            "test-go-test",
            "go test",
            "go test ./...",
            Some("component-go-module"),
            0.9,
            evidence_id,
        );
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
        builder.add_workspace(
            "workspace-go",
            "go-workspace",
            Vec::new(),
            evidence_id.clone(),
        );
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::PartialSupport,
            "go.work was detected but workspace members are not parsed in Phase 1B.",
            Some(Path::new("go.work")),
            Some(evidence_id),
        );
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
        builder.add_command(
            "cmd-make",
            RepoCommandKind::Other,
            "make",
            Some("component-make-project"),
            0.5,
            evidence_id.clone(),
        );

        let targets = read_simple_targets(&makefile);
        if targets.contains(&"test".to_string()) {
            builder.add_command(
                "cmd-make-test",
                RepoCommandKind::Test,
                "make test",
                Some("component-make-project"),
                0.75,
                evidence_id.clone(),
            );
            builder.add_test(
                "test-make-test",
                "make test",
                "make test",
                Some("component-make-project"),
                0.75,
                evidence_id.clone(),
            );
        } else {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "Makefile detected but no test target was parsed.",
                Some(Path::new("Makefile")),
                Some(evidence_id.clone()),
            );
        }

        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::PartialSupport,
            "Makefile target parsing is shallow in Phase 1B.",
            Some(Path::new("Makefile")),
            Some(evidence_id),
        );
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
        builder.add_command(
            "cmd-just",
            RepoCommandKind::Other,
            "just",
            Some("component-just-project"),
            0.5,
            evidence_id.clone(),
        );

        let targets = read_simple_targets(&justfile);
        if targets.contains(&"test".to_string()) {
            builder.add_command(
                "cmd-just-test",
                RepoCommandKind::Test,
                "just test",
                Some("component-just-project"),
                0.75,
                evidence_id.clone(),
            );
            builder.add_test(
                "test-just-test",
                "just test",
                "just test",
                Some("component-just-project"),
                0.75,
                evidence_id.clone(),
            );
        } else {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "justfile detected but no test recipe was parsed.",
                Some(Path::new("justfile")),
                Some(evidence_id.clone()),
            );
        }

        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::PartialSupport,
            "justfile recipe parsing is shallow in Phase 1B.",
            Some(Path::new("justfile")),
            Some(evidence_id),
        );
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
        "python -m pytest",
        Some("component-python-project"),
        0.75,
        evidence_id.clone(),
    );
    builder.add_test(
        "test-python-pytest",
        "pytest",
        "python -m pytest",
        Some("component-python-project"),
        0.75,
        evidence_id,
    );
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
    for ignored_path in [
        "node_modules",
        "target",
        "dist",
        "build",
        ".cache",
        ".venv",
        "coverage",
    ] {
        if root.join(ignored_path).exists() {
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

fn read_simple_targets(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    contents
        .lines()
        .filter_map(|line| {
            if line.starts_with(char::is_whitespace) || line.trim_start().starts_with('#') {
                return None;
            }

            let (target, _) = line.split_once(':')?;
            let target = target.trim();

            if target.is_empty() || target.contains(' ') || target.contains('=') {
                None
            } else {
                Some(target.to_string())
            }
        })
        .collect()
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

    false
}

fn command_applies_to_components(command: &RepoCommand, components: &[Component]) -> bool {
    match command.scope_ref.as_deref() {
        Some("repo") | None => true,
        Some(scope_ref) => components.iter().any(|component| component.id == scope_ref),
    }
}

fn test_applies_to_components(test: &TestTarget, components: &[Component]) -> bool {
    match test.scope_ref.as_deref() {
        Some("repo") | None => true,
        Some(scope_ref) => components.iter().any(|component| component.id == scope_ref),
    }
}

fn is_test_path(path: &str) -> bool {
    path.starts_with("tests/")
        || path.starts_with("test/")
        || path.ends_with("_test.go")
        || path.contains(".test.")
        || path.contains(".spec.")
}

fn push_all_components(repo_graph: &RepoInspection, target: &mut Vec<Component>) {
    for component in &repo_graph.components {
        push_component(target, component.clone());
    }
}

fn push_all_workspaces(repo_graph: &RepoInspection, target: &mut Vec<Workspace>) {
    for workspace in &repo_graph.workspaces {
        if !target.iter().any(|existing| existing.id == workspace.id) {
            target.push(workspace.clone());
        }
    }
}

fn push_all_commands(repo_graph: &RepoInspection, target: &mut Vec<RepoCommand>) {
    for command in &repo_graph.commands {
        push_command(target, command.clone());
    }
}

fn push_all_tests(repo_graph: &RepoInspection, target: &mut Vec<TestTarget>) {
    for test in &repo_graph.tests {
        push_test(target, test.clone());
    }
}

fn push_component(target: &mut Vec<Component>, component: Component) {
    if !target.iter().any(|existing| existing.id == component.id) {
        target.push(component);
    }
}

fn push_command(target: &mut Vec<RepoCommand>, command: RepoCommand) {
    if !target.iter().any(|existing| existing.id == command.id) {
        target.push(command);
    }
}

fn push_test(target: &mut Vec<TestTarget>, test: TestTarget) {
    if !target.iter().any(|existing| existing.id == test.id) {
        target.push(test);
    }
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
