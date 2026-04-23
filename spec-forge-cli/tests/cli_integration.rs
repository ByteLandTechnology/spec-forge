use assert_cmd::Command;
use serde_json::Value as JsonValue;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use toml::Value as TomlValue;

fn bin() -> Command {
    Command::cargo_bin("spec-forge-cli").expect("binary should build")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("package should live under repo root")
        .to_path_buf()
}

fn temp_target() -> TempDir {
    tempfile::tempdir().expect("tempdir should be created")
}

fn read_json(path: &Path) -> JsonValue {
    let raw = fs::read_to_string(path).expect("file should be readable");
    serde_yaml::from_str(&raw).expect("yaml should parse")
}

fn assert_matching_recommendations(value: &JsonValue, expected: &str) {
    assert_eq!(value["recommended_script"], expected);
    assert_eq!(value["recommended_command"], expected);
}

fn assert_matching_handoff_snapshot(workspace: &Path) {
    let handoff = read_json(&workspace.join("handoff.yaml"));
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    assert_eq!(pipeline["handoff"], handoff);
}

fn run_json_success(args: &[&str]) -> JsonValue {
    let assert = bin().args(args).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    serde_json::from_str(&stdout).expect("json output")
}

fn run_json_failure(args: &[&str]) -> JsonValue {
    let assert = bin().args(args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    serde_json::from_str(&stderr).expect("json error output")
}

fn run_json_stdout_failure(args: &[&str]) -> JsonValue {
    let assert = bin().args(args).assert().failure();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    serde_json::from_str(&stdout).expect("json failure output")
}

fn run_toml_success(args: &[&str]) -> TomlValue {
    let assert = bin().args(args).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    toml::from_str(&stdout).expect("toml output")
}

fn run_yaml_success(args: &[&str]) -> JsonValue {
    let assert = bin().args(args).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    serde_yaml::from_str(&stdout).expect("yaml output")
}

fn run_toml_failure(args: &[&str]) -> TomlValue {
    let assert = bin().args(args).assert().failure();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    toml::from_str(&stdout).expect("toml failure output")
}

fn run_yaml_failure(args: &[&str]) -> JsonValue {
    let assert = bin().args(args).assert().failure();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    serde_yaml::from_str(&stdout).expect("yaml failure output")
}

fn init_workspace(temp: &TempDir, request_title: &str, spec_id: &str) {
    run_json_success(&[
        "init",
        "--target",
        temp.path().to_str().expect("temp path utf8"),
        "--spec-id",
        spec_id,
        "--request-title",
        request_title,
        "--format",
        "json",
    ]);
}

#[test]
fn top_level_help_renders_man_sections() {
    let assert = bin().arg("--help").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("SYNOPSIS"));
    assert!(stdout.contains("spec-forge-cli - Native Rust CLI for the spec-forge YAML workflow."));
}

#[test]
fn top_level_command_without_args_renders_man_sections() {
    let assert = bin().assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("SYNOPSIS"));
    assert!(stdout.contains("spec-forge-cli - Native Rust CLI for the spec-forge YAML workflow."));
}

#[test]
fn init_accepts_spec_id_request_summary_and_force_reinitializes_workspace() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    let initial_payload = run_json_success(&[
        "init",
        "--target",
        target,
        "--spec-id",
        "Custom Spec 42",
        "--request-title",
        "Initial Title",
        "--request-summary",
        "Initial summary for the workspace.",
        "--format",
        "json",
    ]);

    assert_eq!(initial_payload["spec_id"], "custom-spec-42");

    let workspace = temp.path().join(".spec-forge/specs/custom-spec-42");
    let initial_pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    assert_eq!(initial_pipeline["spec"]["id"], "custom-spec-42");
    assert_eq!(
        initial_pipeline["spec"]["workspace_path"],
        "specs/custom-spec-42"
    );
    assert_eq!(initial_pipeline["request"]["title"], "Initial Title");
    assert_eq!(
        initial_pipeline["request"]["summary"],
        "Initial summary for the workspace."
    );

    let exists_error = run_json_failure(&[
        "init",
        "--target",
        target,
        "--spec-id",
        "Custom Spec 42",
        "--request-title",
        "Second Title",
        "--request-summary",
        "Second summary should require force.",
        "--format",
        "json",
    ]);
    assert_eq!(exists_error["error"]["code"], "workspace_exists");

    let forced_payload = run_json_success(&[
        "init",
        "--target",
        target,
        "--spec-id",
        "Custom Spec 42",
        "--request-title",
        "Forced Title",
        "--request-summary",
        "Forced summary overwrites the workspace.",
        "--force",
        "--format",
        "json",
    ]);

    assert_eq!(forced_payload["spec_id"], "custom-spec-42");

    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    let request_context = read_json(&workspace.join("framing/request-context.yaml"));
    assert_eq!(pipeline["request"]["title"], "Forced Title");
    assert_eq!(
        pipeline["request"]["summary"],
        "Forced summary overwrites the workspace."
    );
    assert_eq!(
        request_context["problem"]["statement"],
        "Forced summary overwrites the workspace."
    );
}

#[test]
fn init_creates_workspace() {
    let temp = temp_target();

    bin()
        .args([
            "init",
            "--target",
            temp.path().to_str().expect("temp path utf8"),
            "--request-title",
            "Spec Execution Stage",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let workspace = temp.path().join(".spec-forge/specs/spec-execution-stage");
    assert!(workspace.join("pipeline-state.yaml").exists());
    assert!(workspace.join("handoff.yaml").exists());
    assert!(workspace.join("framing/request-context.yaml").exists());
    assert!(temp.path().join(".spec-forge/registry.yaml").exists());
}

#[test]
fn init_persists_matching_handoff_recommendations() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    bin()
        .args([
            "init",
            "--target",
            target,
            "--request-title",
            "Init Recommendations",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let workspace = temp.path().join(".spec-forge/specs/init-recommendations");
    let handoff = read_json(&workspace.join("handoff.yaml"));
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));

    assert_matching_recommendations(&handoff, "spec-forge-cli resolve");
    assert_matching_recommendations(&pipeline["handoff"], "spec-forge-cli resolve");
    assert_matching_handoff_snapshot(&workspace);
}

#[test]
fn artifact_get_rejects_traversal_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Path Invalid", "artifact-path-invalid");

    let payload = run_json_failure(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "artifact-path-invalid",
        "--file",
        "../pipeline-state.yaml",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("traversal"))
    );
}

