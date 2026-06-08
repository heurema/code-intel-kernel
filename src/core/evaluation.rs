use crate::core::lsp_bridge::{
    collect_rust_lsp_diagnostics_with_options, lsp_diagnostics_evidence_valid,
    LspAvailabilityStatus, LspDiagnosticsOptions, LspDiagnosticsReport, LspWarning,
    LspWarningCategory,
};
use crate::core::repo_graph::{
    analyze_impact, inspect_repo, DetectionCategory, DetectionIssue, Evidence, ImpactConfidence,
    ImpactReport, ImpactScope, ImpactStatus, RepoInspection,
};
use crate::core::source_context::{
    build_source_context_report, source_context_evidence_valid, LineRange, SourceContextReport,
    SourceContextSelector, SourceContextSlice, SourceContextWarning, SourceContextWarningCategory,
};
use crate::core::source_evidence::{
    build_source_evidence_bundle, source_evidence_bundle_evidence_valid, BundleWarning,
    BundleWarningCategory, CandidateSymbol, RepoContextRole, SourceContextSelectorHint,
    SourceEvidenceBundle,
};
use crate::core::symbol_graph::{
    build_symbol_graph, symbol_graph_evidence_valid, SourceSymbol, SymbolGraph, SymbolKind,
    SymbolWarning, SymbolWarningCategory,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const EVAL_CONTRACT_VERSION: &str = "0.5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalCaseKind {
    Inspect,
    Impact,
    Symbols,
    SourceEvidence,
    SourceContext,
    LspDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCase {
    pub name: String,
    pub fixture: String,
    pub kind: EvalCaseKind,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub selector_file: String,
    #[serde(default)]
    pub selector_symbol_id: String,
    #[serde(default)]
    pub selector_lines: Option<String>,
    #[serde(default)]
    pub lsp_command: String,
    #[serde(default)]
    pub lsp_timeout_ms: Option<u64>,
    #[serde(default)]
    pub lsp_max_diagnostics: Option<usize>,
    #[serde(default)]
    pub changed_files: Vec<String>,
    #[serde(default)]
    pub expect: EvalExpect,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalExpect {
    pub status: Option<ImpactStatus>,
    pub confidence: Option<ImpactConfidence>,
    pub impact_scope: Option<ImpactScope>,
    #[serde(default)]
    pub components_contains: Vec<String>,
    #[serde(default)]
    pub commands_contains: Vec<String>,
    #[serde(default)]
    pub tests_contains: Vec<String>,
    #[serde(default)]
    pub warnings_contains_categories: Vec<String>,
    #[serde(default)]
    pub warnings_not_contains_categories: Vec<String>,
    #[serde(default)]
    pub forbidden_components_contains: Vec<String>,
    #[serde(default)]
    pub forbidden_commands_contains: Vec<String>,
    #[serde(default)]
    pub forbidden_tests_contains: Vec<String>,
    #[serde(default)]
    pub source_files_contains: Vec<String>,
    #[serde(default)]
    pub source_files_not_contains: Vec<String>,
    #[serde(default)]
    pub symbols_contains: Vec<ExpectedSymbol>,
    #[serde(default)]
    pub symbols_not_contains: Vec<ExpectedSymbol>,
    #[serde(default)]
    pub candidate_files_contains: Vec<String>,
    #[serde(default)]
    pub candidate_symbols_contains: Vec<ExpectedSymbol>,
    pub max_candidate_files: Option<usize>,
    pub max_candidate_symbols: Option<usize>,
    #[serde(default)]
    pub repo_context_contains: Vec<String>,
    #[serde(default)]
    pub repo_context_roles_contains: Vec<String>,
    #[serde(default)]
    pub missing_evidence_contains: Vec<String>,
    #[serde(default)]
    pub selector_hints_contains: Vec<String>,
    #[serde(default)]
    pub selector_hint_symbols_contains: Vec<ExpectedSymbol>,
    pub max_source_context_selectors: Option<usize>,
    #[serde(default)]
    pub lsp_status: Option<String>,
    #[serde(default)]
    pub slices_contains: Vec<String>,
    #[serde(default)]
    pub slice_symbols_contains: Vec<ExpectedSymbol>,
    #[serde(default)]
    pub slice_text_contains: Vec<String>,
    #[serde(default)]
    pub slice_text_not_contains: Vec<String>,
    #[serde(default)]
    pub output_not_contains: Vec<String>,
    pub max_slice_lines: Option<usize>,
    pub max_impacted_components: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedSymbol {
    pub name: String,
    #[serde(default)]
    pub kind: Option<SymbolKind>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub eval_contract_version: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub inspect_cases: usize,
    pub impact_cases: usize,
    pub symbol_cases: usize,
    pub source_evidence_cases: usize,
    pub source_context_cases: usize,
    pub lsp_diagnostics_cases: usize,
    pub metrics: EvalMetrics,
    pub cases: Vec<EvalCaseResult>,
    pub failures: Vec<EvalFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub evidence_coverage_pass_rate: f64,
    pub expected_fact_recall: f64,
    pub unexpected_warning_count: usize,
    pub missing_expected_warning_count: usize,
    pub false_broad_count: usize,
    pub false_narrow_count: usize,
    pub deterministic_output_pass_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCaseResult {
    pub name: String,
    pub kind: EvalCaseKind,
    pub passed: bool,
    pub failures: Vec<EvalFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalFailure {
    pub case_name: String,
    pub expected: String,
    pub actual: String,
    pub severity: String,
    pub category: String,
}

#[derive(Debug, Default)]
struct EvalCounters {
    evidence_cases: usize,
    evidence_passed: usize,
    deterministic_cases: usize,
    deterministic_passed: usize,
    expected_checks: usize,
    expected_checks_passed: usize,
    unexpected_warning_count: usize,
    missing_expected_warning_count: usize,
    false_broad_count: usize,
    false_narrow_count: usize,
}

pub fn run_fixture_evaluation(cases_dir: impl AsRef<Path>) -> Result<EvaluationReport, String> {
    evaluate_cases(load_eval_cases(cases_dir)?)
}

pub fn load_eval_cases(cases_dir: impl AsRef<Path>) -> Result<Vec<EvalCase>, String> {
    let mut entries = fs::read_dir(cases_dir.as_ref())
        .map_err(|error| {
            format!(
                "Failed to read eval cases directory {}: {error}",
                cases_dir.as_ref().display()
            )
        })?
        .flatten()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    entries
        .into_iter()
        .map(|entry| {
            let path = entry.path();
            let contents = fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read eval case {}: {error}", path.display()))?;
            serde_json::from_str::<EvalCase>(&contents)
                .map_err(|error| format!("Failed to parse eval case {}: {error}", path.display()))
        })
        .collect()
}

pub fn evaluate_cases(cases: Vec<EvalCase>) -> Result<EvaluationReport, String> {
    let mut counters = EvalCounters::default();
    let mut results = Vec::new();
    let mut all_failures = Vec::new();
    let mut inspect_cases = 0;
    let mut impact_cases = 0;
    let mut symbol_cases = 0;
    let mut source_evidence_cases = 0;
    let mut source_context_cases = 0;
    let mut lsp_diagnostics_cases = 0;

    for case in cases {
        match case.kind {
            EvalCaseKind::Inspect => inspect_cases += 1,
            EvalCaseKind::Impact => impact_cases += 1,
            EvalCaseKind::Symbols => symbol_cases += 1,
            EvalCaseKind::SourceEvidence => source_evidence_cases += 1,
            EvalCaseKind::SourceContext => source_context_cases += 1,
            EvalCaseKind::LspDiagnostics => lsp_diagnostics_cases += 1,
        }

        let result = evaluate_case(&case, &mut counters);
        all_failures.extend(result.failures.clone());
        results.push(result);
    }

    let passed_cases = results.iter().filter(|case| case.passed).count();
    let total_cases = results.len();

    Ok(EvaluationReport {
        eval_contract_version: EVAL_CONTRACT_VERSION.to_string(),
        total_cases,
        passed_cases,
        failed_cases: total_cases.saturating_sub(passed_cases),
        inspect_cases,
        impact_cases,
        symbol_cases,
        source_evidence_cases,
        source_context_cases,
        lsp_diagnostics_cases,
        metrics: EvalMetrics {
            evidence_coverage_pass_rate: rate(counters.evidence_passed, counters.evidence_cases),
            expected_fact_recall: rate(counters.expected_checks_passed, counters.expected_checks),
            unexpected_warning_count: counters.unexpected_warning_count,
            missing_expected_warning_count: counters.missing_expected_warning_count,
            false_broad_count: counters.false_broad_count,
            false_narrow_count: counters.false_narrow_count,
            deterministic_output_pass_rate: rate(
                counters.deterministic_passed,
                counters.deterministic_cases,
            ),
        },
        cases: results,
        failures: all_failures,
    })
}

fn evaluate_case(case: &EvalCase, counters: &mut EvalCounters) -> EvalCaseResult {
    let mut failures = Vec::new();

    match case.kind {
        EvalCaseKind::Inspect => {
            let first = inspect_repo(&case.fixture);
            let second = inspect_repo(&case.fixture);
            check_deterministic(case, first == second, counters, &mut failures);
            check_inspect_evidence(case, &first, counters, &mut failures);
            check_inspect_expectations(case, &first, counters, &mut failures);
        }
        EvalCaseKind::Impact => {
            let graph = inspect_repo(&case.fixture);
            let first = analyze_impact(&graph, case.changed_files.clone());
            let second = analyze_impact(&graph, case.changed_files.clone());
            check_deterministic(case, first == second, counters, &mut failures);
            check_impact_evidence(case, &first, counters, &mut failures);
            check_impact_expectations(case, &first, counters, &mut failures);
        }
        EvalCaseKind::Symbols => {
            let first = build_symbol_graph(&case.fixture);
            let second = build_symbol_graph(&case.fixture);
            check_deterministic(case, first == second, counters, &mut failures);
            check_symbols_evidence(case, &first, counters, &mut failures);
            check_symbols_expectations(case, &first, counters, &mut failures);
        }
        EvalCaseKind::SourceEvidence => {
            let first = build_source_evidence_bundle(&case.fixture, &case.query);
            let second = build_source_evidence_bundle(&case.fixture, &case.query);
            check_deterministic(case, first == second, counters, &mut failures);
            check_source_evidence(case, &first, counters, &mut failures);
            check_source_evidence_expectations(case, &first, counters, &mut failures);
        }
        EvalCaseKind::SourceContext => {
            let selectors = source_context_selectors(case);
            let first = build_source_context_report(&case.fixture, selectors.clone());
            let second = build_source_context_report(&case.fixture, selectors);
            check_deterministic(case, first == second, counters, &mut failures);
            check_source_context(case, &first, counters, &mut failures);
            check_source_context_expectations(case, &first, counters, &mut failures);
        }
        EvalCaseKind::LspDiagnostics => {
            let files = lsp_diagnostics_files(case);
            let options = lsp_diagnostics_options(case);
            let first = collect_rust_lsp_diagnostics_with_options(
                &case.fixture,
                files.clone(),
                options.clone(),
            );
            let second = collect_rust_lsp_diagnostics_with_options(&case.fixture, files, options);
            check_deterministic(case, first == second, counters, &mut failures);
            check_lsp_diagnostics(case, &first, counters, &mut failures);
            check_lsp_diagnostics_expectations(case, &first, counters, &mut failures);
        }
    }

    EvalCaseResult {
        name: case.name.clone(),
        kind: case.kind.clone(),
        passed: failures.is_empty(),
        failures,
    }
}

fn lsp_diagnostics_files(case: &EvalCase) -> Vec<String> {
    if case.selector_file.is_empty() {
        Vec::new()
    } else {
        vec![case.selector_file.clone()]
    }
}

fn lsp_diagnostics_options(case: &EvalCase) -> LspDiagnosticsOptions {
    let mut options = LspDiagnosticsOptions::default();
    if !case.lsp_command.is_empty() {
        options.command = Some(case.lsp_command.clone());
    }
    if let Some(timeout_ms) = case.lsp_timeout_ms {
        options.timeout_ms = timeout_ms;
    }
    if let Some(max_diagnostics) = case.lsp_max_diagnostics {
        options.max_diagnostics = max_diagnostics;
    }
    options
}

fn source_context_selectors(case: &EvalCase) -> Vec<SourceContextSelector> {
    let mut selectors = Vec::new();
    if !case.selector_file.is_empty() {
        selectors.push(SourceContextSelector::File {
            path: case.selector_file.clone(),
            line_range: case.selector_lines.as_deref().and_then(parse_line_range),
        });
    }
    if !case.selector_symbol_id.is_empty() {
        selectors.push(SourceContextSelector::SymbolId {
            symbol_id: case.selector_symbol_id.clone(),
        });
    }
    selectors
}

fn parse_line_range(value: &str) -> Option<LineRange> {
    let (start, end) = value.split_once(':')?;
    Some(LineRange {
        start_line: start.trim().parse().ok()?,
        end_line: end.trim().parse().ok()?,
    })
}

fn check_source_context_expectations(
    case: &EvalCase,
    report: &SourceContextReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    if let Some(expected) = &case.expect.status {
        check_value(
            case,
            "status",
            format!("{expected:?}"),
            format!("{:?}", report.status),
            format!("{:?}", report.status) == format!("{expected:?}"),
            counters,
            failures,
        );
    }

    let slice_paths = report
        .slices
        .iter()
        .map(|slice| slice.file_path.as_str())
        .collect::<Vec<_>>();
    let warning_categories = source_context_warning_categories(&report.warnings);

    check_contains_all(
        case,
        "slices_contains",
        &case.expect.slices_contains,
        &slice_paths,
        "false_narrow",
        counters,
        failures,
    );
    check_slice_symbol_contains_all(
        case,
        "slice_symbols_contains",
        &case.expect.slice_symbols_contains,
        &report.slices,
        counters,
        failures,
    );
    check_slice_text_contains_all(case, report, counters, failures);
    check_warning_expectations(case, &warning_categories, counters, failures);

    if let Some(max_lines) = case.expect.max_slice_lines {
        let actual_max = report
            .slices
            .iter()
            .map(source_context_slice_line_count)
            .max()
            .unwrap_or(0);
        check_count_at_most(
            case,
            "max_slice_lines",
            actual_max,
            max_lines,
            counters,
            failures,
        );
    }
    check_runtime_output_forbidden(case, report, counters, failures);
}

fn check_lsp_diagnostics_expectations(
    case: &EvalCase,
    report: &LspDiagnosticsReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    if let Some(expected) = &case.expect.lsp_status {
        let actual = lsp_status_name(&report.status);
        check_value(
            case,
            "lsp_status",
            expected.clone(),
            actual.to_string(),
            actual == expected,
            counters,
            failures,
        );
    }

    let warning_categories = lsp_warning_categories(&report.warnings);
    check_warning_expectations(case, &warning_categories, counters, failures);
    check_lsp_missing_evidence_expectations(case, report, counters, failures);
    check_runtime_output_forbidden(case, report, counters, failures);
}

fn source_context_slice_line_count(slice: &SourceContextSlice) -> usize {
    slice
        .end_line
        .saturating_sub(slice.start_line)
        .saturating_add(1)
}

fn check_count_at_most(
    case: &EvalCase,
    field: &str,
    actual: usize,
    max: usize,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    let passed = actual <= max;
    counters.expected_checks += 1;
    if passed {
        counters.expected_checks_passed += 1;
    } else {
        counters.false_broad_count += 1;
        failures.push(failure(
            case,
            field,
            format!("at most {max}"),
            actual.to_string(),
            "false_broad",
        ));
    }
}

fn check_runtime_output_forbidden<T: Serialize>(
    case: &EvalCase,
    output: &T,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    if case.expect.output_not_contains.is_empty() {
        return;
    }

    let Ok(serialized) = serde_json::to_string(output) else {
        failures.push(failure(
            case,
            "output_not_contains",
            "serializable output".to_string(),
            "serialization failed".to_string(),
            "mismatch",
        ));
        return;
    };
    let normalized = serialized.to_ascii_lowercase();

    for forbidden in &case.expect.output_not_contains {
        let passed = !normalized.contains(&forbidden.to_ascii_lowercase());
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                "output_not_contains",
                format!("not {forbidden}"),
                forbidden.clone(),
                "false_broad",
            ));
        }
    }
}

fn check_source_evidence_expectations(
    case: &EvalCase,
    bundle: &SourceEvidenceBundle,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    if let Some(expected) = &case.expect.status {
        check_value(
            case,
            "status",
            format!("{expected:?}"),
            format!("{:?}", bundle.status),
            format!("{:?}", bundle.status) == format!("{expected:?}"),
            counters,
            failures,
        );
    }

    if let Some(expected) = &case.expect.confidence {
        check_value(
            case,
            "confidence",
            format!("{expected:?}"),
            format!("{:?}", bundle.confidence),
            format!("{:?}", bundle.confidence) == format!("{expected:?}"),
            counters,
            failures,
        );
    }

    let candidate_files = bundle
        .candidate_files
        .iter()
        .map(|candidate| candidate.path.as_str())
        .collect::<Vec<_>>();
    let repo_context = bundle
        .repo_context
        .iter()
        .map(|context| context.label.as_str())
        .collect::<Vec<_>>();
    let repo_context_roles = bundle
        .repo_context
        .iter()
        .map(|context| repo_context_role_name(&context.role))
        .collect::<Vec<_>>();
    let selector_hints = bundle
        .source_context_selectors
        .iter()
        .map(|selector| selector.file_path.as_str())
        .collect::<Vec<_>>();
    let warning_categories = source_evidence_warning_categories(&bundle.warnings);

    check_contains_all(
        case,
        "candidate_files_contains",
        &case.expect.candidate_files_contains,
        &candidate_files,
        "false_narrow",
        counters,
        failures,
    );
    check_candidate_symbol_contains_all(
        case,
        "candidate_symbols_contains",
        &case.expect.candidate_symbols_contains,
        &bundle.candidate_symbols,
        counters,
        failures,
    );
    if let Some(max) = case.expect.max_candidate_files {
        check_count_at_most(
            case,
            "max_candidate_files",
            bundle.candidate_files.len(),
            max,
            counters,
            failures,
        );
    }
    if let Some(max) = case.expect.max_candidate_symbols {
        check_count_at_most(
            case,
            "max_candidate_symbols",
            bundle.candidate_symbols.len(),
            max,
            counters,
            failures,
        );
    }
    check_contains_all(
        case,
        "repo_context_contains",
        &case.expect.repo_context_contains,
        &repo_context,
        "false_narrow",
        counters,
        failures,
    );
    check_contains_all(
        case,
        "repo_context_roles_contains",
        &case.expect.repo_context_roles_contains,
        &repo_context_roles,
        "false_narrow",
        counters,
        failures,
    );
    check_warning_expectations(case, &warning_categories, counters, failures);
    check_missing_evidence_expectations(case, bundle, counters, failures);
    check_contains_all(
        case,
        "selector_hints_contains",
        &case.expect.selector_hints_contains,
        &selector_hints,
        "false_narrow",
        counters,
        failures,
    );
    check_selector_hint_symbol_contains_all(
        case,
        "selector_hint_symbols_contains",
        &case.expect.selector_hint_symbols_contains,
        &bundle.source_context_selectors,
        counters,
        failures,
    );
    if let Some(max) = case.expect.max_source_context_selectors {
        let passed = bundle.source_context_selectors.len() <= max;
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                "max_source_context_selectors",
                format!("at most {max}"),
                bundle.source_context_selectors.len().to_string(),
                "false_broad",
            ));
        }
    }
    check_runtime_output_forbidden(case, bundle, counters, failures);
}

