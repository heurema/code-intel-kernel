use code_intel_kernel::{
    create_evidence_bundle, inspect_repo, DetectionCategory, DetectionSeverity, EvidenceRequest,
    KernelProfile, RepoInspection, INSPECT_CONTRACT_VERSION,
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

    assert_eq!(graph.contract_version, "0.1");
    assert!(graph
        .detected_files
        .iter()
        .any(|file| file.path == "Cargo.toml"));
    assert!(graph
        .components
        .iter()
        .any(|component| component.name == "minimal-cargo"));
    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "cargo test"));
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
    assert!(graph
        .tests
        .iter()
        .any(|test| test.command == "python -m pytest"));
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
}

#[test]
fn detects_generic_makefile_fixture() {
    let graph = inspect_repo("tests/fixtures/generic-make");

    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "make test"));
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::PartialSupport));
}

#[test]
fn detects_generic_justfile_fixture() {
    let graph = inspect_repo("tests/fixtures/generic-just");

    assert!(graph
        .commands
        .iter()
        .any(|command| command.command == "just test"));
    assert!(graph
        .warnings
        .iter()
        .any(|warning| warning.category == DetectionCategory::PartialSupport));
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
        "evidence",
        "warnings",
    ] {
        assert!(json.get(key).is_some(), "missing top-level key: {key}");
    }
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

fn assert_all_warnings_are_structured(graph: &RepoInspection) {
    for warning in &graph.warnings {
        assert!(warning.id.starts_with("warning-"));
        assert!(!warning.message.is_empty());
    }
}
