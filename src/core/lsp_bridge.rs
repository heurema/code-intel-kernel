use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub const LSP_DIAGNOSTICS_CONTRACT_VERSION: &str = "0.1";

const DEFAULT_RUST_ANALYZER_COMMAND: &str = "rust-analyzer";
const DEFAULT_TIMEOUT_MS: u64 = 1_500;
const DEFAULT_MAX_DIAGNOSTICS: usize = 100;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspDiagnosticsReport {
    pub contract_version: String,
    pub status: LspAvailabilityStatus,
    pub language: String,
    pub server: LspServerInfo,
    pub workspace_root: String,
    pub request: LspRequestInfo,
    pub diagnostics: Vec<LspDiagnostic>,
    pub evidence: Vec<LspEvidence>,
    pub warnings: Vec<LspWarning>,
    pub limitations: Vec<String>,
    pub missing_evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LspAvailabilityStatus {
    Ok,
    Partial,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspServerInfo {
    pub name: String,
    pub command: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspRequestInfo {
    pub kind: String,
    pub files: Vec<String>,
    pub timeout_ms: u64,
    pub max_diagnostics: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspDiagnostic {
    pub diagnostic_id: String,
    pub file_path: String,
    pub start_line: usize,
    pub start_character: usize,
    pub end_line: usize,
    pub end_character: usize,
    pub severity: LspDiagnosticSeverity,
    pub code: Option<String>,
    pub source: Option<String>,
    pub message: String,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LspDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspEvidence {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub method: String,
    pub workspace_root: String,
    pub server: String,
    pub range: Option<LspRange>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspRange {
    pub start_line: usize,
    pub start_character: usize,
    pub end_line: usize,
    pub end_character: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LspWarning {
    pub id: String,
    pub severity: String,
    pub category: LspWarningCategory,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LspWarningCategory {
    RustAnalyzerUnavailable,
    LspDiagnosticsUnavailable,
    PathOutsideRepo,
    IgnoredPath,
    SymlinkIgnored,
    MissingFile,
    UnsupportedLanguage,
    RequestTimeout,
    ResultLimitExceeded,
    ServerError,
    NoFilesRequested,
    LspNotLocalization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspDiagnosticsOptions {
    pub command: Option<String>,
    pub timeout_ms: u64,
    pub max_diagnostics: usize,
}

impl Default for LspDiagnosticsOptions {
    fn default() -> Self {
        Self {
            command: std::env::var("CODE_INTEL_RUST_ANALYZER")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_diagnostics: DEFAULT_MAX_DIAGNOSTICS,
        }
    }
}

impl LspDiagnosticsOptions {
    fn command(&self) -> String {
        self.command
            .clone()
            .unwrap_or_else(|| DEFAULT_RUST_ANALYZER_COMMAND.to_string())
    }
}

pub fn collect_rust_lsp_diagnostics(
    repo_path: impl AsRef<Path>,
    files: Vec<String>,
) -> LspDiagnosticsReport {
    collect_rust_lsp_diagnostics_with_options(repo_path, files, LspDiagnosticsOptions::default())
}

pub fn collect_rust_lsp_diagnostics_with_options(
    repo_path: impl AsRef<Path>,
    files: Vec<String>,
    options: LspDiagnosticsOptions,
) -> LspDiagnosticsReport {
    let command = options.command();
    let root = match canonicalize_root(repo_path.as_ref()) {
        Ok(root) => root,
        Err(message) => {
            let mut builder = LspDiagnosticsBuilder::new(
                ".".to_string(),
                files,
                command,
                None,
                options.timeout_ms,
                options.max_diagnostics,
            );
            builder.warn(LspWarningCategory::PathOutsideRepo, message, None, "error");
            builder.missing("lsp_diagnostics_unavailable");
            builder.missing("no_lsp_diagnostics");
            return builder.finish(LspAvailabilityStatus::Error);
        }
    };
    let workspace_root = display_path(&root);
    let mut builder = LspDiagnosticsBuilder::new(
        workspace_root.clone(),
        files.clone(),
        command.clone(),
        None,
        options.timeout_ms,
        options.max_diagnostics,
    );

    let valid_files = validate_requested_files(&root, &files, &mut builder);
    if files.is_empty() {
        builder.warn(
            LspWarningCategory::NoFilesRequested,
            "No files were requested for LSP diagnostics.",
            None,
            "warning",
        );
    }

    if valid_files.is_empty() {
        builder.missing("lsp_diagnostics_unavailable");
        builder.missing("no_lsp_diagnostics");
        return builder.finish(LspAvailabilityStatus::Error);
    }

    match rust_analyzer_version(&command) {
        Ok(version) => builder.server.version = Some(version),
        Err(message) => {
            builder.warn(
                LspWarningCategory::RustAnalyzerUnavailable,
                message,
                None,
                "warning",
            );
            builder.warn(
                LspWarningCategory::LspDiagnosticsUnavailable,
                "rust-analyzer is unavailable; LSP diagnostics were not collected.",
                None,
                "warning",
            );
            builder.missing("rust_analyzer_unavailable");
            builder.missing("lsp_diagnostics_unavailable");
            builder.missing("no_lsp_diagnostics");
            return builder.finish(LspAvailabilityStatus::Unavailable);
        }
    }

    match collect_with_rust_analyzer(&root, &command, &valid_files, &options, &mut builder) {
        Ok(()) => {
            let status = if builder.has_partial_warning() {
                LspAvailabilityStatus::Partial
            } else {
                LspAvailabilityStatus::Ok
            };
            builder.finish(status)
        }
        Err(message) => {
            builder.warn(LspWarningCategory::ServerError, message, None, "error");
            builder.missing("lsp_diagnostics_unavailable");
            builder.finish(LspAvailabilityStatus::Error)
        }
    }
}

pub fn lsp_diagnostics_evidence_valid(report: &LspDiagnosticsReport) -> bool {
    report.diagnostics.iter().all(|diagnostic| {
        !diagnostic.evidence_ids.is_empty()
            && diagnostic
                .evidence_ids
                .iter()
                .all(|id| report.evidence.iter().any(|evidence| &evidence.id == id))
    })
}

fn canonicalize_root(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("Workspace root could not be canonicalized: {error}"))
}

fn validate_requested_files(
    root: &Path,
    files: &[String],
    builder: &mut LspDiagnosticsBuilder,
) -> Vec<ValidatedFile> {
    let mut valid = Vec::new();

    for file in files {
        if has_parent_or_root_component(file) {
            builder.warn(
                LspWarningCategory::PathOutsideRepo,
                "Requested diagnostics file must be repository-relative and contained in the workspace root.",
                Some(file.clone()),
                "error",
            );
            continue;
        }

        if has_ignored_component(file) {
            builder.warn(
                LspWarningCategory::IgnoredPath,
                "Ignored/generated path was skipped by LSP diagnostics.",
                Some(file.clone()),
                "warning",
            );
            continue;
        }

        let candidate = root.join(file);
        let metadata = match fs::symlink_metadata(&candidate) {
            Ok(metadata) => metadata,
            Err(_) => {
                builder.warn(
                    LspWarningCategory::MissingFile,
                    "Requested diagnostics file does not exist.",
                    Some(file.clone()),
                    "warning",
                );
                continue;
            }
        };

        if metadata.file_type().is_symlink() {
            builder.warn(
                LspWarningCategory::SymlinkIgnored,
                "LSP diagnostics do not follow symlinks.",
                Some(file.clone()),
                "warning",
            );
            continue;
        }

        let canonical = match candidate.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                builder.warn(
                    LspWarningCategory::MissingFile,
                    "Requested diagnostics file could not be canonicalized.",
                    Some(file.clone()),
                    "warning",
                );
                continue;
            }
        };

        if !canonical.starts_with(root) {
            builder.warn(
                LspWarningCategory::PathOutsideRepo,
                "Requested diagnostics file is outside the workspace root.",
                Some(file.clone()),
                "error",
            );
            continue;
        }

        if canonical.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            builder.warn(
                LspWarningCategory::UnsupportedLanguage,
                "Phase 3B-A supports Rust diagnostics only.",
                Some(file.clone()),
                "warning",
            );
            continue;
        }

        valid.push(ValidatedFile {
            relative_path: normalize_slashes(file),
            absolute_path: canonical,
        });
    }

    valid.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    valid
}

fn rust_analyzer_version(command: &str) -> Result<String, String> {
    let output = Command::new(command)
        .arg("--version")
        .output()
        .map_err(|error| format!("rust-analyzer command `{command}` is unavailable: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "rust-analyzer command `{command}` exited with status {}.",
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        Ok("unknown".to_string())
    } else {
        Ok(stdout)
    }
}

fn collect_with_rust_analyzer(
    root: &Path,
    command: &str,
    files: &[ValidatedFile],
    options: &LspDiagnosticsOptions,
    builder: &mut LspDiagnosticsBuilder,
) -> Result<(), String> {
    let mut child = Command::new(command)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start rust-analyzer: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to open rust-analyzer stdin.".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to open rust-analyzer stdout.".to_string())?;
    let (sender, receiver) = mpsc::channel();
    let _reader_handle = thread::spawn(move || read_lsp_messages(stdout, sender));

    send_lsp_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": file_uri(root),
                "capabilities": {},
                "workspaceFolders": [{
                    "uri": file_uri(root),
                    "name": root.file_name().and_then(|name| name.to_str()).unwrap_or("workspace")
                }]
            }
        }),
    )?;

    if !wait_for_response(&receiver, 1, Duration::from_millis(options.timeout_ms)) {
        let _ = child.kill();
        return Err("Timed out waiting for rust-analyzer initialize response.".to_string());
    }

    send_lsp_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }),
    )?;

    for file in files {
        let text = match fs::read_to_string(&file.absolute_path) {
            Ok(text) => text,
            Err(error) => {
                builder.warn(
                    LspWarningCategory::ServerError,
                    format!("Failed to read diagnostics file: {error}"),
                    Some(file.relative_path.clone()),
                    "warning",
                );
                continue;
            }
        };
        send_lsp_message(
            &mut stdin,
            &json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": file_uri(&file.absolute_path),
                        "languageId": "rust",
                        "version": 1,
                        "text": text
                    }
                }
            }),
        )?;
    }

    let deadline = Instant::now() + Duration::from_millis(options.timeout_ms);
    while Instant::now() < deadline && builder.diagnostics.len() < options.max_diagnostics {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(message) => {
                if message.get("method").and_then(Value::as_str)
                    == Some("textDocument/publishDiagnostics")
                {
                    collect_publish_diagnostics(root, &message, builder);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if builder.diagnostics.len() >= options.max_diagnostics {
        builder.warn(
            LspWarningCategory::ResultLimitExceeded,
            "LSP diagnostics were truncated by max_diagnostics.",
            None,
            "warning",
        );
    }

    let _ = send_lsp_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null
        }),
    );
    let _ = send_lsp_message(
        &mut stdin,
        &json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        }),
    );
    let _ = terminate_child(child);

    Ok(())
}