fn check_symbols_expectations(
    case: &EvalCase,
    graph: &SymbolGraph,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    let source_files = graph
        .source_files
        .iter()
        .map(|source_file| source_file.path.as_str())
        .collect::<Vec<_>>();
    let warning_categories = symbol_warning_categories(&graph.warnings);

    check_contains_all(
        case,
        "source_files_contains",
        &case.expect.source_files_contains,
        &source_files,
        "false_narrow",
        counters,
        failures,
    );
    check_forbidden(
        case,
        "source_files_not_contains",
        &case.expect.source_files_not_contains,
        &source_files,
        counters,
        failures,
    );
    check_symbol_contains_all(
        case,
        "symbols_contains",
        &case.expect.symbols_contains,
        &graph.symbols,
        counters,
        failures,
    );
    check_symbol_forbidden(
        case,
        "symbols_not_contains",
        &case.expect.symbols_not_contains,
        &graph.symbols,
        counters,
        failures,
    );
    check_warning_expectations(case, &warning_categories, counters, failures);
}

fn check_inspect_expectations(
    case: &EvalCase,
    graph: &RepoInspection,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    let components = graph
        .components
        .iter()
        .map(|component| component.name.as_str())
        .collect::<Vec<_>>();
    let commands = graph
        .commands
        .iter()
        .map(|command| command.command.as_str())
        .collect::<Vec<_>>();
    let tests = graph
        .tests
        .iter()
        .map(|test| test.command.as_str())
        .collect::<Vec<_>>();
    let warning_categories = warning_categories(&graph.warnings);

    check_contains_all(
        case,
        "components_contains",
        &case.expect.components_contains,
        &components,
        "false_narrow",
        counters,
        failures,
    );
    check_contains_all(
        case,
        "commands_contains",
        &case.expect.commands_contains,
        &commands,
        "false_narrow",
        counters,
        failures,
    );
    check_contains_all(
        case,
        "tests_contains",
        &case.expect.tests_contains,
        &tests,
        "false_narrow",
        counters,
        failures,
    );
    check_warning_expectations(case, &warning_categories, counters, failures);
    check_forbidden(
        case,
        "forbidden_components_contains",
        &case.expect.forbidden_components_contains,
        &components,
        counters,
        failures,
    );
    check_forbidden(
        case,
        "forbidden_commands_contains",
        &case.expect.forbidden_commands_contains,
        &commands,
        counters,
        failures,
    );
    check_forbidden(
        case,
        "forbidden_tests_contains",
        &case.expect.forbidden_tests_contains,
        &tests,
        counters,
        failures,
    );
}

