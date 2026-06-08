use std::fs;
use std::path::Path;
use toml::Value as TomlValue;

use super::types::*;
use super::{manifest_warning_category, read_toml, RepoGraphBuilder};

pub(super) fn detect_python(root: &Path, builder: &mut RepoGraphBuilder) {
    let mut python_project_detected = false;
    let mut python_project_evidence = None;
    let mut pytest_evidence = None;

    let pyproject = root.join("pyproject.toml");
    if pyproject.exists() {
        python_project_detected = true;
        let manifest_evidence = builder.add_detected_file(
            Path::new("pyproject.toml"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Python project manifest detected.",
        );
        python_project_evidence = Some(manifest_evidence.clone());

        match read_toml(&pyproject) {
            Ok(manifest) => {
                let name = manifest
                    .get("project")
                    .and_then(|project| project.get("name"))
                    .and_then(TomlValue::as_str)
                    .or_else(|| {
                        manifest
                            .get("tool")
                            .and_then(|tool| tool.get("poetry"))
                            .and_then(|poetry| poetry.get("name"))
                            .and_then(TomlValue::as_str)
                    });

                if let Some(name) = name {
                    let evidence_id = builder.add_evidence(
                        Path::new("pyproject.toml"),
                        "manifest",
                        Some("project.name"),
                        "Python project name.",
                    );
                    builder.add_component(
                        "component-python-project",
                        name,
                        "python_project",
                        ".",
                        vec![
                            "pyproject.toml".to_string(),
                            "requirements.txt".to_string(),
                            "src/**".to_string(),
                            "tests/**".to_string(),
                        ],
                        evidence_id,
                    );
                }

                if manifest
                    .get("tool")
                    .and_then(|tool| tool.get("poetry"))
                    .is_some()
                {
                    builder.add_package_manager(
                        PackageManagerKind::Poetry,
                        "poetry",
                        manifest_evidence.clone(),
                    );
                }

                if let Some(field) = pyproject_pytest_field(&manifest) {
                    let evidence_id = builder.add_evidence(
                        Path::new("pyproject.toml"),
                        "test_config",
                        Some(&field),
                        "pytest evidence detected in pyproject.toml.",
                    );
                    pytest_evidence = Some(evidence_id);
                }
            }
            Err(message) => builder.add_warning(
                DetectionSeverity::Error,
                manifest_warning_category(&message),
                &message,
                Some(Path::new("pyproject.toml")),
                Some(manifest_evidence),
            ),
        }
    }

    python_project_detected |=
        detect_python_lockfile(builder, root, "uv.lock", PackageManagerKind::Uv, "uv");
    python_project_detected |= detect_python_lockfile(
        builder,
        root,
        "poetry.lock",
        PackageManagerKind::Poetry,
        "poetry",
    );

    let requirements = root.join("requirements.txt");
    if requirements.exists() {
        python_project_detected = true;
        let evidence_id = builder.add_detected_file(
            Path::new("requirements.txt"),
            DetectedFileKind::Manifest,
            "manifest",
            None,
            "Python requirements file detected.",
        );
        python_project_evidence = Some(evidence_id.clone());
        builder.add_package_manager(PackageManagerKind::Pip, "pip", evidence_id.clone());
        if requirements_mentions_pytest(&requirements) {
            let pytest_requirement_evidence = builder.add_evidence(
                Path::new("requirements.txt"),
                "test_config",
                Some("requirements.pytest"),
                "pytest dependency detected in requirements.txt.",
            );
            pytest_evidence = Some(pytest_requirement_evidence);
        }
    }

    let pytest_ini = root.join("pytest.ini");
    if pytest_ini.exists() {
        python_project_detected = true;
        let evidence_id = builder.add_detected_file(
            Path::new("pytest.ini"),
            DetectedFileKind::TestConfig,
            "test_config",
            None,
            "pytest.ini detected.",
        );
        pytest_evidence = Some(evidence_id);
    }

    let tests_dir_evidence = if python_project_detected && root.join("tests").is_dir() {
        Some(builder.add_detected_file(
            Path::new("tests"),
            DetectedFileKind::TestConfig,
            "directory",
            None,
            "tests directory detected.",
        ))
    } else {
        None
    };

    if let Some(evidence_id) = pytest_evidence {
        add_pytest(builder, evidence_id);
    } else if let Some(evidence_id) = tests_dir_evidence {
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::AmbiguousDetection,
            "Python tests directory detected, but no pytest evidence was found; no test command was inferred.",
            Some(Path::new("tests")),
            Some(evidence_id),
        );
    } else if python_project_detected {
        let warning_path = if pyproject.exists() {
            Some(Path::new("pyproject.toml"))
        } else if requirements.exists() {
            Some(Path::new("requirements.txt"))
        } else {
            None
        };
        builder.add_warning(
            DetectionSeverity::Info,
            DetectionCategory::MissingCommand,
            "Python project detected but no pytest configuration or tests directory was found.",
            warning_path,
            python_project_evidence,
        );
    }
}

