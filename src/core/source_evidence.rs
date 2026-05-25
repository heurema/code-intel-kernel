use crate::core::repo_graph::{
    analyze_impact, inspect_repo, Component, DetectionSeverity, Evidence, ImpactKind, RepoCommand,
    RepoCommandKind, RepoInspection, TestTarget, Workspace,
};
use crate::core::symbol_graph::{
    build_symbol_graph, ParseStatus, SourceFile, SourceLanguage, SourceRange, SourceSymbol,
    SymbolGraph, SymbolKind, SymbolWarningCategory,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const SOURCE_EVIDENCE_CONTRACT_VERSION: &str = "0.3";
const MAX_CANDIDATE_FILES: usize = 8;
const MAX_CANDIDATE_SYMBOLS: usize = 12;
const MAX_REPO_CONTEXT_ITEMS: usize = 12;
const MAX_SOURCE_CONTEXT_SELECTORS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEvidenceBundle {
    pub contract_version: String,
    pub status: BundleStatus,
    pub query: String,
    pub confidence: BundleConfidence,
    pub candidate_files: Vec<CandidateFile>,
    pub candidate_symbols: Vec<CandidateSymbol>,
    pub source_context_selectors: Vec<SourceContextSelectorHint>,
    pub repo_context: Vec<RepoContextItem>,
    pub source_evidence: Vec<Evidence>,
    pub warnings: Vec<BundleWarning>,
    pub limitations: Vec<String>,
    pub missing_evidence: Vec<MissingEvidence>,
    pub refusal_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleStatus {
    Ok,
    Partial,
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleConfidence {
    High,
    Medium,
    Low,
    Insufficient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFile {
    pub path: String,
    pub language: SourceLanguage,
    pub parse_status: ParseStatus,
    pub confidence: BundleConfidence,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateSymbol {
    pub symbol_id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub path: String,
    pub range: SourceRange,
    pub confidence: BundleConfidence,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContextSelectorHint {
    pub selector_id: String,
    pub selector_kind: SourceContextSelectorKind,
    pub file_path: String,
    pub symbol_id: Option<String>,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<SymbolKind>,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub reason: String,
    pub confidence: BundleConfidence,
    pub evidence_ids: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceContextSelectorKind {
    File,
    FileRange,
    SymbolId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoContextItem {
    pub id: String,
    pub kind: RepoContextKind,
    pub role: RepoContextRole,
    pub label: String,
    pub path: Option<String>,
    pub reason: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoContextKind {
    Component,
    Command,
    Workspace,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoContextRole {
    AmbiguousContext,
    ContainingComponent,
    ContainingWorkspace,
    DependencyContext,
    ImpactContext,
    TestCommandContext,
    VerificationCommandContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleWarning {
    pub id: String,
    pub severity: DetectionSeverity,
    pub category: BundleWarningCategory,
    pub message: String,
    pub path: Option<String>,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleWarningCategory {
    AmbiguousQuery,
    CandidateLimitExceeded,
    InsufficientEvidenceForLocalization,
    LocalizationNotSupported,
    MultipleCandidates,
    NoRepoComponentContext,
    NoMatchingSourceFiles,
    NoMatchingSourceSymbols,
    ParseErrorPresent,
    QueryTooBroad,
    RepoGraphContextUnavailable,
    SelectorHintLimitExceeded,
    SymbolGraphParseWarning,
    UnsupportedLanguage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingEvidence {
    pub id: String,
    pub category: MissingEvidenceCategory,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingEvidenceCategory {
    AmbiguousSourceMatch,
    CandidateLimitExceeded,
    LocalizationNotSupported,
    NoCallGraph,
    NoLspDiagnostics,
    NoRepoComponentContext,
    NoSourceMatch,
    NoSymbolReferenceLayer,
    ParseErrorPresent,
    QueryRelevance,
    QueryTooBroad,
    UnsupportedLanguage,
}

pub fn build_source_evidence_bundle(
    repo_path: impl AsRef<Path>,
    query: impl AsRef<str>,
) -> SourceEvidenceBundle {
    let repo_graph = inspect_repo(repo_path.as_ref());
    let symbol_graph = build_symbol_graph(repo_path);
    build_source_evidence_bundle_from_graphs(&repo_graph, &symbol_graph, query)
}

pub fn build_source_evidence_bundle_from_graphs(
    repo_graph: &RepoInspection,
    symbol_graph: &SymbolGraph,
    query: impl AsRef<str>,
) -> SourceEvidenceBundle {
    let query = query.as_ref().trim().to_string();
    let query_terms = tokenize(&query);

    let mut candidate_symbols = symbol_graph
        .symbols
        .iter()
        .filter_map(|symbol| candidate_symbol(symbol, &query, &query_terms))
        .collect::<Vec<_>>();
    candidate_symbols.sort_by(compare_candidate_symbols);
    candidate_symbols.dedup_by(|left, right| left.symbol_id == right.symbol_id);

    let mut candidate_files = symbol_graph
        .source_files
        .iter()
        .filter_map(|source_file| candidate_file(source_file, &query, &query_terms))
        .collect::<Vec<_>>();

    for symbol in &candidate_symbols {
        if let Some(source_file) = symbol_graph
            .source_files
            .iter()
            .find(|source_file| source_file.path == symbol.path)
        {
            candidate_files.push(CandidateFile {
                path: source_file.path.clone(),
                language: source_file.language.clone(),
                parse_status: source_file.parse_status.clone(),
                confidence: symbol.confidence.clone(),
                reason: "contains_candidate_symbol".to_string(),
                evidence_ids: source_file.evidence_ids.clone(),
            });
        }
    }

    candidate_files.sort_by(compare_candidate_files);
    candidate_files.dedup_by(|left, right| left.path == right.path);

    let mut limits = CandidateLimitState {
        files_before: candidate_files.len(),
        symbols_before: candidate_symbols.len(),
        ..CandidateLimitState::default()
    };
    candidate_files.truncate(MAX_CANDIDATE_FILES);
    candidate_symbols.truncate(MAX_CANDIDATE_SYMBOLS);

    let mut source_context_selectors =
        source_context_selectors(&candidate_files, &candidate_symbols);
    limits.selectors_before = source_context_selectors.len();
    source_context_selectors.truncate(MAX_SOURCE_CONTEXT_SELECTORS);

    let mut repo_context = repo_context(repo_graph, &candidate_files, &candidate_symbols);
    repo_context.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.role.cmp(&right.role))
            .then_with(|| left.id.cmp(&right.id))
    });
    repo_context.dedup_by(|left, right| {
        left.id == right.id && left.kind == right.kind && left.role == right.role
    });
    limits.repo_context_before = repo_context.len();
    repo_context.truncate(MAX_REPO_CONTEXT_ITEMS);

    let mut source_evidence = collect_evidence(repo_graph, symbol_graph);
    source_evidence.sort_by(|left, right| left.id.cmp(&right.id));

    let mut warnings = bundle_warnings(
        symbol_graph,
        &query,
        &query_terms,
        &candidate_files,
        &candidate_symbols,
        &repo_context,
        limits,
    );
    warnings.sort_by(|left, right| left.id.cmp(&right.id));

    let mut missing_evidence = missing_evidence(
        symbol_graph,
        &query_terms,
        &candidate_files,
        &candidate_symbols,
        &repo_context,
        limits,
    );
    missing_evidence.sort_by(|left, right| left.id.cmp(&right.id));

    let confidence = bundle_confidence(&candidate_files, &candidate_symbols, limits);
    let status = if candidate_files.is_empty() && candidate_symbols.is_empty() {
        BundleStatus::InsufficientEvidence
    } else {
        BundleStatus::Partial
    };
    let refusal_reason = Some(refusal_reason(
        &status,
        &candidate_files,
        &candidate_symbols,
        limits,
    ));

    SourceEvidenceBundle {
        contract_version: SOURCE_EVIDENCE_CONTRACT_VERSION.to_string(),
        status,
        query,
        confidence,
        candidate_files,
        candidate_symbols,
        source_context_selectors,
        repo_context,
        source_evidence,
        warnings,
        limitations: vec![
            "SourceEvidenceBundle does not perform edit localization.".to_string(),
            "SymbolGraph-lite extracts top-level Rust declarations only.".to_string(),
            "No references, call graph, or import/export semantic resolution.".to_string(),
            "No LSP diagnostics.".to_string(),
            "No runtime behavior or test coverage inference.".to_string(),
        ],
        missing_evidence,
        refusal_reason,
    }
}

pub fn source_evidence_bundle_evidence_valid(bundle: &SourceEvidenceBundle) -> bool {
    let evidence_ids = bundle
        .source_evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<BTreeSet<_>>();

    bundle.candidate_files.iter().all(|candidate| {
        !candidate.evidence_ids.is_empty()
            && candidate
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && bundle.candidate_symbols.iter().all(|candidate| {
        !candidate.evidence_ids.is_empty()
            && candidate
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && bundle.source_context_selectors.iter().all(|selector| {
        !selector.evidence_ids.is_empty()
            && selector
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && bundle.repo_context.iter().all(|context| {
        !context.evidence_ids.is_empty()
            && context
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && bundle.warnings.iter().all(|warning| {
        warning
            .evidence_id
            .as_deref()
            .is_none_or(|id| evidence_ids.contains(id))
    })
}

#[derive(Debug, Clone, Copy, Default)]
struct CandidateLimitState {
    files_before: usize,
    symbols_before: usize,
    repo_context_before: usize,
    selectors_before: usize,
}

impl CandidateLimitState {
    fn any_exceeded(self) -> bool {
        self.files_before > MAX_CANDIDATE_FILES
            || self.symbols_before > MAX_CANDIDATE_SYMBOLS
            || self.repo_context_before > MAX_REPO_CONTEXT_ITEMS
    }

    fn selectors_exceeded(self) -> bool {
        self.selectors_before > MAX_SOURCE_CONTEXT_SELECTORS
    }
}

fn candidate_symbol(
    symbol: &SourceSymbol,
    query: &str,
    query_terms: &BTreeSet<String>,
) -> Option<CandidateSymbol> {
    let (confidence, reason) = match symbol_match_quality(&symbol.name, query, query_terms) {
        Some(match_quality) => match_quality,
        None => file_match_quality(&symbol.path, query, query_terms)?,
    };

    Some(CandidateSymbol {
        symbol_id: symbol.id.clone(),
        kind: symbol.kind.clone(),
        name: symbol.name.clone(),
        path: symbol.path.clone(),
        range: symbol.range.clone(),
        confidence,
        reason,
        evidence_ids: symbol.evidence_ids.clone(),
    })
}

fn candidate_file(
    source_file: &SourceFile,
    query: &str,
    query_terms: &BTreeSet<String>,
) -> Option<CandidateFile> {
    let (confidence, reason) = file_match_quality(&source_file.path, query, query_terms)?;
    Some(CandidateFile {
        path: source_file.path.clone(),
        language: source_file.language.clone(),
        parse_status: source_file.parse_status.clone(),
        confidence,
        reason,
        evidence_ids: source_file.evidence_ids.clone(),
    })
}

fn source_context_selectors(
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
) -> Vec<SourceContextSelectorHint> {
    let mut selectors = Vec::new();

    for symbol in candidate_symbols {
        if symbol.evidence_ids.is_empty() {
            continue;
        }
        selectors.push(SourceContextSelectorHint {
            selector_id: stable_id(
                "source-context-selector",
                &format!("symbol-id-{}-{}", symbol.path, symbol.symbol_id),
            ),
            selector_kind: SourceContextSelectorKind::SymbolId,
            file_path: symbol.path.clone(),
            symbol_id: Some(symbol.symbol_id.clone()),
            symbol_name: Some(symbol.name.clone()),
            symbol_kind: Some(symbol.kind.clone()),
            start_line: Some(symbol.range.start_line),
            end_line: Some(symbol.range.end_line),
            reason: selector_reason_for_symbol(symbol),
            confidence: symbol.confidence.clone(),
            evidence_ids: symbol.evidence_ids.clone(),
            limitations: selector_limitations(),
        });
    }

    for file in candidate_files {
        if file.evidence_ids.is_empty() {
            continue;
        }
        selectors.push(SourceContextSelectorHint {
            selector_id: stable_id("source-context-selector", &format!("file-{}", file.path)),
            selector_kind: SourceContextSelectorKind::File,
            file_path: file.path.clone(),
            symbol_id: None,
            symbol_name: None,
            symbol_kind: None,
            start_line: None,
            end_line: None,
            reason: selector_reason_for_file(file),
            confidence: file.confidence.clone(),
            evidence_ids: file.evidence_ids.clone(),
            limitations: selector_limitations(),
        });
    }

    selectors.sort_by(compare_source_context_selectors);
    selectors.dedup_by(|left, right| left.selector_id == right.selector_id);
    selectors
}

fn selector_reason_for_symbol(symbol: &CandidateSymbol) -> String {
    match symbol.reason.as_str() {
        "exact_symbol_match" => "exact_symbol_match",
        "symbol_substring_match" => "matched_candidate_symbol",
        "query_token_overlap" => "token_overlap_candidate",
        _ => "matched_candidate_symbol",
    }
    .to_string()
}

fn selector_reason_for_file(file: &CandidateFile) -> String {
    match file.reason.as_str() {
        "exact_file_match" => "exact_file_match",
        "path_substring_match" => "matched_candidate_file",
        "query_token_overlap" => "token_overlap_candidate",
        "contains_candidate_symbol" => "matched_candidate_symbol",
        _ => "matched_candidate_file",
    }
    .to_string()
}

fn selector_limitations() -> Vec<String> {
    vec![
        "Selector is for read-only SourceContext retrieval only.".to_string(),
        "Selector does not include source text.".to_string(),
        "Selector does not imply correctness or ownership.".to_string(),
    ]
}

fn symbol_match_quality(
    value: &str,
    query: &str,
    query_terms: &BTreeSet<String>,
) -> Option<(BundleConfidence, String)> {
    if query.is_empty() {
        return None;
    }

    let normalized_value = value.to_ascii_lowercase();
    let normalized_query = query.to_ascii_lowercase();
    if normalized_value == normalized_query {
        return Some((BundleConfidence::High, "exact_symbol_match".to_string()));
    }
    if normalized_value.contains(&normalized_query) {
        return Some((
            BundleConfidence::Medium,
            "symbol_substring_match".to_string(),
        ));
    }

    let value_terms = tokenize(value);
    let overlap = query_terms
        .iter()
        .filter(|term| value_terms.contains(*term))
        .count();
    if overlap > 0 {
        return Some((BundleConfidence::Low, "query_token_overlap".to_string()));
    }

    None
}

fn file_match_quality(
    value: &str,
    query: &str,
    query_terms: &BTreeSet<String>,
) -> Option<(BundleConfidence, String)> {
    if query.is_empty() {
        return None;
    }

    let normalized_value = value.to_ascii_lowercase();
    let normalized_query = query.to_ascii_lowercase();
    if normalized_value == normalized_query {
        return Some((BundleConfidence::High, "exact_file_match".to_string()));
    }
    if normalized_value.contains(&normalized_query) {
        return Some((BundleConfidence::Medium, "path_substring_match".to_string()));
    }

    let value_terms = tokenize(value);
    let overlap = query_terms
        .iter()
        .filter(|term| value_terms.contains(*term))
        .count();
    if overlap > 0 {
        return Some((BundleConfidence::Low, "query_token_overlap".to_string()));
    }

    None
}

fn repo_context(
    repo_graph: &RepoInspection,
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
) -> Vec<RepoContextItem> {
    let candidate_paths = candidate_files
        .iter()
        .map(|file| file.path.as_str())
        .chain(candidate_symbols.iter().map(|symbol| symbol.path.as_str()))
        .collect::<BTreeSet<_>>();
    if candidate_paths.is_empty() {
        return Vec::new();
    }

    let mut context = Vec::new();
    let matched_components = repo_graph
        .components
        .iter()
        .filter(|component| {
            candidate_paths
                .iter()
                .any(|path| component_matches_path(component, path))
        })
        .collect::<Vec<_>>();

    for component in &matched_components {
        context.push(component_context(
            component,
            RepoContextRole::ContainingComponent,
            "component_scope_matches_candidate_source_path",
        ));
    }

    for workspace in &repo_graph.workspaces {
        if candidate_paths
            .iter()
            .any(|path| workspace_matches_path(workspace, path))
        {
            context.push(workspace_context(workspace));
        }
    }

    if !matched_components.is_empty() {
        for command in &repo_graph.commands {
            context.push(command_context(command));
        }
        for test in &repo_graph.tests {
            context.push(test_context(test));
        }
    }

    let changed_files = candidate_paths
        .iter()
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();
    let impact = analyze_impact(repo_graph, changed_files);
    for impacted in impact
        .impacted_components
        .iter()
        .filter(|component| component.impact_kind != ImpactKind::Uncertain)
    {
        context.push(RepoContextItem {
            id: format!("impact-context-{}", impacted.component_id),
            kind: RepoContextKind::Component,
            role: RepoContextRole::ImpactContext,
            label: impacted.name.clone(),
            path: Some(impacted.path.clone()),
            reason: format!(
                "repo_graph_impact_{}",
                impact_kind_name(&impacted.impact_kind)
            ),
            evidence_ids: impacted.evidence_ids.clone(),
        });
    }

    context
}

fn component_context(
    component: &Component,
    role: RepoContextRole,
    reason: &str,
) -> RepoContextItem {
    RepoContextItem {
        id: component.id.clone(),
        kind: RepoContextKind::Component,
        role,
        label: component.name.clone(),
        path: Some(component.path.clone()),
        reason: reason.to_string(),
        evidence_ids: vec![component.evidence_id.clone()],
    }
}

fn workspace_context(workspace: &Workspace) -> RepoContextItem {
    RepoContextItem {
        id: workspace.id.clone(),
        kind: RepoContextKind::Workspace,
        role: RepoContextRole::ContainingWorkspace,
        label: workspace.name.clone(),
        path: None,
        reason: "workspace_member_path_contains_candidate_source_path".to_string(),
        evidence_ids: vec![workspace.evidence_id.clone()],
    }
}

fn command_context(command: &RepoCommand) -> RepoContextItem {
    RepoContextItem {
        id: command.id.clone(),
        kind: RepoContextKind::Command,
        role: match &command.kind {
            RepoCommandKind::Test => RepoContextRole::TestCommandContext,
            _ => RepoContextRole::VerificationCommandContext,
        },
        label: command.command.clone(),
        path: Some(command.scope.clone()),
        reason: "repo_command_context_for_candidate_source".to_string(),
        evidence_ids: vec![command.evidence_id.clone()],
    }
}

fn test_context(test: &TestTarget) -> RepoContextItem {
    RepoContextItem {
        id: test.id.clone(),
        kind: RepoContextKind::Test,
        role: RepoContextRole::TestCommandContext,
        label: test.command.clone(),
        path: Some(test.scope.clone()),
        reason: "repo_test_context_for_candidate_source".to_string(),
        evidence_ids: vec![test.evidence_id.clone()],
    }
}

fn workspace_matches_path(workspace: &Workspace, candidate_path: &str) -> bool {
    workspace.members.iter().any(|member| {
        let normalized = member.trim_start_matches("./");
        candidate_path.starts_with(normalized)
    })
}

fn impact_kind_name(kind: &ImpactKind) -> &'static str {
    match kind {
        ImpactKind::Direct => "direct",
        ImpactKind::Transitive => "transitive",
        ImpactKind::Broad => "broad",
        ImpactKind::Uncertain => "uncertain",
    }
}

fn component_matches_path(component: &Component, candidate_path: &str) -> bool {
    component
        .file_patterns
        .iter()
        .any(|pattern| pattern_matches_path(pattern, candidate_path))
        || component.path != "." && candidate_path.starts_with(&component.path)
}

fn pattern_matches_path(pattern: &str, candidate_path: &str) -> bool {
    if pattern == candidate_path {
        return true;
    }

    if let Some(prefix) = pattern.strip_suffix("/**") {
        return candidate_path.starts_with(prefix);
    }

    false
}

fn collect_evidence(repo_graph: &RepoInspection, symbol_graph: &SymbolGraph) -> Vec<Evidence> {
    let mut evidence = BTreeMap::new();
    for item in repo_graph
        .evidence
        .iter()
        .chain(symbol_graph.evidence.iter())
    {
        evidence.insert(item.id.clone(), item.clone());
    }
    evidence.into_values().collect()
}

fn bundle_warnings(
    symbol_graph: &SymbolGraph,
    query: &str,
    query_terms: &BTreeSet<String>,
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
    repo_context: &[RepoContextItem],
    limits: CandidateLimitState,
) -> Vec<BundleWarning> {
    let mut warnings = Vec::new();
    if query.trim().is_empty() {
        warnings.push(bundle_warning(
            BundleWarningCategory::AmbiguousQuery,
            DetectionSeverity::Warning,
            "Query is empty; no source evidence can be matched.",
            None,
            None,
        ));
    }
    if candidate_files.is_empty() {
        warnings.push(bundle_warning(
            BundleWarningCategory::NoMatchingSourceFiles,
            DetectionSeverity::Warning,
            "No source files matched the query.",
            None,
            None,
        ));
    }
    if candidate_symbols.is_empty() {
        warnings.push(bundle_warning(
            BundleWarningCategory::NoMatchingSourceSymbols,
            DetectionSeverity::Warning,
            "No source symbols matched the query.",
            None,
            None,
        ));
    }
    if candidate_files.len() + candidate_symbols.len() > 1 {
        warnings.push(bundle_warning(
            BundleWarningCategory::MultipleCandidates,
            DetectionSeverity::Info,
            "Multiple evidence candidates matched without disambiguating reference evidence.",
            None,
            None,
        ));
    }
    if query_terms.len() <= 1 && candidate_files.len() + candidate_symbols.len() > 1 {
        warnings.push(bundle_warning(
            BundleWarningCategory::QueryTooBroad,
            DetectionSeverity::Info,
            "Query is broad and matched multiple candidates.",
            None,
            None,
        ));
    }
    if limits.any_exceeded() {
        warnings.push(bundle_warning(
            BundleWarningCategory::CandidateLimitExceeded,
            DetectionSeverity::Warning,
            "Candidate output was deterministically truncated; provide a narrower query.",
            None,
            None,
        ));
    }
    if limits.selectors_exceeded() {
        warnings.push(bundle_warning(
            BundleWarningCategory::SelectorHintLimitExceeded,
            DetectionSeverity::Warning,
            "SourceContext selector hints were deterministically truncated; provide a narrower query.",
            None,
            None,
        ));
    }
    if repo_context.is_empty() && (!candidate_files.is_empty() || !candidate_symbols.is_empty()) {
        warnings.push(bundle_warning(
            BundleWarningCategory::NoRepoComponentContext,
            DetectionSeverity::Warning,
            "No RepoGraph component context could be attached to source candidates.",
            None,
            None,
        ));
    }
    warnings.push(bundle_warning(
        BundleWarningCategory::LocalizationNotSupported,
        DetectionSeverity::Info,
        "SourceEvidenceBundle is not an edit localization result.",
        None,
        None,
    ));

    for warning in &symbol_graph.warnings {
        if warning.category == SymbolWarningCategory::ParseError {
            warnings.push(bundle_warning(
                BundleWarningCategory::ParseErrorPresent,
                warning.severity.clone(),
                "SymbolGraph-lite reported a parse warning.",
                warning.path.as_deref(),
                warning.evidence_id.as_deref(),
            ));
        }
    }

    warnings
}

fn bundle_warning(
    category: BundleWarningCategory,
    severity: DetectionSeverity,
    message: &str,
    path: Option<&str>,
    evidence_id: Option<&str>,
) -> BundleWarning {
    BundleWarning {
        id: stable_id(
            "bundle-warning",
            &format!("{:?}-{}-{}", category, path.unwrap_or("repo"), message),
        ),
        severity,
        category,
        message: message.to_string(),
        path: path.map(str::to_string),
        evidence_id: evidence_id.map(str::to_string),
    }
}

fn missing_evidence(
    symbol_graph: &SymbolGraph,
    query_terms: &BTreeSet<String>,
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
    repo_context: &[RepoContextItem],
    limits: CandidateLimitState,
) -> Vec<MissingEvidence> {
    let mut missing = vec![
        missing_item(
            MissingEvidenceCategory::NoSymbolReferenceLayer,
            "No symbol reference layer is available.",
        ),
        missing_item(
            MissingEvidenceCategory::NoCallGraph,
            "No call graph is available.",
        ),
        missing_item(
            MissingEvidenceCategory::NoLspDiagnostics,
            "No LSP diagnostics or language-server facts are available.",
        ),
        missing_item(
            MissingEvidenceCategory::LocalizationNotSupported,
            "No evaluated localization layer is available.",
        ),
        missing_item(
            MissingEvidenceCategory::QueryRelevance,
            "Query matching uses deterministic string/token overlap only.",
        ),
    ];

    if candidate_files.is_empty() && candidate_symbols.is_empty() {
        missing.push(missing_item(
            MissingEvidenceCategory::NoSourceMatch,
            "No source file or symbol matched the query.",
        ));
    }

    if repo_context.is_empty() {
        missing.push(missing_item(
            MissingEvidenceCategory::NoRepoComponentContext,
            "No RepoGraph context could be attached to source candidates.",
        ));
    }

    if candidate_files.len() + candidate_symbols.len() > 1 {
        missing.push(missing_item(
            MissingEvidenceCategory::AmbiguousSourceMatch,
            "Multiple source candidates matched without reference-level disambiguation.",
        ));
    }

    if query_terms.len() <= 1 && candidate_files.len() + candidate_symbols.len() > 1 {
        missing.push(missing_item(
            MissingEvidenceCategory::QueryTooBroad,
            "The query is broad and needs more disambiguating terms.",
        ));
    }

    if limits.any_exceeded() {
        missing.push(missing_item(
            MissingEvidenceCategory::CandidateLimitExceeded,
            "Candidate limits were exceeded and output was truncated.",
        ));
    }

    if symbol_graph
        .warnings
        .iter()
        .any(|warning| warning.category == SymbolWarningCategory::ParseError)
    {
        missing.push(missing_item(
            MissingEvidenceCategory::ParseErrorPresent,
            "At least one Rust source file has parse errors.",
        ));
    }

    missing
}

fn missing_item(category: MissingEvidenceCategory, message: &str) -> MissingEvidence {
    MissingEvidence {
        id: stable_id("missing-evidence", &format!("{:?}-{message}", category)),
        category,
        message: message.to_string(),
    }
}

fn bundle_confidence(
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
    limits: CandidateLimitState,
) -> BundleConfidence {
    let confidence = candidate_files
        .iter()
        .map(|candidate| candidate.confidence.clone())
        .chain(
            candidate_symbols
                .iter()
                .map(|candidate| candidate.confidence.clone()),
        )
        .min()
        .unwrap_or(BundleConfidence::Insufficient);

    if limits.any_exceeded() && confidence < BundleConfidence::Low {
        BundleConfidence::Low
    } else {
        confidence
    }
}

fn refusal_reason(
    status: &BundleStatus,
    candidate_files: &[CandidateFile],
    candidate_symbols: &[CandidateSymbol],
    limits: CandidateLimitState,
) -> String {
    if *status == BundleStatus::InsufficientEvidence {
        return "no_source_match: no evidence-backed source candidates matched the query."
            .to_string();
    }

    if limits.any_exceeded() {
        return "candidate_limit_exceeded: evidence candidates were truncated; localization_not_supported."
            .to_string();
    }

    if candidate_files.len() + candidate_symbols.len() > 1 {
        return "ambiguous_source_match: multiple evidence candidates matched; localization_not_supported."
            .to_string();
    }

    "localization_not_supported: SourceEvidenceBundle is evidence assembly only.".to_string()
}

fn compare_candidate_files(left: &CandidateFile, right: &CandidateFile) -> std::cmp::Ordering {
    confidence_rank(&left.confidence)
        .cmp(&confidence_rank(&right.confidence))
        .then_with(|| left.path.cmp(&right.path))
}

fn compare_candidate_symbols(
    left: &CandidateSymbol,
    right: &CandidateSymbol,
) -> std::cmp::Ordering {
    confidence_rank(&left.confidence)
        .cmp(&confidence_rank(&right.confidence))
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.symbol_id.cmp(&right.symbol_id))
}

fn compare_source_context_selectors(
    left: &SourceContextSelectorHint,
    right: &SourceContextSelectorHint,
) -> std::cmp::Ordering {
    selector_kind_rank(&left.selector_kind)
        .cmp(&selector_kind_rank(&right.selector_kind))
        .then_with(|| confidence_rank(&left.confidence).cmp(&confidence_rank(&right.confidence)))
        .then_with(|| left.file_path.cmp(&right.file_path))
        .then_with(|| left.symbol_id.cmp(&right.symbol_id))
        .then_with(|| left.selector_id.cmp(&right.selector_id))
}

fn selector_kind_rank(kind: &SourceContextSelectorKind) -> u8 {
    match kind {
        SourceContextSelectorKind::SymbolId => 0,
        SourceContextSelectorKind::FileRange => 1,
        SourceContextSelectorKind::File => 2,
    }
}

fn confidence_rank(confidence: &BundleConfidence) -> u8 {
    match confidence {
        BundleConfidence::High => 0,
        BundleConfidence::Medium => 1,
        BundleConfidence::Low => 2,
        BundleConfidence::Insufficient => 3,
    }
}

fn tokenize(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn stable_id(prefix: &str, value: &str) -> String {
    let suffix = value
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
        .join("-");
    if suffix.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}-{suffix}")
    }
}
