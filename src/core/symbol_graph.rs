#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Interface,
    TypeAlias,
    Import,
    Export,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolRecord {
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolGraphSnapshot {
    pub repo_path: String,
    pub symbols: Vec<SymbolRecord>,
    pub warnings: Vec<String>,
}

pub fn build_symbol_graph(repo_path: impl Into<String>) -> SymbolGraphSnapshot {
    SymbolGraphSnapshot {
        repo_path: repo_path.into(),
        symbols: Vec::new(),
        warnings: vec!["SymbolGraph MVP is not implemented yet.".to_string()],
    }
}
