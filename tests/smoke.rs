use code_intel_kernel::{
    analyze_impact, build_source_context_report, build_source_evidence_bundle, build_symbol_graph,
    create_evidence_bundle, evaluate_cases, inspect_repo, load_eval_cases, run_fixture_evaluation,
    source_context_evidence_valid, source_evidence_bundle_evidence_valid,
    symbol_graph_evidence_valid, BundleConfidence, BundleStatus, BundleWarningCategory,
    DetectionCategory, DetectionSeverity, EvalCase, EvalCaseKind, EvalExpect, EvidenceRequest,
    ImpactConfidence, ImpactKind, ImpactReport, ImpactScope, ImpactStatus, KernelProfile,
    LineRange, ParseStatus, RelationshipKind, RepoContextRole, RepoInspection, SourceContextReport,
    SourceContextSelector, SourceContextSelectorKind, SourceContextStatus,
    SourceContextWarningCategory, SourceEvidenceBundle, SymbolGraph, SymbolKind,
    SymbolWarningCategory, EVAL_CONTRACT_VERSION, IMPACT_CONTRACT_VERSION,
    INSPECT_CONTRACT_VERSION, SOURCE_CONTEXT_CONTRACT_VERSION, SOURCE_EVIDENCE_CONTRACT_VERSION,
    SYMBOLS_CONTRACT_VERSION,
};
use serde_json::Value as JsonValue;
use std::process::Command;

#[test]
fn skeleton_exposes_read_only_repo_inspection() {
    let snapshot = inspect_repo(".");

    assert!(snapshot.repo.read_only);
    assert_eq!(snapshot.contract_version, INSPECT_CONTRACT_VERSION);
    assert!(!snapshot.evidence.is_empty());
    assert!(snapshot
        .components
        .iter()
        .any(|component| component.name == "code-intel-kernel"));
}

#[test]
fn core_uses_generic_profiles_instead_of_consumer_names() {
    let bundle = create_evidence_bundle(EvidenceRequest {
        task: "change login validation copy".to_string(),
        repo_path: ".".to_string(),
        profile: KernelProfile::Strict,
    });

    assert_eq!(bundle.profile, KernelProfile::Strict);
    assert_eq!(bundle.confidence, 0.0);
    assert!(bundle.missing_evidence[0].contains("not implemented yet"));
}

#[test]
fn parses_minimal_cargo_toml_fixture() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");

    assert_eq!(graph.contract_version, "0.2");
    assert!(graph
        .detected_files
        .iter()
        .any(|file| file.path == "Cargo.toml"));
    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "minimal-cargo"));
    assert!(graph
        .components
        .iter()
        .any(|component| component.kind == "rust_lib_target"
            && component.file_patterns == vec!["src/lib.rs"]));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "cargo test"));
}

#[test]
fn detects_explicit_cargo_bin_target_fixture() {
    let graph = inspect_repo("tests/fixtures/cargo-explicit-bin");

    assert!(graph.components.iter().any(|component| {
        component.name == "worker"
            && component.kind == "rust_bin_target"
            && component.file_patterns == vec!["src/bin/worker.rs"]
    }));
    assert_all_evidence_refs_exist(&graph);
}

#[test]
fn detects_npm_package_json_fixture() {
    let graph = inspect_repo("tests/fixtures/minimal-node");

    assert!(graph
        .detected_files
        .iter()
        .any(|file| file.path == "package.json"));
    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "minimal-node"));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "npm run test"));
}

#[test]
fn detects_minimal_python_project_fixture() {
    let graph = inspect_repo("tests/fixtures/minimal-python");

    assert!(graph
        .detected_files
        .iter()
        .any(|file| file.path == "pyproject.toml"));
    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "minimal-python"));
    assert!(graph.tests.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::AmbiguousDetection));
}

#[test]
fn detects_minimal_go_module_fixture() {
    let graph = inspect_repo("tests/fixtures/minimal-go");

    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "example.com/minimal-go"));
    assert!(graph
        .tests
        .iter()
        .any(|test| test.command == "go test ./..."));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "go build ./..."));
}

#[test]
fn detects_generic_makefile_fixture() {
    let graph = inspect_repo("tests/fixtures/generic-make");

    for command in [
        "make test",
        "make check",
        "make build",
        "make lint",
        "make fmt",
    ] {
        assert!(graph
            .commands
            .iter()
            .any(|candidate| candidate.command == command));
    }
    assert!(graph.tests.iter().any(|test| test.command == "make test"));
    assert_all_evidence_refs_exist(&graph);
}

#[test]
fn detects_generic_justfile_fixture() {
    let graph = inspect_repo("tests/fixtures/generic-just");

    for command in [
        "just test",
        "just check",
        "just build",
        "just lint",
        "just format",
    ] {
        assert!(graph
            .commands
            .iter()
            .any(|candidate| candidate.command == command));
    }
    assert!(graph.tests.iter().any(|test| test.command == "just test"));
    assert_all_evidence_refs_exist(&graph);
}

#[test]
fn detects_workspace_members_fixture() {
    let graph = inspect_repo("tests/fixtures/cargo-workspace");

    assert_eq!(graph.workspaces.len(), 1);
    assert_eq!(graph.workspaces[0].members, vec!["crates/core"]);
}

#[test]
fn every_component_command_test_and_package_manager_has_evidence() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");

    assert_all_evidence_refs_exist(&graph);
    assert_all_graph_facts_have_evidence(&graph);
}

#[test]
fn every_fixture_has_valid_inspect_json_and_evidence_refs() {
    for fixture in [
        "tests/fixtures/minimal-cargo",
        "tests/fixtures/cargo-explicit-bin",
        "tests/fixtures/cargo-workspace",
        "tests/fixtures/cargo-workspace-deps",
        "tests/fixtures/minimal-node",
        "tests/fixtures/minimal-python",
        "tests/fixtures/python-pyproject-basic",
        "tests/fixtures/python-pytest-evidence",
        "tests/fixtures/python-tests-ambiguous",
        "tests/fixtures/python-malformed-pyproject",
        "tests/fixtures/minimal-go",
        "tests/fixtures/go-module-basic",
        "tests/fixtures/go-module-with-tests",
        "tests/fixtures/go-workspace-basic",
        "tests/fixtures/go-malformed-mod",
        "tests/fixtures/generic-make",
        "tests/fixtures/generic-just",
        "tests/fixtures/malformed-manifest",
    ] {
        let graph = inspect_repo(fixture);
        serde_json::to_value(&graph).expect("fixture inspect output should serialize");
        assert_eq!(graph.contract_version, INSPECT_CONTRACT_VERSION);
        assert_all_evidence_refs_exist(&graph);
    }
}