#[test]
fn artifact_get_rejects_absolute_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Absolute Path", "artifact-absolute-path");

    let payload = run_json_failure(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "artifact-absolute-path",
        "--file",
        "/etc/passwd",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert_eq!(
        payload["error"]["message"],
        "Artifact path must be relative to the spec workspace."
    );
}

#[test]
fn artifact_put_emits_toml_success_payload() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Put Success", "artifact-put-success");

    let payload = run_toml_success(&[
        "artifact",
        "put",
        "--target",
        target,
        "--spec-id",
        "artifact-put-success",
        "--file",
        "framing/request-context.yaml",
        "--value",
        r#"{"problem":{"statement":"Artifact Put Success Updated"}}"#,
        "--format",
        "toml",
    ]);

    assert_eq!(
        payload.get("spec_id").and_then(TomlValue::as_str),
        Some("artifact-put-success")
    );
    assert_eq!(
        payload.get("operation").and_then(TomlValue::as_str),
        Some("put")
    );
    assert_eq!(
        payload.get("file").and_then(TomlValue::as_str),
        Some("framing/request-context.yaml")
    );
    assert_eq!(
        payload
            .get("artifact")
            .and_then(|value| value.get("problem"))
            .and_then(|value| value.get("statement"))
            .and_then(TomlValue::as_str),
        Some("Artifact Put Success Updated")
    );

    let artifact = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/artifact-put-success/framing/request-context.yaml"),
    );
    assert_eq!(
        artifact["problem"]["statement"],
        "Artifact Put Success Updated"
    );
}

#[test]
fn artifact_merge_emits_toml_success_payload() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Merge Success", "artifact-merge-success");

    let payload = run_toml_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "artifact-merge-success",
        "--file",
        "framing/request-context.yaml",
        "--value",
        r#"{"problem":{"goal":"Merged Goal"}}"#,
        "--format",
        "toml",
    ]);

    assert_eq!(
        payload.get("spec_id").and_then(TomlValue::as_str),
        Some("artifact-merge-success")
    );
    assert_eq!(
        payload.get("operation").and_then(TomlValue::as_str),
        Some("merge")
    );
    assert_eq!(
        payload.get("file").and_then(TomlValue::as_str),
        Some("framing/request-context.yaml")
    );
    assert_eq!(
        payload
            .get("artifact")
            .and_then(|value| value.get("problem"))
            .and_then(|value| value.get("statement"))
            .and_then(TomlValue::as_str),
        Some("Artifact Merge Success")
    );
    assert_eq!(
        payload
            .get("artifact")
            .and_then(|value| value.get("problem"))
            .and_then(|value| value.get("goal"))
            .and_then(TomlValue::as_str),
        Some("Merged Goal")
    );

    let artifact = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/artifact-merge-success/framing/request-context.yaml"),
    );
    assert_eq!(artifact["problem"]["statement"], "Artifact Merge Success");
    assert_eq!(artifact["problem"]["goal"], "Merged Goal");
}

#[test]
fn approve_with_note_writes_custom_confirmation_note() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Approve Custom Note", "approve-custom-note");

    let payload = run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "approve-custom-note",
        "--file",
        "framing/request-context.yaml",
        "--note",
        "Reviewed with product and approved for the next step.",
        "--format",
        "json",
    ]);

    assert_eq!(payload["operation"], "approve");
    assert_eq!(payload["artifact"]["approval"]["status"], "approved");
    assert_eq!(
        payload["artifact"]["approval"]["confirmed_by_user"],
        JsonValue::Bool(true)
    );
    assert_eq!(
        payload["artifact"]["approval"]["confirmation_note"],
        "Reviewed with product and approved for the next step."
    );

    let artifact = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/approve-custom-note/framing/request-context.yaml"),
    );
    assert_eq!(artifact["approval"]["status"], "approved");
    assert_eq!(
        artifact["approval"]["confirmed_by_user"],
        JsonValue::Bool(true)
    );
    assert_eq!(
        artifact["approval"]["confirmation_note"],
        "Reviewed with product and approved for the next step."
    );
}

#[test]
fn resolve_after_init_returns_intake_context() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    bin()
        .args([
            "init",
            "--target",
            target,
            "--request-title",
            "Resolve After Init",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let assert = bin()
        .args([
            "resolve",
            "--target",
            target,
            "--spec-id",
            "resolve-after-init",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(payload["spec_id"], "resolve-after-init");
    assert_eq!(payload["resolved_stage"], "intake");
    assert!(payload["workspace_exists"].as_bool().unwrap_or(false));
    assert!(payload.get("interactive_requirements").is_some());
}

#[test]
fn resolve_accepts_mode_and_emits_toml_output() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Resolve Toml Mode",
        "--format",
        "json",
    ]);

    let payload = run_toml_success(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "resolve-toml-mode",
        "--stage",
        "intake",
        "--mode",
        "guided_intake",
        "--format",
        "toml",
    ]);

    assert_eq!(
        payload.get("spec_id").and_then(TomlValue::as_str),
        Some("resolve-toml-mode")
    );
    assert_eq!(
        payload.get("resolved_stage").and_then(TomlValue::as_str),
        Some("intake")
    );
    assert_eq!(
        payload
            .get("ux")
            .and_then(|value| value.get("mode"))
            .and_then(|value| value.get("id"))
            .and_then(TomlValue::as_str),
        Some("guided_intake")
    );
    assert_eq!(
        payload
            .get("interactive_requirements")
            .and_then(|value| value.get("runtime_policy"))
            .and_then(TomlValue::as_str),
        Some("plan_mode_only")
    );
}

#[test]
fn resolve_rejects_unknown_mode_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Resolve Invalid Mode",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "resolve-invalid-mode",
        "--stage",
        "intake",
        "--mode",
        "missing-mode",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "ux_mode_missing");
    assert_eq!(
        payload["error"]["message"],
        "No UX mode \"missing-mode\" defined for skill \"spec-forge-intake\"."
    );
}

#[test]
fn resolve_rejects_invalid_mode_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Resolve Invalid Mode",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "resolve-invalid-mode",
        "--stage",
        "intake",
        "--mode",
        "not-a-real-mode",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "ux_mode_missing");
    assert_eq!(
        payload["error"]["message"],
        "No UX mode \"not-a-real-mode\" defined for skill \"spec-forge-intake\"."
    );
    assert_eq!(payload["error"]["details"], JsonValue::Null);
}