fn wait_for_response(receiver: &mpsc::Receiver<Value>, id: u64, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(message) if message.get("id").and_then(Value::as_u64) == Some(id) => return true,
            Ok(_) => continue,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return false,
        }
    }
    false
}

fn read_lsp_messages<R: Read>(reader: R, sender: mpsc::Sender<Value>) {
    let mut reader = BufReader::new(reader);
    loop {
        let Some(length) = read_content_length(&mut reader) else {
            break;
        };
        let mut body = vec![0; length];
        if reader.read_exact(&mut body).is_err() {
            break;
        }
        if let Ok(message) = serde_json::from_slice::<Value>(&body) {
            if sender.send(message).is_err() {
                break;
            }
        }
    }
}

fn read_content_length<R: BufRead>(reader: &mut R) -> Option<usize> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return content_length;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }
}

fn send_lsp_message(stdin: &mut impl Write, message: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(message).map_err(|error| error.to_string())?;
    write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).map_err(|error| error.to_string())?;
    stdin.write_all(&body).map_err(|error| error.to_string())?;
    stdin.flush().map_err(|error| error.to_string())
}

fn collect_publish_diagnostics(root: &Path, message: &Value, builder: &mut LspDiagnosticsBuilder) {
    let Some(params) = message.get("params") else {
        return;
    };
    let uri = params.get("uri").and_then(Value::as_str).or_else(|| {
        params
            .get("textDocument")
            .and_then(|doc| doc.get("uri"))
            .and_then(Value::as_str)
    });
    let Some(uri) = uri else {
        return;
    };
    let Some(path) = path_from_file_uri(uri) else {
        return;
    };
    let Ok(relative) = path.strip_prefix(root) else {
        return;
    };
    let relative = display_path(relative);
    let Some(items) = params.get("diagnostics").and_then(Value::as_array) else {
        return;
    };

    for item in items {
        if builder.diagnostics.len() >= builder.request.max_diagnostics {
            break;
        }
        let Some(range_value) = item.get("range") else {
            continue;
        };
        let range = parse_lsp_range(range_value);
        let severity = item
            .get("severity")
            .and_then(Value::as_u64)
            .map(lsp_severity)
            .unwrap_or(LspDiagnosticSeverity::Unknown);
        let message = item
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let code = item.get("code").map(code_to_string);
        let source = item
            .get("source")
            .and_then(Value::as_str)
            .map(str::to_string);
        builder.add_diagnostic(relative.clone(), range, severity, code, source, message);
    }
}