#[test]
fn ignored_directories_do_not_create_graph_facts() {
    let root = std::env::temp_dir().join(format!(
        "code-intel-kernel-ignored-paths-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for ignored_path in [
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        ".cache",
        ".venv",
        "__pycache__",
    ] {
        std::fs::create_dir_all(root.join(ignored_path))
            .expect("ignored directory should be created");
    }
    std::fs::write(
        root.join("target").join("Cargo.toml"),
        "[package]\nname = \"ignored-target\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("ignored manifest should be written");
    std::fs::write(
        root.join("node_modules").join("package.json"),
        "{\"name\":\"ignored-node-module\"}\n",
    )
    .expect("ignored package manifest should be written");

    let graph = inspect_repo(&root);
    let _ = std::fs::remove_dir_all(&root);

    assert!(graph.components.is_empty());
    assert!(graph.package_managers.is_empty());
    assert!(graph.commands.is_empty());
    assert!(graph.tests.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::IgnoredPath));
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::NoSupportedManifests));
}

#[test]
fn evidence_ids_are_deterministic_for_same_repo_state() {
    let first = inspect_repo("tests/fixtures/minimal-cargo");
    let second = inspect_repo("tests/fixtures/minimal-cargo");

    let first_ids = first
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<Vec<_>>();
    let second_ids = second
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(first_ids, second_ids);
    assert!(first_ids.iter().all(|id| id.starts_with("evidence-")));
}

#[test]
fn malformed_manifest_produces_structured_warning() {
    let graph = inspect_repo("tests/fixtures/malformed-manifest");

    assert!(graph.components.is_empty());
    assert!(graph.commands.is_empty());
    assert!(graph.warnings.iter().any(|warning| {
        warning.severity == DetectionSeverity::Error
            && warning.category == DetectionCategory::MalformedManifest
            && warning.path.as_deref() == Some("package.json")
            && warning.evidence_id.is_some()
    }));
    assert_all_warnings_are_structured(&graph);
}

#[test]
fn python_pyproject_name_is_extracted() {
    let graph = inspect_repo("tests/fixtures/python-pyproject-basic");

    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "python-basic"));
    assert!(graph.tests.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::MissingCommand));
}

#[test]
fn python_pytest_evidence_produces_test_command() {
    let graph = inspect_repo("tests/fixtures/python-pytest-evidence");

    assert!(graph
        .tests
        .iter()
        .any(|test| test.command == "pytest" && !test.evidence_id.is_empty()));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "pytest"));
    assert_all_evidence_refs_exist(&graph);
}

#[test]
fn python_tests_without_pytest_evidence_warns_without_guessing() {
    let graph = inspect_repo("tests/fixtures/python-tests-ambiguous");

    assert!(graph.tests.is_empty());
    assert!(graph.commands.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::AmbiguousDetection));
}

#[test]
fn malformed_pyproject_produces_warning_without_panic() {
    let graph = inspect_repo("tests/fixtures/python-malformed-pyproject");

    assert!(graph.components.is_empty());
    assert!(graph.commands.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::MalformedManifest));
}

#[test]
fn go_module_recommends_test_and_build_with_evidence() {
    let graph = inspect_repo("tests/fixtures/go-module-basic");

    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "example.com/go-basic"));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "go test ./..."));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "go build ./..."));
    assert_all_evidence_refs_exist(&graph);
}

#[test]
fn go_test_file_strengthens_test_command_evidence() {
    let graph = inspect_repo("tests/fixtures/go-module-with-tests");

    assert!(graph
        .detected_files
        .iter()
        .any(|file| file.path == "foo_test.go"));
    let test = graph
        .tests
        .iter()
        .find(|test| test.command == "go test ./...")
        .expect("go test target should exist");
    assert!(graph
        .evidence
        .iter()
        .any(|evidence| evidence.id == test.evidence_id && evidence.path == "foo_test.go"));
}

#[test]
fn go_workspace_members_are_parsed_when_simple() {
    let graph = inspect_repo("tests/fixtures/go-workspace-basic");

    assert_eq!(graph.workspaces.len(), 1);
    assert_eq!(
        graph.workspaces[0].members,
        vec!["./apps/api", "./libs/core"]
    );
}

#[test]
fn malformed_go_mod_warns_without_fake_component() {
    let graph = inspect_repo("tests/fixtures/go-malformed-mod");

    assert!(graph.components.is_empty());
    assert!(graph.commands.is_empty());
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::MalformedManifest));
}

#[test]
fn inspect_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args(["inspect", "tests/fixtures/minimal-cargo", "--json"])
        .output()
        .expect("inspect command should run");

    assert!(
        output.status.success(),
        "inspect command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let graph: RepoInspection =
        serde_json::from_slice(&output.stdout).expect("inspect output should be JSON");
    assert_eq!(graph.contract_version, INSPECT_CONTRACT_VERSION);
    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "minimal-cargo"));
}

#[test]
fn inspect_cli_output_has_contract_top_level_fields() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args(["inspect", "tests/fixtures/minimal-cargo", "--json"])
        .output()
        .expect("inspect command should run");

    assert!(output.status.success());

    let json: JsonValue =
        serde_json::from_slice(&output.stdout).expect("inspect output should be JSON");

    for key in [
        "contract_version",
        "repo",
        "detected_files",
        "package_managers",
        "workspaces",
        "components",
        "commands",
        "tests",
        "relationships",
        "evidence",
        "warnings",
    ] {
        assert!(json.get(key).is_some(), "missing top-level key: {key}");
    }
}

