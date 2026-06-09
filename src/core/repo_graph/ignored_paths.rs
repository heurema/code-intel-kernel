use std::path::Path;

use super::types::*;
use super::RepoGraphBuilder;

pub(super) fn detect_ignored_paths(root: &Path, builder: &mut RepoGraphBuilder) {
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