fn detect_python_lockfile(
    builder: &mut RepoGraphBuilder,
    root: &Path,
    file_name: &str,
    kind: PackageManagerKind,
    name: &str,
) -> bool {
    if root.join(file_name).exists() {
        let evidence_id = builder.add_detected_file(
            Path::new(file_name),
            DetectedFileKind::Lockfile,
            "lockfile",
            None,
            &format!("{name} lockfile detected."),
        );
        builder.add_package_manager(kind, name, evidence_id);
        true
    } else {
        false
    }
}

fn add_pytest(builder: &mut RepoGraphBuilder, evidence_id: String) {
    builder.add_command(
        "cmd-python-pytest",
        RepoCommandKind::Test,
        "pytest",
        Some("component-python-project"),
        0.75,
        evidence_id.clone(),
    );
    builder.add_test(
        "test-python-pytest",
        "pytest",
        "pytest",
        Some("component-python-project"),
        0.75,
        evidence_id,
    );
}

fn pyproject_pytest_field(manifest: &TomlValue) -> Option<String> {
    if manifest
        .get("tool")
        .and_then(|tool| tool.get("pytest"))
        .is_some()
    {
        return Some("tool.pytest".to_string());
    }

    if toml_array_contains_package(
        manifest
            .get("project")
            .and_then(|project| project.get("dependencies")),
        "pytest",
    ) {
        return Some("project.dependencies.pytest".to_string());
    }

    if let Some(optional_dependencies) = manifest
        .get("project")
        .and_then(|project| project.get("optional-dependencies"))
        .and_then(TomlValue::as_table)
    {
        for (group, dependencies) in optional_dependencies {
            if toml_array_contains_package(Some(dependencies), "pytest") {
                return Some(format!("project.optional-dependencies.{group}.pytest"));
            }
        }
    }

    if manifest
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dev-dependencies"))
        .and_then(|dependencies| dependencies.get("pytest"))
        .is_some()
    {
        return Some("tool.poetry.dev-dependencies.pytest".to_string());
    }

    if manifest
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("group"))
        .and_then(|group| group.get("dev"))
        .and_then(|dev| dev.get("dependencies"))
        .and_then(|dependencies| dependencies.get("pytest"))
        .is_some()
    {
        return Some("tool.poetry.group.dev.dependencies.pytest".to_string());
    }

    None
}

fn toml_array_contains_package(value: Option<&TomlValue>, package_name: &str) -> bool {
    value.and_then(TomlValue::as_array).is_some_and(|items| {
        items
            .iter()
            .any(|item| toml_dependency_matches(item, package_name))
    })
}

fn toml_dependency_matches(value: &TomlValue, package_name: &str) -> bool {
    value
        .as_str()
        .is_some_and(|dependency| dependency_name_matches(dependency, package_name))
}

fn requirements_mentions_pytest(path: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };

    contents.lines().any(|line| {
        let line = line.split('#').next().unwrap_or("").trim();
        dependency_name_matches(line, "pytest")
    })
}

fn dependency_name_matches(dependency: &str, package_name: &str) -> bool {
    let dependency = dependency.trim().to_ascii_lowercase();
    dependency == package_name
        || dependency
            .strip_prefix(package_name)
            .and_then(|rest| rest.chars().next())
            .is_some_and(|character| matches!(character, '=' | '<' | '>' | '~' | '[' | '!' | ' '))
}