#[test]
fn repeated_inspect_output_has_stable_ordering() {
    let first = inspect_repo("tests/fixtures/minimal-cargo");
    let second = inspect_repo("tests/fixtures/minimal-cargo");

    assert_eq!(first.detected_files, second.detected_files);
    assert_eq!(first.package_managers, second.package_managers);
    assert_eq!(first.components, second.components);
    assert_eq!(first.commands, second.commands);
    assert_eq!(first.tests, second.tests);
    assert_eq!(first.relationships, second.relationships);
    assert_eq!(first.evidence, second.evidence);
    assert_eq!(first.warnings, second.warnings);
}

#[test]
fn impact_for_rust_source_recommends_rust_commands() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["src/lib.rs"]);

    assert_eq!(impact.contract_version, IMPACT_CONTRACT_VERSION);
    assert_eq!(impact.status, ImpactStatus::Partial);
    assert_eq!(impact.impact_scope, ImpactScope::Targeted);
    assert_eq!(impact.confidence, ImpactConfidence::Medium);
    let component = impact
        .impacted_components
        .iter()
        .find(|component| component.name == "minimal-cargo")
        .expect("minimal-cargo component should be impacted");
    assert_eq!(component.impact_kind, ImpactKind::Direct);
    assert_eq!(component.distance, Some(0));
    assert!(impact
        .recommended_commands
        .iter()
        .any(|command| command.command == "cargo check"));
    assert_all_impact_facts_have_evidence(&impact);
}

#[test]
fn impact_for_manifest_change_broadens_to_all_repo_commands() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["Cargo.toml"]);

    assert_eq!(impact.status, ImpactStatus::Partial);
    assert_eq!(impact.impact_scope, ImpactScope::Broad);
    assert_eq!(impact.impacted_components.len(), graph.components.len());
    assert!(impact
        .impacted_components
        .iter()
        .all(|component| component.impact_kind == ImpactKind::Broad));
    assert_eq!(impact.recommended_commands.len(), graph.commands.len());
    assert_eq!(impact.recommended_tests.len(), graph.tests.len());
}

#[test]
fn impact_for_test_file_recommends_test_command() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["tests/smoke.rs"]);

    assert!(impact
        .recommended_tests
        .iter()
        .any(|test| test.command == "cargo test" && !test.reason.is_empty()));
}

#[test]
fn impact_for_unknown_file_is_insufficient_evidence() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["docs/unknown.md"]);

    assert_eq!(impact.status, ImpactStatus::InsufficientEvidence);
    assert_eq!(impact.impact_scope, ImpactScope::Unknown);
    assert_eq!(impact.confidence, ImpactConfidence::Insufficient);
    assert!(impact
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::UnmappedChange));
}

#[test]
fn impact_for_python_manifest_is_broad_and_conservative() {
    let graph = inspect_repo("tests/fixtures/python-pytest-evidence");
    let impact = analyze_impact(&graph, ["pyproject.toml"]);

    assert_eq!(impact.status, ImpactStatus::Partial);
    assert_eq!(impact.impact_scope, ImpactScope::Broad);
    assert!(impact
        .impacted_components
        .iter()
        .any(|component| component.name == "python-pytest-evidence"
            && component.impact_kind == ImpactKind::Broad));
    assert!(impact
        .recommended_tests
        .iter()
        .any(|test| test.command == "pytest"));
}

#[test]
fn impact_for_python_ambiguous_test_file_does_not_guess_test_command() {
    let graph = inspect_repo("tests/fixtures/python-tests-ambiguous");
    let impact = analyze_impact(&graph, ["tests/test_example.py"]);

    assert_eq!(impact.status, ImpactStatus::Partial);
    assert!(impact.recommended_tests.is_empty());
    assert!(impact
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::AmbiguousDetection));
}

#[test]
fn impact_for_go_mod_is_broad_and_recommends_go_commands() {
    let graph = inspect_repo("tests/fixtures/go-module-basic");
    let impact = analyze_impact(&graph, ["go.mod"]);

    assert_eq!(impact.impact_scope, ImpactScope::Broad);
    assert!(impact
        .recommended_commands
        .iter()
        .any(|command| command.command == "go test ./..."));
    assert!(impact
        .recommended_commands
        .iter()
        .any(|command| command.command == "go build ./..."));
}

#[test]
fn impact_for_go_test_file_recommends_go_test() {
    let graph = inspect_repo("tests/fixtures/go-module-with-tests");
    let impact = analyze_impact(&graph, ["foo_test.go"]);

    assert_eq!(impact.status, ImpactStatus::Partial);
    assert!(impact
        .recommended_tests
        .iter()
        .any(|test| test.command == "go test ./..."));
}

#[test]
fn impact_for_reverse_dependency_includes_transitive_dependents() {
    let graph = inspect_repo("tests/fixtures/cargo-workspace-deps");
    let impact = analyze_impact(&graph, ["crates/b/src/lib.rs"]);

    assert!(graph
        .relationships
        .iter()
        .any(|relationship| relationship.kind == RelationshipKind::DependsOn));

    let direct = impact
        .impacted_components
        .iter()
        .find(|component| component.name == "b")
        .expect("changed crate should be directly impacted");
    assert_eq!(direct.impact_kind, ImpactKind::Direct);

    let transitive = impact
        .impacted_components
        .iter()
        .find(|component| component.name == "a")
        .expect("dependent crate should be transitively impacted");
    assert_eq!(transitive.impact_kind, ImpactKind::Transitive);
    assert_eq!(transitive.distance, Some(1));
    assert!(transitive.evidence_ids.len() >= 2);
}

#[test]
fn impact_does_not_claim_transitive_without_dependency_edges() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["src/lib.rs"]);

    assert!(!impact
        .impacted_components
        .iter()
        .any(|component| component.impact_kind == ImpactKind::Transitive));
    assert!(impact
        .limitations
        .iter()
        .any(|limitation| limitation.contains("No depends_on relationships")));
}

