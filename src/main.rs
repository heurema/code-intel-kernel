use code_intel_kernel::{
    analyze_impact, create_evidence_bundle, inspect_repo, EvidenceRequest, KernelProfile,
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
                        "SymbolGraph is not implemented",
                        "No file/symbol relevance model yet"
                    ]
                },
                "evidence": [],
                "confidence": 0,
                "warnings": ["where-to-edit is intentionally a placeholder until SymbolGraph exists"]
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
        "code-intel\n\nUsage:\n  code-intel inspect <repo-path> [--json]\n  code-intel repo-map [--json]\n  code-intel impact <changed-file>... [--json]\n  code-intel impact --changed-files src/main.rs,Cargo.toml [--json]\n  code-intel where-to-edit \"<task>\" [--profile=strict|standard|prototype|research|custom] [--json]\n\nThis is a documentation-first Rust skeleton. RepoGraph impact is repository/build/test-level only; SymbolGraph, LSP, SQLite, MCP, EvidenceBundle, and ProcessReward implementations are intentionally deferred."
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