#[test]
fn resolve_write_persists_matching_handoff_snapshot() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Resolve Write Snapshot",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "resolve-write-snapshot",
        "--runtime-mode",
        "plan",
        "--write",
        "--format",
        "json",
    ]);

    let workspace = temp.path().join(".spec-forge/specs/resolve-write-snapshot");
    let handoff = read_json(&workspace.join("handoff.yaml"));
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));

    assert_matching_recommendations(&handoff, "spec-forge-cli apply");
    assert_matching_recommendations(&pipeline["handoff"], "spec-forge-cli apply");
    assert_matching_handoff_snapshot(&workspace);
}

#[test]
fn resolve_write_without_workspace_persists_invocation_state() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    let payload = run_json_success(&[
        "resolve",
        "--target",
        target,
        "--skill",
        "spec-forge",
        "--stage",
        "router",
        "--mode",
        "initialize_new_spec",
        "--runtime-mode",
        "plan",
        "--write",
        "--format",
        "json",
    ]);

    assert_eq!(payload["resolved_skill"], "spec-forge");
    assert_eq!(payload["resolved_stage"], "router");
    assert_eq!(payload["workspace_exists"], JsonValue::Bool(false));
    assert_eq!(payload["runtime_mode"], "plan");
    assert_eq!(payload["next_interaction"]["type"], "ask_parameter");

    let invocation = read_json(&temp.path().join(".spec-forge/invocation-state.yaml"));
    assert_eq!(invocation["skill"], "spec-forge");
    assert_eq!(invocation["stage"], "router");
    assert_eq!(invocation["mode"], "initialize_new_spec");
    assert_eq!(invocation["status"], "collecting");
    assert_eq!(invocation["next_interaction"]["type"], "ask_parameter");
    assert_eq!(
        invocation["next_interaction"]["parameter"]["id"],
        "chat_language"
    );
}

#[test]
fn resolve_infers_spec_id_from_workspace_target_path() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Workspace Target Resolve",
        "--format",
        "json",
    ]);

    let workspace = temp
        .path()
        .join(".spec-forge/specs/workspace-target-resolve");
    let payload = run_json_success(&[
        "resolve",
        "--target",
        workspace.to_str().expect("workspace path utf8"),
        "--format",
        "json",
    ]);

    assert_eq!(payload["spec_id"], "workspace-target-resolve");
    assert_eq!(
        payload["workspace"],
        workspace.to_str().expect("workspace path utf8")
    );
    assert_eq!(payload["resolved_stage"], "intake");
    assert_eq!(payload["target_dir"], target);
}

#[test]
fn gate_check_incomplete_returns_nonzero() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    bin()
        .args([
            "init",
            "--target",
            target,
            "--request-title",
            "Incomplete Gate",
            "--format",
            "json",
        ])
        .assert()
        .success();

    let assert = bin()
        .args([
            "gate",
            "check",
            "--target",
            target,
            "--spec-id",
            "incomplete-gate",
            "--stage",
            "intake",
            "--format",
            "json",
        ])
        .assert()
        .failure();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(payload["spec_id"], "incomplete-gate");
    assert_eq!(payload["stage"], "intake");
    assert_eq!(payload["passed"], JsonValue::Bool(false));
}

#[test]
fn gate_check_write_persists_report_file() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Gate Write Report",
        "--format",
        "json",
    ]);

    let payload = run_json_stdout_failure(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "gate-write-report",
        "--stage",
        "intake",
        "--write",
        "--format",
        "json",
    ]);

    let report = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/gate-write-report/gates/intake.yaml"),
    );
    assert_eq!(payload["stage"], "intake");
    assert_eq!(payload["passed"], JsonValue::Bool(false));
    assert_eq!(report["stage"], "intake");
    assert_eq!(report["passed"], JsonValue::Bool(false));
    assert_eq!(report["spec_id"], "gate-write-report");
    assert!(
        report["checks"]
            .as_array()
            .is_some_and(|checks| !checks.is_empty())
    );
}

#[test]
fn gate_check_requires_resolved_spec_id_when_workspace_context_is_ambiguous() {
    let temp = temp_target();

    let payload = run_json_failure(&[
        "gate",
        "check",
        "--target",
        temp.path().to_str().expect("temp path utf8"),
        "--stage",
        "intake",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "spec_id_unresolved");
    assert_eq!(
        payload["error"]["message"],
        "No spec_id could be resolved. Pass --spec-id or point to .spec-forge/specs/<spec-id>."
    );
}

#[test]
fn gate_check_reports_missing_workspace_for_explicit_spec_id() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    let payload = run_json_failure(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "missing-spec",
        "--stage",
        "intake",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "workspace_missing");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Spec workspace does not exist"))
    );
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("missing-spec"))
    );
}

