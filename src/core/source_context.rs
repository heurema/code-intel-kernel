use crate::core::repo_graph::{DetectionSeverity, Evidence};
use crate::core::symbol_graph::{
    build_symbol_graph, SourceLanguage, SourceSymbol, SymbolGraph, SymbolKind,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const SOURCE_CONTEXT_CONTRACT_VERSION: &str = "0.1";
const DEFAULT_FILE_LINES: usize = 80;
const SYMBOL_CONTEXT_LINES: usize = 3;
const MAX_SLICES: usize = 8;
const MAX_LINES_PER_SLICE: usize = 80;
const MAX_BYTES_PER_SLICE: usize = 8_000;
const MAX_TOTAL_BYTES: usize = 20_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContextReport {
    pub contract_version: String,
    pub status: SourceContextStatus,
    pub selectors: Vec<SourceContextSelector>,
    pub slices: Vec<SourceContextSlice>,
    pub evidence: Vec<Evidence>,
    pub warnings: Vec<SourceContextWarning>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceContextStatus {
    Ok,
    Partial,
    InsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SourceContextSelector {
    File {
        path: String,
        line_range: Option<LineRange>,
    },
    SymbolId {
        symbol_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContextSlice {
    pub slice_id: String,
    pub file_path: String,
    pub language: SourceLanguage,
    pub reason: String,
    pub symbol_id: Option<String>,
    pub symbol_name: Option<String>,
    pub symbol_kind: Option<SymbolKind>,
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    pub context_before_lines: usize,
    pub context_after_lines: usize,
    pub text: String,
    pub truncated: bool,
    pub content_hash: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceContextWarning {
    pub id: String,
    pub severity: DetectionSeverity,
    pub category: SourceContextWarningCategory,
    pub message: String,
    pub path: Option<String>,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceContextWarningCategory {
    AmbiguousSymbolSelector,
    BinaryFile,
    IgnoredPath,
    MissingFile,
    NonUtf8File,
    PathOutsideRepo,
    SliceTruncated,
    SourceContextLimitExceeded,
    SourceContextNotLocalization,
    SymbolNotFound,
    SymlinkIgnored,
    UnsupportedLanguage,
}

pub fn build_source_context_report(
    repo_path: impl AsRef<Path>,
    selectors: Vec<SourceContextSelector>,
) -> SourceContextReport {
    let root_path = repo_path.as_ref();
    let root = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let symbol_graph = build_symbol_graph(root_path);
    build_source_context_report_from_symbols(&root, &symbol_graph, selectors)
}

pub fn build_source_context_report_from_symbols(
    root: &Path,
    symbol_graph: &SymbolGraph,
    selectors: Vec<SourceContextSelector>,
) -> SourceContextReport {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut builder = SourceContextBuilder::new(selectors.clone());

    if selectors.is_empty() {
        builder.add_warning(
            DetectionSeverity::Warning,
            SourceContextWarningCategory::MissingFile,
            "No source context selectors were provided.",
            None,
            None,
        );
    }

    for selector in selectors.iter().take(MAX_SLICES) {
        match selector {
            SourceContextSelector::File { path, line_range } => {
                builder.add_file_slice(&root, path, line_range.clone());
            }
            SourceContextSelector::SymbolId { symbol_id } => {
                builder.add_symbol_slice(&root, symbol_graph, symbol_id);
            }
        }
    }

    if selectors.len() > MAX_SLICES {
        builder.add_warning(
            DetectionSeverity::Warning,
            SourceContextWarningCategory::SourceContextLimitExceeded,
            "Source context selector limit was exceeded; selectors were truncated.",
            None,
            None,
        );
    }

    builder.finish()
}

pub fn source_context_evidence_valid(report: &SourceContextReport) -> bool {
    let evidence_ids = report
        .evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<BTreeSet<_>>();

    report.slices.iter().all(|slice| {
        !slice.evidence_ids.is_empty()
            && slice
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && report.warnings.iter().all(|warning| {
        warning
            .evidence_id
            .as_deref()
            .is_none_or(|id| evidence_ids.contains(id))
    })
}

struct SourceContextBuilder {
    selectors: Vec<SourceContextSelector>,
    slices: Vec<SourceContextSlice>,
    evidence: Vec<Evidence>,
    warnings: Vec<SourceContextWarning>,
    total_bytes: usize,
}

impl SourceContextBuilder {
    fn new(selectors: Vec<SourceContextSelector>) -> Self {
        Self {
            selectors,
            slices: Vec::new(),
            evidence: Vec::new(),
            warnings: Vec::new(),
            total_bytes: 0,
        }
    }

    fn add_file_slice(&mut self, root: &Path, path: &str, line_range: Option<LineRange>) {
        let Some(relative_path) = self.resolve_relative_path(root, path) else {
            return;
        };

        let range = line_range.unwrap_or(LineRange {
            start_line: 1,
            end_line: DEFAULT_FILE_LINES,
        });
        self.slice_file(root, &relative_path, range, "explicit_file_selector", None);
    }

    fn add_symbol_slice(&mut self, root: &Path, symbol_graph: &SymbolGraph, symbol_id: &str) {
        let Some(symbol) = symbol_graph
            .symbols
            .iter()
            .find(|symbol| symbol.id == symbol_id)
        else {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::SymbolNotFound,
                "No SymbolGraph-lite symbol matched the selector.",
                None,
                None,
            );
            return;
        };

        let range = LineRange {
            start_line: symbol
                .range
                .start_line
                .saturating_sub(SYMBOL_CONTEXT_LINES)
                .max(1),
            end_line: symbol.range.end_line + SYMBOL_CONTEXT_LINES,
        };
        self.slice_file(
            root,
            Path::new(&symbol.path),
            range,
            "symbol_id_selector",
            Some(symbol),
        );
    }

    fn slice_file(
        &mut self,
        root: &Path,
        relative_path: &Path,
        requested_range: LineRange,
        reason: &str,
        symbol: Option<&SourceSymbol>,
    ) {
        if self.slices.len() >= MAX_SLICES {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::SourceContextLimitExceeded,
                "Source context slice limit was exceeded; slice was skipped.",
                Some(relative_path),
                None,
            );
            return;
        }

        if relative_path
            .extension()
            .is_none_or(|extension| extension != "rs")
        {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::UnsupportedLanguage,
                "SourceContext currently slices Rust source files only.",
                Some(relative_path),
                None,
            );
            return;
        }

        let absolute_path = root.join(relative_path);
        let Ok(bytes) = fs::read(&absolute_path) else {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::MissingFile,
                "Source file could not be read.",
                Some(relative_path),
                None,
            );
            return;
        };

        if bytes.contains(&0) {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::BinaryFile,
                "Source file looks binary and was not sliced.",
                Some(relative_path),
                None,
            );
            return;
        }

        let Ok(contents) = String::from_utf8(bytes) else {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::NonUtf8File,
                "Source file is not valid UTF-8 and was not sliced.",
                Some(relative_path),
                None,
            );
            return;
        };

        let line_count = contents.lines().count().max(1);
        let start_line = requested_range.start_line.max(1).min(line_count);
        let requested_end = requested_range.end_line.max(start_line).min(line_count);
        let capped_end = (start_line + MAX_LINES_PER_SLICE - 1).min(requested_end);
        let mut truncated = capped_end < requested_end;

        let (mut text, start_byte, end_byte) = slice_text(&contents, start_line, capped_end);
        if text.len() > MAX_BYTES_PER_SLICE {
            text = truncate_utf8(&text, MAX_BYTES_PER_SLICE);
            truncated = true;
        }

        let remaining_total = MAX_TOTAL_BYTES.saturating_sub(self.total_bytes);
        if text.len() > remaining_total {
            text = truncate_utf8(&text, remaining_total);
            truncated = true;
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::SourceContextLimitExceeded,
                "Source context report byte limit was exceeded; slice was truncated.",
                Some(relative_path),
                None,
            );
        }
        self.total_bytes += text.len();

        let path = normalize_path(relative_path);
        let evidence_id = self.add_evidence(
            &path,
            "source_context_slice",
            Some(&format!("lines.{start_line}-{capped_end}")),
            "Read-only source context slice.",
        );
        let mut evidence_ids = vec![evidence_id.clone()];
        if let Some(symbol) = symbol {
            for symbol_evidence_id in &symbol.evidence_ids {
                evidence_ids.push(symbol_evidence_id.clone());
            }
            for symbol_evidence in symbol_graph_evidence_for(symbol) {
                self.evidence.push(symbol_evidence);
            }
        }
        evidence_ids.sort();
        evidence_ids.dedup();

        if truncated {
            self.add_warning(
                DetectionSeverity::Info,
                SourceContextWarningCategory::SliceTruncated,
                "Source context slice was truncated by line or byte limits.",
                Some(relative_path),
                Some(evidence_id.clone()),
            );
        }

        self.slices.push(SourceContextSlice {
            slice_id: stable_id(
                "source-slice",
                &format!(
                    "{}-{}-{}-{}",
                    path,
                    symbol.map(|symbol| symbol.id.as_str()).unwrap_or("file"),
                    start_line,
                    capped_end
                ),
            ),
            file_path: path,
            language: SourceLanguage::Rust,
            reason: reason.to_string(),
            symbol_id: symbol.map(|symbol| symbol.id.clone()),
            symbol_name: symbol.map(|symbol| symbol.name.clone()),
            symbol_kind: symbol.map(|symbol| symbol.kind.clone()),
            start_line,
            end_line: capped_end,
            start_byte: Some(start_byte),
            end_byte: Some(end_byte),
            context_before_lines: symbol
                .map(|symbol| symbol.range.start_line.saturating_sub(start_line))
                .unwrap_or(0),
            context_after_lines: symbol
                .map(|symbol| capped_end.saturating_sub(symbol.range.end_line))
                .unwrap_or(0),
            content_hash: content_hash(&text),
            text,
            truncated,
            evidence_ids,
        });
    }

    fn resolve_relative_path(&mut self, root: &Path, path: &str) -> Option<PathBuf> {
        let raw_path = Path::new(path);
        if path.trim().is_empty() || has_parent_or_prefix(raw_path) {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::PathOutsideRepo,
                "Source context selector path must stay inside the repository.",
                Some(raw_path),
                None,
            );
            return None;
        }

        let relative_path = if raw_path.is_absolute() {
            let Ok(canonical) = fs::canonicalize(raw_path) else {
                self.add_warning(
                    DetectionSeverity::Warning,
                    SourceContextWarningCategory::MissingFile,
                    "Source context selector path does not exist.",
                    Some(raw_path),
                    None,
                );
                return None;
            };
            if !canonical.starts_with(root) {
                self.add_warning(
                    DetectionSeverity::Warning,
                    SourceContextWarningCategory::PathOutsideRepo,
                    "Source context selector path resolved outside the repository.",
                    Some(raw_path),
                    None,
                );
                return None;
            }
            canonical.strip_prefix(root).ok()?.to_path_buf()
        } else {
            raw_path.to_path_buf()
        };

        if is_ignored_path(&relative_path) {
            self.add_warning(
                DetectionSeverity::Info,
                SourceContextWarningCategory::IgnoredPath,
                "Generated, dependency, or cache path was not sliced.",
                Some(&relative_path),
                None,
            );
            return None;
        }

        let absolute_path = root.join(&relative_path);
        if !absolute_path.exists() {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::MissingFile,
                "Source context selector path does not exist.",
                Some(&relative_path),
                None,
            );
            return None;
        }

        if has_symlink_component(root, &relative_path) {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::SymlinkIgnored,
                "SourceContext does not follow symlinks.",
                Some(&relative_path),
                None,
            );
            return None;
        }

        let Ok(canonical) = fs::canonicalize(&absolute_path) else {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::MissingFile,
                "Source context selector path could not be resolved.",
                Some(&relative_path),
                None,
            );
            return None;
        };
        if !canonical.starts_with(root) {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::PathOutsideRepo,
                "Source context selector path resolved outside the repository.",
                Some(&relative_path),
                None,
            );
            return None;
        }

        if !canonical.is_file() {
            self.add_warning(
                DetectionSeverity::Warning,
                SourceContextWarningCategory::MissingFile,
                "Source context selector path is not a file.",
                Some(&relative_path),
                None,
            );
            return None;
        }

        canonical.strip_prefix(root).ok().map(Path::to_path_buf)
    }

    fn add_evidence(
        &mut self,
        path: &str,
        kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let id = stable_id(
            "evidence-source-context",
            &format!("{}-{}-{}", path, kind, field.unwrap_or("")),
        );
        self.evidence.push(Evidence {
            id: id.clone(),
            path: path.to_string(),
            kind: kind.to_string(),
            field: field.map(str::to_string),
            reason: reason.to_string(),
        });
        id
    }

    fn add_warning(
        &mut self,
        severity: DetectionSeverity,
        category: SourceContextWarningCategory,
        message: &str,
        path: Option<&Path>,
        evidence_id: Option<String>,
    ) {
        let path_string = path.map(normalize_path);
        self.warnings.push(SourceContextWarning {
            id: stable_id(
                "source-context-warning",
                &format!(
                    "{:?}-{}-{}",
                    category,
                    path_string.as_deref().unwrap_or("repo"),
                    message
                ),
            ),
            severity,
            category,
            message: message.to_string(),
            path: path_string,
            evidence_id,
        });
    }

    fn finish(mut self) -> SourceContextReport {
        self.slices.sort_by(|left, right| {
            left.file_path
                .cmp(&right.file_path)
                .then_with(|| left.symbol_id.cmp(&right.symbol_id))
                .then_with(|| left.start_line.cmp(&right.start_line))
        });
        self.evidence.sort_by(|left, right| left.id.cmp(&right.id));
        self.evidence.dedup_by(|left, right| left.id == right.id);
        self.warnings.push(SourceContextWarning {
            id: stable_id("source-context-warning", "source-context-not-localization"),
            severity: DetectionSeverity::Info,
            category: SourceContextWarningCategory::SourceContextNotLocalization,
            message:
                "SourceContext is read-only source text context, not an edit localization result."
                    .to_string(),
            path: None,
            evidence_id: None,
        });
        self.warnings.sort_by(|left, right| left.id.cmp(&right.id));
        self.warnings.dedup_by(|left, right| left.id == right.id);

        let status = if self.slices.is_empty() {
            SourceContextStatus::InsufficientEvidence
        } else if self.warnings.iter().any(|warning| {
            warning.severity == DetectionSeverity::Warning
                || warning.severity == DetectionSeverity::Error
        }) {
            SourceContextStatus::Partial
        } else {
            SourceContextStatus::Ok
        };

        SourceContextReport {
            contract_version: SOURCE_CONTEXT_CONTRACT_VERSION.to_string(),
            status,
            selectors: self.selectors,
            slices: self.slices,
            evidence: self.evidence,
            warnings: self.warnings,
            limitations: vec![
                "SourceContext uses explicit selectors only.".to_string(),
                "SourceContext returns read-only bounded snippets, not edit targets.".to_string(),
                "No reference resolution, call graph, LSP diagnostics, or patch planning."
                    .to_string(),
                "Rust source files only in this phase.".to_string(),
            ],
        }
    }
}

