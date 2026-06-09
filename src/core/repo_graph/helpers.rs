use std::path::Path;

use super::RelationshipKind;

pub(super) fn stable_id(prefix: &str, value: &str) -> String {
    format!("{prefix}-{}", sanitize_id(value))
}

pub(super) fn stable_relationship_id(
    kind: &RelationshipKind,
    src_id: &str,
    dst_id: &str,
) -> String {
    format!(
        "relationship-{}-{}-{}",
        sanitize_id(&format!("{kind:?}")),
        sanitize_id(src_id),
        sanitize_id(dst_id)
    )
}

pub(super) fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub(super) fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn display_path(path: &Path) -> String {
    normalize_path(path)
}