#[test]
fn ux_validate_succeeds_against_repo_root() {
    let root = repo_root();

    let assert = bin()
        .args([
            "ux",
            "validate",
            "--target",
            root.to_str().expect("repo root utf8"),
            "--format",
            "json",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(payload["problem_count"], 0);
    assert!(
        payload["validated_files"]
            .as_array()
            .is_some_and(|files| !files.is_empty())
    );
}

#[test]
fn ux_validate_reports_problems_for_empty_directory() {
    let temp = temp_target();

    let assert = bin()
        .args([
            "ux",
            "validate",
            "--target",
            temp.path().to_str().expect("temp path utf8"),
            "--format",
            "json",
        ])
        .assert()
        .failure();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    assert!(payload["problem_count"].as_u64().unwrap_or(0) >= 1);
    assert!(payload["problems"].as_array().is_some_and(|problems| {
        problems.iter().any(|problem| {
            problem
                .as_str()
                .is_some_and(|message| message.contains("Missing shared UX contract"))
        })
    }));
}

#[test]
fn focus_help_includes_write_option() {
    let assert = bin()
        .args(["help", "focus", "--format", "json"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    let options = payload["options"].as_array().expect("options array");
    assert!(options.iter().any(|option| option["name"] == "--write"));
}

#[test]
fn resolve_structured_help_emits_toml_payload() {
    let payload = run_toml_success(&["help", "resolve", "--format", "toml"]);

    assert_eq!(
        payload.get("name").and_then(TomlValue::as_str),
        Some("spec-forge-cli resolve")
    );

    let command_path = payload
        .get("command_path")
        .and_then(TomlValue::as_array)
        .expect("command_path array");
    assert_eq!(command_path.len(), 1);
    assert_eq!(command_path[0].as_str(), Some("resolve"));

    let formats = payload
        .get("formats")
        .and_then(TomlValue::as_array)
        .expect("formats array");
    assert!(formats.iter().any(|format| format.as_str() == Some("toml")));

    let options = payload
        .get("options")
        .and_then(TomlValue::as_array)
        .expect("options array");
    assert!(options.iter().any(|option| {
        option.get("name").and_then(TomlValue::as_str) == Some("--runtime-mode")
    }));
    assert!(
        options
            .iter()
            .any(|option| { option.get("name").and_then(TomlValue::as_str) == Some("--write") })
    );

    let runtime_directories = payload
        .get("runtime_directories")
        .and_then(TomlValue::as_array)
        .expect("runtime_directories array");
    assert!(runtime_directories.iter().any(|entry| {
        entry.get("kind").and_then(TomlValue::as_str) == Some("data")
            && entry.get("default").and_then(TomlValue::as_str)
                == Some("<target>/.spec-forge/specs/<spec-id>/")
    }));

    let active_context = payload
        .get("active_context")
        .and_then(TomlValue::as_array)
        .expect("active_context array");
    assert!(active_context.iter().any(|entry| {
        entry.get("capability").and_then(TomlValue::as_str) == Some("precedence")
    }));
}

#[test]
fn artifact_get_structured_help_emits_yaml_payload() {
    let payload = run_yaml_success(&["help", "artifact", "get", "--format", "yaml"]);

    assert_eq!(payload["name"], "spec-forge-cli artifact get");
    assert_eq!(payload["command_path"], json!(["artifact", "get"]));
    assert!(
        payload["formats"]
            .as_array()
            .is_some_and(|formats| formats.iter().any(|format| format == "yaml (default)"))
    );
    assert!(payload["options"].as_array().is_some_and(|options| {
        options
            .iter()
            .any(|option| option["name"] == "--file" && option["required"] == JsonValue::Bool(true))
    }));
    assert!(
        payload["options"]
            .as_array()
            .is_some_and(|options| { options.iter().any(|option| option["name"] == "--format") })
    );
}

#[test]
fn resolve_help_flag_renders_command_manual() {
    let assert = bin().args(["resolve", "--help"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli resolve - Resolve the next workflow interaction"));
    assert!(stdout.contains("spec-forge-cli resolve [--target PATH]"));
    assert!(stdout.contains("--runtime-mode default|plan"));
}

#[test]
fn resolve_help_flag_renders_command_manual_even_with_format_before_help() {
    let assert = bin()
        .args(["resolve", "--format", "json", "--help"])
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli resolve - Resolve the next workflow interaction"));
    assert!(stdout.contains("spec-forge-cli resolve [--target PATH]"));
}

#[test]
fn artifact_get_help_flag_renders_command_manual() {
    let assert = bin().args(["artifact", "get", "--help"]).assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains(
        "spec-forge-cli artifact get - Read one artifact from the resolved spec workspace."
    ));
    assert!(stdout.contains(
        "spec-forge-cli artifact get [--target PATH] [--spec-id ID] --file RELATIVE_PATH"
    ));
}

#[test]
fn artifact_group_without_subcommand_renders_help_page() {
    let assert = bin().arg("artifact").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli artifact - Non-leaf command group"));
    assert!(stdout.contains("spec-forge-cli artifact get [OPTIONS]"));
    assert!(stdout.contains("spec-forge-cli artifact merge [OPTIONS]"));
}

#[test]
fn gate_group_without_subcommand_renders_help_page() {
    let assert = bin().arg("gate").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli gate - Non-leaf command group"));
    assert!(stdout.contains("spec-forge-cli gate check [OPTIONS]"));
    assert!(stdout.contains("Evaluate one workflow stage gate."));
}

#[test]
fn stage_group_without_subcommand_renders_help_page() {
    let assert = bin().arg("stage").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli stage - Non-leaf command group"));
    assert!(stdout.contains("spec-forge-cli stage advance [OPTIONS]"));
    assert!(stdout.contains("workflow stage transitions"));
}

#[test]
fn ux_group_without_subcommand_renders_help_page() {
    let assert = bin().arg("ux").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");

    assert!(stdout.contains("spec-forge-cli ux - Non-leaf command group"));
    assert!(stdout.contains("spec-forge-cli ux validate [OPTIONS]"));
    assert!(stdout.contains("UX-contract tooling"));
}

#[test]
fn help_unknown_path_returns_structured_error() {
    let payload = run_json_failure(&["help", "unknown-command", "--format", "json"]);

    assert_eq!(payload["error"]["code"], "help_path_unknown");
    assert_eq!(
        payload["error"]["message"],
        "Unknown help command path: unknown-command"
    );
}

#[test]
fn init_writes_expected_pipeline_request_title() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    bin()
        .args([
            "init",
            "--target",
            target,
            "--request-title",
            "Pipeline Title",
        ])
        .assert()
        .success();

    let pipeline = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/pipeline-title/pipeline-state.yaml"),
    );
    assert_eq!(pipeline["request"]["title"], "Pipeline Title");
    assert_eq!(pipeline["phase"]["current"], "intake");
}

#[test]
fn focus_components_respects_max_items_and_writes_selected_ids() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Focus Components",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "focus-components",
        "--file",
        "architecture/component-index.yaml",
        "--value",
        r#"{"items":[{"id":"component-low","title":"Component Low","priority":"low","sequence":4,"status":"pending"},{"id":"component-auth","title":"Component Auth","priority":"high","sequence":2,"status":"ready"},{"id":"component-billing","title":"Component Billing","priority":"high","sequence":1,"review_status":"draft"},{"id":"component-legacy","title":"Component Legacy","priority":"medium","sequence":0,"status":"approved"}]}"#,
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "focus",
        "--target",
        target,
        "--spec-id",
        "focus-components",
        "--stage",
        "components",
        "--max-items",
        "2",
        "--write",
        "--format",
        "json",
    ]);

    assert_eq!(payload["stage"], "components");
    assert_eq!(payload["max_items"], json!(2));
    assert_eq!(
        payload["selected_ids"],
        json!(["component-billing", "component-auth"])
    );
    assert_eq!(
        payload["reason"],
        "Use the next pending item set derived from the stage index."
    );

    let workspace = temp.path().join(".spec-forge/specs/focus-components");
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    assert_eq!(pipeline["focus"]["kind"], "components");
    assert_eq!(
        pipeline["focus"]["ids"],
        json!(["component-billing", "component-auth"])
    );
}

#[test]
fn focus_rejects_invalid_stage_with_argument_parse_failed_error() {
    let payload = run_json_failure(&["focus", "--stage", "invalid", "--format", "json"]);

    assert_eq!(payload["error"]["code"], "argument_parse_failed");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("invalid value"))
    );
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("journeys"))
    );
}