fn check_impact_expectations(
    case: &EvalCase,
    report: &ImpactReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    if let Some(expected) = &case.expect.status {
        check_value(
            case,
            "status",
            format!("{expected:?}"),
            format!("{:?}", report.status),
            &report.status == expected,
            counters,
            failures,
        );
    }

    if let Some(expected) = &case.expect.confidence {
        check_value(
            case,
            "confidence",
            format!("{expected:?}"),
            format!("{:?}", report.confidence),
            &report.confidence == expected,
            counters,
            failures,
        );
    }

    if let Some(expected) = &case.expect.impact_scope {
        check_value(
            case,
            "impact_scope",
            format!("{expected:?}"),
            format!("{:?}", report.impact_scope),
            &report.impact_scope == expected,
            counters,
            failures,
        );
    }

    let components = report
        .impacted_components
        .iter()
        .map(|component| component.name.as_str())
        .collect::<Vec<_>>();
    let commands = report
        .recommended_commands
        .iter()
        .map(|command| command.command.as_str())
        .collect::<Vec<_>>();
    let tests = report
        .recommended_tests
        .iter()
        .map(|test| test.command.as_str())
        .collect::<Vec<_>>();
    let warning_categories = warning_categories(&report.warnings);

    check_contains_all(
        case,
        "impacted_components_contains",
        &case.expect.components_contains,
        &components,
        "false_narrow",
        counters,
        failures,
    );
    check_contains_all(
        case,
        "recommended_commands_contains",
        &case.expect.commands_contains,
        &commands,
        "false_narrow",
        counters,
        failures,
    );
    check_contains_all(
        case,
        "recommended_tests_contains",
        &case.expect.tests_contains,
        &tests,
        "false_narrow",
        counters,
        failures,
    );
    check_warning_expectations(case, &warning_categories, counters, failures);
    check_forbidden(
        case,
        "forbidden_impacted_components_contains",
        &case.expect.forbidden_components_contains,
        &components,
        counters,
        failures,
    );
    check_forbidden(
        case,
        "forbidden_recommended_commands_contains",
        &case.expect.forbidden_commands_contains,
        &commands,
        counters,
        failures,
    );
    check_forbidden(
        case,
        "forbidden_recommended_tests_contains",
        &case.expect.forbidden_tests_contains,
        &tests,
        counters,
        failures,
    );

    if let Some(max) = case.expect.max_impacted_components {
        let passed = report.impacted_components.len() <= max;
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                "max_impacted_components",
                format!("at most {max}"),
                report.impacted_components.len().to_string(),
                "false_broad",
            ));
        }
    }
}

