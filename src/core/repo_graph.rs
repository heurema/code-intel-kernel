use std::fs;
use std::path::Path;

mod builder;
mod generic;
mod go;
mod helpers;
mod impact;
mod manifest;
mod node;
mod python;
mod rust;
mod types;

use builder::RepoGraphBuilder;
use helpers::display_path;

pub use impact::analyze_impact;
pub use types::*;

pub fn inspect_repo(repo_path: impl AsRef<Path>) -> RepoInspection {
    let root_path = repo_path.as_ref();
    let root = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let mut builder = RepoGraphBuilder::new(display_path(&root));

    rust::detect_rust(&root, &mut builder);
    node::detect_node(&root, &mut builder);
    python::detect_python(&root, &mut builder);
    go::detect_go(&root, &mut builder);
    generic::detect_generic(&root, &mut builder);
    detect_ignored_paths(&root, &mut builder);

    if !builder.has_detected_files() {
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::NoSupportedManifests,
            "No supported repository manifests were detected.",
            None,
            None,
        );
    }

    builder.finish()
}

fn detect_ignored_paths(root: &Path, builder: &mut RepoGraphBuilder) {
    for (ignored_path, emit_warning) in [
        (".git", false),
        ("node_modules", true),
        ("target", true),
        ("dist", true),
        ("build", true),
        (".cache", true),
        (".venv", true),
        ("__pycache__", true),
        ("coverage", true),
    ] {
        if emit_warning && root.join(ignored_path).exists() {
            builder.add_warning(
                DetectionSeverity::Info,
                DetectionCategory::IgnoredPath,
                "Generated, dependency, or cache directory was ignored by RepoGraph inspection.",
                Some(Path::new(ignored_path)),
                None,
            );
        }
    }
}