#[test]
fn focus_journeys_respects_max_items_and_writes_selected_ids() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Focus Journeys",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "focus-journeys",
        "--file",
        "architecture/journey-index.yaml",
        "--value",
        r#"{"items":[{"id":"journey-low","title":"Journey Low","priority":"low","sequence":5,"status":"pending"},{"id":"journey-onboarding","title":"Journey Onboarding","priority":"high","sequence":2,"status":"ready"},{"id":"journey-checkout","title":"Journey Checkout","priority":"high","sequence":1,"review_status":"draft"},{"id":"journey-approved","title":"Journey Approved","priority":"medium","sequence":0,"status":"approved"}]}"#,
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "focus",
        "--target",
        target,
        "--spec-id",
        "focus-journeys",
        "--stage",
        "journeys",
        "--max-items",
        "2",
        "--write",
        "--format",
        "json",
    ]);

    assert_eq!(payload["stage"], "journeys");
    assert_eq!(payload["max_items"], json!(2));
    assert_eq!(
        payload["selected_ids"],
        json!(["journey-checkout", "journey-onboarding"])
    );
    assert_eq!(
        payload["reason"],
        "Use the next pending item set derived from the stage index."
    );

    let workspace = temp.path().join(".spec-forge/specs/focus-journeys");
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    assert_eq!(pipeline["focus"]["kind"], "journeys");
    assert_eq!(
        pipeline["focus"]["ids"],
        json!(["journey-checkout", "journey-onboarding"])
    );
}

#[test]
fn focus_components_fails_when_no_pending_items_are_available() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Focus Components Empty",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "focus-components-empty",
        "--file",
        "architecture/component-index.yaml",
        "--value",
        r#"{"items":[{"id":"component-approved","title":"Approved","priority":"high","status":"approved"},{"id":"component-deferred","title":"Deferred","priority":"medium","status":"deferred"}]}"#,
        "--format",
        "json",
    ]);

    let assert = bin()
        .args([
            "focus",
            "--target",
            target,
            "--spec-id",
            "focus-components-empty",
            "--stage",
            "components",
            "--max-items",
            "2",
            "--format",
            "json",
        ])
        .assert()
        .failure();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let payload: JsonValue = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(payload["stage"], "components");
    assert_eq!(payload["max_items"], json!(2));
    assert_eq!(payload["selected_ids"], json!([]));
    assert_eq!(
        payload["reason"],
        "Use the next pending item set derived from the stage index."
    );
}

#[test]
fn focus_write_persists_matching_handoff_recommendations() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    bin()
        .args([
            "init",
            "--target",
            target,
            "--request-title",
            "Focus Recommendations",
            "--format",
            "json",
        ])
        .assert()
        .success();

    bin()
        .args([
            "focus",
            "--target",
            target,
            "--spec-id",
            "focus-recommendations",
            "--stage",
            "journeys",
            "--write",
            "--format",
            "json",
        ])
        .assert()
        .failure();

    let workspace = temp.path().join(".spec-forge/specs/focus-recommendations");
    let handoff = read_json(&workspace.join("handoff.yaml"));
    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));

    assert_matching_recommendations(&handoff, "spec-forge-cli resolve");
    assert_matching_recommendations(&pipeline["handoff"], "spec-forge-cli resolve");
    assert_matching_handoff_snapshot(&workspace);
}

#[test]
fn focus_components_returns_empty_selection_payload_on_failure() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Focus Empty Components",
        "--format",
        "json",
    ]);

    let payload = run_json_stdout_failure(&[
        "focus",
        "--target",
        target,
        "--spec-id",
        "focus-empty-components",
        "--stage",
        "components",
        "--format",
        "json",
    ]);

    assert_eq!(payload["stage"], "components");
    assert_eq!(payload["selected_ids"], json!([]));
    assert_eq!(
        payload["reason"],
        "Use the next pending item set derived from the stage index."
    );
}

#[test]
fn apply_choice_updates_active_roles_across_request_context_and_role_map() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Active Roles",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-active-roles",
        "--stage",
        "intake",
        "--choice",
        "active_roles",
        "--value",
        "\"product-manager\"",
        "--value",
        "\"tech-lead\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["applied"]["kind"], "choice");
    assert_eq!(payload["applied"]["id"], "active_roles");
    assert_eq!(
        payload["applied"]["value"],
        json!(["product-manager", "tech-lead"])
    );

    let workspace = temp.path().join(".spec-forge/specs/apply-active-roles");
    let request_context = read_json(&workspace.join("framing/request-context.yaml"));
    let role_map = read_json(&workspace.join("framing/role-map.yaml"));

    assert_eq!(
        request_context["actors"]["reviewers"],
        json!(["product-manager", "tech-lead"])
    );

    let roles = role_map["roles"].as_array().expect("roles array");
    let enabled_role_ids = roles
        .iter()
        .filter(|role| role["enabled"].as_bool().unwrap_or(false))
        .filter_map(|role| role["id"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(enabled_role_ids, vec!["product-manager", "tech-lead"]);
}

#[test]
fn apply_choice_rejects_invalid_option_value() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Invalid Choice Value",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-invalid-choice-value",
        "--stage",
        "intake",
        "--choice",
        "active_roles",
        "--value",
        "\"not-a-real-role\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "choice_value_invalid");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("not-a-real-role"))
    );
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("active_roles"))
    );
}

#[test]
fn apply_rejects_parameter_and_choice_together() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Invalid Interaction Target",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-invalid-interaction-target",
        "--stage",
        "intake",
        "--parameter",
        "problem_goal",
        "--choice",
        "active_roles",
        "--value",
        "\"Ship the CLI\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "invalid_interaction_target");
}

#[test]
fn apply_requires_at_least_one_value() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Missing Value",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-missing-value",
        "--stage",
        "intake",
        "--choice",
        "active_roles",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "missing_value");
    assert_eq!(
        payload["error"]["message"],
        "At least one --value must be provided."
    );
}

#[test]
fn apply_rejects_invalid_yaml_value() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Apply Parse Failure", "apply-parse-failure");

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-parse-failure",
        "--stage",
        "intake",
        "--parameter",
        "problem_goal",
        "--value",
        "{broken",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "value_parse_failed");
}

#[test]
fn apply_rejects_invalid_choice_option_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Invalid Choice Option",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-invalid-choice-option",
        "--stage",
        "intake",
        "--choice",
        "active_roles",
        "--value",
        "\"not-a-real-role\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "choice_value_invalid");
    assert_eq!(
        payload["error"]["message"],
        "Choice value(s) [\"not-a-real-role\"] are not in the available options for \"active_roles\"."
    );
    assert_eq!(payload["error"]["details"], JsonValue::Null);
}

