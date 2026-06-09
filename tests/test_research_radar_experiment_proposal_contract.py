from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
TEMPLATE = ROOT / "research-radar" / "templates" / "experiment-proposal.md"
PROMPT = ROOT / "research-radar" / "codex-experiment-proposal-prompt.md"

REQUIRED_HEADINGS = [
    "Title",
    "Source URL",
    "Source Type",
    "Hypothesis",
    "Affected Modules",
    "Minimal Reversible Change",
    "Expected Signal",
    "Evaluation Plan",
    "Agent Bench Lab Fit",
    "Agent Bench Lab Evaluation Handoff",
    "Agent Bench Lab Blockers",
    "Fixtures or Benchmarks Needed",
    "Contract Risk",
    "Licensing and Attribution Notes",
    "Security Notes",
    "Stop Condition",
    "Reason Not To Implement Immediately",
]

REQUIRED_PROMPT_FIELDS = [
    "title",
    "source_url",
    "source_type",
    "hypothesis",
    "affected_modules",
    "minimal_reversible_change",
    "expected_signal",
    "evaluation_plan",
    "agent_bench_lab_fit",
    "agent_bench_lab_eval_handoff",
    "agent_bench_lab_blockers",
    "fixtures_or_benchmarks_needed",
    "contract_risk",
    "licensing_attribution_notes",
    "security_notes",
    "stop_condition",
    "reason_not_to_implement_immediately",
]

REQUIRED_HANDOFF_LABELS = [
    "Candidate suite or task family",
    "Public smoke check",
    "Private holdout need",
    "Run-validity or harness blocker",
    "Baseline setup",
    "Candidate setup",
    "Comparison metric",
]

REQUIRED_HANDOFF_PROMPT_FIELDS = [
    "candidate_suite_or_task_family",
    "public_smoke_check",
    "private_holdout_need",
    "run_validity_or_harness_blocker",
    "baseline_setup",
    "candidate_setup",
    "comparison_metric",
]


class ExperimentProposalContractTests(unittest.TestCase):
    def test_template_contains_required_sections(self):
        text = TEMPLATE.read_text(encoding="utf-8")

        for heading in REQUIRED_HEADINGS:
            with self.subTest(heading=heading):
                self.assertIn(f"## {heading}", text)

    def test_template_expands_agent_bench_lab_handoff(self):
        text = TEMPLATE.read_text(encoding="utf-8")

        for label in REQUIRED_HANDOFF_LABELS:
            with self.subTest(label=label):
                self.assertIn(f"- {label}:", text)

    def test_prompt_lists_required_fields_and_handoff_fields(self):
        text = PROMPT.read_text(encoding="utf-8")

        for field in REQUIRED_PROMPT_FIELDS + REQUIRED_HANDOFF_PROMPT_FIELDS:
            with self.subTest(field=field):
                self.assertIn(field, text)


if __name__ == "__main__":
    unittest.main()
