use code_intel_kernel::{
    collect_rust_lsp_diagnostics_with_options, lsp_diagnostics_evidence_valid,
    run_fixture_evaluation, LspAvailabilityStatus, LspDiagnosticsOptions, LspWarningCategory,
    LSP_DIAGNOSTICS_CONTRACT_VERSION,
};
use serde_json::Value as JsonValue;
use std::process::Command;

#[test]
fn lsp_diagnostics_unavailable_when_rust_analyzer_is_missing() {
    let report = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["src/lib.rs".to_string()],
        missing_rust_analyzer_options(),
    );

    assert_eq!(report.contract_version, LSP_DIAGNOSTICS_CONTRACT_VERSION);
    assert_eq!(report.status, LspAvailabilityStatus::Unavailable);
    assert!(report.diagnostics.is_empty());
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.category == LspWarningCategory::RustAnalyzerUnavailable));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.category == LspWarningCategory::LspDiagnosticsUnavailable));
    assert!(report
        .missing_evidence
        .iter()
        .any(|item| item == "no_lsp_diagnostics"));
    assert!(lsp_diagnostics_evidence_valid(&report));
}

#[test]
fn lsp_diagnostics_blocks_unsafe_or_unsupported_paths_before_server_start() {
    let outside = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["../Cargo.toml".to_string()],
        missing_rust_analyzer_options(),
    );
    assert_eq!(outside.status, LspAvailabilityStatus::Error);
    assert!(outside
        .warnings
        .iter()
        .any(|warning| warning.category == LspWarningCategory::PathOutsideRepo));

    let ignored = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["target/generated.rs".to_string()],
        missing_rust_analyzer_options(),
    );
    assert_eq!(ignored.status, LspAvailabilityStatus::Error);
    assert!(ignored
        .warnings
        .iter()
        .any(|warning| warning.category == LspWarningCategory::IgnoredPath));

    let missing = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["src/missing.rs".to_string()],
        missing_rust_analyzer_options(),
    );
    assert_eq!(missing.status, LspAvailabilityStatus::Error);
    assert!(missing
        .warnings
        .iter()
        .any(|warning| warning.category == LspWarningCategory::MissingFile));
}

#[test]
fn lsp_diagnostics_output_is_deterministic_and_has_no_edit_target_language() {
    let first = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["src/lib.rs".to_string()],
        missing_rust_analyzer_options(),
    );
    let second = collect_rust_lsp_diagnostics_with_options(
        "tests/fixtures/rust-symbols-basic",
        vec!["src/lib.rs".to_string()],
        missing_rust_analyzer_options(),
    );

    let first_json = serde_json::to_value(&first).expect("LSP diagnostics should serialize");
    let second_json = serde_json::to_value(&second).expect("LSP diagnostics should serialize");
    assert_eq!(first_json, second_json);
    assert_json_has_no_edit_target_language(&first_json);
}

#[test]
fn lsp_diagnostics_cli_output_is_valid_json_without_rust_analyzer() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .env(
            "CODE_INTEL_RUST_ANALYZER",
            "code-intel-missing-rust-analyzer",
        )
        .args([
            "lsp-diagnostics",
            "--file",
            "src/lib.rs",
            "--timeout-ms",
            "50",
            "--json",
        ])
        .output()
        .expect("lsp-diagnostics command should run");
    assert!(output.status.success());
    let json: JsonValue =
        serde_json::from_slice(&output.stdout).expect("lsp-diagnostics output should be JSON");
    assert_eq!(json["contract_version"], LSP_DIAGNOSTICS_CONTRACT_VERSION);
    assert_eq!(json["status"], "unavailable");
    assert_json_has_no_edit_target_language(&json);
}

#[test]
fn lsp_diagnostics_eval_cases_pass() {
    let report =
        run_fixture_evaluation("tests/eval/cases").expect("eval report should include LSP cases");

    for case_name in [
        "lsp_diagnostics_unavailable",
        "lsp_diagnostics_path_outside",
    ] {
        let result = report
            .cases
            .iter()
            .find(|case| case.name == case_name)
            .expect("LSP diagnostics eval case should be present");
        assert!(result.passed, "{case_name} failed: {:?}", result.failures);
    }

    let json = serde_json::to_value(&report).expect("eval report should serialize");
    assert!(json["lsp_diagnostics_cases"]
        .as_u64()
        .is_some_and(|count| count >= 2));
}

fn missing_rust_analyzer_options() -> LspDiagnosticsOptions {
    LspDiagnosticsOptions {
        command: Some("code-intel-missing-rust-analyzer".to_string()),
        timeout_ms: 50,
        max_diagnostics: 16,
    }
}

fn assert_json_has_no_edit_target_language(value: &JsonValue) {
    let json = serde_json::to_string(value).expect("JSON value should serialize");
    for forbidden in [
        "edit this",
        "edit here",
        "target_edit",
        "edit_location",
        "patch target",
        "apply patch",
        "root cause",
        "change this",
        "correct edit location",
    ] {
        assert!(
            !json.contains(forbidden),
            "runtime output should not contain edit-target phrase: {forbidden}"
        );
    }
}