#[test]
fn apply_rejects_unknown_parameter_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Missing Parameter",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-missing-parameter",
        "--stage",
        "intake",
        "--parameter",
        "not_a_real_parameter",
        "--value",
        "\"ignored\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "parameter_missing");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("not_a_real_parameter"))
    );
}

#[test]
fn apply_rejects_unknown_choice_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Missing Choice",
        "--format",
        "json",
    ]);

    let payload = run_json_failure(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-missing-choice",
        "--stage",
        "intake",
        "--choice",
        "not_a_real_choice",
        "--value",
        "\"ignored\"",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "choice_missing");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("not_a_real_choice"))
    );
}

#[test]
fn apply_updates_workspace_artifact_and_returns_follow_up_context() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Workspace Updates",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-workspace-updates",
        "--stage",
        "intake",
        "--parameter",
        "problem_goal",
        "--value",
        "\"Ship the dedicated Rust CLI.\"",
        "--format",
        "json",
    ]);

    let request_context = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/apply-workspace-updates/framing/request-context.yaml"),
    );
    assert_eq!(
        request_context["problem"]["goal"],
        "Ship the dedicated Rust CLI."
    );
    assert_eq!(payload["applied"]["id"], "problem_goal");
    assert!(payload["updated_files"].as_array().is_some_and(|files| {
        files
            .iter()
            .any(|file| file == "framing/request-context.yaml")
    }));
    assert!(payload.get("next_interaction").is_some());
}

#[test]
fn apply_runtime_mode_default_requires_plan_mode_for_plan_only_skill() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Runtime Default",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-runtime-default",
        "--stage",
        "intake",
        "--parameter",
        "problem_goal",
        "--value",
        "\"Ship CLI\"",
        "--runtime-mode",
        "default",
        "--format",
        "json",
    ]);

    assert_eq!(payload["runtime_mode"], "default");
    assert_eq!(
        payload["interactive_requirements"]["runtime_mode"],
        "default"
    );
    assert_eq!(
        payload["interactive_requirements"]["requires_plan_mode"],
        JsonValue::Bool(true)
    );
    assert_eq!(payload["next_interaction"]["type"], "switch_to_plan_mode");
}

#[test]
fn apply_runtime_mode_plan_allows_interactive_collection() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Runtime Plan",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-runtime-plan",
        "--stage",
        "intake",
        "--parameter",
        "problem_goal",
        "--value",
        "\"Ship CLI\"",
        "--runtime-mode",
        "plan",
        "--format",
        "json",
    ]);

    assert_eq!(payload["runtime_mode"], "plan");
    assert_eq!(payload["interactive_requirements"]["runtime_mode"], "plan");
    assert_eq!(
        payload["interactive_requirements"]["requires_plan_mode"],
        JsonValue::Bool(false)
    );
    assert_eq!(payload["next_interaction"]["type"], "ask_parameter");
    assert_eq!(
        payload["next_interaction"]["parameter"]["id"],
        "chat_language"
    );
}

#[test]
fn apply_target_dir_uses_generic_invocation_writeback() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Apply Generic Writeback",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "apply-generic-writeback",
        "--stage",
        "intake",
        "--parameter",
        "target_dir",
        "--value",
        "\"/tmp/spec-forge-generic\"",
        "--format",
        "json",
    ]);

    let invocation = read_json(&temp.path().join(".spec-forge/invocation-state.yaml"));
    assert_eq!(
        invocation["parameters"]["target_dir"],
        "/tmp/spec-forge-generic"
    );
    assert_eq!(payload["applied"]["id"], "target_dir");
    assert!(
        payload["updated_files"]
            .as_array()
            .is_some_and(|files| files.iter().any(|file| file == "invocation-state.yaml"))
    );
}

#[test]
fn artifact_get_missing_file_returns_structured_json_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Missing", "artifact-missing");

    let payload = run_json_failure(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "artifact-missing",
        "--file",
        "journeys/not-created-yet.yaml",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_missing");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("journeys/not-created-yet.yaml"))
    );
}

#[test]
fn artifact_get_emits_toml_success_payload_for_existing_artifact() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Get Toml", "artifact-get-toml");

    let payload = run_toml_success(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "artifact-get-toml",
        "--file",
        "handoff.yaml",
        "--format",
        "toml",
    ]);

    assert_eq!(
        payload.get("spec_id").and_then(TomlValue::as_str),
        Some("artifact-get-toml")
    );
    assert_eq!(
        payload.get("file").and_then(TomlValue::as_str),
        Some("handoff.yaml")
    );
    assert_eq!(
        payload
            .get("artifact")
            .and_then(|value| value.get("recommended_script"))
            .and_then(TomlValue::as_str),
        Some("spec-forge-cli resolve")
    );
    assert_eq!(
        payload
            .get("artifact")
            .and_then(|value| value.get("next_interaction"))
            .and_then(|value| value.get("type"))
            .and_then(TomlValue::as_str),
        Some("switch_to_plan_mode")
    );
}

#[test]
fn artifact_put_rejects_invalid_yaml_value_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Put Invalid Value", "artifact-put-invalid");

    let payload = run_json_failure(&[
        "artifact",
        "put",
        "--target",
        target,
        "--spec-id",
        "artifact-put-invalid",
        "--file",
        "framing/request-context.yaml",
        "--value",
        "[1, 2",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "value_parse_failed");
}

#[test]
fn artifact_merge_rejects_invalid_yaml_value_with_structured_error() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(
        &temp,
        "Artifact Merge Invalid Value",
        "artifact-merge-invalid",
    );

    let payload = run_json_failure(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "artifact-merge-invalid",
        "--file",
        "framing/request-context.yaml",
        "--value",
        "[1, 2",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "value_parse_failed");
}

#[test]
fn approve_without_note_writes_default_confirmation_note() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Approve Default Note", "approve-default-note");

    let payload = run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "approve-default-note",
        "--file",
        "architecture/solution-outline.yaml",
        "--format",
        "json",
    ]);

    assert_eq!(
        payload["artifact"]["approval"]["confirmation_note"],
        "Approved via spec-forge-cli approve."
    );

    let artifact = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/approve-default-note/architecture/solution-outline.yaml"),
    );
    assert_eq!(artifact["approval"]["status"], "approved");
    assert_eq!(
        artifact["approval"]["confirmed_by_user"],
        JsonValue::Bool(true)
    );
    assert_eq!(
        artifact["approval"]["confirmation_note"],
        "Approved via spec-forge-cli approve."
    );
}