fn parse_lsp_range(value: &Value) -> LspRange {
    let start = value.get("start").unwrap_or(&Value::Null);
    let end = value.get("end").unwrap_or(&Value::Null);
    LspRange {
        start_line: start.get("line").and_then(Value::as_u64).unwrap_or(0) as usize + 1,
        start_character: start.get("character").and_then(Value::as_u64).unwrap_or(0) as usize,
        end_line: end.get("line").and_then(Value::as_u64).unwrap_or(0) as usize + 1,
        end_character: end.get("character").and_then(Value::as_u64).unwrap_or(0) as usize,
    }
}

fn code_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn lsp_severity(value: u64) -> LspDiagnosticSeverity {
    match value {
        1 => LspDiagnosticSeverity::Error,
        2 => LspDiagnosticSeverity::Warning,
        3 => LspDiagnosticSeverity::Information,
        4 => LspDiagnosticSeverity::Hint,
        _ => LspDiagnosticSeverity::Unknown,
    }
}

fn terminate_child(mut child: Child) -> Result<(), String> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            child.kill().map_err(|error| error.to_string())?;
            let _ = child.wait();
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    }
}

fn has_ignored_component(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| match component {
            Component::Normal(name) => matches!(
                name.to_str(),
                Some(
                    ".git"
                        | "target"
                        | "node_modules"
                        | "dist"
                        | "build"
                        | ".cache"
                        | ".venv"
                        | "__pycache__"
                        | "coverage"
                )
            ),
            _ => false,
        })
}

