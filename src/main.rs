use code_intel_kernel::{
    analyze_impact, build_source_context_report, build_source_evidence_bundle, build_symbol_graph,
    create_evidence_bundle, inspect_repo, run_fixture_evaluation, EvidenceRequest, KernelProfile,
    LineRange, SourceContextSelector,
};
use serde::Serialize;
use serde_json::json;

fn main() {
    let exit_code = run(std::env::args().skip(1).collect());
    std::process::exit(exit_code);
}

fn run(args: Vec<String>) -> i32 {
    match args.first().map(String::as_str) {
        Some("inspect") => {
            let repo_path = args.get(1).map(String::as_str).unwrap_or(".");
            let snapshot = inspect_repo(repo_path);
            print_json(&snapshot);
            0
        }
        Some("repo-map") => {
            let snapshot = inspect_repo(".");
            print_json(&snapshot);
            0
        }
        Some("impact") => {
            let changed_files = parse_changed_files(&args[1..]);
            let snapshot = inspect_repo(".");
            let report = analyze_impact(&snapshot, changed_files);
            print_json(&report);
            0
        }
        Some("symbols") | Some("symbol-graph") => {
            let repo_path = args
                .iter()
                .skip(1)
                .find(|arg| !arg.starts_with("--"))
                .map(String::as_str)
                .unwrap_or(".");
            let snapshot = build_symbol_graph(repo_path);
            print_json(&snapshot);
            0
        }
        Some("source-evidence") => {
            let (repo_path, query) = parse_source_evidence_args(&args[1..]);
            let bundle = build_source_evidence_bundle(repo_path, query);
            print_json(&bundle);
            0
        }
        Some("source-context") => {
            let (repo_path, selectors) = parse_source_context_args(&args[1..]);
            let report = build_source_context_report(repo_path, selectors);
            print_json(&report);
            0
        }
        Some("eval-fixtures") => match run_fixture_evaluation("tests/eval/cases") {
            Ok(report) => {
                print_json(&report);
                if report.failed_cases == 0 {
                    0
                } else {
                    7
                }
            }
            Err(error) => {
                eprintln!("{error}");
                7
            }
        },
        Some("where-to-edit") => {
            let task = args
                .iter()
                .skip(1)
                .find(|arg| !arg.starts_with("--"))
                .map(String::as_str)
                .unwrap_or("");
            let profile = parse_profile(&args).unwrap_or(KernelProfile::Standard);
            let bundle = create_evidence_bundle(EvidenceRequest {
                task: task.to_string(),
                repo_path: ".".to_string(),
                profile,
            });
            print_json(&json!({
                "ok": false,
                "status": "insufficient_evidence",
                "data": {
                    "claim": bundle.claim,
                    "profile": bundle.profile.as_str(),
                    "confidence": bundle.confidence,
                    "files": [],
                    "symbols": [],
                    "commands": [],
                    "risks": [],
                    "missing_evidence": [
                        "SymbolGraph-lite is not evaluated for edit localization",
                        "No file/symbol relevance model yet"
                    ]
                },
                "evidence": [],
                "confidence": 0,
                "warnings": ["where-to-edit is intentionally a placeholder until evaluated localization evidence exists"]
            }));
            0
        }
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            0
        }
        Some(command) => {
            eprintln!("Unknown command: {command}");
            print_help();
            5
        }
    }
}

fn parse_profile(args: &[String]) -> Option<KernelProfile> {
    args.iter()
        .find_map(|arg| arg.strip_prefix("--profile="))
        .and_then(KernelProfile::parse)
}

fn parse_changed_files(args: &[String]) -> Vec<String> {
    let mut changed_files = Vec::new();
    let mut args_iter = args.iter();

    while let Some(arg) = args_iter.next() {
        if arg == "--json" {
            continue;
        }

        if arg == "--changed-files" {
            if let Some(value) = args_iter.next() {
                changed_files.extend(split_changed_files(value));
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--changed-files=") {
            changed_files.extend(split_changed_files(value));
            continue;
        }

        changed_files.extend(split_changed_files(arg));
    }

    changed_files
}

fn parse_source_evidence_args(args: &[String]) -> (&str, &str) {
    let mut repo_path = ".";
    let mut query = "";
    let mut args_iter = args.iter();

    while let Some(arg) = args_iter.next() {
        if arg == "--json" {
            continue;
        }

        if arg == "--repo" {
            if let Some(value) = args_iter.next() {
                repo_path = value;
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--repo=") {
            repo_path = value;
            continue;
        }

        if query.is_empty() {
            query = arg;
        }
    }

    (repo_path, query)
}

fn parse_source_context_args(args: &[String]) -> (&str, Vec<SourceContextSelector>) {
    let mut repo_path = ".";
    let mut selectors = Vec::new();
    let mut pending_file: Option<String> = None;
    let mut pending_lines: Option<LineRange> = None;
    let mut args_iter = args.iter();

    while let Some(arg) = args_iter.next() {
        if arg == "--json" {
            continue;
        }

        if arg == "--repo" {
            if let Some(value) = args_iter.next() {
                repo_path = value;
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--repo=") {
            repo_path = value;
            continue;
        }

        if arg == "--file" {
            if let Some(value) = args_iter.next() {
                pending_file = Some(value.to_string());
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--file=") {
            pending_file = Some(value.to_string());
            continue;
        }

        if arg == "--lines" {
            if let Some(value) = args_iter.next() {
                pending_lines = parse_line_range(value);
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--lines=") {
            pending_lines = parse_line_range(value);
            continue;
        }

        if arg == "--symbol-id" {
            if let Some(value) = args_iter.next() {
                selectors.push(SourceContextSelector::SymbolId {
                    symbol_id: value.to_string(),
                });
            }
            continue;
        }

        if let Some(value) = arg.strip_prefix("--symbol-id=") {
            selectors.push(SourceContextSelector::SymbolId {
                symbol_id: value.to_string(),
            });
        }
    }

    if let Some(path) = pending_file {
        selectors.push(SourceContextSelector::File {
            path,
            line_range: pending_lines,
        });
    }

    (repo_path, selectors)
}

fn parse_line_range(value: &str) -> Option<LineRange> {
    let (start, end) = value.split_once(':')?;
    let start_line = start.trim().parse::<usize>().ok()?;
    let end_line = end.trim().parse::<usize>().ok()?;
    Some(LineRange {
        start_line,
        end_line,
    })
}

fn split_changed_files(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn print_help() {
    println!(
        "code-intel\n\nUsage:\n  code-intel inspect <repo-path> [--json]\n  code-intel repo-map [--json]\n  code-intel impact <changed-file>... [--json]\n  code-intel impact --changed-files src/main.rs,Cargo.toml [--json]\n  code-intel symbols <repo-path> [--json]\n  code-intel source-evidence \"<query>\" [--repo <repo-path>] [--json]\n  code-intel source-context --file src/lib.rs [--lines 1:80] [--repo <repo-path>] [--json]\n  code-intel source-context --symbol-id <symbol-id> [--repo <repo-path>] [--json]\n  code-intel eval-fixtures [--json]\n  code-intel where-to-edit \"<task>\" [--profile=strict|standard|prototype|research|custom] [--json]\n\nRepoGraph impact is repository/build/test-level only. SymbolGraph-lite extracts top-level Rust source facts only. SourceEvidenceBundle is evidence assembly only. SourceContext returns explicit read-only source slices only. LSP, SQLite, MCP, ProcessReward, call graph, references, and edit localization are intentionally deferred."
    );
}

fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("Failed to serialize JSON: {error}");
            std::process::exit(1);
        }
    }
}