#[test]
fn artifact_get_rejects_empty_relative_path() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Empty Path", "artifact-empty-path");

    let payload = run_json_failure(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "artifact-empty-path",
        "--file",
        "   ",
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert_eq!(
        payload["error"]["message"],
        "Artifact path cannot be empty."
    );
}

#[test]
fn router_requires_plan_mode_even_after_language_handshake_is_resolved() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Plan Only Router",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "plan-only-router",
        "--skill",
        "spec-forge",
        "--stage",
        "router",
        "--parameter",
        "chat_language",
        "--value",
        "\"Chinese\"",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "apply",
        "--target",
        target,
        "--spec-id",
        "plan-only-router",
        "--skill",
        "spec-forge",
        "--stage",
        "router",
        "--parameter",
        "file_language",
        "--value",
        "\"English\"",
        "--format",
        "json",
    ]);

    let payload = run_json_success(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "plan-only-router",
        "--skill",
        "spec-forge",
        "--stage",
        "router",
        "--format",
        "json",
    ]);

    assert_eq!(payload["resolved_skill"], "spec-forge");
    assert_eq!(payload["resolved_stage"], "router");
    assert_eq!(
        payload["interactive_requirements"]["runtime_policy"],
        "plan_mode_only"
    );
    assert_eq!(
        payload["interactive_requirements"]["plan_mode_only"],
        JsonValue::Bool(true)
    );
    assert!(
        payload["interactive_requirements"]["missing_parameters"]
            .as_array()
            .is_some_and(|items| items.is_empty())
    );
    assert_eq!(
        payload["interactive_requirements"]["next_interaction"]["type"],
        "switch_to_plan_mode"
    );
}

#[test]
fn artifact_merge_creates_missing_file_from_empty_object() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Merge Create", "artifact-merge-create");

    let payload = run_yaml_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "artifact-merge-create",
        "--file",
        "journeys/research-notes.yaml",
        "--value",
        r#"{"items":[{"id":"journey-research","title":"Research Notes"}]}"#,
        "--format",
        "yaml",
    ]);

    assert_eq!(payload["spec_id"], "artifact-merge-create");
    assert_eq!(payload["operation"], "merge");
    assert_eq!(payload["file"], "journeys/research-notes.yaml");
    assert_eq!(payload["artifact"]["items"][0]["id"], "journey-research");

    let artifact = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/artifact-merge-create/journeys/research-notes.yaml"),
    );
    assert_eq!(artifact["items"][0]["id"], "journey-research");
    assert_eq!(artifact["items"][0]["title"], "Research Notes");
}

#[test]
fn artifact_put_rejects_traversal_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Put Traversal", "artifact-put-traversal");

    let payload = run_json_failure(&[
        "artifact",
        "put",
        "--target",
        target,
        "--spec-id",
        "artifact-put-traversal",
        "--file",
        "../pipeline-state.yaml",
        "--value",
        r#"{"bad":true}"#,
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("traversal components"))
    );
}

#[test]
fn artifact_put_rejects_absolute_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Put Absolute", "artifact-put-absolute");

    let payload = run_json_failure(&[
        "artifact",
        "put",
        "--target",
        target,
        "--spec-id",
        "artifact-put-absolute",
        "--file",
        "/etc/passwd",
        "--value",
        r#"{"bad":true}"#,
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert_eq!(
        payload["error"]["message"],
        "Artifact path must be relative to the spec workspace."
    );
}

#[test]
fn artifact_merge_rejects_traversal_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(
        &temp,
        "Artifact Merge Traversal",
        "artifact-merge-traversal",
    );

    let payload = run_json_failure(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "artifact-merge-traversal",
        "--file",
        "../pipeline-state.yaml",
        "--value",
        r#"{"bad":true}"#,
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert!(
        payload["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("traversal components"))
    );
}

#[test]
fn artifact_merge_rejects_absolute_paths() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Artifact Merge Absolute", "artifact-merge-absolute");

    let payload = run_json_failure(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "artifact-merge-absolute",
        "--file",
        "/etc/passwd",
        "--value",
        r#"{"bad":true}"#,
        "--format",
        "json",
    ]);

    assert_eq!(payload["error"]["code"], "artifact_path_invalid");
    assert_eq!(
        payload["error"]["message"],
        "Artifact path must be relative to the spec workspace."
    );
}

#[test]
fn stage_advance_returns_failure_when_requested_stage_does_not_match_pipeline_state() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(&temp, "Stage Advance Mismatch", "stage-advance-mismatch");

    let payload = run_json_stdout_failure(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "stage-advance-mismatch",
        "--stage",
        "architecture",
        "--format",
        "json",
    ]);

    assert_eq!(payload["spec_id"], "stage-advance-mismatch");
    assert_eq!(payload["advanced_from"], "architecture");
    assert_eq!(payload["next_stage"], "intake");
    assert!(
        payload["error"]
            .as_str()
            .is_some_and(|message| message.contains("Cannot advance architecture"))
    );
}

#[test]
fn stage_advance_failure_still_writes_gate_report_file() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(
        &temp,
        "Stage Advance Writes Gate Report",
        "stage-advance-writes-gate-report",
    );

    let payload = run_json_stdout_failure(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "stage-advance-writes-gate-report",
        "--stage",
        "architecture",
        "--format",
        "json",
    ]);

    let report = read_json(
        &temp
            .path()
            .join(".spec-forge/specs/stage-advance-writes-gate-report/gates/architecture.yaml"),
    );
    assert_eq!(payload["advanced_from"], "architecture");
    assert_eq!(payload["next_stage"], "intake");
    assert_eq!(report["stage"], "architecture");
    assert_eq!(report["passed"], JsonValue::Bool(false));
    assert!(
        report["checks"]
            .as_array()
            .is_some_and(|checks| !checks.is_empty())
    );
}

#[test]
fn stage_advance_returns_gate_report_when_gate_has_not_passed() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    init_workspace(
        &temp,
        "Stage Advance Gate Failure",
        "stage-advance-gate-failure",
    );

    let payload = run_json_stdout_failure(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "stage-advance-gate-failure",
        "--stage",
        "intake",
        "--format",
        "json",
    ]);

    assert_eq!(payload["stage"], "intake");
    assert_eq!(payload["passed"], JsonValue::Bool(false));
    assert_eq!(payload["recommended_next_stage"], "intake");
    assert!(
        payload["blocking_checks"]
            .as_array()
            .is_some_and(|checks| checks.iter().any(|check| check == "problem_goal"))
    );
    assert!(payload["checks"].as_array().is_some_and(|checks| {
        checks.iter().any(|check| {
            check["id"] == "request_context_approved" && check["passed"] == JsonValue::Bool(false)
        })
    }));
}

