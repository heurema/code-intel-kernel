use crate::core::repo_graph::{DetectionSeverity, Evidence, RepoInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

pub const SYMBOLS_CONTRACT_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGraph {
    pub contract_version: String,
    pub repo: RepoInfo,
    pub source_files: Vec<SourceFile>,
    pub symbols: Vec<SourceSymbol>,
    pub evidence: Vec<Evidence>,
    pub warnings: Vec<SymbolWarning>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub id: String,
    pub path: String,
    pub language: SourceLanguage,
    pub parse_status: ParseStatus,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLanguage {
    Rust,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Ok,
    Error,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSymbol {
    pub id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub path: String,
    pub range: SourceRange,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Const,
    Static,
    Module,
    ImplBlock,
}

impl SymbolKind {
    fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Function => "function",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Const => "const",
            SymbolKind::Static => "static",
            SymbolKind::Module => "module",
            SymbolKind::ImplBlock => "impl_block",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolWarning {
    pub id: String,
    pub severity: DetectionSeverity,
    pub category: SymbolWarningCategory,
    pub message: String,
    pub path: Option<String>,
    pub evidence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolWarningCategory {
    IgnoredPath,
    ParseError,
    SymlinkIgnored,
    UnreadableSource,
}

pub fn build_symbol_graph(repo_path: impl AsRef<Path>) -> SymbolGraph {
    let root_path = repo_path.as_ref();
    let root = fs::canonicalize(root_path).unwrap_or_else(|_| root_path.to_path_buf());
    let mut builder = SymbolGraphBuilder::new(display_path(&root));

    for &ignored_path in ignored_paths() {
        if ignored_path != ".git" && root.join(ignored_path).exists() {
            builder.add_warning(
                DetectionSeverity::Info,
                SymbolWarningCategory::IgnoredPath,
                "Generated, dependency, or cache directory was ignored by SymbolGraph inspection.",
                Some(Path::new(ignored_path)),
                None,
            );
        }
    }

    let rust_files = discover_rust_files(&root, &root, &mut builder);
    for relative_path in rust_files {
        inspect_rust_file(&root, &relative_path, &mut builder);
    }

    builder.finish()
}

fn inspect_rust_file(root: &Path, relative_path: &Path, builder: &mut SymbolGraphBuilder) {
    let absolute_path = root.join(relative_path);
    let source_evidence = builder.add_evidence(
        relative_path,
        "source_file",
        None,
        "Rust source file discovered by SymbolGraph-lite.",
    );

    let Ok(contents) = fs::read_to_string(&absolute_path) else {
        builder.add_source_file(
            relative_path,
            ParseStatus::Unreadable,
            source_evidence.clone(),
        );
        builder.add_warning(
            DetectionSeverity::Warning,
            SymbolWarningCategory::UnreadableSource,
            "Rust source file could not be read.",
            Some(relative_path),
            Some(source_evidence),
        );
        return;
    };

    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    if parser.set_language(&language.into()).is_err() {
        builder.add_source_file(relative_path, ParseStatus::Error, source_evidence.clone());
        builder.add_warning(
            DetectionSeverity::Error,
            SymbolWarningCategory::ParseError,
            "Rust parser could not be initialized.",
            Some(relative_path),
            Some(source_evidence),
        );
        return;
    }

    let Some(tree) = parser.parse(&contents, None) else {
        builder.add_source_file(relative_path, ParseStatus::Error, source_evidence.clone());
        builder.add_warning(
            DetectionSeverity::Error,
            SymbolWarningCategory::ParseError,
            "Rust source file could not be parsed.",
            Some(relative_path),
            Some(source_evidence),
        );
        return;
    };

    let root_node = tree.root_node();
    if root_node.has_error() {
        builder.add_source_file(relative_path, ParseStatus::Error, source_evidence.clone());
        builder.add_warning(
            DetectionSeverity::Warning,
            SymbolWarningCategory::ParseError,
            "Rust source file parsed with syntax errors; no symbols were extracted.",
            Some(relative_path),
            Some(source_evidence),
        );
        return;
    }

    builder.add_source_file(relative_path, ParseStatus::Ok, source_evidence);
    extract_top_level_symbols(relative_path, &contents, root_node, builder);
}

fn extract_top_level_symbols(
    relative_path: &Path,
    contents: &str,
    root_node: Node,
    builder: &mut SymbolGraphBuilder,
) {
    let mut cursor = root_node.walk();
    for node in root_node.named_children(&mut cursor) {
        let Some((kind, name)) = top_level_symbol(&node, contents) else {
            continue;
        };
        builder.add_symbol(relative_path, kind, name, source_range(&node));
    }
}

fn top_level_symbol(node: &Node, contents: &str) -> Option<(SymbolKind, String)> {
    match node.kind() {
        "function_item" => named_symbol(node, contents, SymbolKind::Function),
        "struct_item" => named_symbol(node, contents, SymbolKind::Struct),
        "enum_item" => named_symbol(node, contents, SymbolKind::Enum),
        "trait_item" => named_symbol(node, contents, SymbolKind::Trait),
        "type_item" => named_symbol(node, contents, SymbolKind::TypeAlias),
        "const_item" => named_symbol(node, contents, SymbolKind::Const),
        "static_item" => named_symbol(node, contents, SymbolKind::Static),
        "mod_item" => named_symbol(node, contents, SymbolKind::Module),
        "impl_item" => {
            let point = node.start_position();
            Some((
                SymbolKind::ImplBlock,
                format!("impl@{}:{}", point.row + 1, point.column),
            ))
        }
        _ => None,
    }
}

fn named_symbol(node: &Node, contents: &str, kind: SymbolKind) -> Option<(SymbolKind, String)> {
    node.child_by_field_name("name")
        .and_then(|name| name.utf8_text(contents.as_bytes()).ok())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| (kind, name.to_string()))
}

fn source_range(node: &Node) -> SourceRange {
    let start = node.start_position();
    let end = node.end_position();
    SourceRange {
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_line: start.row + 1,
        start_column: start.column,
        end_line: end.row + 1,
        end_column: end.column,
    }
}

fn discover_rust_files(
    root: &Path,
    current: &Path,
    builder: &mut SymbolGraphBuilder,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut entries = fs::read_dir(current)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let relative_path = path.strip_prefix(root).ok().unwrap_or(&path);

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            builder.add_warning(
                DetectionSeverity::Info,
                SymbolWarningCategory::SymlinkIgnored,
                "SymbolGraph-lite does not follow symlinks.",
                Some(relative_path),
                None,
            );
            continue;
        }

        if file_type.is_dir() {
            if is_ignored_dir_name(&file_name) {
                continue;
            }
            files.extend(discover_rust_files(root, &path, builder));
            continue;
        }

        if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            files.push(relative_path.to_path_buf());
        }
    }

    files.sort();
    files
}

struct SymbolGraphBuilder {
    repo_root: String,
    source_files: Vec<SourceFile>,
    symbols: Vec<SourceSymbol>,
    evidence: Vec<Evidence>,
    warnings: Vec<SymbolWarning>,
}

impl SymbolGraphBuilder {
    fn new(repo_root: String) -> Self {
        Self {
            repo_root,
            source_files: Vec::new(),
            symbols: Vec::new(),
            evidence: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn finish(mut self) -> SymbolGraph {
        self.source_files.sort_by(|a, b| a.id.cmp(&b.id));
        self.symbols.sort_by(|a, b| a.id.cmp(&b.id));
        self.evidence.sort_by(|a, b| a.id.cmp(&b.id));
        self.warnings.sort_by(|a, b| a.id.cmp(&b.id));

        SymbolGraph {
            contract_version: SYMBOLS_CONTRACT_VERSION.to_string(),
            repo: RepoInfo {
                root: self.repo_root,
                read_only: true,
            },
            source_files: self.source_files,
            symbols: self.symbols,
            evidence: self.evidence,
            warnings: self.warnings,
            limitations: vec![
                "SymbolGraph-lite extracts top-level Rust declarations only.".to_string(),
                "No call graph, references, imports, LSP diagnostics, or edit localization."
                    .to_string(),
            ],
        }
    }

    fn add_source_file(&mut self, path: &Path, parse_status: ParseStatus, evidence_id: String) {
        let path = normalize_path(path);
        self.source_files.push(SourceFile {
            id: stable_id("source-file", &path),
            path,
            language: SourceLanguage::Rust,
            parse_status,
            evidence_ids: vec![evidence_id],
        });
    }

    fn add_symbol(&mut self, path: &Path, kind: SymbolKind, name: String, range: SourceRange) {
        let path = normalize_path(path);
        let evidence_id = stable_id(
            "evidence-symbol",
            &format!(
                "{}-{}-{}-{}-{}",
                path,
                kind.as_str(),
                name,
                range.start_byte,
                range.end_byte
            ),
        );
        self.evidence.push(Evidence {
            id: evidence_id.clone(),
            path: path.clone(),
            kind: "source_symbol".to_string(),
            field: Some(format!(
                "symbol.{}.{}@{}:{}",
                kind.as_str(),
                name,
                range.start_line,
                range.start_column
            )),
            reason: "Rust top-level symbol declaration.".to_string(),
        });
        self.symbols.push(SourceSymbol {
            id: stable_id(
                "symbol",
                &format!("{}-{}-{}-{}", path, kind.as_str(), name, range.start_byte),
            ),
            kind,
            name,
            path,
            range,
            evidence_ids: vec![evidence_id],
        });
    }

    fn add_evidence(
        &mut self,
        path: &Path,
        kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let path = normalize_path(path);
        let id = stable_id(
            "evidence",
            &format!("{}-{}-{}", path, kind, field.unwrap_or("")),
        );
        self.evidence.push(Evidence {
            id: id.clone(),
            path,
            kind: kind.to_string(),
            field: field.map(str::to_string),
            reason: reason.to_string(),
        });
        id
    }

    fn add_warning(
        &mut self,
        severity: DetectionSeverity,
        category: SymbolWarningCategory,
        message: &str,
        path: Option<&Path>,
        evidence_id: Option<String>,
    ) {
        let path_string = path.map(normalize_path);
        self.warnings.push(SymbolWarning {
            id: stable_id(
                "warning",
                &format!(
                    "{:?}-{}",
                    category,
                    path_string.as_deref().unwrap_or("repo")
                ),
            ),
            severity,
            category,
            message: message.to_string(),
            path: path_string,
            evidence_id,
        });
    }
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

fn is_ignored_dir_name(name: &str) -> bool {
    ignored_paths().contains(&name)
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

fn display_path(path: &Path) -> String {
    normalize_path(path)
}

fn evidence_ids(evidence: &[Evidence]) -> BTreeSet<&str> {
    evidence
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect()
}

pub fn symbol_graph_evidence_valid(graph: &SymbolGraph) -> bool {
    let evidence_ids = evidence_ids(&graph.evidence);
    graph.source_files.iter().all(|source_file| {
        !source_file.evidence_ids.is_empty()
            && source_file
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && graph.symbols.iter().all(|symbol| {
        !symbol.evidence_ids.is_empty()
            && symbol
                .evidence_ids
                .iter()
                .all(|id| evidence_ids.contains(id.as_str()))
    }) && graph.warnings.iter().all(|warning| {
        warning
            .evidence_id
            .as_deref()
            .is_none_or(|id| evidence_ids.contains(id))
    })
}
