use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::*;
use super::{stable_id, RepoGraphBuilder};

pub(super) fn detect_generic(root: &Path, builder: &mut RepoGraphBuilder) {
    let makefile = root.join("Makefile");
    if makefile.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("Makefile"),
            DetectedFileKind::BuildConfig,
            "build_config",
            None,
            "Makefile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Make, "make", evidence_id.clone());
        builder.add_component(
            "component-make-project",
            "make-project",
            "generic_make_project",
            ".",
            vec!["Makefile".to_string()],
            evidence_id.clone(),
        );
        let parsed_targets = read_command_file_targets(&makefile);
        add_command_file_targets(
            builder,
            "Makefile",
            "make",
            "component-make-project",
            &parsed_targets.targets,
        );

        if !parsed_targets.targets.iter().any(|target| target == "test") {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "Makefile detected but no test target was parsed.",
                Some(Path::new("Makefile")),
                Some(evidence_id.clone()),
            );
        }

        if parsed_targets.ambiguous_lines > 0 {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::PartialSupport,
                "Makefile contains target-like lines that were not parsed conservatively.",
                Some(Path::new("Makefile")),
                Some(evidence_id),
            );
        }
    }

    let justfile = root.join("justfile");
    if justfile.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("justfile"),
            DetectedFileKind::BuildConfig,
            "build_config",
            None,
            "justfile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Just, "just", evidence_id.clone());
        builder.add_component(
            "component-just-project",
            "just-project",
            "generic_just_project",
            ".",
            vec!["justfile".to_string()],
            evidence_id.clone(),
        );
        let parsed_targets = read_command_file_targets(&justfile);
        add_command_file_targets(
            builder,
            "justfile",
            "just",
            "component-just-project",
            &parsed_targets.targets,
        );

        if !parsed_targets.targets.iter().any(|target| target == "test") {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::MissingCommand,
                "justfile detected but no test recipe was parsed.",
                Some(Path::new("justfile")),
                Some(evidence_id.clone()),
            );
        }

        if parsed_targets.ambiguous_lines > 0 {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::PartialSupport,
                "justfile contains recipe-like lines that were not parsed conservatively.",
                Some(Path::new("justfile")),
                Some(evidence_id),
            );
        }
    }

    if root.join("Dockerfile").exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("Dockerfile"),
            DetectedFileKind::ContainerConfig,
            "container_config",
            None,
            "Dockerfile detected.",
        );
        builder.add_package_manager(PackageManagerKind::Docker, "docker", evidence_id);
    }

    for compose_file in [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        let path = root.join(compose_file);
        if path.exists() {
            let evidence_id = builder.add_detected_file(
                Path::new(compose_file),
                DetectedFileKind::ContainerConfig,
                "container_config",
                None,
                "Docker Compose file detected.",
            );
            builder.add_package_manager(PackageManagerKind::Docker, "docker", evidence_id);
        }
    }

    let workflows_dir = root.join(".github").join("workflows");
    if let Ok(entries) = fs::read_dir(workflows_dir) {
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();
            let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
                continue;
            };

            if matches!(extension, "yml" | "yaml") {
                let relative = PathBuf::from(".github")
                    .join("workflows")
                    .join(entry.file_name());
                let evidence_id = builder.add_detected_file(
                    &relative,
                    DetectedFileKind::Workflow,
                    "workflow",
                    None,
                    "GitHub Actions workflow detected.",
                );
                builder.add_package_manager(
                    PackageManagerKind::GitHubActions,
                    "github_actions",
                    evidence_id,
                );
            }
        }
    }
}

struct CommandFileTargets {
    targets: Vec<String>,
    ambiguous_lines: usize,
}

fn add_command_file_targets(
    builder: &mut RepoGraphBuilder,
    file_name: &str,
    tool_name: &str,
    scope_ref: &str,
    targets: &[String],
) {
    for target in targets {
        let Some(kind) = command_kind_for_target(target) else {
            continue;
        };

        let evidence_id = builder.add_evidence(
            Path::new(file_name),
            "build_target",
            Some(&format!("target.{target}")),
            "Build file target detected.",
        );
        let command = format!("{tool_name} {target}");
        builder.add_command(
            &stable_id(&format!("cmd-{tool_name}"), target),
            kind.clone(),
            &command,
            Some(scope_ref),
            0.75,
            evidence_id.clone(),
        );

        if kind == RepoCommandKind::Test {
            builder.add_test(
                &stable_id(&format!("test-{tool_name}"), target),
                &command,
                &command,
                Some(scope_ref),
                0.75,
                evidence_id,
            );
        }
    }
}

fn command_kind_for_target(target: &str) -> Option<RepoCommandKind> {
    match target {
        "test" => Some(RepoCommandKind::Test),
        "check" => Some(RepoCommandKind::Check),
        "build" => Some(RepoCommandKind::Build),
        "lint" => Some(RepoCommandKind::Lint),
        "fmt" | "format" => Some(RepoCommandKind::Format),
        _ => None,
    }
}

fn read_command_file_targets(path: &Path) -> CommandFileTargets {
    let Ok(contents) = fs::read_to_string(path) else {
        return CommandFileTargets {
            targets: Vec::new(),
            ambiguous_lines: 0,
        };
    };

    let mut targets = BTreeSet::new();
    let mut ambiguous_lines = 0;

    for line in contents.lines() {
        let trimmed = line.trim_start();
        if line.starts_with(char::is_whitespace)
            || trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with(".PHONY:")
            || trimmed.contains(":=")
            || trimmed.contains("?=")
            || trimmed.contains("+=")
        {
            continue;
        }

        let Some((target, _)) = trimmed.split_once(':') else {
            continue;
        };
        let target = target.trim();

        if is_simple_command_target(target) {
            targets.insert(target.to_string());
        } else if is_ambiguous_target_syntax(target) {
            ambiguous_lines += 1;
        }
    }

    CommandFileTargets {
        targets: targets.into_iter().collect(),
        ambiguous_lines,
    }
}

fn is_simple_command_target(target: &str) -> bool {
    command_kind_for_target(target).is_some()
}

fn is_ambiguous_target_syntax(target: &str) -> bool {
    !target.is_empty()
        && !target.starts_with('.')
        && (target.chars().any(char::is_whitespace)
            || target.contains('%')
            || target.contains('$')
            || target.contains('/'))
}
