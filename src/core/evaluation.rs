use crate::core::repo_graph::{
    analyze_impact, inspect_repo, DetectionCategory, DetectionIssue, Evidence, ImpactConfidence,
    ImpactReport, ImpactScope, ImpactStatus, RepoInspection,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub const EVAL_CONTRACT_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalCaseKind {
    Inspect,
    Impact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalCase {
    pub name: String,
    pub fixture: String,
    pub kind: EvalCaseKind,
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
    pub max_impacted_components: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub eval_contract_version: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub inspect_cases: usize,
    pub impact_cases: usize,
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

    for case in cases {
        match case.kind {
            EvalCaseKind::Inspect => inspect_cases += 1,
            EvalCaseKind::Impact => impact_cases += 1,
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
    }

    EvalCaseResult {
        name: case.name.clone(),
        kind: case.kind.clone(),
        passed: failures.is_empty(),
        failures,
    }
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
