use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEventType {
    TaskStarted,
    RepoScoped,
    EvidenceCollected,
    HypothesisCreated,
    HypothesisRejected,
    EditPlanned,
    PatchPreflighted,
    DiagnosticDeltaRecorded,
    TestRunRecorded,
    DecisionRecorded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentEvent {
    pub id: String,
    pub task_id: Option<String>,
    pub event_type: AgentEventType,
    pub payload_json: String,
    pub created_at_unix_ms: u128,
}

pub fn create_session_event(
    event_type: AgentEventType,
    task_id: Option<String>,
    payload_json: impl Into<String>,
) -> AgentEvent {
    let created_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    AgentEvent {
        id: format!("event-{created_at_unix_ms}"),
        task_id,
        event_type,
        payload_json: payload_json.into(),
        created_at_unix_ms,
    }
}