#[test]
fn impact_recommendations_include_rank_reason_confidence_and_evidence() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let impact = analyze_impact(&graph, ["src/lib.rs"]);

    let command = impact
        .recommended_commands
        .iter()
        .find(|command| command.command == "cargo check")
        .expect("cargo check should be recommended");
    assert!(command.rank > 0);
    assert!(!command.reason.is_empty());
    assert_eq!(command.confidence, ImpactConfidence::Medium);
    assert!(!command.evidence_ids.is_empty());
}

#[test]
fn repeated_impact_output_has_stable_ordering() {
    let graph = inspect_repo("tests/fixtures/minimal-cargo");
    let first = analyze_impact(&graph, ["src/lib.rs", "Cargo.toml"]);
    let second = analyze_impact(&graph, ["Cargo.toml", "src/lib.rs"]);

    assert_eq!(first.changed_files, second.changed_files);
    assert_eq!(first.impacted_components, second.impacted_components);
    assert_eq!(first.impacted_workspaces, second.impacted_workspaces);
    assert_eq!(first.recommended_commands, second.recommended_commands);
    assert_eq!(first.recommended_tests, second.recommended_tests);
    assert_eq!(first.warnings, second.warnings);
}

#[test]
fn impact_with_malformed_manifest_keeps_warning_and_does_not_panic() {
    let graph = inspect_repo("tests/fixtures/malformed-manifest");
    let impact = analyze_impact(&graph, ["package.json"]);

    assert_eq!(impact.contract_version, IMPACT_CONTRACT_VERSION);
    assert!(impact
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::MalformedManifest));
}

#[test]
fn impact_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args(["impact", "src/main.rs", "Cargo.toml", "--json"])
        .output()
        .expect("impact command should run");

    assert!(
        output.status.success(),
        "impact command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: ImpactReport =
        serde_json::from_slice(&output.stdout).expect("impact output should be JSON");
    assert_eq!(report.contract_version, IMPACT_CONTRACT_VERSION);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::RepoGraphOnly));
}

#[test]
fn where_to_edit_remains_insufficient_evidence_placeholder() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args([
            "where-to-edit",
            "change login validation copy",
            "--profile=strict",
            "--json",
        ])
        .output()
        .expect("where-to-edit command should run");

    assert!(output.status.success());

    let json: JsonValue =
        serde_json::from_slice(&output.stdout).expect("where-to-edit output should be JSON");
    assert_eq!(json["ok"], false);
    assert_eq!(json["status"], "insufficient_evidence");
    assert!(json["data"]["files"].as_array().is_some_and(Vec::is_empty));
    assert!(json["data"]["symbols"]
        .as_array()
        .is_some_and(Vec::is_empty));
}

#[test]
fn symbol_graph_extracts_top_level_rust_symbols() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-basic");

    assert_eq!(graph.contract_version, SYMBOLS_CONTRACT_VERSION);
    assert!(graph
        .source_files
        .iter()
        .any(|file| file.path == "src/lib.rs" && file.parse_status == ParseStatus::Ok));
    for (kind, name) in [
        (SymbolKind::Function, "top_level_function"),
        (SymbolKind::Struct, "Widget"),
        (SymbolKind::Enum, "Mode"),
        (SymbolKind::Trait, "Runner"),
        (SymbolKind::TypeAlias, "WidgetId"),
        (SymbolKind::Const, "DEFAULT_LIMIT"),
        (SymbolKind::Static, "GLOBAL_LIMIT"),
        (SymbolKind::Module, "nested"),
    ] {
        assert!(
            graph
                .symbols
                .iter()
                .any(|symbol| symbol.kind == kind && symbol.name == name),
            "missing symbol {kind:?} {name}"
        );
    }
    assert!(graph
        .symbols
        .iter()
        .any(|symbol| symbol.kind == SymbolKind::ImplBlock && symbol.name.starts_with("impl@")));
    assert_symbol_graph_evidence_refs_exist(&graph);
}

#[test]
fn symbol_graph_does_not_extract_nested_functions_as_top_level() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-basic");

    assert!(!graph
        .symbols
        .iter()
        .any(|symbol| symbol.name == "nested_helper"));
    assert!(!graph.symbols.iter().any(|symbol| symbol.name == "new"));
}

#[test]
fn malformed_rust_source_produces_symbol_warning_without_panic() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-malformed");

    assert!(graph
        .source_files
        .iter()
        .any(|file| file.path == "src/lib.rs" && file.parse_status == ParseStatus::Error));
    assert!(graph.symbols.is_empty());
    assert!(graph.warnings.iter().any(|warning| warning.category
        == SymbolWarningCategory::ParseError
        && warning.evidence_id.is_some()));
    assert_symbol_graph_evidence_refs_exist(&graph);
}

#[test]
fn symbol_graph_ignores_generated_and_dependency_directories() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-ignored");

    assert!(graph.symbols.iter().any(|symbol| symbol.name == "visible"));
    assert!(!graph
        .source_files
        .iter()
        .any(|file| file.path.contains("target/") || file.path.contains("node_modules/")));
    assert!(!graph.symbols.iter().any(
        |symbol| symbol.name == "ignored_target_symbol" || symbol.name == "ignored_node_symbol"
    ));
}

#[test]
fn symbol_graph_ids_and_order_are_deterministic() {
    let first = build_symbol_graph("tests/fixtures/rust-symbols-basic");
    let second = build_symbol_graph("tests/fixtures/rust-symbols-basic");

    assert_eq!(first.source_files, second.source_files);
    assert_eq!(first.symbols, second.symbols);
    assert_eq!(first.evidence, second.evidence);
    assert_eq!(first.warnings, second.warnings);
    assert!(first
        .symbols
        .iter()
        .all(|symbol| symbol.id.starts_with("symbol-")));
}

#[test]
fn every_symbol_graph_source_file_and_symbol_has_valid_evidence() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-basic");

    assert!(symbol_graph_evidence_valid(&graph));
    assert_symbol_graph_evidence_refs_exist(&graph);
}

