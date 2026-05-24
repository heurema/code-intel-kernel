#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspBridge {
    pub languages: Vec<&'static str>,
    pub status: &'static str,
}

pub fn create_lsp_bridge() -> LspBridge {
    LspBridge {
        languages: vec!["typescript"],
        status: "placeholder",
    }
}