fn has_parent_or_root_component(path: &str) -> bool {
    Path::new(path).components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", display_path(path))
}

fn path_from_file_uri(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(PathBuf::from)
}

fn stable_id(prefix: &str, parts: &[&str]) -> String {
    let mut id = prefix.to_string();
    for part in parts {
        id.push('-');
        id.push_str(&sanitize_id(part));
    }
    id
}

fn sanitize_id(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output.trim_matches('-').to_string()
}

#[derive(Debug, Clone)]
struct ValidatedFile {
    relative_path: String,
    absolute_path: PathBuf,
}

#[derive(Debug)]
struct LspDiagnosticsBuilder {
    workspace_root: String,
    server: LspServerInfo,
    request: LspRequestInfo,
    diagnostics: Vec<LspDiagnostic>,
    evidence: Vec<LspEvidence>,
    warnings: Vec<LspWarning>,
    missing_evidence: Vec<String>,
}

impl LspDiagnosticsBuilder {
    fn new(
        workspace_root: String,
        files: Vec<String>,
        command: String,
        version: Option<String>,
        timeout_ms: u64,
        max_diagnostics: usize,
    ) -> Self {
        Self {
            workspace_root,
            server: LspServerInfo {
                name: "rust-analyzer".to_string(),
                command,
                version,
            },
            request: LspRequestInfo {
                kind: "diagnostics".to_string(),
                files: files
                    .into_iter()
                    .map(|file| normalize_slashes(&file))
                    .collect(),
                timeout_ms,
                max_diagnostics,
            },
            diagnostics: Vec::new(),
            evidence: Vec::new(),
            warnings: Vec::new(),
            missing_evidence: Vec::new(),
        }
    }

