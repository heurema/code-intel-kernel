pub mod adapters;
pub mod core;
pub mod storage;

pub use crate::core::evaluation::{
    evaluate_cases, load_eval_cases, run_fixture_evaluation, EvalCase, EvalCaseKind,
    EvalCaseResult, EvalExpect, EvalFailure, EvalMetrics, EvaluationReport, ExpectedSymbol,
    EVAL_CONTRACT_VERSION,
};
pub use crate::core::evidence::{
    create_evidence_bundle, EvidenceBundle, EvidenceCommand, EvidenceFile, EvidenceRequest,
    KernelProfile,
};
pub use crate::core::memory::{create_session_event, AgentEvent, AgentEventType};
pub use crate::core::process_reward::{score_process_reward, ProcessReward, ProcessRewardInput};
pub use crate::core::repo_graph::{
    analyze_impact, inspect_repo, Component, DetectedFile, DetectedFileKind, DetectionCategory,
    DetectionIssue, DetectionSeverity, Evidence, ImpactConfidence, ImpactKind, ImpactReport,
    ImpactScope, ImpactStatus, ImpactedComponent, ImpactedWorkspace, PackageManager,
    PackageManagerKind, RecommendedCommand, RecommendedTest, Relationship, RelationshipKind,
    RepoCommand, RepoCommandKind, RepoInfo, RepoInspection, TestTarget, Workspace,
    IMPACT_CONTRACT_VERSION, INSPECT_CONTRACT_VERSION,
};
pub use crate::core::source_evidence::{
    build_source_evidence_bundle, build_source_evidence_bundle_from_graphs,
    source_evidence_bundle_evidence_valid, BundleConfidence, BundleStatus, BundleWarning,
    BundleWarningCategory, CandidateFile, CandidateSymbol, MissingEvidence,
    MissingEvidenceCategory, RepoContextItem, RepoContextKind, RepoContextRole,
    SourceEvidenceBundle, SOURCE_EVIDENCE_CONTRACT_VERSION,
};
pub use crate::core::symbol_graph::{
    build_symbol_graph, symbol_graph_evidence_valid, ParseStatus, SourceFile, SourceLanguage,
    SourceRange, SourceSymbol, SymbolGraph, SymbolKind, SymbolWarning, SymbolWarningCategory,
    SYMBOLS_CONTRACT_VERSION,
};
pub use crate::storage::sqlite::{open_kernel_database, KernelDatabase};