#[test]
fn full_workflow_can_advance_to_complete() {
    let temp = temp_target();
    let target = temp.path().to_str().expect("temp path utf8");

    run_json_success(&[
        "init",
        "--target",
        target,
        "--request-title",
        "Full Workflow",
        "--format",
        "json",
    ]);

    let workspace = temp.path().join(".spec-forge/specs/full-workflow");

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "framing/request-context.yaml",
        "--value",
        r#"{"problem":{"goal":"Ship the native spec-forge CLI."},"scope":{"in_scope":["Replace the script surface"]},"actors":{"primary_users":["spec-forge operators"]}}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "framing/request-context.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "framing/role-map.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "intake",
        "--write",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "intake",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/solution-outline.yaml",
        "--value",
        r#"{"solution":{"summary":"Drive the full workflow through the native Rust CLI."}}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/solution-outline.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/journey-index.yaml",
        "--value",
        r#"{"items":[{"id":"journey-alpha","title":"Journey Alpha"}]}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/journey-index.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/component-index.yaml",
        "--value",
        r#"{"items":[{"id":"component-alpha","title":"Component Alpha"}]}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "architecture/component-index.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "architecture",
        "--write",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "architecture",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "journeys/batches.yaml",
        "--value",
        r#"{"batches":[{"id":"journey-batch-1","item_ids":["journey-alpha"],"status":"approved"}]}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "journeys/batches.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "journeys/journey-alpha.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "journeys",
        "--write",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "journeys",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "components/batches.yaml",
        "--value",
        r#"{"batches":[{"id":"component-batch-1","item_ids":["component-alpha"],"status":"approved"}]}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "components/batches.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "components/component-alpha.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "components",
        "--write",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "components",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "merge",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "synthesis/implementation-spec.yaml",
        "--value",
        r#"{"summary":{"scope":["Ship the native CLI"]},"acceptance_criteria":["All workflow stages pass"],"unresolved_items":[],"implementation_gate":{"ready":true}}"#,
        "--format",
        "json",
    ]);
    run_json_success(&[
        "approve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "synthesis/implementation-spec.yaml",
        "--note",
        "Approved in integration test.",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "readiness",
        "--write",
        "--format",
        "json",
    ]);
    run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "readiness",
        "--format",
        "json",
    ]);

    run_json_success(&[
        "artifact",
        "put",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "synthesis/implementation-report.yaml",
        "--value",
        r#"{"implementation":{"completed":true,"summary":"Workflow implementation and validation completed.","changed_files":["spec-forge-cli/src/workflow.rs"]},"validations":[{"name":"cargo test","status":"passed"}],"blockers":[]}"#,
        "--format",
        "json",
    ]);

    let implement_gate_payload = run_json_success(&[
        "gate",
        "check",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "implement",
        "--write",
        "--format",
        "json",
    ]);
    assert_eq!(implement_gate_payload["passed"], JsonValue::Bool(true));

    let advance_payload = run_json_success(&[
        "stage",
        "advance",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--stage",
        "implement",
        "--format",
        "json",
    ]);
    assert_eq!(advance_payload["next_stage"], "complete");

    let pipeline = read_json(&workspace.join("pipeline-state.yaml"));
    let handoff_payload = run_json_success(&[
        "artifact",
        "get",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--file",
        "handoff.yaml",
        "--format",
        "json",
    ]);
    let handoff = &handoff_payload["artifact"];
    assert_eq!(pipeline["phase"]["status"], "complete");
    assert_eq!(
        pipeline["stages"]["implement"]["approved"],
        JsonValue::Bool(true)
    );
    assert_eq!(pipeline["ux"]["stage"], "complete");
    assert_eq!(handoff["to_stage"], "complete");
    assert_eq!(handoff["next_interaction"]["type"], "ready");
    assert_eq!(handoff["required_inputs"], json!([]));
    assert_matching_recommendations(handoff, "spec-forge-cli resolve");
    assert_matching_recommendations(&pipeline["handoff"], "spec-forge-cli resolve");
    assert_matching_handoff_snapshot(&workspace);

    let resolved_complete = run_json_success(&[
        "resolve",
        "--target",
        target,
        "--spec-id",
        "full-workflow",
        "--write",
        "--format",
        "json",
    ]);
    assert_eq!(resolved_complete["resolved_stage"], "complete");
    assert_eq!(resolved_complete["next_interaction"]["type"], "ready");
    assert!(
        resolved_complete["interactive_requirements"]["missing_parameters"]
            .as_array()
            .is_some_and(|items| items.is_empty())
    );
    let pipeline_after_resolve = read_json(&workspace.join("pipeline-state.yaml"));
    assert_eq!(pipeline_after_resolve["ux"]["stage"], "complete");
    assert_eq!(
        pipeline_after_resolve["handoff"]["next_interaction"]["type"],
        "ready"
    );
    assert_matching_handoff_snapshot(&workspace);
}

#[test]
fn ux_validate_failure_emits_toml_payload() {
    let temp = temp_target();

    let payload = run_toml_failure(&[
        "ux",
        "validate",
        "--target",
        temp.path().to_str().expect("temp path utf8"),
        "--format",
        "toml",
    ]);

    let problems = payload
        .get("problems")
        .and_then(TomlValue::as_array)
        .expect("problems array");
    assert!(
        payload
            .get("problem_count")
            .and_then(TomlValue::as_integer)
            .is_some_and(|count| count >= 1)
    );
    assert_eq!(
        payload.get("problem_count").and_then(TomlValue::as_integer),
        Some(problems.len() as i64)
    );
    assert!(problems.iter().any(|problem| {
        problem
            .as_str()
            .is_some_and(|message| message.contains("Missing shared UX contract"))
    }));
}

#[test]
fn ux_validate_failure_emits_yaml_payload() {
    let temp = temp_target();

    let payload = run_yaml_failure(&[
        "ux",
        "validate",
        "--target",
        temp.path().to_str().expect("temp path utf8"),
        "--format",
        "yaml",
    ]);

    let problems = payload["problems"].as_array().expect("problems array");
    assert!(
        payload["problem_count"]
            .as_u64()
            .is_some_and(|count| count >= 1)
    );
    assert_eq!(payload["problem_count"], json!(problems.len()));
    assert!(problems.iter().any(|problem| {
        problem
            .as_str()
            .is_some_and(|message| message.contains("Missing shared UX contract"))
    }));
}
