/**
 * Domain model draft for Code Intelligence Kernel.
 * This is not final implementation. Use it as a contract sketch.
 */

export type KernelProfile = "strict" | "standard" | "prototype" | "research" | "custom";

export type NodeKind =
  | "repo"
  | "package"
  | "file"
  | "module"
  | "function"
  | "class"
  | "method"
  | "interface"
  | "type_alias"
  | "import"
  | "export"
  | "test"
  | "config"
  | "command"
  | "decision"
  | "event";

export type EdgeKind =
  | "contains"
  | "imports"
  | "exports"
  | "defines"
  | "calls"
  | "references"
  | "tests"
  | "depends_on"
  | "belongs_to_package"
  | "has_command"
  | "modified_in_session"
  | "mentioned_in_decision"
  | "risk_related_to";

export type Confidence = number; // 0.0 .. 1.0

export interface CodeNode {
  id: string;
  kind: NodeKind;
  name: string;
  path?: string;
  startLine?: number;
  endLine?: number;
  hash?: string;
  metadata?: Record<string, unknown>;
}

export interface CodeEdge {
  srcId: string;
  dstId: string;
  kind: EdgeKind;
  confidence: Confidence;
  evidence?: EvidenceRef[];
}

export interface EvidenceRef {
  kind: "file" | "symbol" | "config" | "diagnostic" | "event" | "decision" | "external";
  ref: string;
  reason?: string;
}

export interface TaskIntent {
  id?: string;
  goal: string;
  repoPath: string;
  profile?: KernelProfile;
  constraints?: string[];
}

export interface FileEvidence {
  path: string;
  reason: string;
  score: Confidence;
  evidence?: EvidenceRef[];
}

export interface SymbolEvidence {
  name: string;
  kind: NodeKind;
  path: string;
  startLine?: number;
  endLine?: number;
  reason: string;
  score: Confidence;
}

export interface CommandEvidence {
  kind: "test" | "lint" | "build" | "typecheck" | "other";
  command: string;
  scope?: string;
  reason: string;
  confidence: Confidence;
}

export interface EvidenceBundle {
  taskId?: string;
  claim: string;
  confidence: Confidence;
  files: FileEvidence[];
  symbols: SymbolEvidence[];
  commands: CommandEvidence[];
  risks: RiskFlag[];
  missingEvidence: string[];
}

export interface RiskFlag {
  code: string;
  severity: "low" | "medium" | "high" | "critical";
  message: string;
  evidence?: EvidenceRef[];
}

export interface DiagnosticSummary {
  before: number;
  after: number;
  newErrors: number;
  fixedErrors: number;
  newWarnings?: number;
  fixedWarnings?: number;
}

export interface ProcessReward {
  score: Confidence;
  diagnosticsDelta?: DiagnosticSummary;
  editScopeOk: boolean;
  impactedTestsKnown: boolean;
  affectedSymbolsConfidence: Confidence;
  testPlanConfidence: Confidence;
  riskFlags: RiskFlag[];
}

export type AgentEventType =
  | "task_started"
  | "repo_scoped"
  | "evidence_collected"
  | "hypothesis_created"
  | "hypothesis_rejected"
  | "edit_planned"
  | "patch_preflighted"
  | "diagnostic_delta_recorded"
  | "test_run_recorded"
  | "decision_recorded";

export interface AgentEvent {
  id: string;
  taskId?: string;
  eventType: AgentEventType;
  payload: Record<string, unknown>;
  createdAt: string;
}
