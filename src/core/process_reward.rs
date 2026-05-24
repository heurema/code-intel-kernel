#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessRewardInput {
    pub diagnostics_before: Option<u32>,
    pub diagnostics_after: Option<u32>,
    pub edit_scope_ok: Option<bool>,
    pub impacted_tests_known: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessReward {
    pub score: f32,
    pub edit_scope_ok: bool,
    pub impacted_tests_known: bool,
    pub risk_flags: Vec<String>,
}

pub fn score_process_reward(input: ProcessRewardInput) -> ProcessReward {
    ProcessReward {
        score: 0.0,
        edit_scope_ok: input.edit_scope_ok.unwrap_or(false),
        impacted_tests_known: input.impacted_tests_known.unwrap_or(false),
        risk_flags: vec!["ProcessReward scoring is not implemented yet.".to_string()],
    }
}