fn check_contains_all(
    case: &EvalCase,
    field: &str,
    expected: &[String],
    actual: &[&str],
    category: &str,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in expected {
        let passed = actual
            .iter()
            .any(|actual_item| actual_item == expected_item);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            if category == "false_narrow" {
                counters.false_narrow_count += 1;
            }
            failures.push(failure(
                case,
                field,
                expected_item.clone(),
                format!("{actual:?}"),
                category,
            ));
        }
    }
}

fn check_forbidden(
    case: &EvalCase,
    field: &str,
    forbidden: &[String],
    actual: &[&str],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for forbidden_item in forbidden {
        let passed = !actual
            .iter()
            .any(|actual_item| actual_item == forbidden_item);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                field,
                format!("not {forbidden_item}"),
                format!("{actual:?}"),
                "false_broad",
            ));
        }
    }
}

fn check_symbol_contains_all(
    case: &EvalCase,
    field: &str,
    expected: &[ExpectedSymbol],
    actual: &[SourceSymbol],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in expected {
        let passed = actual
            .iter()
            .any(|actual_item| symbol_matches(actual_item, expected_item));
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                field,
                format_expected_symbol(expected_item),
                format_actual_symbols(actual),
                "false_narrow",
            ));
        }
    }
}

fn check_symbol_forbidden(
    case: &EvalCase,
    field: &str,
    forbidden: &[ExpectedSymbol],
    actual: &[SourceSymbol],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for forbidden_item in forbidden {
        let passed = !actual
            .iter()
            .any(|actual_item| symbol_matches(actual_item, forbidden_item));
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                field,
                format!("not {}", format_expected_symbol(forbidden_item)),
                format_actual_symbols(actual),
                "false_broad",
            ));
        }
    }
}

