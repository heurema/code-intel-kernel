use serde_json::Value as JsonValue;
use std::fs;
use std::path::Path;

use super::helpers::normalize_path;
use super::types::*;
use super::{manifest_warning_category, RepoGraphBuilder};

pub(super) fn detect_node(root: &Path, builder: &mut RepoGraphBuilder) {
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

fn read_json(path: &Path) -> Result<JsonValue, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {error}", normalize_path(path)))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Failed to parse {}: {error}", normalize_path(path)))
}
