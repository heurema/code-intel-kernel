#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSitterExtractor {
    pub languages: Vec<&'static str>,
    pub status: &'static str,
}

pub fn create_tree_sitter_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor {
        languages: vec!["rust"],
        status: "implemented in SymbolGraph-lite for top-level Rust facts",
    }
}
