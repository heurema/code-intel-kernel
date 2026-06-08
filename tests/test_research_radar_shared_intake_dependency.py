import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "research-radar" / "bin" / "check_shared_intake_dependency.py"


def load_module():
    spec = importlib.util.spec_from_file_location(
        "check_shared_intake_dependency",
        SCRIPT,
    )
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def write_lock(path: Path, **overrides):
    payload = {
        "schema_version": "shared-intake-consumer.v1",
        "consumer": "code-intel-kernel/research-radar",
        "upstream": {
            "repository": "https://github.com/heurema/shared-intake-governance",
            "pinned_commit": "abc123",
            "default_relative_path": "../shared-intake-governance",
        },
        "paths": {
            "profile": "research-radar/shared-intake/profile.json",
            "sources_glob": "research-radar/shared-intake/sources/*.json",
        },
    }
    for key, value in overrides.items():
        payload[key] = value
    path.write_text(json.dumps(payload), encoding="utf-8")


class SharedIntakeDependencyTests(unittest.TestCase):
    def test_load_lock_requires_consumer_schema(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp_dir:
            lock_path = Path(tmp_dir) / "shared-intake.lock.json"
            write_lock(lock_path)

            lock = module.load_lock(lock_path)

            self.assertEqual(lock["upstream"]["pinned_commit"], "abc123")

    def test_load_lock_rejects_unknown_schema(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp_dir:
            lock_path = Path(tmp_dir) / "shared-intake.lock.json"
            write_lock(lock_path, schema_version="other.v1")

            with self.assertRaisesRegex(module.DependencyError, "schema_version"):
                module.load_lock(lock_path)

    def test_resolve_shared_repo_root_prefers_explicit_root(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            explicit = root / "shared-intake-governance"
            marker = (
                explicit
                / "src"
                / "shared_intake_governance"
                / "cli"
                / "__main__.py"
            )
            marker.parent.mkdir(parents=True)
            marker.write_text("", encoding="utf-8")

            resolved = module.resolve_shared_repo_root(
                {
                    "upstream": {
                        "default_relative_path": "../missing-shared-intake"
                    }
                },
                consumer_repo_root=root / "consumer",
                explicit=explicit,
                env={},
            )

            self.assertEqual(resolved, explicit.resolve())

    def test_verify_pinned_commit_reports_mismatch(self):
        module = load_module()

        with self.assertRaisesRegex(
            module.DependencyError,
            "shared-intake pinned commit mismatch",
        ):
            module.verify_pinned_commit(
                expected_commit="abc123",
                actual_commit="def456",
            )

    def test_consumer_preflight_commands_validate_profile_then_sorted_sources(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp_dir:
            consumer_root = Path(tmp_dir)
            profile = consumer_root / "research-radar" / "shared-intake" / "profile.json"
            source_dir = profile.parent / "sources"
            source_dir.mkdir(parents=True)
            profile.write_text("{}", encoding="utf-8")
            (source_dir / "z-source.json").write_text("{}", encoding="utf-8")
            (source_dir / "a-source.json").write_text("{}", encoding="utf-8")

            commands = module.consumer_preflight_commands(
                {
                    "paths": {
                        "profile": "research-radar/shared-intake/profile.json",
                        "sources_glob": "research-radar/shared-intake/sources/*.json",
                    }
                },
                consumer_root,
            )

            self.assertEqual(commands[0][:2], ["inspect-profile", "--profile"])
            self.assertEqual(
                [Path(command[-1]).name for command in commands[1:]],
                ["a-source.json", "z-source.json"],
            )


if __name__ == "__main__":
    unittest.main()