#[test]
fn symbols_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args(["symbols", "tests/fixtures/rust-symbols-basic", "--json"])
        .output()
        .expect("symbols command should run");

    assert!(
        output.status.success(),
        "symbols command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let graph: SymbolGraph =
        serde_json::from_slice(&output.stdout).expect("symbols output should be JSON");
    assert_eq!(graph.contract_version, SYMBOLS_CONTRACT_VERSION);
    assert!(graph
        .symbols
        .iter()
        .any(|symbol| symbol.name == "top_level_function"));
}

#[test]
fn source_evidence_exact_symbol_returns_evidence_candidate() {
    let bundle =
        build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "top_level_function");

    assert_eq!(bundle.contract_version, SOURCE_EVIDENCE_CONTRACT_VERSION);
    assert_eq!(bundle.status, BundleStatus::Partial);
    assert_eq!(bundle.confidence, BundleConfidence::High);
    assert!(bundle
        .candidate_symbols
        .iter()
        .any(|symbol| symbol.name == "top_level_function"
            && symbol.kind == SymbolKind::Function
            && !symbol.evidence_ids.is_empty()));
    assert!(bundle
        .candidate_files
        .iter()
        .any(|file| file.path == "src/lib.rs" && !file.evidence_ids.is_empty()));
    assert!(bundle.source_context_selectors.iter().any(|selector| {
        selector.selector_kind == SourceContextSelectorKind::SymbolId
            && selector.symbol_name.as_deref() == Some("top_level_function")
            && selector.file_path == "src/lib.rs"
            && !selector.evidence_ids.is_empty()
    }));
    assert!(source_evidence_bundle_evidence_valid(&bundle));
    assert!(bundle
        .repo_context
        .iter()
        .any(|context| context.role == RepoContextRole::ContainingComponent));
    assert!(bundle
        .refusal_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("localization_not_supported")));
}

#[test]
fn source_evidence_file_path_query_returns_candidate_file() {
    let bundle = build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "src/lib.rs");

    assert_eq!(bundle.status, BundleStatus::Partial);
    assert!(bundle
        .candidate_files
        .iter()
        .any(|file| file.path == "src/lib.rs"));
    assert!(bundle.source_context_selectors.iter().any(|selector| {
        selector.selector_kind == SourceContextSelectorKind::File
            && selector.file_path == "src/lib.rs"
    }));
    assert!(source_evidence_bundle_evidence_valid(&bundle));
}

#[test]
fn source_evidence_no_match_returns_insufficient_evidence() {
    let bundle =
        build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "no_such_symbol_zz");

    assert_eq!(bundle.status, BundleStatus::InsufficientEvidence);
    assert_eq!(bundle.confidence, BundleConfidence::Insufficient);
    assert!(bundle.candidate_files.is_empty());
    assert!(bundle.candidate_symbols.is_empty());
    assert!(bundle.source_context_selectors.is_empty());
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.category == BundleWarningCategory::NoMatchingSourceSymbols));
    assert!(bundle
        .refusal_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("no_source_match")));
    assert!(source_evidence_bundle_evidence_valid(&bundle));
}

#[test]
fn source_evidence_ambiguous_query_returns_partial_with_warning() {
    let bundle = build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "widget");

    assert_eq!(bundle.status, BundleStatus::Partial);
    assert!(bundle.candidate_symbols.len() > 1);
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.category == BundleWarningCategory::MultipleCandidates));
}

#[test]
fn source_evidence_broad_query_truncates_candidates() {
    let bundle = build_source_evidence_bundle(".", "repo");

    assert_eq!(bundle.status, BundleStatus::Partial);
    assert!(bundle.candidate_files.len() <= 8);
    assert!(bundle.candidate_symbols.len() <= 12);
    assert!(bundle.repo_context.len() <= 12);
    assert!(bundle.source_context_selectors.len() <= 12);
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.category == BundleWarningCategory::CandidateLimitExceeded));
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.category == BundleWarningCategory::SelectorHintLimitExceeded));
}

#[test]
fn source_evidence_malformed_source_warns_without_panic() {
    let bundle = build_source_evidence_bundle("tests/fixtures/rust-symbols-malformed", "broken");

    assert_eq!(bundle.status, BundleStatus::InsufficientEvidence);
    assert!(bundle
        .warnings
        .iter()
        .any(|warning| warning.category == BundleWarningCategory::ParseErrorPresent));
    assert!(source_evidence_bundle_evidence_valid(&bundle));
}

#[test]
fn source_evidence_ignored_directories_do_not_produce_candidates() {
    let bundle = build_source_evidence_bundle(
        "tests/fixtures/rust-symbols-ignored",
        "ignored_target_symbol",
    );

    assert_eq!(bundle.status, BundleStatus::InsufficientEvidence);
    assert!(!bundle
        .candidate_files
        .iter()
        .any(|file| file.path.contains("target/") || file.path.contains("node_modules/")));
    assert!(!bundle
        .candidate_symbols
        .iter()
        .any(|symbol| symbol.name == "ignored_target_symbol"));
}

#[test]
fn source_evidence_output_is_deterministic() {
    let first = build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "widget");
    let second = build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "widget");

    assert_eq!(first, second);
}

#[test]
fn source_evidence_selector_hint_can_feed_source_context_manually() {
    let bundle =
        build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "top_level_function");
    let selector = bundle
        .source_context_selectors
        .iter()
        .find(|selector| selector.selector_kind == SourceContextSelectorKind::SymbolId)
        .expect("symbol selector hint should exist");
    let symbol_id = selector
        .symbol_id
        .as_ref()
        .expect("symbol selector should include symbol_id");

    let report = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::SymbolId {
            symbol_id: symbol_id.clone(),
        }],
    );

    assert_eq!(report.status, SourceContextStatus::Ok);
    assert!(report
        .slices
        .iter()
        .any(|slice| slice.symbol_name.as_deref() == Some("top_level_function")));
}

#[test]
fn source_evidence_output_has_no_edit_target_language() {
    let bundle = build_source_evidence_bundle("tests/fixtures/rust-symbols-basic", "widget");
    let json = serde_json::to_string(&bundle).expect("bundle should serialize");

    let value: JsonValue = serde_json::from_str(&json).expect("bundle should be JSON");
    assert!(value.get("source_context_selectors").is_some());
    assert!(value.get("slices").is_none());

    for forbidden in [
        "edit this",
        "edit here",
        "target_edit",
        "edit_location",
        "apply patch",
        "change this",
        "correct edit location",
    ] {
        assert!(
            !json.contains(forbidden),
            "bundle output should not contain edit-target phrase: {forbidden}"
        );
    }
}

