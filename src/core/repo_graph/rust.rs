use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

use super::types::*;
use super::{manifest_warning_category, normalize_path, read_toml, stable_id, RepoGraphBuilder};

struct CargoWorkspaceMember {
    relative_manifest: PathBuf,
    package_name: String,
    component_id: String,
    manifest: TomlValue,
}

struct CargoDependency {
    name: String,
    field: String,
    path_dependency: bool,
}

pub(super) fn detect_rust(root: &Path, builder: &mut RepoGraphBuilder) {
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

                let package_name = manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
                    .and_then(TomlValue::as_str);

                if let Some(name) = package_name {
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
                        .collect::<BTreeSet<_>>()
                        .into_iter()
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
                            workspace_members.clone(),
                            evidence_id.clone(),
                        );
                        detect_cargo_workspace_members(
                            root,
                            &workspace_members,
                            &evidence_id,
                            builder,
                        );
                    }
                }

                detect_cargo_targets(root, &manifest, package_name, builder);
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

fn detect_cargo_workspace_members(
    root: &Path,
    workspace_members: &[String],
    workspace_evidence_id: &str,
    builder: &mut RepoGraphBuilder,
) {
    let mut members = Vec::<CargoWorkspaceMember>::new();

    for member in workspace_members {
        let relative_manifest = PathBuf::from(member).join("Cargo.toml");
        let absolute_manifest = root.join(&relative_manifest);

        if !absolute_manifest.exists() {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::UnsupportedPattern,
                "Cargo workspace member was listed, but its Cargo.toml was not found.",
                Some(&relative_manifest),
                Some(workspace_evidence_id.to_string()),
            );
            continue;
        }

        let manifest_evidence = builder.add_detected_file(
            &relative_manifest,
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Cargo workspace member manifest detected.",
        );

        match read_toml(&absolute_manifest) {
            Ok(manifest) => {
                let Some(package_name) = manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
                    .and_then(TomlValue::as_str)
                    .map(str::to_string)
                else {
                    builder.add_warning(
                        DetectionSeverity::Warning,
                        DetectionCategory::UnsupportedPattern,
                        "Cargo workspace member manifest did not define package.name.",
                        Some(&relative_manifest),
                        Some(manifest_evidence.clone()),
                    );
                    continue;
                };

                let component_id = stable_id("component-rust-crate", &package_name);
                let package_name_field = format!("{}/package.name", member);
                let component_evidence = builder.add_evidence(
                    &relative_manifest,
                    "manifest",
                    Some(&package_name_field),
                    "Cargo workspace member package name.",
                );
                builder.add_component(
                    &component_id,
                    &package_name,
                    "rust_crate",
                    member,
                    vec![
                        normalize_path(&relative_manifest),
                        format!("{member}/src/**"),
                        format!("{member}/tests/**"),
                    ],
                    component_evidence.clone(),
                );
                builder.add_relationship(
                    RelationshipKind::BelongsToWorkspace,
                    &component_id,
                    "workspace-cargo",
                    component_evidence.clone(),
                );
                builder.add_relationship(
                    RelationshipKind::UsesPackageManager,
                    &component_id,
                    "package-manager-cargo",
                    manifest_evidence,
                );

                members.push(CargoWorkspaceMember {
                    relative_manifest,
                    package_name,
                    component_id,
                    manifest,
                });
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(&relative_manifest),
                Some(manifest_evidence),
            ),
        }
    }

    let component_by_name = members
        .iter()
        .map(|member| (member.package_name.as_str(), member.component_id.as_str()))
        .collect::<BTreeMap<_, _>>();

    for member in &members {
        for dependency in cargo_dependencies(&member.manifest) {
            if !dependency.path_dependency {
                continue;
            }
            let Some(dependency_component_id) = component_by_name.get(dependency.name.as_str())
            else {
                continue;
            };

            let evidence_id = builder.add_evidence(
                &member.relative_manifest,
                "manifest",
                Some(&dependency.field),
                "Cargo workspace path dependency.",
            );
            builder.add_relationship(
                RelationshipKind::DependsOn,
                &member.component_id,
                dependency_component_id,
                evidence_id,
            );
        }
    }
}

fn detect_cargo_targets(
    root: &Path,
    manifest: &TomlValue,
    package_name: Option<&str>,
    builder: &mut RepoGraphBuilder,
) {
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
    } else if root.join("src/lib.rs").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("src/lib.rs"),
            DetectedFileKind::SourceHint,
            "source_hint",
            None,
            "Cargo default library target source detected.",
        );
        builder.add_component(
            "component-rust-lib",
            "lib",
            "rust_lib_target",
            ".",
            vec!["src/lib.rs".to_string()],
            evidence_id,
        );
    }

    let mut explicit_bin_detected = false;
    if let Some(bin_targets) = manifest.get("bin").and_then(TomlValue::as_array) {
        for (index, bin) in bin_targets.iter().enumerate() {
            explicit_bin_detected = true;
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

    if !explicit_bin_detected && root.join("src/main.rs").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("src/main.rs"),
            DetectedFileKind::SourceHint,
            "source_hint",
            None,
            "Cargo default binary target source detected.",
        );
        builder.add_component(
            "component-rust-bin-0",
            package_name.unwrap_or("bin"),
            "rust_bin_target",
            ".",
            vec!["src/main.rs".to_string()],
            evidence_id,
        );
    }
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

fn cargo_dependencies(manifest: &TomlValue) -> Vec<CargoDependency> {
    let mut dependencies = Vec::new();
    for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(section) = manifest.get(section_name).and_then(TomlValue::as_table) else {
            continue;
        };

        for (name, value) in section {
            dependencies.push(CargoDependency {
                name: name.to_string(),
                field: format!("{section_name}.{name}"),
                path_dependency: value
                    .as_table()
                    .is_some_and(|dependency| dependency.contains_key("path")),
            });
        }
    }

    dependencies.sort_by(|a, b| a.field.cmp(&b.field));
    dependencies
}