fn check_candidate_symbol_contains_all(
    case: &EvalCase,
    field: &str,
    expected: &[ExpectedSymbol],
    actual: &[CandidateSymbol],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in expected {
        let passed = actual
            .iter()
            .any(|actual_item| candidate_symbol_matches(actual_item, expected_item));
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                field,
                format_expected_symbol(expected_item),
                format_actual_candidate_symbols(actual),
                "false_narrow",
            ));
        }
    }
}

fn check_selector_hint_symbol_contains_all(
    case: &EvalCase,
    field: &str,
    expected: &[ExpectedSymbol],
    actual: &[SourceContextSelectorHint],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in expected {
        let passed = actual
            .iter()
            .any(|actual_item| selector_hint_symbol_matches(actual_item, expected_item));
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                field,
                format_expected_symbol(expected_item),
                format_actual_selector_hints(actual),
                "false_narrow",
            ));
        }
    }
}

fn check_slice_symbol_contains_all(
    case: &EvalCase,
    field: &str,
    expected: &[ExpectedSymbol],
    actual: &[SourceContextSlice],
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in expected {
        let passed = actual
            .iter()
            .any(|actual_item| slice_symbol_matches(actual_item, expected_item));
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                field,
                format_expected_symbol(expected_item),
                format_actual_slice_symbols(actual),
                "false_narrow",
            ));
        }
    }
}

