use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::*;
use super::RepoGraphBuilder;

pub(super) fn detect_go(root: &Path, builder: &mut RepoGraphBuilder) {
    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        let manifest_evidence = builder.add_detected_file(
            Path::new("go.mod"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Go module manifest detected.",
        );
        builder.add_package_manager(PackageManagerKind::Go, "go", manifest_evidence.clone());
        if let Some(module_name) = read_go_module_name(&go_mod) {
            let test_file_evidence = first_go_test_file(root).map(|path| {
                builder.add_detected_file(
                    &path,
                    DetectedFileKind::TestConfig,
                    "source_hint",
                    None,
                    "Go test file detected.",
                )
            });
            let test_confidence = if test_file_evidence.is_some() {
                0.95
            } else {
                0.85
            };

            builder.add_component(
                "component-go-module",
                &module_name,
                "go_module",
                ".",
                vec![
                    "go.mod".to_string(),
                    "go.work".to_string(),
                    "*.go".to_string(),
                    "**/*.go".to_string(),
                ],
                manifest_evidence.clone(),
            );
            builder.add_command(
                "cmd-go-test",
                RepoCommandKind::Test,
                "go test ./...",
                Some("component-go-module"),
                test_confidence,
                test_file_evidence
                    .clone()
                    .unwrap_or_else(|| manifest_evidence.clone()),
            );
            builder.add_command(
                "cmd-go-build",
                RepoCommandKind::Build,
                "go build ./...",
                Some("component-go-module"),
                0.85,
                manifest_evidence.clone(),
            );
            builder.add_test(
                "test-go-test",
                "go test",
                "go test ./...",
                Some("component-go-module"),
                test_confidence,
                test_file_evidence.unwrap_or(manifest_evidence),
            );
        } else {
            builder.add_warning(
                DetectionSeverity::Warning,
                DetectionCategory::MalformedManifest,
                "go.mod was detected but no module declaration was parsed; Go commands were not inferred.",
                Some(Path::new("go.mod")),
                Some(manifest_evidence),
            );
        }
    }

    let go_work = root.join("go.work");
    if go_work.exists() {
        let evidence_id = builder.add_detected_file(
            Path::new("go.work"),
            DetectedFileKind::WorkspaceConfig,
            "workspace_config",
            None,
            "Go workspace detected.",
        );
        let members = read_go_work_members(&go_work);
        builder.add_workspace(
            "workspace-go",
            "go-workspace",
            members.clone(),
            evidence_id.clone(),
        );
        if members.is_empty() {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::PartialSupport,
                "go.work was detected but no simple use members were parsed.",
                Some(Path::new("go.work")),
                Some(evidence_id),
            );
        }
    }
}

fn read_go_module_name(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("module ")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
    })
}

fn read_go_work_members(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut members = BTreeSet::new();
    let mut in_use_block = false;

    for line in contents.lines() {
        let trimmed = line.split("//").next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "use (" {
            in_use_block = true;
            continue;
        }

        if in_use_block && trimmed == ")" {
            in_use_block = false;
            continue;
        }

        if in_use_block {
            if is_simple_go_work_member(trimmed) {
                members.insert(trimmed.to_string());
            }
            continue;
        }

        if let Some(member) = trimmed.strip_prefix("use ").map(str::trim) {
            if is_simple_go_work_member(member) {
                members.insert(member.to_string());
            }
        }
    }

    members.into_iter().collect()
}

fn is_simple_go_work_member(member: &str) -> bool {
    !member.is_empty()
        && !member.contains('"')
        && !member.contains(' ')
        && (member == "." || member.starts_with("./") || member.starts_with("../"))
}

fn first_go_test_file(root: &Path) -> Option<PathBuf> {
    first_matching_file(root, root, &|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_test.go"))
    })
}

fn first_matching_file(
    root: &Path,
    current: &Path,
    predicate: &impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let mut entries = fs::read_dir(current).ok()?.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if path.is_dir() {
            if is_ignored_dir_name(&file_name) {
                continue;
            }
            if let Some(found) = first_matching_file(root, &path, predicate) {
                return Some(found);
            }
        } else if predicate(&path) {
            return path
                .strip_prefix(root)
                .ok()
                .map(Path::to_path_buf)
                .or(Some(path));
        }
    }

    None
}

fn is_ignored_dir_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "dist" | "build" | ".cache" | ".venv" | "__pycache__"
    )
}
