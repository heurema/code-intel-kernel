import importlib.util
import io
import os
import unittest
from pathlib import Path
from unittest import mock


MODULE_PATH = Path(__file__).resolve().parents[1] / "research-radar" / "bin" / "run_daily.py"
SPEC = importlib.util.spec_from_file_location("research_radar_run_daily", MODULE_PATH)
assert SPEC and SPEC.loader
run_daily = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(run_daily)


class FetchTextTests(unittest.TestCase):
    def setUp(self) -> None:
        run_daily._GITHUB_TOKEN_CACHE = False
        run_daily._HOST_LAST_REQUEST_AT.clear()

    def test_fetch_text_uses_gh_token_env(self) -> None:
        captured_headers = {}

        class Response:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                return b"{}"

        def fake_urlopen(request, timeout):
            captured_headers["authorization"] = request.get_header("Authorization")
            return Response()

        env = {"GH_TOKEN": "test-gh-token"}
        with mock.patch.dict(os.environ, env, clear=False):
            with mock.patch.object(run_daily.urllib.request, "urlopen", side_effect=fake_urlopen):
                body = run_daily.fetch_text("https://api.github.com/repos/example/project")
        self.assertEqual(body, "{}")
        self.assertEqual(captured_headers["authorization"], "Bearer test-gh-token")

    def test_fetch_text_retries_retryable_http_errors(self) -> None:
        response = mock.MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = b"<feed />"
        retry_error = run_daily.urllib.error.HTTPError(
            "https://export.arxiv.org/api/query",
            429,
            "rate limited",
            hdrs=None,
            fp=io.BytesIO(b""),
        )
        with mock.patch.object(
            run_daily.urllib.request,
            "urlopen",
            side_effect=[retry_error, response],
        ) as urlopen_mock:
            with mock.patch.object(run_daily.time, "sleep") as sleep_mock:
                body = run_daily.fetch_text(
                    "https://export.arxiv.org/api/query",
                    accept="application/atom+xml",
                    timeout=3,
                    max_attempts=2,
                    retry_statuses={429},
                )
        self.assertEqual(body, "<feed />")
        self.assertEqual(urlopen_mock.call_count, 2)
        sleep_mock.assert_called_once()

    def test_maybe_throttle_request_waits_for_arxiv_gap(self) -> None:
        run_daily._HOST_LAST_REQUEST_AT["export.arxiv.org"] = 10.0
        with mock.patch.object(run_daily.time, "monotonic", return_value=11.0):
            with mock.patch.object(run_daily.time, "sleep") as sleep_mock:
                run_daily.maybe_throttle_request("https://export.arxiv.org/api/query?search_query=all:test")
        sleep_mock.assert_called_once_with(2.0)

    def test_compute_retry_delay_prefers_retry_after_header(self) -> None:
        headers = {"Retry-After": "7"}
        delay = run_daily.compute_retry_delay("https://export.arxiv.org/api/query", 1, headers)
        self.assertEqual(delay, 7.0)


class GithubFallbackTests(unittest.TestCase):
    def test_collect_github_repo_falls_back_to_public_feeds_on_403(self) -> None:
        source = {
            "id": "codebase-memory",
            "name": "Codebase-Memory GitHub repository",
            "repo": "DeusData/codebase-memory-mcp",
            "url": "https://github.com/DeusData/codebase-memory-mcp",
            "license_hint": "MIT",
            "max_commits": 2,
            "max_releases": 1,
            "priority": 82,
        }

        commits_atom = """\
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <title>Fix indexing bug</title>
    <updated>2026-05-29T05:00:00Z</updated>
    <content>Repair source graph handling.</content>
    <link href="https://github.com/DeusData/codebase-memory-mcp/commit/abc123" />
  </entry>
</feed>
"""
        releases_atom = """\
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <title>v0.2.0</title>
    <updated>2026-05-28T05:00:00Z</updated>
    <content>Release summary.</content>
    <link href="https://github.com/DeusData/codebase-memory-mcp/releases/tag/v0.2.0" />
  </entry>
</feed>
"""

        def fake_fetch_json(url):
            raise RuntimeError("HTTP 403 for https://api.github.com/repos/DeusData/codebase-memory-mcp")

        def fake_fetch_text(url, **kwargs):
            if url.endswith("/commits.atom"):
                return commits_atom
            if url.endswith("/releases.atom"):
                return releases_atom
            if url == source["url"]:
                return '<meta property="og:description" content="Repository summary" />'
            raise AssertionError(url)

        with mock.patch.object(run_daily, "fetch_json", side_effect=fake_fetch_json):
            with mock.patch.object(run_daily, "fetch_text", side_effect=fake_fetch_text):
                items, health = run_daily.collect_github_repo(source)

        self.assertEqual(health["status"], "ok")
        self.assertEqual(health["checked_via"], "github_public_feeds")
        self.assertEqual(len(items), 3)
        self.assertEqual(items[0]["source_id"], "codebase-memory")


class ArxivRssKeywordTests(unittest.TestCase):
    def test_collect_arxiv_rss_keywords_filters_and_dedupes_items(self) -> None:
        context = run_daily.RunContext(Path("/tmp/radar"), run_daily.dt.date(2026, 5, 29))
        source = {
            "id": "arxiv-code-agents",
            "name": "arXiv code agents query",
            "type": "arxiv_rss_keywords",
            "rss_categories": ["cs.SE", "cs.AI"],
            "keywords": ["program repair", "coding agent"],
            "per_feed_items": 10,
            "submitted_after": "${TODAY_MINUS_2D}",
            "max_items": 5,
            "priority": 68,
            "affected_module": "research only",
        }
        feed_map = {
            "cs.SE": [
                {
                    "title": "Agentic Program Repair for Build Failures",
                    "canonical_url": "https://arxiv.org/abs/2605.10001",
                    "summary": "A coding agent for program repair in CI.",
                    "published_at": "2026-05-29T00:00:00Z",
                },
                {
                    "title": "Unrelated Paper",
                    "canonical_url": "https://arxiv.org/abs/2605.10002",
                    "summary": "Nothing relevant here.",
                    "published_at": "2026-05-29T00:00:00Z",
                },
            ],
            "cs.AI": [
                {
                    "title": "Agentic Program Repair for Build Failures",
                    "canonical_url": "https://arxiv.org/abs/2605.10001",
                    "summary": "Duplicate in another category.",
                    "published_at": "2026-05-29T00:00:00Z",
                },
                {
                    "title": "Older Program Repair Study",
                    "canonical_url": "https://arxiv.org/abs/2605.09999",
                    "summary": "program repair",
                    "published_at": "2026-05-20T00:00:00Z",
                },
            ],
        }

        def fake_fetch(category, *, limit):
            return feed_map[category][:limit]

        with mock.patch.object(run_daily, "fetch_arxiv_rss_feed", side_effect=fake_fetch):
            items, health = run_daily.collect_arxiv_rss_keywords(context, source)

        self.assertEqual(health["status"], "ok")
        self.assertEqual(health["checked_via"], "arxiv_rss_keyword_filter")
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]["canonical_url"], "https://arxiv.org/abs/2605.10001")


if __name__ == "__main__":
    unittest.main()