fn check_slice_text_contains_all(
    case: &EvalCase,
    report: &SourceContextReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    let combined_text = report
        .slices
        .iter()
        .map(|slice| slice.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for expected_item in &case.expect.slice_text_contains {
        let passed = combined_text.contains(expected_item);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                "slice_text_contains",
                expected_item.clone(),
                combined_text.clone(),
                "false_narrow",
            ));
        }
    }

    for forbidden_item in &case.expect.slice_text_not_contains {
        let passed = !combined_text.contains(forbidden_item);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_broad_count += 1;
            failures.push(failure(
                case,
                "slice_text_not_contains",
                format!("not {forbidden_item}"),
                combined_text.clone(),
                "false_broad",
            ));
        }
    }
}

fn symbol_matches(actual: &SourceSymbol, expected: &ExpectedSymbol) -> bool {
    actual.name == expected.name
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| &actual.kind == kind)
        && expected
            .path
            .as_deref()
            .is_none_or(|path| actual.path == path)
}

fn candidate_symbol_matches(actual: &CandidateSymbol, expected: &ExpectedSymbol) -> bool {
    actual.name == expected.name
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| &actual.kind == kind)
        && expected
            .path
            .as_deref()
            .is_none_or(|path| actual.path == path)
}

fn selector_hint_symbol_matches(
    actual: &SourceContextSelectorHint,
    expected: &ExpectedSymbol,
) -> bool {
    actual.symbol_name.as_deref() == Some(expected.name.as_str())
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| actual.symbol_kind.as_ref() == Some(kind))
        && expected
            .path
            .as_deref()
            .is_none_or(|path| actual.file_path == path)
}

fn slice_symbol_matches(actual: &SourceContextSlice, expected: &ExpectedSymbol) -> bool {
    actual.symbol_name.as_deref() == Some(expected.name.as_str())
        && expected
            .kind
            .as_ref()
            .is_none_or(|kind| actual.symbol_kind.as_ref() == Some(kind))
        && expected
            .path
            .as_deref()
            .is_none_or(|path| actual.file_path == path)
}

fn format_expected_symbol(expected: &ExpectedSymbol) -> String {
    let kind = expected
        .kind
        .as_ref()
        .map(|kind| format!("{kind:?}"))
        .unwrap_or_else(|| "*".to_string());
    let path = expected.path.as_deref().unwrap_or("*");
    format!("{kind}:{}@{path}", expected.name)
}