#[test]
fn source_evidence_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args([
            "source-evidence",
            "top_level_function",
            "--repo",
            "tests/fixtures/rust-symbols-basic",
            "--json",
        ])
        .output()
        .expect("source-evidence command should run");

    assert!(
        output.status.success(),
        "source-evidence command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle: SourceEvidenceBundle =
        serde_json::from_slice(&output.stdout).expect("source-evidence output should be JSON");
    assert_eq!(bundle.contract_version, SOURCE_EVIDENCE_CONTRACT_VERSION);
    assert!(bundle
        .candidate_symbols
        .iter()
        .any(|symbol| symbol.name == "top_level_function"));
}

#[test]
fn source_context_file_selector_returns_bounded_slice_with_evidence() {
    let report = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::File {
            path: "src/lib.rs".to_string(),
            line_range: Some(LineRange {
                start_line: 1,
                end_line: 6,
            }),
        }],
    );

    assert_eq!(report.contract_version, SOURCE_CONTEXT_CONTRACT_VERSION);
    assert_eq!(report.status, SourceContextStatus::Ok);
    assert_eq!(report.slices.len(), 1);
    let slice = &report.slices[0];
    assert_eq!(slice.file_path, "src/lib.rs");
    assert_eq!(slice.start_line, 1);
    assert_eq!(slice.end_line, 6);
    assert!(slice.text.contains("pub fn top_level_function"));
    assert!(!slice.text.contains("nested_helper"));
    assert!(!slice.evidence_ids.is_empty());
    assert!(source_context_evidence_valid(&report));
}

#[test]
fn source_context_symbol_id_selector_returns_symbol_slice() {
    let graph = build_symbol_graph("tests/fixtures/rust-symbols-basic");
    let symbol = graph
        .symbols
        .iter()
        .find(|symbol| symbol.name == "top_level_function")
        .expect("fixture should contain top_level_function");
    let report = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::SymbolId {
            symbol_id: symbol.id.clone(),
        }],
    );

    assert_eq!(report.status, SourceContextStatus::Ok);
    assert!(report.slices.iter().any(|slice| {
        slice.symbol_id.as_deref() == Some(symbol.id.as_str())
            && slice.symbol_name.as_deref() == Some("top_level_function")
            && slice.context_after_lines <= 3
            && slice.text.contains("pub fn top_level_function")
    }));
    assert!(source_context_evidence_valid(&report));
}

#[test]
fn source_context_missing_ignored_and_outside_paths_warn_without_slices() {
    let missing = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::File {
            path: "src/missing.rs".to_string(),
            line_range: None,
        }],
    );
    assert_eq!(missing.status, SourceContextStatus::InsufficientEvidence);
    assert!(missing
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::MissingFile));

    let ignored = build_source_context_report(
        "tests/fixtures/rust-symbols-ignored",
        vec![SourceContextSelector::File {
            path: "target/generated.rs".to_string(),
            line_range: None,
        }],
    );
    assert_eq!(ignored.status, SourceContextStatus::InsufficientEvidence);
    assert!(ignored
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::IgnoredPath));

    let outside = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::File {
            path: "../Cargo.toml".to_string(),
            line_range: None,
        }],
    );
    assert_eq!(outside.status, SourceContextStatus::InsufficientEvidence);
    assert!(outside
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::PathOutsideRepo));
}