fn symbol_graph_evidence_for(symbol: &SourceSymbol) -> Vec<Evidence> {
    symbol
        .evidence_ids
        .iter()
        .map(|evidence_id| Evidence {
            id: evidence_id.clone(),
            path: symbol.path.clone(),
            kind: "source_symbol".to_string(),
            field: Some(format!("symbol.{}", symbol.name)),
            reason: "SymbolGraph-lite symbol evidence referenced by SourceContext.".to_string(),
        })
        .collect()
}

fn has_parent_or_prefix(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    })
}

fn is_ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|part| ignored_paths().contains(&part))
    })
}

fn has_symlink_component(root: &Path, relative_path: &Path) -> bool {
    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        current.push(component.as_os_str());
        if fs::symlink_metadata(&current)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

fn slice_text(contents: &str, start_line: usize, end_line: usize) -> (String, usize, usize) {
    let offsets = line_offsets(contents);
    let start_byte = offsets
        .get(start_line.saturating_sub(1))
        .copied()
        .unwrap_or(0);
    let end_byte = offsets.get(end_line).copied().unwrap_or(contents.len());
    let lines = contents
        .lines()
        .skip(start_line.saturating_sub(1))
        .take(end_line.saturating_sub(start_line) + 1)
        .collect::<Vec<_>>()
        .join("\n");

    (lines, start_byte, end_byte)
}

fn line_offsets(contents: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (index, character) in contents.char_indices() {
        if character == '\n' {
            offsets.push(index + 1);
        }
    }
    if offsets.last().copied() != Some(contents.len()) {
        offsets.push(contents.len());
    }
    offsets
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    if max_bytes == 0 {
        return String::new();
    }

    let mut end = max_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn content_hash(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn ignored_paths() -> &'static [&'static str] {
    &[
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        ".cache",
        ".venv",
        "__pycache__",
        "coverage",
    ]
}

fn stable_id(prefix: &str, value: &str) -> String {
    let suffix = value
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
        .join("-");
    if suffix.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}-{suffix}")
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
