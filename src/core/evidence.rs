#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelProfile {
    Strict,
    Standard,
    Prototype,
    Research,
    Custom,
}

impl KernelProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "strict" => Some(Self::Strict),
            "standard" => Some(Self::Standard),
            "prototype" => Some(Self::Prototype),
            "research" => Some(Self::Research),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Standard => "standard",
            Self::Prototype => "prototype",
            Self::Research => "research",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRequest {
    pub task: String,
    pub repo_path: String,
    pub profile: KernelProfile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceFile {
    pub path: String,
    pub reason: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCommand {
    pub kind: String,
    pub command: String,
    pub reason: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundle {
    pub claim: String,
    pub profile: KernelProfile,
    pub confidence: f32,
    pub files: Vec<EvidenceFile>,
    pub symbols: Vec<String>,
    pub commands: Vec<EvidenceCommand>,
    pub risks: Vec<String>,
    pub missing_evidence: Vec<String>,
}

pub fn create_evidence_bundle(request: EvidenceRequest) -> EvidenceBundle {
    EvidenceBundle {
        claim: request.task,
        profile: request.profile,
        confidence: 0.0,
        files: Vec::new(),
        symbols: Vec::new(),
        commands: Vec::new(),
        risks: Vec::new(),
        missing_evidence: vec![
            "RepoGraph, SymbolGraph, and LSP evidence are not implemented yet.".to_string(),
        ],
    }
}