#[test]
fn source_context_large_and_non_utf8_files_warn_without_panic() {
    let root = std::env::temp_dir().join(format!(
        "code-intel-kernel-source-context-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).expect("source directory should be created");
    let large_text = (0..120)
        .map(|index| format!("pub fn function_{index}() {{}}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(root.join("src").join("large.rs"), large_text)
        .expect("large source should be written");
    std::fs::write(root.join("src").join("bad.rs"), vec![0xff, 0xfe, 0xfd])
        .expect("non-UTF8 source should be written");

    let large = build_source_context_report(
        &root,
        vec![SourceContextSelector::File {
            path: "src/large.rs".to_string(),
            line_range: Some(LineRange {
                start_line: 1,
                end_line: 120,
            }),
        }],
    );
    assert_eq!(large.slices.len(), 1);
    assert!(large.slices[0].truncated);
    assert!(large.slices[0].end_line - large.slices[0].start_line < 80);
    assert!(large
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::SliceTruncated));

    let non_utf8 = build_source_context_report(
        &root,
        vec![SourceContextSelector::File {
            path: "src/bad.rs".to_string(),
            line_range: None,
        }],
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(non_utf8.status, SourceContextStatus::InsufficientEvidence);
    assert!(non_utf8
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::NonUtf8File));
}

#[cfg(unix)]
#[test]
fn source_context_symlink_selector_is_blocked() {
    let root = std::env::temp_dir().join(format!(
        "code-intel-kernel-source-context-symlink-{}",
        std::process::id()
    ));
    let outside = std::env::temp_dir().join(format!(
        "code-intel-kernel-source-context-outside-{}.rs",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_file(&outside);
    std::fs::create_dir_all(root.join("src")).expect("source directory should be created");
    std::fs::write(&outside, "pub fn outside() {}\n").expect("outside file should be written");
    std::os::unix::fs::symlink(&outside, root.join("src").join("link.rs"))
        .expect("symlink should be created");

    let report = build_source_context_report(
        &root,
        vec![SourceContextSelector::File {
            path: "src/link.rs".to_string(),
            line_range: None,
        }],
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_file(&outside);

    assert_eq!(report.status, SourceContextStatus::InsufficientEvidence);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.category == SourceContextWarningCategory::SymlinkIgnored));
}

#[test]
fn source_context_output_is_deterministic_and_has_no_edit_target_language() {
    let first = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::File {
            path: "src/lib.rs".to_string(),
            line_range: Some(LineRange {
                start_line: 1,
                end_line: 12,
            }),
        }],
    );
    let second = build_source_context_report(
        "tests/fixtures/rust-symbols-basic",
        vec![SourceContextSelector::File {
            path: "src/lib.rs".to_string(),
            line_range: Some(LineRange {
                start_line: 1,
                end_line: 12,
            }),
        }],
    );
    assert_eq!(first, second);
    assert!(source_context_evidence_valid(&first));

    let json = serde_json::to_string(&first).expect("source context should serialize");
    for forbidden in [
        "edit this",
        "edit here",
        "target_edit",
        "edit_location",
        "apply patch",
        "change this",
        "correct edit location",
    ] {
        assert!(
            !json.contains(forbidden),
            "source-context output should not contain edit-target phrase: {forbidden}"
        );
    }
}

#[test]
fn source_context_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args([
            "source-context",
            "--file",
            "src/lib.rs",
            "--lines",
            "1:8",
            "--repo",
            "tests/fixtures/rust-symbols-basic",
            "--json",
        ])
        .output()
        .expect("source-context command should run");

    assert!(
        output.status.success(),
        "source-context command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: SourceContextReport =
        serde_json::from_slice(&output.stdout).expect("source-context output should be JSON");
    assert_eq!(report.contract_version, SOURCE_CONTEXT_CONTRACT_VERSION);
    assert!(report
        .slices
        .iter()
        .any(|slice| slice.file_path == "src/lib.rs"));
}

#[test]
fn evaluator_loads_fixture_cases() {
    let cases = load_eval_cases("tests/eval/cases").expect("eval cases should load");

    assert!(cases.len() >= 29);
    assert!(cases
        .iter()
        .any(|case| case.name == "cargo_workspace_dependency_impact"));
    assert!(cases
        .iter()
        .any(|case| case.name == "rust_symbols_basic_symbols"));
    assert!(cases.iter().any(|case| case.kind == EvalCaseKind::Inspect));
    assert!(cases.iter().any(|case| case.kind == EvalCaseKind::Impact));
    assert!(cases.iter().any(|case| case.kind == EvalCaseKind::Symbols));
    assert!(cases
        .iter()
        .any(|case| case.kind == EvalCaseKind::SourceEvidence));
    assert!(cases
        .iter()
        .any(|case| case.kind == EvalCaseKind::SourceContext));
}

#[test]
fn evaluator_report_json_includes_metrics_and_current_cases_pass() {
    let report = run_fixture_evaluation("tests/eval/cases").expect("eval report should run");
    let json = serde_json::to_value(&report).expect("eval report should serialize");

    assert_eq!(report.eval_contract_version, EVAL_CONTRACT_VERSION);
    assert_eq!(report.failed_cases, 0, "{:#?}", report.failures);
    assert!(report.total_cases >= 29);
    assert!(report.inspect_cases > 0);
    assert!(report.impact_cases > 0);
    assert!(report.symbol_cases > 0);
    assert!(report.source_evidence_cases > 0);
    assert!(report.source_context_cases > 0);
    assert_eq!(report.metrics.evidence_coverage_pass_rate, 1.0);
    assert_eq!(report.metrics.deterministic_output_pass_rate, 1.0);
    assert!(json.get("metrics").is_some());
    assert!(json.get("symbol_cases").is_some());
    assert!(json.get("source_evidence_cases").is_some());
    assert!(json.get("source_context_cases").is_some());
}

#[test]
fn eval_fixtures_cli_output_is_valid_json() {
    let binary = env!("CARGO_BIN_EXE_code-intel");
    let output = Command::new(binary)
        .args(["eval-fixtures", "--json"])
        .output()
        .expect("eval-fixtures command should run");

    assert!(
        output.status.success(),
        "eval-fixtures command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json: JsonValue =
        serde_json::from_slice(&output.stdout).expect("eval-fixtures output should be JSON");
    assert_eq!(json["eval_contract_version"], EVAL_CONTRACT_VERSION);
    assert_eq!(json["failed_cases"], 0);
    assert!(json["symbol_cases"].as_u64().is_some_and(|count| count > 0));
    assert!(json["source_evidence_cases"]
        .as_u64()
        .is_some_and(|count| count > 0));
    assert!(json["source_context_cases"]
        .as_u64()
        .is_some_and(|count| count > 0));
    assert!(json.get("metrics").is_some());
}

#[test]
fn evaluator_report_detects_evidence_and_deterministic_checks() {
    let report = run_fixture_evaluation("tests/eval/cases").expect("eval report should run");

    assert_eq!(report.metrics.evidence_coverage_pass_rate, 1.0);
    assert_eq!(report.metrics.deterministic_output_pass_rate, 1.0);
    assert_eq!(report.metrics.false_narrow_count, 0);
    assert_eq!(report.metrics.false_broad_count, 0);
}

#[test]
fn symbol_eval_cases_cover_parse_warning_and_ignored_paths() {
    let report = run_fixture_evaluation("tests/eval/cases").expect("eval report should run");

    for case_name in [
        "rust_symbols_basic_symbols",
        "rust_symbols_malformed_warning",
        "rust_symbols_ignored_paths",
    ] {
        let result = report
            .cases
            .iter()
            .find(|case| case.name == case_name)
            .expect("symbol eval case should be present");
        assert!(result.passed, "{case_name} failed: {:?}", result.failures);
        assert_eq!(result.kind, EvalCaseKind::Symbols);
    }
}

#[test]
fn source_evidence_bundle_contract_docs_exist() {
    let contract = std::fs::read_to_string("docs/source-evidence-bundle.md")
        .expect("SourceEvidenceBundle contract should exist");
    let checklist = std::fs::read_to_string("docs/localization-readiness-checklist.md")
        .expect("localization readiness checklist should exist");

    assert!(contract.contains("contract_version"));
    assert!(contract.contains("SourceEvidenceBundle"));
    assert!(checklist.contains("not_ready_for_confident_localization"));
}

#[test]
fn source_evidence_eval_cases_pass() {
    let report = run_fixture_evaluation("tests/eval/cases").expect("eval report should run");

    for case_name in [
        "source_evidence_function_match",
        "source_evidence_file_match",
        "source_evidence_no_match",
        "source_evidence_broad_query_limit",
        "source_evidence_malformed_source",
    ] {
        let result = report
            .cases
            .iter()
            .find(|case| case.name == case_name)
            .expect("source evidence eval case should be present");
        assert!(result.passed, "{case_name} failed: {:?}", result.failures);
        assert_eq!(result.kind, EvalCaseKind::SourceEvidence);
    }
}

#[test]
fn source_context_eval_cases_pass() {
    let report = run_fixture_evaluation("tests/eval/cases").expect("eval report should run");

    for case_name in [
        "source_context_file_slice",
        "source_context_symbol_slice",
        "source_context_missing_file",
        "source_context_ignored_path",
    ] {
        let result = report
            .cases
            .iter()
            .find(|case| case.name == case_name)
            .expect("source context eval case should be present");
        assert!(result.passed, "{case_name} failed: {:?}", result.failures);
        assert_eq!(result.kind, EvalCaseKind::SourceContext);
    }
}

#[test]
fn evaluator_reports_deliberate_false_narrow() {
    let report = evaluate_cases(vec![EvalCase {
        name: "deliberate_false_narrow".to_string(),
        fixture: "tests/fixtures/minimal-cargo".to_string(),
        kind: EvalCaseKind::Inspect,
        query: String::new(),
        selector_file: String::new(),
        selector_symbol_id: String::new(),
        selector_lines: None,
        changed_files: Vec::new(),
        expect: EvalExpect {
            components_contains: vec!["missing-component".to_string()],
            ..EvalExpect::default()
        },
    }])
    .expect("deliberately failing eval case should still produce a report");

    assert_eq!(report.failed_cases, 1);
    assert_eq!(report.metrics.false_narrow_count, 1);
    assert!(report
        .failures
        .iter()
        .any(|failure| failure.category == "false_narrow"));
}

#[test]
fn evaluator_reports_deliberate_false_broad() {
    let report = evaluate_cases(vec![EvalCase {
        name: "deliberate_false_broad".to_string(),
        fixture: "tests/fixtures/minimal-cargo".to_string(),
        kind: EvalCaseKind::Impact,
        query: String::new(),
        selector_file: String::new(),
        selector_symbol_id: String::new(),
        selector_lines: None,
        changed_files: vec!["Cargo.toml".to_string()],
        expect: EvalExpect {
            max_impacted_components: Some(0),
            ..EvalExpect::default()
        },
    }])
    .expect("deliberately broad eval case should still produce a report");

    assert_eq!(report.failed_cases, 1);
    assert_eq!(report.metrics.false_broad_count, 1);
    assert!(report
        .failures
        .iter()
        .any(|failure| failure.category == "false_broad"));
}

fn assert_all_evidence_refs_exist(graph: &RepoInspection) {
    let evidence_ids = graph
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for package_manager in &graph.package_managers {
        assert!(!package_manager.evidence_id.is_empty());
        assert!(evidence_ids.contains(package_manager.evidence_id.as_str()));
    }

    for workspace in &graph.workspaces {
        assert!(!workspace.evidence_id.is_empty());
        assert!(evidence_ids.contains(workspace.evidence_id.as_str()));
    }

    for detected_file in &graph.detected_files {
        assert!(!detected_file.evidence_id.is_empty());
        assert!(evidence_ids.contains(detected_file.evidence_id.as_str()));
    }

    for component in &graph.components {
        assert!(!component.evidence_id.is_empty());
        assert!(evidence_ids.contains(component.evidence_id.as_str()));
    }

    for command in &graph.commands {
        assert!(!command.evidence_id.is_empty());
        assert!(evidence_ids.contains(command.evidence_id.as_str()));
    }

    for test in &graph.tests {
        assert!(!test.evidence_id.is_empty());
        assert!(evidence_ids.contains(test.evidence_id.as_str()));
    }

    for relationship in &graph.relationships {
        assert!(!relationship.evidence_id.is_empty());
        assert!(evidence_ids.contains(relationship.evidence_id.as_str()));
    }

    for warning in &graph.warnings {
        if let Some(evidence_id) = &warning.evidence_id {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }
}

fn assert_all_graph_facts_have_evidence(graph: &RepoInspection) {
    assert!(graph
        .package_managers
        .iter()
        .all(|fact| !fact.evidence_id.is_empty()));
    assert!(graph
        .components
        .iter()
        .all(|fact| !fact.evidence_id.is_empty()));
    assert!(graph
        .commands
        .iter()
        .all(|fact| !fact.evidence_id.is_empty()));
    assert!(graph.tests.iter().all(|fact| !fact.evidence_id.is_empty()));
}

fn assert_all_impact_facts_have_evidence(report: &ImpactReport) {
    let evidence_ids = report
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for component in &report.impacted_components {
        assert!(!component.evidence_ids.is_empty());
        for evidence_id in &component.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }

    for workspace in &report.impacted_workspaces {
        assert!(!workspace.evidence_ids.is_empty());
        for evidence_id in &workspace.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }

    for command in &report.recommended_commands {
        assert!(!command.evidence_ids.is_empty());
        for evidence_id in &command.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }

    for test in &report.recommended_tests {
        assert!(!test.evidence_ids.is_empty());
        for evidence_id in &test.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }
}

fn assert_all_warnings_are_structured(graph: &RepoInspection) {
    for warning in &graph.warnings {
        assert!(warning.id.starts_with("warning-"));
        assert!(!warning.message.is_empty());
    }
}

fn assert_symbol_graph_evidence_refs_exist(graph: &SymbolGraph) {
    let evidence_ids = graph
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for source_file in &graph.source_files {
        assert!(!source_file.evidence_ids.is_empty());
        for evidence_id in &source_file.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }

    for symbol in &graph.symbols {
        assert!(!symbol.evidence_ids.is_empty());
        for evidence_id in &symbol.evidence_ids {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }

    for warning in &graph.warnings {
        if let Some(evidence_id) = &warning.evidence_id {
            assert!(evidence_ids.contains(evidence_id.as_str()));
        }
    }
}