    fn warn(
        &mut self,
        category: LspWarningCategory,
        message: impl Into<String>,
        path: Option<String>,
        severity: &str,
    ) -> &mut Self {
        let category_id = format!("{category:?}");
        let path_id = path.clone().unwrap_or_else(|| "workspace".to_string());
        self.warnings.push(LspWarning {
            id: stable_id("lsp-warning", &[&category_id, &path_id]),
            severity: severity.to_string(),
            category,
            message: message.into(),
            path,
        });
        self
    }

    fn missing(&mut self, item: &str) -> &mut Self {
        if !self
            .missing_evidence
            .iter()
            .any(|existing| existing == item)
        {
            self.missing_evidence.push(item.to_string());
        }
        self
    }

    fn add_diagnostic(
        &mut self,
        file_path: String,
        range: LspRange,
        severity: LspDiagnosticSeverity,
        code: Option<String>,
        source: Option<String>,
        message: String,
    ) {
        let diagnostic_id = stable_id(
            "lsp-diagnostic",
            &[
                &file_path,
                &range.start_line.to_string(),
                &range.start_character.to_string(),
                &message,
            ],
        );
        let evidence_id = stable_id("lsp-evidence", &[&diagnostic_id]);
        self.evidence.push(LspEvidence {
            id: evidence_id.clone(),
            path: file_path.clone(),
            kind: "lsp_diagnostic".to_string(),
            method: "textDocument/publishDiagnostics".to_string(),
            workspace_root: self.workspace_root.clone(),
            server: self.server.name.clone(),
            range: Some(range.clone()),
        });
        self.diagnostics.push(LspDiagnostic {
            diagnostic_id,
            file_path,
            start_line: range.start_line,
            start_character: range.start_character,
            end_line: range.end_line,
            end_character: range.end_character,
            severity,
            code,
            source,
            message,
            evidence_ids: vec![evidence_id],
        });
    }

    fn has_partial_warning(&self) -> bool {
        self.warnings.iter().any(|warning| {
            matches!(
                warning.category,
                LspWarningCategory::ResultLimitExceeded | LspWarningCategory::RequestTimeout
            )
        })
    }

    fn finish(mut self, status: LspAvailabilityStatus) -> LspDiagnosticsReport {
        self.diagnostics.sort_by(|left, right| {
            (
                &left.file_path,
                left.start_line,
                left.start_character,
                &left.diagnostic_id,
            )
                .cmp(&(
                    &right.file_path,
                    right.start_line,
                    right.start_character,
                    &right.diagnostic_id,
                ))
        });
        self.evidence.sort_by(|left, right| left.id.cmp(&right.id));
        self.warnings.sort_by(|left, right| left.id.cmp(&right.id));
        self.missing_evidence.sort();

        self.warnings.push(LspWarning {
            id: stable_id("lsp-warning", &["lsp-not-localization"]),
            severity: "info".to_string(),
            category: LspWarningCategory::LspNotLocalization,
            message: "LSP diagnostics are evidence, not edit localization or fix instructions."
                .to_string(),
            path: None,
        });
        self.warnings.sort_by(|left, right| left.id.cmp(&right.id));

        LspDiagnosticsReport {
            contract_version: LSP_DIAGNOSTICS_CONTRACT_VERSION.to_string(),
            status,
            language: "rust".to_string(),
            server: self.server,
            workspace_root: self.workspace_root,
            request: self.request,
            diagnostics: self.diagnostics,
            evidence: self.evidence,
            warnings: self.warnings,
            limitations: vec![
                "Phase 3B-A collects Rust diagnostics only.".to_string(),
                "No definitions, references, hover, call hierarchy, formatting, code actions, or rename.".to_string(),
                "Diagnostics are evidence, not fixes or edit targets.".to_string(),
                "where-to-edit remains insufficient_evidence.".to_string(),
            ],
            missing_evidence: self.missing_evidence,
        }
    }
}
