use serde::{Deserialize, Serialize};

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