fn format_actual_symbols(actual: &[SourceSymbol]) -> String {
    actual
        .iter()
        .map(|symbol| format!("{:?}:{}@{}", symbol.kind, symbol.name, symbol.path))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_actual_candidate_symbols(actual: &[CandidateSymbol]) -> String {
    actual
        .iter()
        .map(|symbol| format!("{:?}:{}@{}", symbol.kind, symbol.name, symbol.path))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_actual_selector_hints(actual: &[SourceContextSelectorHint]) -> String {
    actual
        .iter()
        .map(|selector| {
            format!(
                "{:?}:{}@{}",
                selector.selector_kind,
                selector.symbol_name.as_deref().unwrap_or(""),
                selector.file_path
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_actual_slice_symbols(actual: &[SourceContextSlice]) -> String {
    actual
        .iter()
        .map(|slice| {
            format!(
                "{:?}:{}@{}",
                slice.symbol_kind,
                slice.symbol_name.as_deref().unwrap_or(""),
                slice.file_path
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn check_missing_evidence_expectations(
    case: &EvalCase,
    bundle: &SourceEvidenceBundle,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    let actual = bundle
        .missing_evidence
        .iter()
        .map(|missing| {
            format!(
                "{}:{}",
                serde_json::to_value(&missing.category)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_string))
                    .unwrap_or_else(|| format!("{:?}", missing.category)),
                missing.message
            )
        })
        .collect::<Vec<_>>();

    for expected_item in &case.expect.missing_evidence_contains {
        let passed = actual.iter().any(|actual_item| {
            actual_item
                .to_ascii_lowercase()
                .contains(&expected_item.to_ascii_lowercase())
        });
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                "missing_evidence_contains",
                expected_item.clone(),
                format!("{actual:?}"),
                "false_narrow",
            ));
        }
    }
}

fn check_lsp_missing_evidence_expectations(
    case: &EvalCase,
    report: &LspDiagnosticsReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_item in &case.expect.missing_evidence_contains {
        let passed = report
            .missing_evidence
            .iter()
            .any(|actual_item| actual_item == expected_item);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.false_narrow_count += 1;
            failures.push(failure(
                case,
                "missing_evidence_contains",
                expected_item.clone(),
                format!("{:?}", report.missing_evidence),
                "false_narrow",
            ));
        }
    }
}

fn check_warning_expectations(
    case: &EvalCase,
    actual_categories: &BTreeSet<String>,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    for expected_category in &case.expect.warnings_contains_categories {
        let passed = actual_categories.contains(expected_category);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.missing_expected_warning_count += 1;
            failures.push(failure(
                case,
                "warnings_contains_categories",
                expected_category.clone(),
                format!("{actual_categories:?}"),
                "missing_expected_warning",
            ));
        }
    }

    for unexpected_category in &case.expect.warnings_not_contains_categories {
        let passed = !actual_categories.contains(unexpected_category);
        counters.expected_checks += 1;
        if passed {
            counters.expected_checks_passed += 1;
        } else {
            counters.unexpected_warning_count += 1;
            failures.push(failure(
                case,
                "warnings_not_contains_categories",
                format!("not {unexpected_category}"),
                format!("{actual_categories:?}"),
                "unexpected_warning",
            ));
        }
    }
}

fn check_value(
    case: &EvalCase,
    field: &str,
    expected: String,
    actual: String,
    passed: bool,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.expected_checks += 1;
    if passed {
        counters.expected_checks_passed += 1;
    } else {
        failures.push(failure(case, field, expected, actual, "mismatch"));
    }
}

fn check_deterministic(
    case: &EvalCase,
    passed: bool,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.deterministic_cases += 1;
    if passed {
        counters.deterministic_passed += 1;
    } else {
        failures.push(failure(
            case,
            "deterministic_output",
            "same output across repeated runs".to_string(),
            "outputs differed".to_string(),
            "determinism",
        ));
    }
}

fn check_inspect_evidence(
    case: &EvalCase,
    graph: &RepoInspection,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if inspect_evidence_valid(graph) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all inspect facts have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn check_impact_evidence(
    case: &EvalCase,
    report: &ImpactReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if impact_evidence_valid(report) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all impact facts have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn check_symbols_evidence(
    case: &EvalCase,
    graph: &SymbolGraph,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if symbol_graph_evidence_valid(graph) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all source files and symbols have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn check_source_evidence(
    case: &EvalCase,
    bundle: &SourceEvidenceBundle,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if source_evidence_bundle_evidence_valid(bundle) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all source evidence bundle candidates have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn check_source_context(
    case: &EvalCase,
    report: &SourceContextReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if source_context_evidence_valid(report) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all source context slices have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn check_lsp_diagnostics(
    case: &EvalCase,
    report: &LspDiagnosticsReport,
    counters: &mut EvalCounters,
    failures: &mut Vec<EvalFailure>,
) {
    counters.evidence_cases += 1;
    if lsp_diagnostics_evidence_valid(report) {
        counters.evidence_passed += 1;
    } else {
        failures.push(failure(
            case,
            "evidence_coverage",
            "all LSP diagnostics have valid evidence".to_string(),
            "missing or invalid evidence reference".to_string(),
            "evidence_coverage",
        ));
    }
}

fn inspect_evidence_valid(graph: &RepoInspection) -> bool {
    let evidence_ids = evidence_ids(&graph.evidence);

    graph
        .detected_files
        .iter()
        .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .package_managers
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .workspaces
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .components
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .commands
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .tests
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph
            .relationships
            .iter()
            .all(|fact| evidence_ids.contains(fact.evidence_id.as_str()))
        && graph.warnings.iter().all(|warning| {
            warning
                .evidence_id
                .as_deref()
                .is_none_or(|id| evidence_ids.contains(id))
        })
}

fn impact_evidence_valid(report: &ImpactReport) -> bool {
    let evidence_ids = evidence_ids(&report.evidence);

    report.impacted_components.iter().all(|fact| {
        !fact.evidence_ids.is_empty()
            && fact
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && report.impacted_workspaces.iter().all(|fact| {
        !fact.evidence_ids.is_empty()
            && fact
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && report.recommended_commands.iter().all(|fact| {
        !fact.evidence_ids.is_empty()
            && fact
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && report.recommended_tests.iter().all(|fact| {
        !fact.evidence_ids.is_empty()
            && fact
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    })
}

fn evidence_ids(evidence: &[Evidence]) -> BTreeSet<&str> {
    evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect()
}

fn warning_categories(warnings: &[DetectionIssue]) -> BTreeSet<String> {
    warnings
        .iter()
        .map(|warning| category_name(&warning.category).to_string())
        .collect()
}

fn symbol_warning_categories(warnings: &[SymbolWarning]) -> BTreeSet<String> {
    warnings
        .iter()
        .map(|warning| symbol_category_name(&warning.category).to_string())
        .collect()
}

fn source_evidence_warning_categories(warnings: &[BundleWarning]) -> BTreeSet<String> {
    warnings
        .iter()
        .map(|warning| source_evidence_category_name(&warning.category).to_string())
        .collect()
}

fn source_context_warning_categories(warnings: &[SourceContextWarning]) -> BTreeSet<String> {
    warnings
        .iter()
        .map(|warning| source_context_category_name(&warning.category).to_string())
        .collect()
}

fn lsp_warning_categories(warnings: &[LspWarning]) -> BTreeSet<String> {
    warnings
        .iter()
        .map(|warning| lsp_warning_category_name(&warning.category).to_string())
        .collect()
}

fn category_name(category: &DetectionCategory) -> &'static str {
    match category {
        DetectionCategory::AmbiguousDetection => "ambiguous_detection",
        DetectionCategory::IgnoredPath => "ignored_path",
        DetectionCategory::MalformedManifest => "malformed_manifest",
        DetectionCategory::MissingCommand => "missing_command",
        DetectionCategory::NoSupportedManifests => "no_supported_manifests",
        DetectionCategory::PartialSupport => "partial_support",
        DetectionCategory::RepoGraphOnly => "repo_graph_only",
        DetectionCategory::UnmappedChange => "unmapped_change",
        DetectionCategory::UnreadableManifest => "unreadable_manifest",
        DetectionCategory::UnsupportedPattern => "unsupported_pattern",
    }
}

fn symbol_category_name(category: &SymbolWarningCategory) -> &'static str {
    match category {
        SymbolWarningCategory::IgnoredPath => "ignored_path",
        SymbolWarningCategory::ParseError => "parse_error",
        SymbolWarningCategory::SymlinkIgnored => "symlink_ignored",
        SymbolWarningCategory::UnreadableSource => "unreadable_source",
    }
}

fn source_evidence_category_name(category: &BundleWarningCategory) -> &'static str {
    match category {
        BundleWarningCategory::AmbiguousQuery => "ambiguous_query",
        BundleWarningCategory::CandidateLimitExceeded => "candidate_limit_exceeded",
        BundleWarningCategory::InsufficientEvidenceForLocalization => {
            "insufficient_evidence_for_localization"
        }
        BundleWarningCategory::LocalizationNotSupported => "localization_not_supported",
        BundleWarningCategory::MultipleCandidates => "multiple_candidates",
        BundleWarningCategory::NoRepoComponentContext => "no_repo_component_context",
        BundleWarningCategory::NoMatchingSourceFiles => "no_matching_source_files",
        BundleWarningCategory::NoMatchingSourceSymbols => "no_matching_source_symbols",
        BundleWarningCategory::ParseErrorPresent => "parse_error_present",
        BundleWarningCategory::QueryTooBroad => "query_too_broad",
        BundleWarningCategory::RepoGraphContextUnavailable => "repo_graph_context_unavailable",
        BundleWarningCategory::SelectorHintLimitExceeded => "selector_hint_limit_exceeded",
        BundleWarningCategory::SymbolGraphParseWarning => "symbol_graph_parse_warning",
        BundleWarningCategory::UnsupportedLanguage => "unsupported_language",
    }
}

fn source_context_category_name(category: &SourceContextWarningCategory) -> &'static str {
    match category {
        SourceContextWarningCategory::AmbiguousSymbolSelector => "ambiguous_symbol_selector",
        SourceContextWarningCategory::BinaryFile => "binary_file",
        SourceContextWarningCategory::IgnoredPath => "ignored_path",
        SourceContextWarningCategory::MissingFile => "missing_file",
        SourceContextWarningCategory::NonUtf8File => "non_utf8_file",
        SourceContextWarningCategory::PathOutsideRepo => "path_outside_repo",
        SourceContextWarningCategory::SliceTruncated => "slice_truncated",
        SourceContextWarningCategory::SourceContextLimitExceeded => "source_context_limit_exceeded",
        SourceContextWarningCategory::SourceContextNotLocalization => {
            "source_context_not_localization"
        }
        SourceContextWarningCategory::SymbolNotFound => "symbol_not_found",
        SourceContextWarningCategory::SymlinkIgnored => "symlink_ignored",
        SourceContextWarningCategory::UnsupportedLanguage => "unsupported_language",
    }
}

fn lsp_warning_category_name(category: &LspWarningCategory) -> &'static str {
    match category {
        LspWarningCategory::RustAnalyzerUnavailable => "rust_analyzer_unavailable",
        LspWarningCategory::LspDiagnosticsUnavailable => "lsp_diagnostics_unavailable",
        LspWarningCategory::PathOutsideRepo => "path_outside_repo",
        LspWarningCategory::IgnoredPath => "ignored_path",
        LspWarningCategory::SymlinkIgnored => "symlink_ignored",
        LspWarningCategory::MissingFile => "missing_file",
        LspWarningCategory::UnsupportedLanguage => "unsupported_language",
        LspWarningCategory::RequestTimeout => "request_timeout",
        LspWarningCategory::ResultLimitExceeded => "result_limit_exceeded",
        LspWarningCategory::ServerError => "server_error",
        LspWarningCategory::NoFilesRequested => "no_files_requested",
        LspWarningCategory::LspNotLocalization => "lsp_not_localization",
    }
}

fn lsp_status_name(status: &LspAvailabilityStatus) -> &'static str {
    match status {
        LspAvailabilityStatus::Ok => "ok",
        LspAvailabilityStatus::Partial => "partial",
        LspAvailabilityStatus::Unavailable => "unavailable",
        LspAvailabilityStatus::Error => "error",
    }
}

fn repo_context_role_name(role: &RepoContextRole) -> &'static str {
    match role {
        RepoContextRole::AmbiguousContext => "ambiguous_context",
        RepoContextRole::ContainingComponent => "containing_component",
        RepoContextRole::ContainingWorkspace => "containing_workspace",
        RepoContextRole::DependencyContext => "dependency_context",
        RepoContextRole::ImpactContext => "impact_context",
        RepoContextRole::TestCommandContext => "test_command_context",
        RepoContextRole::VerificationCommandContext => "verification_command_context",
    }
}

fn failure(
    case: &EvalCase,
    expected: &str,
    expected_value: String,
    actual: String,
    category: &str,
) -> EvalFailure {
    EvalFailure {
        case_name: case.name.clone(),
        expected: format!("{expected}: {expected_value}"),
        actual,
        severity: "error".to_string(),
        category: category.to_string(),
    }
}

fn rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    }
}
