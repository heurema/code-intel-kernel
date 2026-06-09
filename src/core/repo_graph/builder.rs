use std::path::Path;

use super::types::*;
use super::{normalize_path, stable_id, stable_relationship_id};

pub(super) struct RepoGraphBuilder {
    repo_root: String,
    next_evidence: usize,
    next_warning: usize,
    detected_files: Vec<DetectedFile>,
    package_managers: Vec<PackageManager>,
    workspaces: Vec<Workspace>,
    components: Vec<Component>,
    commands: Vec<RepoCommand>,
    tests: Vec<TestTarget>,
    relationships: Vec<Relationship>,
    evidence: Vec<Evidence>,
    warnings: Vec<DetectionIssue>,
}

impl RepoGraphBuilder {
    pub(super) fn new(repo_root: String) -> Self {
        Self {
            repo_root,
            next_evidence: 1,
            next_warning: 1,
            detected_files: Vec::new(),
            package_managers: Vec::new(),
            workspaces: Vec::new(),
            components: Vec::new(),
            commands: Vec::new(),
            tests: Vec::new(),
            relationships: Vec::new(),
            evidence: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub(super) fn has_detected_files(&self) -> bool {
        !self.detected_files.is_empty()
    }

    pub(super) fn finish(mut self) -> RepoInspection {
        self.detected_files.sort_by(|a, b| a.id.cmp(&b.id));
        self.package_managers.sort_by(|a, b| a.id.cmp(&b.id));
        self.workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        self.components.sort_by(|a, b| a.id.cmp(&b.id));
        self.commands.sort_by(|a, b| a.id.cmp(&b.id));
        self.tests.sort_by(|a, b| a.id.cmp(&b.id));
        self.relationships.sort_by(|a, b| a.id.cmp(&b.id));
        self.warnings.sort_by(|a, b| a.id.cmp(&b.id));

        RepoInspection {
            contract_version: INSPECT_CONTRACT_VERSION.to_string(),
            repo: RepoInfo {
                root: self.repo_root,
                read_only: true,
            },
            detected_files: self.detected_files,
            package_managers: self.package_managers,
            workspaces: self.workspaces,
            components: self.components,
            commands: self.commands,
            tests: self.tests,
            relationships: self.relationships,
            evidence: self.evidence,
            warnings: self.warnings,
        }
    }

    pub(super) fn add_detected_file(
        &mut self,
        path: &Path,
        kind: DetectedFileKind,
        evidence_kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let evidence_id = self.add_evidence(path, evidence_kind, field, reason);
        let file_id = stable_id("file", &normalize_path(path));
        self.detected_files.push(DetectedFile {
            id: file_id.clone(),
            path: normalize_path(path),
            kind,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            &file_id,
            evidence_id.clone(),
        );
        evidence_id
    }

    pub(super) fn add_evidence(
        &mut self,
        path: &Path,
        kind: &str,
        field: Option<&str>,
        reason: &str,
    ) -> String {
        let id = format!("evidence-{}", self.next_evidence);
        self.next_evidence += 1;
        self.evidence.push(Evidence {
            id: id.clone(),
            path: normalize_path(path),
            kind: kind.to_string(),
            field: field.map(str::to_string),
            reason: reason.to_string(),
        });
        id
    }

    pub(super) fn add_warning(
        &mut self,
        severity: DetectionSeverity,
        category: DetectionCategory,
        message: &str,
        path: Option<&Path>,
        evidence_id: Option<String>,
    ) {
        let id = format!("warning-{}", self.next_warning);
        self.next_warning += 1;
        self.warnings.push(DetectionIssue {
            id,
            severity,
            category,
            message: message.to_string(),
            path: path.map(normalize_path),
            evidence_id,
        });
    }

    pub(super) fn add_package_manager(
        &mut self,
        kind: PackageManagerKind,
        name: &str,
        evidence_id: String,
    ) {
        let id = stable_id("package-manager", name);
        if self
            .package_managers
            .iter()
            .any(|manager| manager.kind == kind && manager.name == name)
        {
            return;
        }

        self.package_managers.push(PackageManager {
            id: id.clone(),
            kind,
            name: name.to_string(),
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            &id,
            evidence_id.clone(),
        );
    }

    pub(super) fn add_workspace(
        &mut self,
        id: &str,
        name: &str,
        members: Vec<String>,
        evidence_id: String,
    ) {
        self.workspaces.push(Workspace {
            id: id.to_string(),
            name: name.to_string(),
            members,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    pub(super) fn add_component(
        &mut self,
        id: &str,
        name: &str,
        kind: &str,
        path: &str,
        file_patterns: Vec<String>,
        evidence_id: String,
    ) {
        if self.components.iter().any(|existing| existing.id == id) {
            return;
        }

        self.components.push(Component {
            id: id.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            path: path.to_string(),
            file_patterns,
            evidence_id: evidence_id.clone(),
        });
        self.add_relationship(
            RelationshipKind::DefinesComponent,
            "repo",
            id,
            evidence_id.clone(),
        );
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    pub(super) fn add_command(
        &mut self,
        id: &str,
        kind: RepoCommandKind,
        command: &str,
        scope_ref: Option<&str>,
        confidence: f32,
        evidence_id: String,
    ) {
        if self
            .commands
            .iter()
            .any(|existing| existing.command == command)
        {
            return;
        }

        self.commands.push(RepoCommand {
            id: id.to_string(),
            kind,
            command: command.to_string(),
            scope: ".".to_string(),
            scope_ref: scope_ref.map(str::to_string),
            confidence,
            evidence_id: evidence_id.clone(),
        });
        if let Some(scope_ref) = scope_ref {
            self.add_relationship(
                RelationshipKind::HasCommand,
                scope_ref,
                id,
                evidence_id.clone(),
            );
        }
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    pub(super) fn add_test(
        &mut self,
        id: &str,
        name: &str,
        command: &str,
        scope_ref: Option<&str>,
        confidence: f32,
        evidence_id: String,
    ) {
        if self
            .tests
            .iter()
            .any(|existing| existing.command == command)
        {
            return;
        }

        self.tests.push(TestTarget {
            id: id.to_string(),
            name: name.to_string(),
            command: command.to_string(),
            scope: ".".to_string(),
            scope_ref: scope_ref.map(str::to_string),
            confidence,
            evidence_id: evidence_id.clone(),
        });
        if let Some(scope_ref) = scope_ref {
            self.add_relationship(
                RelationshipKind::HasTest,
                scope_ref,
                id,
                evidence_id.clone(),
            );
            self.add_relationship(RelationshipKind::Tests, id, scope_ref, evidence_id.clone());
        }
        self.add_relationship(
            RelationshipKind::EvidenceFor,
            &evidence_id,
            id,
            evidence_id.clone(),
        );
    }

    pub(super) fn add_relationship(
        &mut self,
        kind: RelationshipKind,
        src_id: &str,
        dst_id: &str,
        evidence_id: String,
    ) {
        let id = stable_relationship_id(&kind, src_id, dst_id);
        if self.relationships.iter().any(|existing| existing.id == id) {
            return;
        }

        self.relationships.push(Relationship {
            id,
            kind,
            src_id: src_id.to_string(),
            dst_id: dst_id.to_string(),
            evidence_id,
        });
    }
}
