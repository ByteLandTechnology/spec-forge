#![allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_if,
    clippy::result_large_err,
    clippy::too_many_arguments
)]

use crate::output::CliError;
use chrono::{SecondsFormat, Utc};
use regex::Regex;
use serde_json::{Map, Value as JsonValue, json};
use std::cmp::Ordering;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const ROUTER_STAGE: &str = "router";
pub const STAGES: [&str; 6] = [
    "intake",
    "architecture",
    "journeys",
    "components",
    "readiness",
    "implement",
];

const WORKSPACE_DIRNAME: &str = ".spec-forge";
const SPECS_DIRNAME: &str = "specs";
const REGISTRY_FILENAME: &str = "registry.yaml";
const INVOCATION_STATE_FILENAME: &str = "invocation-state.yaml";
const UX_CONTRACT_PATH: &str = "spec-forge/assets/contracts/ux-contracts.yaml";

pub struct CommandOutcome {
    pub payload: JsonValue,
    pub exit_code: i32,
}

pub fn init_command(
    target: &str,
    spec_id: Option<&str>,
    request_title: Option<&str>,
    request_summary: Option<&str>,
    force: bool,
) -> Result<CommandOutcome, CliError> {
    let collection = collection_root(target)?;
    ensure_collection_dirs(&collection)?;
    let invocation_state = load_invocation_state(&collection)?;

    let title = request_title
        .and_then(non_empty_string)
        .or_else(|| json_get_string(&invocation_state, "parameters.request_title"))
        .unwrap_or_default();
    let summary = request_summary
        .and_then(non_empty_string)
        .or_else(|| json_get_string(&invocation_state, "parameters.request_summary"))
        .unwrap_or_default();

    let resolved_spec_id = if let Some(explicit) = spec_id.and_then(non_empty_string) {
        slugify_spec_id(&explicit)
    } else if let Some(inferred) = infer_spec_id(target)? {
        inferred
    } else if let Some(from_invocation) = json_get_string(&invocation_state, "parameters.spec_id") {
        slugify_spec_id(&from_invocation)
    } else {
        allocate_spec_id(&collection, if title.is_empty() { "spec" } else { &title })?
    };

    let workspace = spec_workspace(&collection, &resolved_spec_id);
    if workspace.exists() && workspace.read_dir().map_err(io_error)?.next().is_some() && !force {
        return Err(CliError::new(
            "workspace_exists",
            format!(
                "Workspace already exists at {}. Use --force to reinitialize.",
                workspace.display()
            ),
        ));
    }

    ensure_workspace_dirs(&workspace)?;

    let mut pipeline = template("pipeline-state.yaml")?;
    let mut handoff = template("handoff.yaml")?;
    let mut request_context = template("request-context.yaml")?;
    let role_map = template("role-map.yaml")?;
    let solution_outline = template("solution-outline.yaml")?;
    let journey_index = template("journey-index.yaml")?;
    let component_index = template("component-index.yaml")?;
    let journey_batches = template("journey-batches.yaml")?;
    let component_batches = template("component-batches.yaml")?;
    let implementation_spec = template("implementation-spec.yaml")?;
    let implementation_report = template("implementation-report.yaml")?;

    let created_at = now_utc();
    set_path(&mut pipeline, "spec.id", json!(resolved_spec_id));
    set_path(
        &mut pipeline,
        "spec.workspace_path",
        json!(format!("{}/{}", SPECS_DIRNAME, resolved_spec_id)),
    );
    set_path(
        &mut pipeline,
        "workspace.target_path",
        json!(
            collection
                .parent()
                .unwrap_or(&collection)
                .to_string_lossy()
                .to_string()
        ),
    );
    set_path(
        &mut pipeline,
        "workspace.spec_root",
        json!(format!("{}/{}", SPECS_DIRNAME, resolved_spec_id)),
    );
    set_path(&mut pipeline, "workspace.created_at", json!(created_at));
    set_path(&mut pipeline, "request.title", json!(title));
    set_path(&mut pipeline, "request.summary", json!(summary));
    set_path(
        &mut pipeline,
        "handoff.reason",
        json!("Initialize the spec workspace and begin intake."),
    );

    set_path(
        &mut handoff,
        "reason",
        json!("Initialize the spec workspace and begin intake."),
    );
    set_handoff_recommended_command(&mut handoff, "spec-forge-cli init");
    set_path(
        &mut handoff,
        "required_inputs",
        json!(["target_dir", "spec_id", "request_title", "request_summary"]),
    );

    set_path(
        &mut request_context,
        "problem.statement",
        json!(if summary.is_empty() {
            title.clone()
        } else {
            summary.clone()
        }),
    );
    set_path(&mut request_context, "problem.goal", json!(""));

    let resolution = resolve_ux_contract(
        Some("spec-forge-intake"),
        Some("intake"),
        None,
        &json!({
            "workspace": {
                "target_dir": collection.parent().unwrap_or(&collection).to_string_lossy().to_string(),
                "collection_root": collection.to_string_lossy().to_string(),
                "collection_exists": true,
                "spec_id": resolved_spec_id,
                "spec_id_source": "workspace_context",
                "spec_exists": true,
                "spec_root": workspace.to_string_lossy().to_string(),
            },
            "request": {
                "spec_id": resolved_spec_id,
                "title": title,
                "summary": summary,
            },
            "pipeline": pipeline,
            "registry": {},
            "artifacts": {
                "request_context": request_context,
                "solution_outline": solution_outline,
                "journey_index": journey_index,
                "component_index": component_index,
                "implementation_spec": implementation_spec,
                "implementation_report": implementation_report,
            }
        }),
    )?;

    let mut pipeline = template("pipeline-state.yaml")?;
    let mut handoff = template("handoff.yaml")?;
    let mut request_context = template("request-context.yaml")?;
    set_path(&mut pipeline, "spec.id", json!(resolved_spec_id));
    set_path(
        &mut pipeline,
        "spec.workspace_path",
        json!(format!("{}/{}", SPECS_DIRNAME, resolved_spec_id)),
    );
    set_path(
        &mut pipeline,
        "workspace.target_path",
        json!(
            collection
                .parent()
                .unwrap_or(&collection)
                .to_string_lossy()
                .to_string()
        ),
    );
    set_path(
        &mut pipeline,
        "workspace.spec_root",
        json!(format!("{}/{}", SPECS_DIRNAME, resolved_spec_id)),
    );
    set_path(&mut pipeline, "workspace.created_at", json!(created_at));
    set_path(&mut pipeline, "request.title", json!(title));
    set_path(&mut pipeline, "request.summary", json!(summary));
    set_path(
        &mut pipeline,
        "handoff.reason",
        json!("Initialize the spec workspace and begin intake."),
    );
    set_path(
        &mut handoff,
        "reason",
        json!("Initialize the spec workspace and begin intake."),
    );
    set_path(&mut handoff, "required_inputs", json!([]));
    set_path(
        &mut request_context,
        "problem.statement",
        json!(if summary.is_empty() {
            title.clone()
        } else {
            summary.clone()
        }),
    );
    set_path(&mut request_context, "problem.goal", json!(""));
    apply_ux_resolution(&mut pipeline, &mut handoff, &resolution, "default")?;
    finalize_workspace_handoff(&mut pipeline, &mut handoff, "intake", false);

    write_pipeline_and_handoff(&workspace, &mut pipeline, &handoff)?;
    write_json_yaml(
        workspace.join("framing/request-context.yaml"),
        &request_context,
    )?;
    write_json_yaml(workspace.join("framing/role-map.yaml"), &role_map)?;
    write_json_yaml(
        workspace.join("architecture/solution-outline.yaml"),
        &solution_outline,
    )?;
    write_json_yaml(
        workspace.join("architecture/journey-index.yaml"),
        &journey_index,
    )?;
    write_json_yaml(
        workspace.join("architecture/component-index.yaml"),
        &component_index,
    )?;
    write_json_yaml(workspace.join("journeys/batches.yaml"), &journey_batches)?;
    write_json_yaml(
        workspace.join("components/batches.yaml"),
        &component_batches,
    )?;
    write_json_yaml(
        workspace.join("synthesis/implementation-spec.yaml"),
        &implementation_spec,
    )?;
    write_json_yaml(
        workspace.join("synthesis/implementation-report.yaml"),
        &implementation_report,
    )?;

    for stage in STAGES {
        let mut report = blank_gate_report(stage);
        set_path(&mut report, "checked_at", json!(created_at));
        write_json_yaml(workspace.join(format!("gates/{}.yaml", stage)), &report)?;
    }

    sync_registry_from_pipeline(&collection, &resolved_spec_id, &pipeline)?;
    clear_invocation_state(&collection)?;

    Ok(CommandOutcome {
        payload: json!({
            "collection": collection.to_string_lossy().to_string(),
            "workspace": workspace.to_string_lossy().to_string(),
            "spec_id": resolved_spec_id,
            "created_at": created_at,
            "request_title": title,
            "next_stage": "intake",
        }),
        exit_code: 0,
    })
}

pub fn resolve_command(
    target: &str,
    spec_id: Option<&str>,
    skill: Option<&str>,
    stage: Option<&str>,
    mode: Option<&str>,
    runtime_mode: &str,
    write: bool,
) -> Result<CommandOutcome, CliError> {
    let runtime = build_runtime_context(target, spec_id)?;
    let explicit_stage = stage.and_then(non_empty_string);
    let explicit_skill = skill.and_then(non_empty_string);
    let use_completion_resolution = explicit_stage.is_none()
        && explicit_skill.is_none()
        && string_at(&runtime.context, "pipeline.phase.status").as_deref() == Some("complete");
    let resolved_stage = if use_completion_resolution {
        "complete".to_string()
    } else {
        determine_stage(stage, &runtime.context)
    };
    let resolved_skill = if use_completion_resolution {
        stage_to_skill("implement").to_string()
    } else {
        explicit_skill
            .clone()
            .unwrap_or_else(|| stage_to_skill(&resolved_stage).to_string())
    };
    let invocation = nested_get(&runtime.context, "invocation")
        .cloned()
        .unwrap_or_else(empty_object);
    let stored_mode = if string_at(&invocation, "skill").as_deref() == Some(&resolved_skill)
        || string_at(&invocation, "stage").as_deref() == Some(&resolved_stage)
    {
        string_at(&invocation, "mode")
    } else {
        None
    };
    let mode_override = mode.and_then(non_empty_string).or(stored_mode);

    let context_pipeline = nested_get(&runtime.context, "pipeline")
        .cloned()
        .unwrap_or_else(empty_object);
    let context_handoff = nested_get(&runtime.context, "handoff")
        .cloned()
        .unwrap_or_else(empty_object);
    let resolution = if use_completion_resolution {
        completion_resolution(
            &string_at(&runtime.context, "workspace.target_dir").unwrap_or_default(),
            &string_at(&runtime.context, "workspace.spec_id").unwrap_or_default(),
            &context_pipeline,
            &context_handoff,
        )
    } else {
        resolve_ux_contract(
            Some(&resolved_skill),
            Some(&resolved_stage),
            mode_override.as_deref(),
            &runtime.context,
        )?
    };
    let interactive_requirements = build_interactive_requirements(&resolution, runtime_mode)?;

    let payload = json!({
        "target_dir": string_at(&runtime.context, "workspace.target_dir").unwrap_or_default(),
        "collection": runtime.collection.to_string_lossy().to_string(),
        "workspace": runtime.workspace.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
        "workspace_exists": runtime.workspace.as_ref().is_some_and(|path| path.exists()),
        "spec_id": string_at(&runtime.context, "workspace.spec_id").unwrap_or_default(),
        "resolved_skill": resolution["skill"],
        "resolved_stage": resolution["stage"],
        "runtime_mode": runtime_mode,
        "ux": resolution,
        "interactive_requirements": interactive_requirements,
        "next_interaction": interactive_requirements["next_interaction"],
        "next_prompt": interactive_requirements["next_prompt"],
        "next_choice_dialog": interactive_requirements["next_choice_dialog"],
    });

    if write {
        if let Some(workspace) = runtime.workspace.as_ref().filter(|path| path.exists()) {
            let mut pipeline = read_json_yaml(&workspace.join("pipeline-state.yaml"))?;
            let mut handoff = read_json_yaml(&workspace.join("handoff.yaml"))?;
            apply_ux_resolution(&mut pipeline, &mut handoff, &resolution, runtime_mode)?;
            set_path(
                &mut pipeline,
                "ux.interactive_requirements",
                interactive_requirements.clone(),
            );
            set_path(
                &mut handoff,
                "ux.interactive_requirements",
                interactive_requirements.clone(),
            );
            finalize_workspace_handoff(
                &mut pipeline,
                &mut handoff,
                if use_completion_resolution {
                    ROUTER_STAGE
                } else {
                    &resolved_stage
                },
                false,
            );
            write_pipeline_and_handoff(workspace, &mut pipeline, &handoff)?;
        } else {
            ensure_collection_dirs(&runtime.collection)?;
            let mut invocation_state = load_invocation_state(&runtime.collection)?;
            set_path(&mut invocation_state, "skill", resolution["skill"].clone());
            set_path(&mut invocation_state, "stage", resolution["stage"].clone());
            set_path(
                &mut invocation_state,
                "mode",
                resolution["mode"]["id"].clone(),
            );
            let next_type = string_at(&interactive_requirements, "next_interaction.type")
                .unwrap_or_else(|| "ready".to_string());
            set_path(
                &mut invocation_state,
                "status",
                json!(if next_type == "ready" {
                    "ready"
                } else {
                    "collecting"
                }),
            );
            set_path(
                &mut invocation_state,
                "resolved_parameters",
                resolution["resolved_parameters"].clone(),
            );
            set_path(
                &mut invocation_state,
                "missing_parameters",
                resolution["missing_parameters"].clone(),
            );
            set_path(
                &mut invocation_state,
                "available_choices",
                interactive_requirements["available_choices"].clone(),
            );
            set_path(
                &mut invocation_state,
                "next_interaction",
                interactive_requirements["next_interaction"].clone(),
            );
            set_path(
                &mut invocation_state,
                "next_prompt",
                interactive_requirements["next_prompt"].clone(),
            );
            set_path(
                &mut invocation_state,
                "next_choice_dialog",
                interactive_requirements["next_choice_dialog"].clone(),
            );
            set_path(&mut invocation_state, "updated_at", json!(now_utc()));
            save_invocation_state(&runtime.collection, &invocation_state)?;
        }
    }

    Ok(CommandOutcome {
        payload,
        exit_code: 0,
    })
}

pub fn apply_command(
    target: &str,
    spec_id: Option<&str>,
    skill: Option<&str>,
    stage: Option<&str>,
    mode: Option<&str>,
    runtime_mode: &str,
    parameter: Option<&str>,
    choice: Option<&str>,
    values: &[String],
) -> Result<CommandOutcome, CliError> {
    let has_parameter = parameter.and_then(non_empty_string).is_some();
    let has_choice = choice.and_then(non_empty_string).is_some();
    if has_parameter == has_choice {
        return Err(CliError::new(
            "invalid_interaction_target",
            "Provide exactly one of --parameter or --choice.",
        ));
    }

    if values.is_empty() {
        return Err(CliError::new(
            "missing_value",
            "At least one --value must be provided.",
        ));
    }
    let raw_answer = parse_yaml_values(values)?;

    let runtime = build_runtime_context(target, spec_id)?;
    let resolved_stage = determine_stage(stage, &runtime.context);
    let resolved_skill = skill
        .and_then(non_empty_string)
        .unwrap_or_else(|| stage_to_skill(&resolved_stage).to_string());
    let mode_override = mode
        .and_then(non_empty_string)
        .or_else(|| stored_mode_for(&runtime.context, &resolved_skill, &resolved_stage));
    let resolution = resolve_ux_contract(
        Some(&resolved_skill),
        Some(&resolved_stage),
        mode_override.as_deref(),
        &runtime.context,
    )?;

    let mut registry = nested_get(&runtime.context, "registry")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut invocation_state = nested_get(&runtime.context, "invocation")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut pipeline = nested_get(&runtime.context, "pipeline")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut request_context = nested_get(&runtime.context, "artifacts.request_context")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut role_map = nested_get(&runtime.context, "artifacts.role_map")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut solution_outline = nested_get(&runtime.context, "artifacts.solution_outline")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut journey_index = nested_get(&runtime.context, "artifacts.journey_index")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut component_index = nested_get(&runtime.context, "artifacts.component_index")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut journey_batches = nested_get(&runtime.context, "artifacts.journey_batches")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut component_batches = nested_get(&runtime.context, "artifacts.component_batches")
        .cloned()
        .unwrap_or_else(empty_object);
    let mut implementation_spec = nested_get(&runtime.context, "artifacts.implementation_spec")
        .cloned()
        .unwrap_or_else(empty_object);

    set_path(&mut invocation_state, "skill", json!(resolved_skill));
    set_path(&mut invocation_state, "stage", json!(resolved_stage));
    if let Some(mode_override) = mode_override.as_deref() {
        set_path(&mut invocation_state, "mode", json!(mode_override));
    }
    set_path(&mut invocation_state, "status", json!("collecting"));
    set_path(&mut invocation_state, "updated_at", json!(now_utc()));

    let mut updated_files = Vec::new();
    let applied = if let Some(parameter_id) = parameter.and_then(non_empty_string) {
        let param = find_parameter(&resolution, &parameter_id)?;
        updated_files.extend(apply_parameter_answer(
            &resolved_stage,
            param["id"].as_str().unwrap_or_default(),
            &raw_answer,
            runtime.workspace.as_ref().is_some_and(|path| path.exists()),
            &mut invocation_state,
            &mut pipeline,
            &mut request_context,
            &mut solution_outline,
            &mut journey_index,
            &mut component_index,
            &mut journey_batches,
            &mut component_batches,
            &mut implementation_spec,
        )?);
        json!({
            "kind": "parameter",
            "id": param["id"],
            "value": raw_answer,
        })
    } else {
        let choice_id = choice.and_then(non_empty_string).unwrap_or_default();
        let choice_doc = find_choice(&resolution, &choice_id)?;
        let normalized = normalize_choice_value(&choice_doc, &raw_answer)?;
        updated_files.extend(apply_choice_answer(
            &resolved_stage,
            choice_doc["id"].as_str().unwrap_or_default(),
            choice_doc
                .get("answers_parameter")
                .and_then(JsonValue::as_str),
            &normalized,
            &mut invocation_state,
            &mut registry,
            &mut pipeline,
            &mut request_context,
            &mut role_map,
            &mut implementation_spec,
        )?);
        json!({
            "kind": "choice",
            "id": choice_doc["id"],
            "value": normalized,
        })
    };

    ensure_collection_dirs(&runtime.collection)?;
    save_registry(&runtime.collection, &registry)?;
    save_invocation_state(&runtime.collection, &invocation_state)?;
    updated_files.push("registry.yaml".to_string());
    updated_files.push("invocation-state.yaml".to_string());

    if let Some(workspace) = runtime.workspace.as_ref().filter(|path| path.exists()) {
        write_workspace_artifacts(
            workspace,
            &pipeline,
            &request_context,
            &role_map,
            &solution_outline,
            &journey_index,
            &component_index,
            &journey_batches,
            &component_batches,
            &implementation_spec,
        )?;
    }

    let runtime_after = build_runtime_context(target, spec_id)?;
    let stage_after = determine_stage(stage, &runtime_after.context);
    let skill_after = skill
        .and_then(non_empty_string)
        .unwrap_or_else(|| stage_to_skill(&stage_after).to_string());
    let mode_after = mode
        .and_then(non_empty_string)
        .or_else(|| stored_mode_for(&runtime_after.context, &skill_after, &stage_after));
    let resolution_after = resolve_ux_contract(
        Some(&skill_after),
        Some(&stage_after),
        mode_after.as_deref(),
        &runtime_after.context,
    )?;
    let interactive_requirements = build_interactive_requirements(&resolution_after, runtime_mode)?;

    if let Some(workspace_after) = runtime_after
        .workspace
        .as_ref()
        .filter(|path| path.exists() && stage_after != ROUTER_STAGE)
    {
        let mut pipeline_after = read_json_yaml(&workspace_after.join("pipeline-state.yaml"))?;
        let mut handoff_after = read_json_yaml(&workspace_after.join("handoff.yaml"))?;
        apply_ux_resolution(
            &mut pipeline_after,
            &mut handoff_after,
            &resolution_after,
            runtime_mode,
        )?;
        set_path(&mut handoff_after, "from_stage", json!(stage_after));
        set_path(&mut handoff_after, "to_stage", json!(stage_after));
        set_path(
            &mut handoff_after,
            "reason",
            json!(format!(
                "Continue {} by resolving the next interaction.",
                stage_after
            )),
        );
        finalize_workspace_handoff(&mut pipeline_after, &mut handoff_after, &stage_after, false);
        write_pipeline_and_handoff(workspace_after, &mut pipeline_after, &handoff_after)?;
        updated_files.push("pipeline-state.yaml".to_string());
        updated_files.push("handoff.yaml".to_string());
    } else {
        let mut invocation_after = load_invocation_state(&runtime_after.collection)?;
        let next_type = string_at(&interactive_requirements, "next_interaction.type")
            .unwrap_or_else(|| "ready".to_string());
        set_path(
            &mut invocation_after,
            "skill",
            resolution_after["skill"].clone(),
        );
        set_path(
            &mut invocation_after,
            "stage",
            resolution_after["stage"].clone(),
        );
        set_path(
            &mut invocation_after,
            "mode",
            resolution_after["mode"]["id"].clone(),
        );
        set_path(
            &mut invocation_after,
            "status",
            json!(if next_type == "ready" {
                "ready"
            } else {
                "collecting"
            }),
        );
        set_path(
            &mut invocation_after,
            "resolved_parameters",
            resolution_after["resolved_parameters"].clone(),
        );
        set_path(
            &mut invocation_after,
            "missing_parameters",
            resolution_after["missing_parameters"].clone(),
        );
        set_path(
            &mut invocation_after,
            "available_choices",
            interactive_requirements["available_choices"].clone(),
        );
        set_path(
            &mut invocation_after,
            "next_interaction",
            interactive_requirements["next_interaction"].clone(),
        );
        set_path(
            &mut invocation_after,
            "next_prompt",
            interactive_requirements["next_prompt"].clone(),
        );
        set_path(
            &mut invocation_after,
            "next_choice_dialog",
            interactive_requirements["next_choice_dialog"].clone(),
        );
        set_path(&mut invocation_after, "updated_at", json!(now_utc()));
        save_invocation_state(&runtime_after.collection, &invocation_after)?;
    }

    let ready_for_workspace_init = resolution_after["stage"] == json!(ROUTER_STAGE)
        && resolution_after["mode"]["id"] == json!("initialize_new_spec")
        && !bool_at(&runtime_after.context, "workspace.spec_exists")
        && interactive_requirements["next_interaction"]["type"] == json!("ready");

    updated_files.sort();
    updated_files.dedup();

    Ok(CommandOutcome {
        payload: json!({
            "applied": applied,
            "collection": runtime_after.collection.to_string_lossy().to_string(),
            "workspace": runtime_after.workspace.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            "workspace_exists": runtime_after.workspace.as_ref().is_some_and(|path| path.exists()),
            "spec_id": string_at(&runtime_after.context, "workspace.spec_id").unwrap_or_default(),
            "resolved_skill": resolution_after["skill"],
            "resolved_stage": resolution_after["stage"],
            "runtime_mode": runtime_mode,
            "updated_files": updated_files,
            "interactive_requirements": interactive_requirements,
            "next_interaction": interactive_requirements["next_interaction"],
            "next_prompt": interactive_requirements["next_prompt"],
            "next_choice_dialog": interactive_requirements["next_choice_dialog"],
            "ready_for_workspace_init": ready_for_workspace_init,
            "recommended_script": if ready_for_workspace_init {
                "spec-forge-cli init"
            } else {
                "spec-forge-cli resolve"
            },
            "recommended_command": if ready_for_workspace_init {
                "spec-forge-cli init"
            } else {
                "spec-forge-cli resolve"
            }
        }),
        exit_code: 0,
    })
}

pub fn focus_command(
    target: &str,
    spec_id: Option<&str>,
    stage: &str,
    max_items: i64,
    write: bool,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace) = resolve_workspace(target, spec_id, true)?;
    let (index, batches) = if stage == "journeys" {
        (
            read_json_yaml(&workspace.join("architecture/journey-index.yaml"))?,
            read_json_yaml(&workspace.join("journeys/batches.yaml"))?,
        )
    } else {
        (
            read_json_yaml(&workspace.join("architecture/component-index.yaml"))?,
            read_json_yaml(&workspace.join("components/batches.yaml"))?,
        )
    };

    let (selected_ids, reason) = select_batch(
        &batches,
        array_at(&index, "items"),
        max_items.max(1) as usize,
    );
    let payload = json!({
        "spec_id": resolved_spec_id,
        "stage": stage,
        "selected_ids": selected_ids,
        "reason": reason,
        "max_items": max_items.max(1),
    });

    if write {
        let mut pipeline = read_json_yaml(&workspace.join("pipeline-state.yaml"))?;
        let mut handoff = read_json_yaml(&workspace.join("handoff.yaml"))?;
        set_path(&mut pipeline, "focus.kind", json!(stage));
        set_path(&mut pipeline, "focus.ids", json!(selected_ids));
        set_path(&mut pipeline, "focus.note", json!(reason));
        set_path(&mut handoff, "from_stage", json!("router"));
        set_path(&mut handoff, "to_stage", json!(stage));
        set_path(&mut handoff, "reason", json!(reason));
        set_path(&mut handoff, "blocking_items", json!([]));
        set_path(&mut handoff, "required_inputs", json!(selected_ids));
        set_handoff_recommended_command(&mut handoff, "spec-forge-cli focus");
        let resolution = resolve_ux_contract(
            Some(stage_to_skill(stage)),
            Some(stage),
            None,
            &json!({
                "workspace": {
                    "target_dir": collection.parent().unwrap_or(&collection).to_string_lossy().to_string(),
                    "collection_root": collection.to_string_lossy().to_string(),
                    "collection_exists": true,
                    "spec_id": resolved_spec_id,
                    "spec_id_source": "workspace_context",
                    "spec_exists": true,
                    "spec_root": workspace.to_string_lossy().to_string(),
                },
                "registry": read_json_yaml(&collection.join(REGISTRY_FILENAME))?,
                "pipeline": pipeline,
                "handoff": handoff,
                "request": {
                    "spec_id": resolved_spec_id,
                    "title": string_at(&pipeline, "request.title").unwrap_or_default(),
                    "summary": string_at(&pipeline, "request.summary").unwrap_or_default(),
                },
                "artifacts": {
                    "request_context": read_json_yaml(&workspace.join("framing/request-context.yaml"))?,
                    "solution_outline": read_json_yaml(&workspace.join("architecture/solution-outline.yaml"))?,
                    "journey_index": read_json_yaml(&workspace.join("architecture/journey-index.yaml"))?,
                    "component_index": read_json_yaml(&workspace.join("architecture/component-index.yaml"))?,
                    "implementation_spec": read_json_yaml(&workspace.join("synthesis/implementation-spec.yaml"))?,
                }
            }),
        )?;
        apply_ux_resolution(&mut pipeline, &mut handoff, &resolution, "default")?;
        finalize_workspace_handoff(&mut pipeline, &mut handoff, stage, false);
        write_pipeline_and_handoff(&workspace, &mut pipeline, &handoff)?;
        sync_registry_from_pipeline(&collection, &resolved_spec_id, &pipeline)?;
    }

    Ok(CommandOutcome {
        payload,
        exit_code: if selected_ids.is_empty() { 1 } else { 0 },
    })
}

pub fn gate_check_command(
    target: &str,
    spec_id: Option<&str>,
    stage: &str,
    write: bool,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace) = resolve_workspace(target, spec_id, true)?;
    let mut report = evaluate_stage(&workspace, stage)?;
    set_path(&mut report, "spec_id", json!(resolved_spec_id));
    set_path(
        &mut report,
        "collection",
        json!(collection.to_string_lossy().to_string()),
    );
    set_path(
        &mut report,
        "workspace",
        json!(workspace.to_string_lossy().to_string()),
    );
    if write {
        write_json_yaml(workspace.join(format!("gates/{}.yaml", stage)), &report)?;
    }
    Ok(CommandOutcome {
        exit_code: if bool_at(&report, "passed") { 0 } else { 1 },
        payload: report,
    })
}

pub fn stage_advance_command(
    target: &str,
    spec_id: Option<&str>,
    stage: &str,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace) = resolve_workspace(target, spec_id, true)?;
    let report = evaluate_stage(&workspace, stage)?;
    write_json_yaml(workspace.join(format!("gates/{}.yaml", stage)), &report)?;

    let mut pipeline = read_json_yaml(&workspace.join("pipeline-state.yaml"))?;
    let current_stage = string_at(&pipeline, "phase.current").unwrap_or_default();
    if current_stage != stage {
        return Ok(CommandOutcome {
            payload: json!({
                "spec_id": resolved_spec_id,
                "advanced_from": stage,
                "next_stage": current_stage,
                "workspace": workspace.to_string_lossy().to_string(),
                "error": format!(
                    "Cannot advance {} while pipeline current stage is {}.",
                    stage, current_stage
                ),
            }),
            exit_code: 1,
        });
    }
    if !bool_at(&report, "passed") {
        return Ok(CommandOutcome {
            payload: report,
            exit_code: 1,
        });
    }

    let mut handoff = read_json_yaml(&workspace.join("handoff.yaml"))?;
    set_path(
        &mut pipeline,
        &format!("stages.{}.status", stage),
        json!("approved"),
    );
    set_path(
        &mut pipeline,
        &format!("stages.{}.approved", stage),
        json!(true),
    );

    let next_stage = next_stage(stage);
    if let Some(next_stage) = next_stage {
        set_path(
            &mut pipeline,
            &format!("stages.{}.status", next_stage),
            json!("ready"),
        );
        set_path(&mut pipeline, "phase.current", json!(next_stage));
        set_path(&mut pipeline, "phase.next_recommended", json!(next_stage));
        set_path(&mut pipeline, "phase.status", json!("active"));
        set_path(&mut pipeline, "focus.kind", json!("none"));
        set_path(&mut pipeline, "focus.ids", json!([]));
        set_path(&mut pipeline, "focus.note", json!(""));
        set_path(&mut handoff, "from_stage", json!(stage));
        set_path(&mut handoff, "to_stage", json!(next_stage));
        set_path(
            &mut handoff,
            "reason",
            json!(format!(
                "Stage {} passed. Proceed to {}.",
                stage, next_stage
            )),
        );
        set_path(&mut handoff, "blocking_items", json!([]));
        set_path(&mut handoff, "required_inputs", json!([]));
        set_handoff_recommended_command(&mut handoff, "spec-forge-cli stage advance");
    } else {
        set_path(&mut pipeline, "phase.current", json!("implement"));
        set_path(&mut pipeline, "phase.next_recommended", json!(""));
        set_path(&mut pipeline, "phase.status", json!("complete"));
        set_path(&mut handoff, "from_stage", json!(stage));
        set_path(&mut handoff, "to_stage", json!("complete"));
        set_path(
            &mut handoff,
            "reason",
            json!("All stages passed. Implementation is complete."),
        );
        set_path(&mut handoff, "blocking_items", json!([]));
        set_path(&mut handoff, "required_inputs", json!([]));
        set_handoff_recommended_command(&mut handoff, "spec-forge-cli stage advance");
    }

    if let Some(next_stage_name) = next_stage {
        let resolution = resolve_ux_contract(
            Some(stage_to_skill(next_stage_name)),
            Some(next_stage_name),
            None,
            &json!({
                "workspace": {
                    "target_dir": collection.parent().unwrap_or(&collection).to_string_lossy().to_string(),
                    "collection_root": collection.to_string_lossy().to_string(),
                    "collection_exists": true,
                    "spec_id": resolved_spec_id,
                    "spec_id_source": "workspace_context",
                    "spec_exists": true,
                    "spec_root": workspace.to_string_lossy().to_string(),
                },
                "registry": read_json_yaml(&collection.join(REGISTRY_FILENAME))?,
                "pipeline": pipeline,
                "handoff": handoff,
                "request": {
                    "spec_id": resolved_spec_id,
                    "title": string_at(&pipeline, "request.title").unwrap_or_default(),
                    "summary": string_at(&pipeline, "request.summary").unwrap_or_default(),
                },
                "artifacts": {
                    "request_context": read_json_yaml(&workspace.join("framing/request-context.yaml"))?,
                    "solution_outline": read_json_yaml(&workspace.join("architecture/solution-outline.yaml"))?,
                    "journey_index": read_json_yaml(&workspace.join("architecture/journey-index.yaml"))?,
                    "component_index": read_json_yaml(&workspace.join("architecture/component-index.yaml"))?,
                    "implementation_spec": read_json_yaml(&workspace.join("synthesis/implementation-spec.yaml"))?,
                    "implementation_report": read_json_yaml(&workspace.join("synthesis/implementation-report.yaml"))?,
                }
            }),
        )?;
        apply_ux_resolution(&mut pipeline, &mut handoff, &resolution, "default")?;
        finalize_workspace_handoff(&mut pipeline, &mut handoff, next_stage_name, false);
    } else {
        let completion_resolution = completion_resolution(
            &collection.parent().unwrap_or(&collection).to_string_lossy(),
            &resolved_spec_id,
            &pipeline,
            &handoff,
        );
        apply_ux_resolution(
            &mut pipeline,
            &mut handoff,
            &completion_resolution,
            "default",
        )?;
        finalize_workspace_handoff(&mut pipeline, &mut handoff, ROUTER_STAGE, false);
    }
    write_pipeline_and_handoff(&workspace, &mut pipeline, &handoff)?;
    sync_registry_from_pipeline(&collection, &resolved_spec_id, &pipeline)?;

    Ok(CommandOutcome {
        payload: json!({
            "spec_id": resolved_spec_id,
            "advanced_from": stage,
            "next_stage": next_stage.unwrap_or("complete"),
            "workspace": workspace.to_string_lossy().to_string(),
        }),
        exit_code: 0,
    })
}

pub fn validate_ux_command(target: &str) -> Result<CommandOutcome, CliError> {
    let repo_root = absolute_path(target)?;
    let shared_path = repo_root.join(UX_CONTRACT_PATH);
    let shared_contract = if shared_path.exists() {
        let data = read_json_yaml(&shared_path)?;
        validate_shared_ux_contract_value(&shared_path, &data, &mut Vec::new())?;
        Some(data)
    } else {
        None
    };
    let skill_dirs = [
        ("spec-forge", "router", repo_root.join("spec-forge")),
        (
            "spec-forge-intake",
            "intake",
            repo_root.join("spec-forge-intake"),
        ),
        (
            "spec-forge-architecture",
            "architecture",
            repo_root.join("spec-forge-architecture"),
        ),
        (
            "spec-forge-journeys",
            "journeys",
            repo_root.join("spec-forge-journeys"),
        ),
        (
            "spec-forge-components",
            "components",
            repo_root.join("spec-forge-components"),
        ),
        (
            "spec-forge-readiness",
            "readiness",
            repo_root.join("spec-forge-readiness"),
        ),
        (
            "spec-forge-implement",
            "implement",
            repo_root.join("spec-forge-implement"),
        ),
    ];

    let mut problems = Vec::new();
    let mut validated = Vec::new();

    for (skill_name, stage_name, skill_dir) in skill_dirs {
        let ux_path = skill_dir.join("agents/ux.yaml");
        if !ux_path.exists() {
            problems.push(format!("Missing UX contract: {}", ux_path.display()));
            continue;
        }
        validate_agent_ux(&ux_path, &mut problems)?;
        if let Some(shared) = shared_contract.as_ref() {
            validate_agent_ux_against_shared_contract(&ux_path, skill_name, shared, &mut problems)?;
        }
        validated.push(ux_path.to_string_lossy().to_string());

        let openai_path = skill_dir.join("agents/openai.yaml");
        if !openai_path.exists() {
            problems.push(format!(
                "Missing OpenAI agent config: {}",
                openai_path.display()
            ));
            continue;
        }
        if let Some(shared) = shared_contract.as_ref() {
            validate_openai_agent_config(
                &openai_path,
                skill_name,
                stage_name,
                shared,
                &mut problems,
            )?;
        }
        validated.push(openai_path.to_string_lossy().to_string());
    }

    if shared_path.exists() {
        validate_shared_ux_contract(&shared_path, &mut problems)?;
        validated.push(shared_path.to_string_lossy().to_string());
    } else {
        problems.push(format!(
            "Missing shared UX contract: {}",
            shared_path.display()
        ));
    }

    let payload = json!({
        "validated_files": validated,
        "problem_count": problems.len(),
        "problems": problems,
    });
    Ok(CommandOutcome {
        exit_code: if problems.is_empty() { 0 } else { 1 },
        payload,
    })
}

pub fn artifact_get_command(
    target: &str,
    spec_id: Option<&str>,
    file: &str,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace, relative_path, artifact_path) =
        resolve_workspace_artifact(target, spec_id, file)?;
    if !artifact_path.exists() {
        return Err(CliError::new(
            "artifact_missing",
            format!("Artifact does not exist: {}", artifact_path.display()),
        ));
    }
    let artifact = read_json_yaml(&artifact_path)?;
    Ok(CommandOutcome {
        exit_code: 0,
        payload: json!({
            "spec_id": resolved_spec_id,
            "collection": collection.to_string_lossy().to_string(),
            "workspace": workspace.to_string_lossy().to_string(),
            "file": relative_path.to_string_lossy().to_string(),
            "path": artifact_path.to_string_lossy().to_string(),
            "artifact": artifact,
        }),
    })
}

pub fn artifact_put_command(
    target: &str,
    spec_id: Option<&str>,
    file: &str,
    value: &str,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace, relative_path, artifact_path) =
        resolve_workspace_artifact(target, spec_id, file)?;
    let parsed = parse_yaml_values(&[value.to_string()])?;
    write_json_yaml(artifact_path.clone(), &parsed)?;
    maybe_sync_registry_after_artifact_write(
        &collection,
        &resolved_spec_id,
        &relative_path,
        &parsed,
    )?;
    Ok(CommandOutcome {
        exit_code: 0,
        payload: json!({
            "spec_id": resolved_spec_id,
            "collection": collection.to_string_lossy().to_string(),
            "workspace": workspace.to_string_lossy().to_string(),
            "file": relative_path.to_string_lossy().to_string(),
            "path": artifact_path.to_string_lossy().to_string(),
            "artifact": parsed,
            "operation": "put",
        }),
    })
}

pub fn artifact_merge_command(
    target: &str,
    spec_id: Option<&str>,
    file: &str,
    value: &str,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace, relative_path, artifact_path) =
        resolve_workspace_artifact(target, spec_id, file)?;
    let patch = parse_yaml_values(&[value.to_string()])?;
    let mut artifact = if artifact_path.exists() {
        read_json_yaml(&artifact_path)?
    } else {
        empty_object()
    };
    deep_merge_json(&mut artifact, patch);
    write_json_yaml(artifact_path.clone(), &artifact)?;
    maybe_sync_registry_after_artifact_write(
        &collection,
        &resolved_spec_id,
        &relative_path,
        &artifact,
    )?;
    Ok(CommandOutcome {
        exit_code: 0,
        payload: json!({
            "spec_id": resolved_spec_id,
            "collection": collection.to_string_lossy().to_string(),
            "workspace": workspace.to_string_lossy().to_string(),
            "file": relative_path.to_string_lossy().to_string(),
            "path": artifact_path.to_string_lossy().to_string(),
            "artifact": artifact,
            "operation": "merge",
        }),
    })
}

pub fn approve_command(
    target: &str,
    spec_id: Option<&str>,
    file: &str,
    note: Option<&str>,
) -> Result<CommandOutcome, CliError> {
    let (collection, resolved_spec_id, workspace, relative_path, artifact_path) =
        resolve_workspace_artifact(target, spec_id, file)?;
    let mut artifact = if artifact_path.exists() {
        read_json_yaml(&artifact_path)?
    } else {
        empty_object()
    };
    set_path(&mut artifact, "approval.status", json!("approved"));
    set_path(&mut artifact, "approval.confirmed_by_user", json!(true));
    set_path(
        &mut artifact,
        "approval.confirmation_note",
        json!(note.unwrap_or("Approved via spec-forge-cli approve.")),
    );
    write_json_yaml(artifact_path.clone(), &artifact)?;
    maybe_sync_registry_after_artifact_write(
        &collection,
        &resolved_spec_id,
        &relative_path,
        &artifact,
    )?;
    Ok(CommandOutcome {
        exit_code: 0,
        payload: json!({
            "spec_id": resolved_spec_id,
            "collection": collection.to_string_lossy().to_string(),
            "workspace": workspace.to_string_lossy().to_string(),
            "file": relative_path.to_string_lossy().to_string(),
            "path": artifact_path.to_string_lossy().to_string(),
            "artifact": artifact,
            "operation": "approve",
        }),
    })
}

fn validate_agent_ux(path: &Path, problems: &mut Vec<String>) -> Result<(), CliError> {
    let data = read_json_yaml(path)?;
    for key in ["execution", "invocation", "parameters"] {
        if data.get(key).is_none() {
            problems.push(format!("{}: missing top-level key {}", path.display(), key));
        }
    }
    let Some(parameters) = data.get("parameters").and_then(JsonValue::as_array) else {
        problems.push(format!(
            "{}: parameters must be a non-empty list",
            path.display()
        ));
        return Ok(());
    };
    if parameters.is_empty() {
        problems.push(format!(
            "{}: parameters must be a non-empty list",
            path.display()
        ));
        return Ok(());
    }
    for param in parameters {
        for key in ["id", "label", "description", "type", "collect_via"] {
            if param.get(key).is_none() {
                problems.push(format!("{}: parameter missing key {}", path.display(), key));
            }
        }
    }
    Ok(())
}

fn validate_agent_ux_against_shared_contract(
    path: &Path,
    skill_name: &str,
    shared_contract: &JsonValue,
    problems: &mut Vec<String>,
) -> Result<(), CliError> {
    let data = read_json_yaml(path)?;
    let prefix = format!("{}:{}", path.display(), skill_name);
    if data
        .get("invocation")
        .and_then(|value| value.get("skill_name"))
        .and_then(JsonValue::as_str)
        != Some(skill_name)
    {
        problems.push(format!(
            "{}: invocation.skill_name must equal {:?}",
            prefix, skill_name
        ));
    }

    let Some(shared_skill) = shared_contract
        .get("skills")
        .and_then(JsonValue::as_object)
        .and_then(|skills| skills.get(skill_name))
    else {
        problems.push(format!(
            "{}: shared UX contract is missing skill entry {:?}",
            prefix, skill_name
        ));
        return Ok(());
    };

    let mut allowed_ids = std::collections::BTreeSet::new();
    for param in array_at(shared_skill, "parameters") {
        if let Some(id) = param.get("id").and_then(JsonValue::as_str) {
            allowed_ids.insert(id.to_string());
        }
    }
    for choice in array_at(shared_skill, "choices") {
        if let Some(id) = choice.get("id").and_then(JsonValue::as_str) {
            allowed_ids.insert(id.to_string());
        }
    }

    for param in array_at(&data, "parameters") {
        let Some(param_id) = param.get("id").and_then(JsonValue::as_str) else {
            continue;
        };
        if !allowed_ids.contains(param_id) {
            problems.push(format!(
                "{}: parameter {:?} is not declared in the shared UX contract for this skill",
                prefix, param_id
            ));
        }
    }

    Ok(())
}

fn validate_shared_ux_contract(path: &Path, problems: &mut Vec<String>) -> Result<(), CliError> {
    let data = read_json_yaml(path)?;
    validate_shared_ux_contract_value(path, &data, problems)
}

fn validate_shared_ux_contract_value(
    path: &Path,
    data: &JsonValue,
    problems: &mut Vec<String>,
) -> Result<(), CliError> {
    for key in ["family", "version", "defaults", "skills"] {
        if data.get(key).is_none() {
            problems.push(format!("{}: missing top-level key {}", path.display(), key));
        }
    }
    let Some(skills) = data.get("skills").and_then(JsonValue::as_object) else {
        problems.push(format!(
            "{}: skills must be a non-empty mapping",
            path.display()
        ));
        return Ok(());
    };
    if skills.is_empty() {
        problems.push(format!(
            "{}: skills must be a non-empty mapping",
            path.display()
        ));
        return Ok(());
    }
    for (skill_name, contract) in skills {
        let prefix = format!("{}:{}", path.display(), skill_name);
        for key in ["skill", "stage", "collection", "modes", "parameters"] {
            if contract.get(key).is_none() {
                problems.push(format!("{}: missing key {}", prefix, key));
            }
        }
        let Some(parameters) = contract.get("parameters").and_then(JsonValue::as_array) else {
            problems.push(format!("{}: parameters must be a non-empty list", prefix));
            continue;
        };
        if parameters.is_empty() {
            problems.push(format!("{}: parameters must be a non-empty list", prefix));
        } else {
            for param in parameters {
                for key in ["id", "label", "description", "type", "prompt"] {
                    if param.get(key).is_none() {
                        problems.push(format!("{}: parameter missing key {}", prefix, key));
                    }
                }
            }
        }
        if let Some(choices) = contract.get("choices") {
            if let Some(choice_array) = choices.as_array() {
                for choice in choice_array {
                    for key in ["id", "type", "prompt"] {
                        if choice.get(key).is_none() {
                            problems.push(format!("{}: choice missing key {}", prefix, key));
                        }
                    }
                    if choice.get("options").is_none() && choice.get("options_from").is_none() {
                        problems.push(format!(
                            "{}: choice {} must define options or options_from",
                            prefix,
                            choice
                                .get("id")
                                .and_then(JsonValue::as_str)
                                .unwrap_or("<unknown>")
                        ));
                    }
                }
            } else {
                problems.push(format!("{}: choices must be a list when present", prefix));
            }
        }
    }
    Ok(())
}

fn validate_openai_agent_config(
    path: &Path,
    skill_name: &str,
    stage_name: &str,
    shared_contract: &JsonValue,
    problems: &mut Vec<String>,
) -> Result<(), CliError> {
    let data = read_json_yaml(path)?;
    let prefix = format!("{}:{}", path.display(), skill_name);

    for key in ["display_name", "short_description", "default_prompt"] {
        if data
            .get("interface")
            .and_then(|value| value.get(key))
            .is_none()
        {
            problems.push(format!("{}: interface missing key {}", prefix, key));
        }
    }

    let Some(extension) = data
        .get("extensions")
        .and_then(|value| value.get("ux_contract"))
    else {
        problems.push(format!("{}: missing extensions.ux_contract", prefix));
        return Ok(());
    };

    for key in [
        "family",
        "skill",
        "stage",
        "contract_path",
        "contract_id",
        "collection_mode",
        "interactive_collection",
        "choice_presentation",
    ] {
        if extension.get(key).is_none() {
            problems.push(format!("{}: ux_contract missing key {}", prefix, key));
        }
    }

    if extension.get("family").and_then(JsonValue::as_str) != Some("spec-forge") {
        problems.push(format!("{}: ux_contract.family must be spec-forge", prefix));
    }
    if extension.get("skill").and_then(JsonValue::as_str) != Some(skill_name) {
        problems.push(format!(
            "{}: ux_contract.skill must equal {:?}",
            prefix, skill_name
        ));
    }
    if extension.get("stage").and_then(JsonValue::as_str) != Some(stage_name) {
        problems.push(format!(
            "{}: ux_contract.stage must equal {:?}",
            prefix, stage_name
        ));
    }
    if extension.get("contract_id").and_then(JsonValue::as_str) != Some(skill_name) {
        problems.push(format!(
            "{}: ux_contract.contract_id must equal {:?}",
            prefix, skill_name
        ));
    }

    if let Some(contract_path) = extension.get("contract_path").and_then(JsonValue::as_str) {
        let resolved = path.parent().unwrap_or(path).join(contract_path);
        if !resolved.exists() {
            problems.push(format!(
                "{}: ux_contract.contract_path does not resolve to an existing file ({})",
                prefix,
                resolved.display()
            ));
        }
    }

    let Some(shared_skill) = shared_contract
        .get("skills")
        .and_then(JsonValue::as_object)
        .and_then(|skills| skills.get(skill_name))
    else {
        problems.push(format!(
            "{}: shared UX contract is missing skill entry {:?}",
            prefix, skill_name
        ));
        return Ok(());
    };

    if extension.get("collection_mode").and_then(JsonValue::as_str)
        != shared_skill
            .get("collection")
            .and_then(|value| value.get("mode"))
            .and_then(JsonValue::as_str)
    {
        problems.push(format!(
            "{}: ux_contract.collection_mode must match shared contract collection.mode",
            prefix
        ));
    }
    if extension
        .get("interactive_collection")
        .and_then(JsonValue::as_str)
        != shared_skill
            .get("collection")
            .and_then(|value| value.get("interactive_collection"))
            .and_then(JsonValue::as_str)
    {
        problems.push(format!(
            "{}: ux_contract.interactive_collection must match shared contract collection.interactive_collection",
            prefix
        ));
    }
    if extension
        .get("choice_presentation")
        .and_then(JsonValue::as_str)
        != shared_skill
            .get("collection")
            .and_then(|value| value.get("choice_presentation"))
            .and_then(JsonValue::as_str)
    {
        problems.push(format!(
            "{}: ux_contract.choice_presentation must match shared contract collection.choice_presentation",
            prefix
        ));
    }

    Ok(())
}

pub fn next_stage(stage: &str) -> Option<&'static str> {
    match stage {
        "intake" => Some("architecture"),
        "architecture" => Some("journeys"),
        "journeys" => Some("components"),
        "components" => Some("readiness"),
        "readiness" => Some("implement"),
        "implement" => None,
        _ => None,
    }
}

fn resolve_workspace_artifact(
    target: &str,
    spec_id: Option<&str>,
    file: &str,
) -> Result<(PathBuf, String, PathBuf, PathBuf, PathBuf), CliError> {
    let (collection, resolved_spec_id, workspace) = resolve_workspace(target, spec_id, true)?;
    let relative_path = normalize_relative_artifact_path(file)?;
    let artifact_path = workspace.join(&relative_path);
    Ok((
        collection,
        resolved_spec_id,
        workspace,
        relative_path,
        artifact_path,
    ))
}

fn normalize_relative_artifact_path(file: &str) -> Result<PathBuf, CliError> {
    let trimmed = file.trim();
    if trimmed.is_empty() {
        return Err(CliError::new(
            "artifact_path_invalid",
            "Artifact path cannot be empty.",
        ));
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return Err(CliError::new(
            "artifact_path_invalid",
            "Artifact path must be relative to the spec workspace.",
        ));
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(CliError::new(
                "artifact_path_invalid",
                format!(
                    "Artifact path {:?} must stay within the spec workspace and cannot contain traversal components.",
                    file
                ),
            ));
        }
    }
    Ok(path)
}

fn deep_merge_json(base: &mut JsonValue, patch: JsonValue) {
    match (base, patch) {
        (JsonValue::Object(base_map), JsonValue::Object(patch_map)) => {
            for (key, value) in patch_map {
                if let Some(existing) = base_map.get_mut(&key) {
                    deep_merge_json(existing, value);
                } else {
                    base_map.insert(key, value);
                }
            }
        }
        (base_slot, patch_value) => {
            *base_slot = patch_value;
        }
    }
}

fn maybe_sync_registry_after_artifact_write(
    collection: &Path,
    spec_id: &str,
    relative_path: &Path,
    artifact: &JsonValue,
) -> Result<(), CliError> {
    if relative_path == Path::new("pipeline-state.yaml") {
        sync_registry_from_pipeline(collection, spec_id, artifact)?;
    }
    Ok(())
}

fn read_json_yaml(path: &Path) -> Result<JsonValue, CliError> {
    let content = fs::read_to_string(path).map_err(|err| {
        CliError::new(
            "io_error",
            format!("Failed to read {}: {}", path.display(), err),
        )
    })?;
    let value: JsonValue = serde_yaml::from_str(&content)
        .map_err(|err| CliError::new("yaml_parse_failed", err.to_string()))?;
    Ok(value)
}

fn write_json_yaml(path: PathBuf, value: &JsonValue) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            CliError::new(
                "io_error",
                format!("Failed to create directory {}: {}", parent.display(), err),
            )
        })?;
    }
    let content = serde_yaml::to_string(value)
        .map_err(|err| CliError::new("yaml_emit_failed", err.to_string()))?;
    fs::write(&path, content).map_err(|err| {
        CliError::new(
            "io_error",
            format!("Failed to write {}: {}", path.display(), err),
        )
    })
}

fn template(name: &str) -> Result<JsonValue, CliError> {
    let source = match name {
        "component-batches.yaml" => {
            include_str!("../assets/templates/component-batches.yaml")
        }
        "component-detail.yaml" => {
            include_str!("../assets/templates/component-detail.yaml")
        }
        "component-index.yaml" => {
            include_str!("../assets/templates/component-index.yaml")
        }
        "handoff.yaml" => include_str!("../assets/templates/handoff.yaml"),
        "implementation-report.yaml" => {
            include_str!("../assets/templates/implementation-report.yaml")
        }
        "implementation-spec.yaml" => {
            include_str!("../assets/templates/implementation-spec.yaml")
        }
        "invocation-state.yaml" => {
            include_str!("../assets/templates/invocation-state.yaml")
        }
        "journey-batches.yaml" => {
            include_str!("../assets/templates/journey-batches.yaml")
        }
        "journey-detail.yaml" => {
            include_str!("../assets/templates/journey-detail.yaml")
        }
        "journey-index.yaml" => {
            include_str!("../assets/templates/journey-index.yaml")
        }
        "pipeline-state.yaml" => {
            include_str!("../assets/templates/pipeline-state.yaml")
        }
        "registry.yaml" => include_str!("../assets/templates/registry.yaml"),
        "request-context.yaml" => {
            include_str!("../assets/templates/request-context.yaml")
        }
        "role-map.yaml" => include_str!("../assets/templates/role-map.yaml"),
        "solution-outline.yaml" => {
            include_str!("../assets/templates/solution-outline.yaml")
        }
        _ => {
            return Err(CliError::new(
                "unknown_template",
                format!("Unknown embedded template {}", name),
            ));
        }
    };
    serde_yaml::from_str(source)
        .map_err(|err| CliError::new("template_parse_failed", err.to_string()))
}

fn ux_contracts() -> Result<JsonValue, CliError> {
    serde_yaml::from_str(include_str!("../assets/contracts/ux-contracts.yaml"))
        .map_err(|err| CliError::new("ux_contract_parse_failed", err.to_string()))
}

fn registry_template_with_defaults(collection: &Path) -> Result<JsonValue, CliError> {
    let mut registry = template("registry.yaml")?;
    set_path(
        &mut registry,
        "collection.target_path",
        json!(
            collection
                .parent()
                .unwrap_or(collection)
                .to_string_lossy()
                .to_string()
        ),
    );
    if string_at(&registry, "collection.created_at")
        .unwrap_or_default()
        .is_empty()
    {
        set_path(&mut registry, "collection.created_at", json!(now_utc()));
    }
    Ok(registry)
}

fn load_registry(collection: &Path) -> Result<JsonValue, CliError> {
    let path = collection.join(REGISTRY_FILENAME);
    if path.exists() {
        read_json_yaml(&path)
    } else {
        registry_template_with_defaults(collection)
    }
}

fn save_registry(collection: &Path, registry: &JsonValue) -> Result<(), CliError> {
    write_json_yaml(collection.join(REGISTRY_FILENAME), registry)
}

fn invocation_state_template() -> Result<JsonValue, CliError> {
    template("invocation-state.yaml")
}

fn load_invocation_state(collection: &Path) -> Result<JsonValue, CliError> {
    let path = collection.join(INVOCATION_STATE_FILENAME);
    if path.exists() {
        read_json_yaml(&path)
    } else {
        invocation_state_template()
    }
}

fn save_invocation_state(collection: &Path, state: &JsonValue) -> Result<(), CliError> {
    write_json_yaml(collection.join(INVOCATION_STATE_FILENAME), state)
}

fn clear_invocation_state(collection: &Path) -> Result<(), CliError> {
    let mut state = invocation_state_template()?;
    set_path(&mut state, "updated_at", json!(now_utc()));
    save_invocation_state(collection, &state)
}

struct RuntimeBuild {
    context: JsonValue,
    workspace: Option<PathBuf>,
    collection: PathBuf,
}

fn build_runtime_context(
    target_dir: &str,
    explicit_spec_id: Option<&str>,
) -> Result<RuntimeBuild, CliError> {
    let collection = collection_root(target_dir)?;
    let normalized_target = normalize_target_dir(target_dir)?;
    let collection_exists = collection.exists();
    let registry = if collection_exists {
        load_registry(&collection)?
    } else {
        empty_object()
    };
    let invocation_state = if collection_exists {
        load_invocation_state(&collection)?
    } else {
        invocation_state_template()?
    };

    let mut spec_id = explicit_spec_id
        .and_then(non_empty_string)
        .map(|value| slugify_spec_id(&value))
        .unwrap_or_default();
    let mut spec_id_source = if spec_id.is_empty() {
        String::new()
    } else {
        "explicit_argument".to_string()
    };

    if spec_id.is_empty() {
        if let Some(inferred) = infer_spec_id(target_dir)? {
            spec_id = inferred;
            spec_id_source = "explicit_path".to_string();
        }
    }
    if spec_id.is_empty() {
        if let Some(active_spec_id) = json_get_string(&registry, "active_spec_id") {
            spec_id = active_spec_id;
            spec_id_source = "active_registry".to_string();
        }
    }
    if spec_id.is_empty() {
        if let Some(specs) = registry.get("specs").and_then(JsonValue::as_array) {
            if specs.len() == 1 && non_empty(specs[0].get("spec_id")) {
                spec_id = specs[0]["spec_id"].as_str().unwrap_or_default().to_string();
                spec_id_source = "single_registry_match".to_string();
            }
        }
    }
    if spec_id.is_empty() {
        let invocation_spec_id =
            json_get_string(&invocation_state, "parameters.spec_id").unwrap_or_default();
        let invocation_stage = json_get_string(&invocation_state, "stage").unwrap_or_default();
        if !invocation_spec_id.is_empty() {
            let invocation_workspace =
                spec_workspace(&collection, &slugify_spec_id(&invocation_spec_id));
            if invocation_stage == ROUTER_STAGE || invocation_workspace.exists() {
                spec_id = slugify_spec_id(&invocation_spec_id);
                spec_id_source = "invocation_state".to_string();
            }
        }
    }

    let workspace = if spec_id.is_empty() {
        None
    } else {
        Some(spec_workspace(&collection, &spec_id))
    };
    let workspace_exists = workspace.as_ref().is_some_and(|path| path.exists());

    let mut pipeline = empty_object();
    let mut handoff = empty_object();
    let mut request_context = empty_object();
    let mut role_map = empty_object();
    let mut solution_outline = empty_object();
    let mut journey_index = empty_object();
    let mut component_index = empty_object();
    let mut journey_batches = empty_object();
    let mut component_batches = empty_object();
    let mut implementation_spec = empty_object();
    let mut implementation_report = empty_object();

    if workspace_exists {
        let workspace_path = workspace.as_ref().unwrap();
        pipeline = read_json_yaml(&workspace_path.join("pipeline-state.yaml"))?;
        handoff = read_json_yaml(&workspace_path.join("handoff.yaml"))?;
        request_context = read_json_yaml(&workspace_path.join("framing/request-context.yaml"))?;
        role_map = read_json_yaml(&workspace_path.join("framing/role-map.yaml"))?;
        solution_outline =
            read_json_yaml(&workspace_path.join("architecture/solution-outline.yaml"))?;
        journey_index = read_json_yaml(&workspace_path.join("architecture/journey-index.yaml"))?;
        component_index =
            read_json_yaml(&workspace_path.join("architecture/component-index.yaml"))?;
        journey_batches = read_json_yaml(&workspace_path.join("journeys/batches.yaml"))?;
        component_batches = read_json_yaml(&workspace_path.join("components/batches.yaml"))?;
        implementation_spec =
            read_json_yaml(&workspace_path.join("synthesis/implementation-spec.yaml"))?;
        implementation_report =
            read_json_yaml(&workspace_path.join("synthesis/implementation-report.yaml"))?;
    }

    let request_title = string_at(&pipeline, "request.title")
        .or_else(|| json_get_string(&invocation_state, "parameters.request_title"))
        .unwrap_or_default();
    let request_summary = string_at(&pipeline, "request.summary")
        .or_else(|| json_get_string(&invocation_state, "parameters.request_summary"))
        .unwrap_or_default();

    Ok(RuntimeBuild {
        context: json!({
            "workspace": {
                "target_dir": normalized_target.to_string_lossy().to_string(),
                "collection_root": collection.to_string_lossy().to_string(),
                "collection_exists": collection_exists,
                "spec_id": spec_id,
                "spec_id_source": spec_id_source,
                "spec_exists": workspace_exists,
                "spec_root": workspace.as_ref().map(|path| path.to_string_lossy().to_string()).unwrap_or_default(),
            },
            "registry": registry,
            "invocation": invocation_state,
            "pipeline": pipeline,
            "handoff": handoff,
            "request": {
                "spec_id": spec_id,
                "title": request_title,
                "summary": request_summary,
            },
            "artifacts": {
                "request_context": request_context,
                "role_map": role_map,
                "solution_outline": solution_outline,
                "journey_index": journey_index,
                "component_index": component_index,
                "journey_batches": journey_batches,
                "component_batches": component_batches,
                "implementation_spec": implementation_spec,
                "implementation_report": implementation_report,
            }
        }),
        workspace,
        collection,
    })
}

fn resolve_workspace(
    path_like: &str,
    spec_id: Option<&str>,
    require_existing: bool,
) -> Result<(PathBuf, String, PathBuf), CliError> {
    let collection = collection_root(path_like)?;
    let explicit_spec_id = spec_id
        .and_then(non_empty_string)
        .map(|value| slugify_spec_id(&value));
    let inferred = infer_spec_id(path_like)?;
    let mut effective_spec_id = explicit_spec_id.or(inferred);

    if effective_spec_id.is_none() {
        let registry = if collection.exists() {
            load_registry(&collection)?
        } else {
            empty_object()
        };
        if let Some(active_spec_id) = string_at(&registry, "active_spec_id") {
            effective_spec_id = Some(active_spec_id);
        } else if let Some(specs) = registry.get("specs").and_then(JsonValue::as_array) {
            if specs.len() == 1 {
                effective_spec_id = specs[0]
                    .get("spec_id")
                    .and_then(JsonValue::as_str)
                    .map(str::to_string);
            }
        }
    }

    let Some(effective_spec_id) = effective_spec_id else {
        return Err(CliError::new(
            "spec_id_unresolved",
            "No spec_id could be resolved. Pass --spec-id or point to .spec-forge/specs/<spec-id>.",
        ));
    };

    let workspace = spec_workspace(&collection, &effective_spec_id);
    if require_existing && !workspace.exists() {
        return Err(CliError::new(
            "workspace_missing",
            format!(
                "Spec workspace does not exist: {}. Initialize it first or pass a valid --spec-id.",
                workspace.display()
            ),
        ));
    }
    Ok((collection, effective_spec_id, workspace))
}

fn resolve_ux_contract(
    skill: Option<&str>,
    stage: Option<&str>,
    mode: Option<&str>,
    context: &JsonValue,
) -> Result<JsonValue, CliError> {
    let contract = skill_contract(skill, stage)?;
    let modes = contract
        .get("modes")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| CliError::new("ux_contract_invalid", "UX contract is missing modes."))?;
    let mut selected_mode = mode.and_then(non_empty_string).unwrap_or_else(|| {
        contract
            .get("default_mode")
            .and_then(JsonValue::as_str)
            .unwrap_or("guided")
            .to_string()
    });
    if selected_mode == "auto" {
        let collection_exists = bool_at(context, "workspace.collection_exists");
        let workspace_exists = bool_at(context, "workspace.spec_exists");
        selected_mode = if collection_exists && workspace_exists {
            "resume_existing_spec".to_string()
        } else {
            "initialize_new_spec".to_string()
        };
    }
    let Some(mode_config) = modes.get(&selected_mode).cloned() else {
        return Err(CliError::new(
            "ux_mode_missing",
            format!(
                "No UX mode {:?} defined for skill {:?}.",
                selected_mode,
                contract
                    .get("skill")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("")
            ),
        ));
    };

    let required_ids = array_at(&mode_config, "required_parameters")
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let required_id_set = required_ids
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    let mut parameters = Vec::new();
    let mut resolved_parameters = Map::new();
    let mut missing_parameters = Vec::new();
    for mut param in array_at(&contract, "parameters") {
        let param_id = param
            .get("id")
            .and_then(JsonValue::as_str)
            .unwrap_or_default()
            .to_string();
        let (value, source) = resolve_contract_value(&param, context);
        let required_in_mode = required_id_set.contains(&param_id);
        set_path(&mut param, "required_in_mode", json!(required_in_mode));
        set_path(
            &mut param,
            "resolved_from",
            json!(source.clone().unwrap_or_default()),
        );
        if let Some(value) = value {
            set_path(&mut param, "resolved_value", value.clone());
            resolved_parameters.insert(param_id.clone(), value);
        }
        if required_in_mode && !non_empty(param.get("resolved_value")) {
            missing_parameters.push(json!({
                "id": param_id,
                "label": param.get("label").cloned().unwrap_or(json!(param_id)),
                "description": param.get("description").cloned().unwrap_or(json!("")),
                "type": param.get("type").cloned().unwrap_or(json!("string")),
                "prompt": param.get("prompt").cloned().unwrap_or(json!("")),
                "reason": format!("Required by mode {}.", selected_mode),
            }));
        }
        parameters.push(param);
    }

    let mut choice_context = context.clone();
    set_path(&mut choice_context, "ux_mode", json!(selected_mode));
    let mut choices = Vec::new();
    for mut choice in array_at(&contract, "choices") {
        if !choice_should_be_available(&choice, &choice_context) {
            continue;
        }
        let options = choice_options_for(&choice, &choice_context);
        set_path(&mut choice, "options", JsonValue::Array(options));
        choices.push(choice);
    }

    Ok(json!({
        "family": contract.get("family").cloned().unwrap_or(json!("spec-forge")),
        "skill": contract.get("skill").cloned().unwrap_or(json!(skill.unwrap_or(stage_to_skill(stage.unwrap_or(ROUTER_STAGE))))),
        "stage": contract.get("stage").cloned().unwrap_or(json!(stage.unwrap_or(ROUTER_STAGE))),
        "contract_path": UX_CONTRACT_PATH,
        "contract_version": contract.get("version").cloned().unwrap_or(json!(1)),
        "execution": contract.get("execution").cloned().unwrap_or_else(empty_object),
        "mode": {
            "id": selected_mode,
            "label": mode_config.get("label").cloned().unwrap_or(json!("")),
            "description": mode_config.get("description").cloned().unwrap_or(json!("")),
        },
        "collection": contract.get("collection").cloned().unwrap_or_else(empty_object),
        "parameters": parameters,
        "resolved_parameters": JsonValue::Object(resolved_parameters),
        "missing_parameters": missing_parameters,
        "choices": choices,
    }))
}

fn build_interactive_requirements(
    resolution: &JsonValue,
    runtime_mode: &str,
) -> Result<JsonValue, CliError> {
    let collection_cfg = resolution
        .get("collection")
        .cloned()
        .unwrap_or_else(empty_object);
    let execution_cfg = resolution
        .get("execution")
        .cloned()
        .unwrap_or_else(empty_object);
    let missing_parameters = resolution
        .get("missing_parameters")
        .cloned()
        .unwrap_or_else(|| JsonValue::Array(vec![]));
    let mut available_choices = array_at(resolution, "choices")
        .into_iter()
        .filter(|choice| !array_at(choice, "options").is_empty())
        .collect::<Vec<_>>();
    available_choices.sort_by(|left, right| {
        let left_priority = left
            .get("interaction_priority")
            .and_then(JsonValue::as_i64)
            .unwrap_or(100);
        let right_priority = right
            .get("interaction_priority")
            .and_then(JsonValue::as_i64)
            .unwrap_or(100);
        left_priority.cmp(&right_priority).then_with(|| {
            left.get("id")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .cmp(right.get("id").and_then(JsonValue::as_str).unwrap_or(""))
        })
    });

    let interactive_collection = collection_cfg
        .get("interactive_collection")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let runtime_policy = execution_cfg
        .get("runtime_mode")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let plan_only = interactive_collection == "plan_mode_only";
    let runtime_plan_only = runtime_policy == "plan_mode_only";
    let plan_mode_required_if_interactive = runtime_plan_only
        || (plan_only
            && (!available_choices.is_empty() || !array_at(&missing_parameters, "").is_empty()));
    let requires_plan_mode = runtime_mode == "default"
        && (runtime_plan_only
            || (plan_only
                && (!available_choices.is_empty()
                    || !array_at(&missing_parameters, "").is_empty())));
    let missing_parameter_items = array_at(&missing_parameters, "");
    let missing_ids = missing_parameter_items
        .into_iter()
        .filter_map(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .collect::<std::collections::BTreeSet<_>>();
    let blocking_choices = available_choices
        .iter()
        .filter(|choice| choice.get("before_missing_parameters") == Some(&json!(true)))
        .cloned()
        .collect::<Vec<_>>();
    let choice_for_missing_parameter = available_choices.iter().find(|choice| {
        choice
            .get("answers_parameter")
            .and_then(JsonValue::as_str)
            .is_some_and(|parameter| missing_ids.contains(parameter))
    });

    let (next_interaction, next_prompt, next_choice_dialog) = if requires_plan_mode {
        if runtime_plan_only {
            (
                json!({
                    "type": "switch_to_plan_mode",
                    "reason": "This skill family only runs in Plan mode.",
                }),
                json!(
                    "Switch to Plan mode before using this skill. Do not continue in the default runtime."
                ),
                empty_object(),
            )
        } else {
            (
                json!({
                    "type": "switch_to_plan_mode",
                    "reason": "The UX contract requires plan_mode_only for interactive collection.",
                }),
                json!(
                    "Switch to Plan mode before collecting missing inputs or showing a choice dialog."
                ),
                empty_object(),
            )
        }
    } else if let Some(choice) = blocking_choices.first() {
        (
            json!({
                "type": "show_choice_dialog",
                "choice": choice,
            }),
            choice.get("prompt").cloned().unwrap_or(json!("")),
            choice.clone(),
        )
    } else if let Some(choice) = choice_for_missing_parameter {
        (
            json!({
                "type": "show_choice_dialog",
                "choice": choice,
            }),
            choice.get("prompt").cloned().unwrap_or(json!("")),
            choice.clone(),
        )
    } else if let Some(parameter) = missing_parameters
        .as_array()
        .and_then(|items| items.first())
    {
        (
            json!({
                "type": "ask_parameter",
                "parameter": parameter,
            }),
            parameter.get("prompt").cloned().unwrap_or(json!("")),
            empty_object(),
        )
    } else if let Some(choice) = available_choices.first() {
        (
            json!({
                "type": "show_choice_dialog",
                "choice": choice,
            }),
            choice.get("prompt").cloned().unwrap_or(json!("")),
            choice.clone(),
        )
    } else {
        (
            json!({
                "type": "ready",
                "reason": "Invocation has enough information to continue without more input.",
            }),
            json!("No additional interactive input is required."),
            empty_object(),
        )
    };

    Ok(json!({
        "runtime_mode": runtime_mode,
        "runtime_policy": runtime_policy,
        "interactive_collection": interactive_collection,
        "plan_mode_required_if_interactive": plan_mode_required_if_interactive,
        "plan_mode_only": runtime_plan_only,
        "requires_plan_mode": requires_plan_mode,
        "missing_parameters": missing_parameters,
        "available_choices": available_choices,
        "next_interaction": next_interaction,
        "next_prompt": next_prompt,
        "next_choice_dialog": next_choice_dialog,
    }))
}

fn apply_ux_resolution(
    pipeline: &mut JsonValue,
    handoff: &mut JsonValue,
    resolution: &JsonValue,
    runtime_mode: &str,
) -> Result<(), CliError> {
    let interactive_requirements = build_interactive_requirements(resolution, runtime_mode)?;
    let payload = json!({
        "skill": resolution.get("skill").cloned().unwrap_or(json!("")),
        "stage": resolution.get("stage").cloned().unwrap_or(json!("")),
        "contract_path": resolution.get("contract_path").cloned().unwrap_or(json!("")),
        "contract_version": resolution.get("contract_version").cloned().unwrap_or(json!(1)),
        "execution": resolution.get("execution").cloned().unwrap_or_else(empty_object),
        "mode": resolution.get("mode").cloned().unwrap_or_else(empty_object),
        "collection": resolution.get("collection").cloned().unwrap_or_else(empty_object),
        "resolved_parameters": resolution.get("resolved_parameters").cloned().unwrap_or_else(empty_object),
        "missing_parameters": resolution.get("missing_parameters").cloned().unwrap_or_else(|| JsonValue::Array(vec![])),
        "choices": resolution.get("choices").cloned().unwrap_or_else(|| JsonValue::Array(vec![])),
        "interactive_requirements": interactive_requirements,
        "resolved_at": now_utc(),
    });
    set_path(pipeline, "ux", payload.clone());
    set_path(handoff, "ux", payload.clone());
    set_path(
        handoff,
        "next_interaction",
        payload["interactive_requirements"]["next_interaction"].clone(),
    );
    set_path(
        handoff,
        "next_prompt",
        payload["interactive_requirements"]["next_prompt"].clone(),
    );
    set_path(
        handoff,
        "next_choice_dialog",
        payload["interactive_requirements"]["next_choice_dialog"].clone(),
    );
    set_path(
        handoff,
        "mode_requirement",
        json!({
            "runtime_policy": payload["interactive_requirements"]["runtime_policy"],
            "plan_mode_only": payload["interactive_requirements"]["plan_mode_only"],
            "interactive_collection": payload["interactive_requirements"]["interactive_collection"],
            "plan_mode_required_if_interactive": payload["interactive_requirements"]["plan_mode_required_if_interactive"],
        }),
    );
    Ok(())
}

fn completion_resolution(
    target_dir: &str,
    spec_id: &str,
    pipeline: &JsonValue,
    handoff: &JsonValue,
) -> JsonValue {
    let mut resolved_parameters = Map::new();
    for source in [
        nested_get(pipeline, "ux.resolved_parameters"),
        nested_get(handoff, "ux.resolved_parameters"),
    ] {
        if let Some(parameters) = source.and_then(JsonValue::as_object) {
            for (key, value) in parameters {
                resolved_parameters
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }
    }
    if !target_dir.is_empty() {
        resolved_parameters.insert("target_dir".to_string(), json!(target_dir));
    }
    if !spec_id.is_empty() {
        resolved_parameters.insert("spec_id".to_string(), json!(spec_id));
    }

    json!({
        "skill": "spec-forge-implement",
        "stage": "complete",
        "contract_path": UX_CONTRACT_PATH,
        "contract_version": 1,
        "execution": {
            "runtime_mode": "default",
            "approval_style": "chat_summary_confirmation",
            "file_review_policy": "assistant_summarizes_in_chat",
        },
        "mode": {
            "id": "completed_workflow",
            "label": "Completed Workflow",
            "description": "All workflow stages passed. No additional interactive input is required.",
        },
        "collection": {
            "mode": "summary_only",
            "interactive_collection": "none",
            "choice_presentation": "dialog_when_available",
            "fallback_collection": "inline_short_questions",
        },
        "resolved_parameters": JsonValue::Object(resolved_parameters),
        "missing_parameters": [],
        "choices": [],
    })
}

fn finalize_workspace_handoff(
    pipeline: &mut JsonValue,
    handoff: &mut JsonValue,
    stage: &str,
    ready_for_workspace_init: bool,
) {
    let missing_parameters = handoff
        .get("ux")
        .and_then(|value| value.get("interactive_requirements"))
        .and_then(|value| value.get("missing_parameters"))
        .cloned()
        .unwrap_or_else(|| json!([]));
    let next_type = string_at(handoff, "ux.interactive_requirements.next_interaction.type")
        .or_else(|| {
            string_at(
                pipeline,
                "ux.interactive_requirements.next_interaction.type",
            )
        })
        .unwrap_or_else(|| "ready".to_string());
    set_path(
        handoff,
        "required_inputs",
        json!(ids_from_array(&missing_parameters)),
    );
    set_handoff_recommended_command(
        handoff,
        recommended_command_for_interaction(&next_type, stage, ready_for_workspace_init),
    );
    sync_pipeline_handoff(pipeline, handoff);
}

fn sync_pipeline_handoff(pipeline: &mut JsonValue, handoff: &JsonValue) {
    set_path(pipeline, "handoff", handoff.clone());
}

fn write_pipeline_and_handoff(
    workspace: &Path,
    pipeline: &mut JsonValue,
    handoff: &JsonValue,
) -> Result<(), CliError> {
    sync_pipeline_handoff(pipeline, handoff);
    write_json_yaml(workspace.join("pipeline-state.yaml"), pipeline)?;
    write_json_yaml(workspace.join("handoff.yaml"), handoff)?;
    Ok(())
}

fn set_handoff_recommended_command(handoff: &mut JsonValue, command: impl AsRef<str>) {
    let command = command.as_ref();
    set_path(handoff, "recommended_script", json!(command));
    set_path(handoff, "recommended_command", json!(command));
}

fn recommended_command_for_interaction(
    next_interaction_type: &str,
    stage: &str,
    ready_for_workspace_init: bool,
) -> String {
    if ready_for_workspace_init {
        return "spec-forge-cli init".to_string();
    }
    match next_interaction_type {
        "ask_parameter" | "show_choice_dialog" => "spec-forge-cli apply".to_string(),
        "switch_to_plan_mode" => "spec-forge-cli resolve".to_string(),
        "ready" => {
            if stage != ROUTER_STAGE {
                "spec-forge-cli stage advance".to_string()
            } else {
                "spec-forge-cli resolve".to_string()
            }
        }
        _ => "spec-forge-cli resolve".to_string(),
    }
}

fn blank_gate_report(stage: &str) -> JsonValue {
    json!({
        "stage": stage,
        "checked_at": "",
        "passed": false,
        "checks": [],
        "blocking_checks": [],
        "recommended_next_stage": stage,
    })
}

fn evaluate_stage(workspace: &Path, stage: &str) -> Result<JsonValue, CliError> {
    let request_context = read_json_yaml(&workspace.join("framing/request-context.yaml"))?;
    let role_map = read_json_yaml(&workspace.join("framing/role-map.yaml"))?;
    let solution_outline = read_json_yaml(&workspace.join("architecture/solution-outline.yaml"))?;
    let journey_index = read_json_yaml(&workspace.join("architecture/journey-index.yaml"))?;
    let component_index = read_json_yaml(&workspace.join("architecture/component-index.yaml"))?;
    let journey_batches = read_json_yaml(&workspace.join("journeys/batches.yaml"))?;
    let component_batches = read_json_yaml(&workspace.join("components/batches.yaml"))?;
    let implementation_spec =
        read_json_yaml(&workspace.join("synthesis/implementation-spec.yaml"))?;
    let implementation_report =
        read_json_yaml(&workspace.join("synthesis/implementation-report.yaml"))?;
    let readiness_gate = read_json_yaml(&workspace.join("gates/readiness.yaml"))?;

    let mut checks = Vec::new();
    match stage {
        "intake" => {
            checks.extend([
                make_check(
                    "problem_statement",
                    non_empty(
                        request_context
                            .get("problem")
                            .and_then(|v| v.get("statement")),
                    ),
                    "Problem statement is captured.",
                    "Problem statement is missing.",
                ),
                make_check(
                    "problem_goal",
                    non_empty(request_context.get("problem").and_then(|v| v.get("goal"))),
                    "Problem goal is captured.",
                    "Problem goal is missing.",
                ),
                make_check(
                    "scope_in_scope",
                    non_empty(request_context.get("scope").and_then(|v| v.get("in_scope"))),
                    "In-scope items are listed.",
                    "In-scope items are missing.",
                ),
                make_check(
                    "primary_users",
                    non_empty(
                        request_context
                            .get("actors")
                            .and_then(|v| v.get("primary_users")),
                    ),
                    "Primary users are listed.",
                    "Primary users are missing.",
                ),
                make_check(
                    "roles_defined",
                    role_map
                        .get("roles")
                        .and_then(JsonValue::as_array)
                        .map(|items| items.len() >= 3)
                        .unwrap_or(false),
                    "At least three active roles are defined.",
                    "Fewer than three roles are defined.",
                ),
                make_check(
                    "request_context_approved",
                    approval_is_confirmed(&request_context),
                    "Request context is approved.",
                    "Request context is not approved.",
                ),
                make_check(
                    "role_map_approved",
                    approval_is_confirmed(&role_map),
                    "Role map is approved.",
                    "Role map is not approved.",
                ),
            ]);
        }
        "architecture" => {
            checks.extend([
                make_check(
                    "intake_complete",
                    approval_is_confirmed(&request_context) && approval_is_confirmed(&role_map),
                    "Intake artifacts are approved.",
                    "Architecture cannot pass before intake artifacts are approved.",
                ),
                make_check(
                    "solution_summary",
                    non_empty(
                        solution_outline
                            .get("solution")
                            .and_then(|v| v.get("summary")),
                    ),
                    "Solution summary is captured.",
                    "Solution summary is missing.",
                ),
                make_check(
                    "journey_index_items",
                    journey_index
                        .get("items")
                        .and_then(JsonValue::as_array)
                        .map(|items| !items.is_empty())
                        .unwrap_or(false),
                    "Journey index contains at least one item.",
                    "Journey index is empty.",
                ),
                make_check(
                    "component_index_items",
                    component_index
                        .get("items")
                        .and_then(JsonValue::as_array)
                        .map(|items| !items.is_empty())
                        .unwrap_or(false),
                    "Component index contains at least one item.",
                    "Component index is empty.",
                ),
                make_check(
                    "solution_outline_approved",
                    approval_is_confirmed(&solution_outline),
                    "Solution outline is approved.",
                    "Solution outline is not approved.",
                ),
                make_check(
                    "journey_index_approved",
                    approval_is_confirmed(&journey_index),
                    "Journey index is approved.",
                    "Journey index is not approved.",
                ),
                make_check(
                    "component_index_approved",
                    approval_is_confirmed(&component_index),
                    "Component index is approved.",
                    "Component index is not approved.",
                ),
            ]);
        }
        "journeys" => {
            let items = array_at(&journey_index, "items");
            checks.extend([
                make_check(
                    "architecture_complete",
                    approval_is_confirmed(&solution_outline)
                        && approval_is_confirmed(&journey_index)
                        && approval_is_confirmed(&component_index),
                    "Architecture artifacts are approved.",
                    "Journeys cannot pass before architecture artifacts are approved.",
                ),
                make_check(
                    "journey_batches_exist",
                    journey_batches
                        .get("batches")
                        .and_then(JsonValue::as_array)
                        .map(|batches| !batches.is_empty())
                        .unwrap_or(false),
                    "Journey batches are defined.",
                    "Journey batches are missing.",
                ),
                make_check(
                    "journey_batches_approved",
                    approval_is_confirmed(&journey_batches),
                    "Journey batches are approved.",
                    "Journey batches are not approved.",
                ),
            ]);
            checks.extend(detail_approval_checks(
                workspace, "journeys", &items, "journey",
            )?);
        }
        "components" => {
            let items = array_at(&component_index, "items");
            checks.extend([
                make_check(
                    "journeys_complete",
                    approval_is_confirmed(&journey_batches),
                    "Journey stage artifacts are approved.",
                    "Components cannot pass before the journeys stage is approved.",
                ),
                make_check(
                    "component_batches_exist",
                    component_batches
                        .get("batches")
                        .and_then(JsonValue::as_array)
                        .map(|batches| !batches.is_empty())
                        .unwrap_or(false),
                    "Component batches are defined.",
                    "Component batches are missing.",
                ),
                make_check(
                    "component_batches_approved",
                    approval_is_confirmed(&component_batches),
                    "Component batches are approved.",
                    "Component batches are not approved.",
                ),
            ]);
            checks.extend(detail_approval_checks(
                workspace,
                "components",
                &items,
                "component",
            )?);
        }
        "readiness" => {
            checks.extend([
                make_check(
                    "components_complete",
                    approval_is_confirmed(&component_batches),
                    "Component stage artifacts are approved.",
                    "Readiness cannot pass before components are approved.",
                ),
                make_check(
                    "implementation_scope",
                    non_empty(
                        implementation_spec
                            .get("summary")
                            .and_then(|v| v.get("scope")),
                    ),
                    "Final scope is listed.",
                    "Final scope is missing.",
                ),
                make_check(
                    "acceptance_criteria",
                    implementation_spec
                        .get("acceptance_criteria")
                        .and_then(JsonValue::as_array)
                        .map(|items| !items.is_empty())
                        .unwrap_or(false),
                    "Acceptance criteria are listed.",
                    "Acceptance criteria are missing.",
                ),
                make_check(
                    "no_unresolved_items",
                    implementation_spec
                        .get("unresolved_items")
                        .and_then(JsonValue::as_array)
                        .map(|items| items.is_empty())
                        .unwrap_or(true),
                    "No unresolved items remain.",
                    "Unresolved items remain in the final spec.",
                ),
                make_check(
                    "implementation_gate_ready",
                    implementation_spec
                        .get("implementation_gate")
                        .and_then(|v| v.get("ready"))
                        .and_then(JsonValue::as_bool)
                        .unwrap_or(false),
                    "Implementation gate is marked ready.",
                    "Implementation gate is not marked ready.",
                ),
                make_check(
                    "implementation_spec_approved",
                    approval_is_confirmed(&implementation_spec),
                    "Implementation spec is approved.",
                    "Implementation spec is not approved.",
                ),
            ]);
        }
        "implement" => {
            let validations = array_at(&implementation_report, "validations");
            let blockers = array_at(&implementation_report, "blockers");
            let failed_validations = validations
                .iter()
                .filter(|item| {
                    item.get("status")
                        .and_then(JsonValue::as_str)
                        .map(|status| matches!(status.to_lowercase().as_str(), "failed" | "error"))
                        .unwrap_or(false)
                })
                .count();
            checks.extend([
                make_check(
                    "readiness_complete",
                    readiness_gate
                        .get("passed")
                        .and_then(JsonValue::as_bool)
                        .unwrap_or(false)
                        && approval_is_confirmed(&implementation_spec)
                        && implementation_spec
                            .get("implementation_gate")
                            .and_then(|v| v.get("ready"))
                            .and_then(JsonValue::as_bool)
                            .unwrap_or(false),
                    "Readiness artifacts are approved and marked ready.",
                    "Implement cannot pass before readiness is approved and marked ready.",
                ),
                make_check(
                    "implementation_completed",
                    implementation_report
                        .get("implementation")
                        .and_then(|v| v.get("completed"))
                        .and_then(JsonValue::as_bool)
                        .unwrap_or(false),
                    "Implementation report is marked completed.",
                    "Implementation report is not marked completed.",
                ),
                make_check(
                    "validations_recorded",
                    !validations.is_empty(),
                    "Implementation validations are recorded.",
                    "Implementation validations are missing.",
                ),
                make_check(
                    "no_failed_validations",
                    failed_validations == 0,
                    "No failed validations are recorded.",
                    "Implementation report contains failed validations.",
                ),
                make_check(
                    "no_blockers",
                    blockers.is_empty(),
                    "No implementation blockers remain.",
                    "Implementation report still lists blockers.",
                ),
            ]);
        }
        _ => {
            return Err(CliError::new(
                "unknown_stage",
                format!("Unknown stage {}", stage),
            ));
        }
    }

    let blocking_checks = checks
        .iter()
        .filter(|check| !check["passed"].as_bool().unwrap_or(false))
        .filter_map(|check| {
            check
                .get("id")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "stage": stage,
        "checked_at": now_utc(),
        "passed": blocking_checks.is_empty(),
        "checks": checks,
        "blocking_checks": blocking_checks,
        "recommended_next_stage": if blocking_checks.is_empty() {
            next_stage(stage).unwrap_or(stage)
        } else {
            stage
        },
    }))
}

fn detail_approval_checks(
    workspace: &Path,
    folder: &str,
    items: &[JsonValue],
    label: &str,
) -> Result<Vec<JsonValue>, CliError> {
    let mut checks = Vec::new();
    for item in items {
        if item_is_deferred(item) {
            continue;
        }
        let item_id = item
            .get("id")
            .and_then(JsonValue::as_str)
            .unwrap_or("")
            .trim();
        if item_id.is_empty() {
            checks.push(make_check(
                &format!("{}_missing_id", label),
                false,
                "",
                &format!("{} index contains an item without an id.", label),
            ));
            continue;
        }
        let detail_path = workspace.join(folder).join(format!("{}.yaml", item_id));
        let exists = detail_path.exists();
        checks.push(make_check(
            &format!("{}_{}_exists", label, item_id),
            exists,
            &format!("{} detail file exists for {}.", label, item_id),
            &format!("Missing {} detail file for {}.", label, item_id),
        ));
        if exists {
            let detail = read_json_yaml(&detail_path)?;
            checks.push(make_check(
                &format!("{}_{}_approved", label, item_id),
                approval_is_confirmed(&detail),
                &format!("{} detail {} is approved.", label, item_id),
                &format!("{} detail {} is not approved.", label, item_id),
            ));
        }
    }
    Ok(checks)
}

fn make_check(check_id: &str, passed: bool, ok: &str, fail: &str) -> JsonValue {
    json!({
        "id": check_id,
        "passed": passed,
        "message": if passed { ok } else { fail },
    })
}

fn select_batch(
    batches: &JsonValue,
    items: Vec<JsonValue>,
    max_items: usize,
) -> (Vec<String>, String) {
    for batch in array_at(batches, "batches") {
        let status = batch
            .get("status")
            .and_then(JsonValue::as_str)
            .unwrap_or("pending")
            .to_lowercase();
        if matches!(status.as_str(), "pending" | "ready" | "in_progress") {
            let ids = array_at(&batch, "item_ids")
                .into_iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>();
            if !ids.is_empty() {
                return (
                    ids.into_iter().take(max_items).collect(),
                    format!(
                        "Use declared batch {}.",
                        batch
                            .get("batch_id")
                            .and_then(JsonValue::as_str)
                            .unwrap_or("")
                    ),
                );
            }
        }
    }
    let mut pending = items
        .into_iter()
        .filter(|item| {
            let status = item_status(item);
            !matches!(
                status.as_str(),
                "approved" | "deferred" | "out_of_scope" | "cancelled"
            )
        })
        .collect::<Vec<_>>();
    pending.sort_by(compare_pending_items);
    let ids = pending
        .into_iter()
        .filter_map(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .take(max_items)
        .collect::<Vec<_>>();
    (
        ids,
        "Use the next pending item set derived from the stage index.".to_string(),
    )
}

fn compare_pending_items(left: &JsonValue, right: &JsonValue) -> Ordering {
    priority_rank(left)
        .cmp(&priority_rank(right))
        .then_with(|| {
            left.get("sequence")
                .and_then(JsonValue::as_i64)
                .unwrap_or(0)
                .cmp(
                    &right
                        .get("sequence")
                        .and_then(JsonValue::as_i64)
                        .unwrap_or(0),
                )
        })
        .then_with(|| {
            left.get("id")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .cmp(right.get("id").and_then(JsonValue::as_str).unwrap_or(""))
        })
}

fn priority_rank(item: &JsonValue) -> (i32, String) {
    let value = item
        .get("priority")
        .and_then(JsonValue::as_str)
        .unwrap_or("medium")
        .to_lowercase();
    let rank = match value.as_str() {
        "high" => 0,
        "medium" => 1,
        "low" => 2,
        _ => 1,
    };
    (rank, value)
}

fn write_workspace_artifacts(
    workspace: &Path,
    pipeline: &JsonValue,
    request_context: &JsonValue,
    role_map: &JsonValue,
    solution_outline: &JsonValue,
    journey_index: &JsonValue,
    component_index: &JsonValue,
    journey_batches: &JsonValue,
    component_batches: &JsonValue,
    implementation_spec: &JsonValue,
) -> Result<(), CliError> {
    write_json_yaml(workspace.join("pipeline-state.yaml"), pipeline)?;
    write_json_yaml(
        workspace.join("framing/request-context.yaml"),
        request_context,
    )?;
    write_json_yaml(workspace.join("framing/role-map.yaml"), role_map)?;
    write_json_yaml(
        workspace.join("architecture/solution-outline.yaml"),
        solution_outline,
    )?;
    write_json_yaml(
        workspace.join("architecture/journey-index.yaml"),
        journey_index,
    )?;
    write_json_yaml(
        workspace.join("architecture/component-index.yaml"),
        component_index,
    )?;
    write_json_yaml(workspace.join("journeys/batches.yaml"), journey_batches)?;
    write_json_yaml(workspace.join("components/batches.yaml"), component_batches)?;
    write_json_yaml(
        workspace.join("synthesis/implementation-spec.yaml"),
        implementation_spec,
    )?;
    Ok(())
}

fn find_parameter(resolution: &JsonValue, param_id: &str) -> Result<JsonValue, CliError> {
    array_at(resolution, "parameters")
        .into_iter()
        .find(|param| param.get("id").and_then(JsonValue::as_str) == Some(param_id))
        .ok_or_else(|| {
            CliError::new(
                "parameter_missing",
                format!(
                    "Parameter {:?} is not defined for this invocation.",
                    param_id
                ),
            )
        })
}

fn find_choice(resolution: &JsonValue, choice_id: &str) -> Result<JsonValue, CliError> {
    array_at(resolution, "choices")
        .into_iter()
        .find(|choice| choice.get("id").and_then(JsonValue::as_str) == Some(choice_id))
        .ok_or_else(|| {
            CliError::new(
                "choice_missing",
                format!(
                    "Choice {:?} is not available for this invocation.",
                    choice_id
                ),
            )
        })
}

fn parse_yaml_values(values: &[String]) -> Result<JsonValue, CliError> {
    let mut parsed = Vec::new();
    for value in values {
        parsed.push(
            serde_yaml::from_str::<JsonValue>(value)
                .map_err(|err| CliError::new("value_parse_failed", err.to_string()))?,
        );
    }
    Ok(if parsed.len() == 1 {
        parsed.into_iter().next().unwrap()
    } else {
        JsonValue::Array(parsed)
    })
}

fn normalize_choice_value(
    choice: &JsonValue,
    raw_value: &JsonValue,
) -> Result<JsonValue, CliError> {
    let choice_type = choice
        .get("type")
        .and_then(JsonValue::as_str)
        .unwrap_or("single");
    let value = if choice_type == "multi" {
        JsonValue::Array(
            string_list(raw_value)
                .into_iter()
                .map(JsonValue::String)
                .collect(),
        )
    } else {
        let values = string_list(raw_value);
        if values.len() != 1 {
            return Err(CliError::new(
                "choice_value_invalid",
                format!(
                    "Choice {:?} expects exactly one value.",
                    choice
                        .get("id")
                        .and_then(JsonValue::as_str)
                        .unwrap_or("<unknown>")
                ),
            ));
        }
        JsonValue::String(values[0].clone())
    };

    let allowed_options = array_at(choice, "options");
    let allowed = allowed_options
        .into_iter()
        .filter_map(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .collect::<std::collections::BTreeSet<_>>();
    if !allowed.is_empty() {
        let candidate_values = string_list(&value);
        let invalid = candidate_values
            .iter()
            .filter(|item| !allowed.contains(item.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if !invalid.is_empty() {
            return Err(CliError::new(
                "choice_value_invalid",
                format!(
                    "Choice value(s) {:?} are not in the available options for {:?}.",
                    invalid,
                    choice.get("id").and_then(JsonValue::as_str).unwrap_or("")
                ),
            ));
        }
    }
    Ok(value)
}

fn apply_parameter_answer(
    stage: &str,
    param_id: &str,
    value: &JsonValue,
    workspace_exists: bool,
    invocation_state: &mut JsonValue,
    pipeline: &mut JsonValue,
    request_context: &mut JsonValue,
    solution_outline: &mut JsonValue,
    journey_index: &mut JsonValue,
    component_index: &mut JsonValue,
    journey_batches: &mut JsonValue,
    component_batches: &mut JsonValue,
    implementation_spec: &mut JsonValue,
) -> Result<Vec<String>, CliError> {
    let mut touched = Vec::new();
    if !workspace_exists || stage == ROUTER_STAGE {
        let path = if param_id == "spec_id" {
            let raw = value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string());
            json!(slugify_spec_id(&raw))
        } else {
            value.clone()
        };
        set_path(invocation_state, &format!("parameters.{}", param_id), path);
        return Ok(touched);
    }

    match param_id {
        "problem_statement" => {
            set_path(request_context, "problem.statement", value.clone());
            touched.push("framing/request-context.yaml".to_string());
        }
        "problem_goal" => {
            set_path(request_context, "problem.goal", value.clone());
            touched.push("framing/request-context.yaml".to_string());
        }
        "primary_users" => {
            set_path(
                request_context,
                "actors.primary_users",
                json!(string_list(value)),
            );
            touched.push("framing/request-context.yaml".to_string());
        }
        "in_scope" => {
            set_path(request_context, "scope.in_scope", json!(string_list(value)));
            touched.push("framing/request-context.yaml".to_string());
        }
        "request_summary" => {
            set_path(pipeline, "request.summary", value.clone());
            touched.push("pipeline-state.yaml".to_string());
        }
        "solution_summary" => {
            set_path(solution_outline, "solution.summary", value.clone());
            touched.push("architecture/solution-outline.yaml".to_string());
        }
        "journey_index" => {
            set_path(
                journey_index,
                "items",
                if value.is_array() {
                    value.clone()
                } else {
                    JsonValue::Array(vec![value.clone()])
                },
            );
            touched.push("architecture/journey-index.yaml".to_string());
        }
        "component_index" => {
            set_path(
                component_index,
                "items",
                if value.is_array() {
                    value.clone()
                } else {
                    JsonValue::Array(vec![value.clone()])
                },
            );
            touched.push("architecture/component-index.yaml".to_string());
        }
        "selected_focus_ids" => {
            let selected_ids = string_list(value);
            set_stage_focus(
                pipeline,
                stage,
                &selected_ids,
                &format!("Manual {} focus selection.", stage),
            );
            if stage == "journeys" {
                set_path(journey_batches, "current_batch_id", json!(""));
                touched.push("journeys/batches.yaml".to_string());
            }
            if stage == "components" {
                set_path(component_batches, "current_batch_id", json!(""));
                touched.push("components/batches.yaml".to_string());
            }
            touched.push("pipeline-state.yaml".to_string());
        }
        "implementation_ready" => {
            set_path(
                implementation_spec,
                "implementation_gate.ready",
                json!(value.as_bool().unwrap_or(false)),
            );
            touched.push("synthesis/implementation-spec.yaml".to_string());
        }
        _ => {
            set_path(
                invocation_state,
                &format!("parameters.{}", param_id),
                value.clone(),
            );
        }
    }
    Ok(touched)
}

fn apply_choice_answer(
    stage: &str,
    choice_id: &str,
    answers_parameter: Option<&str>,
    value: &JsonValue,
    invocation_state: &mut JsonValue,
    registry: &mut JsonValue,
    pipeline: &mut JsonValue,
    request_context: &mut JsonValue,
    role_map: &mut JsonValue,
    implementation_spec: &mut JsonValue,
) -> Result<Vec<String>, CliError> {
    let mut touched = Vec::new();
    set_path(
        invocation_state,
        &format!("choices.{}", choice_id),
        value.clone(),
    );

    match choice_id {
        "spec_selection_mode" => {
            let selected = value.as_str().unwrap_or_default();
            set_path(invocation_state, "mode", json!(selected));
            if selected == "initialize_new_spec" {
                remove_path(invocation_state, "parameters.spec_id");
                set_path(registry, "active_spec_id", json!(""));
                touched.push("registry.yaml".to_string());
            }
        }
        "existing_spec_id" => {
            let selected = slugify_spec_id(value.as_str().unwrap_or_default());
            set_path(invocation_state, "parameters.spec_id", json!(selected));
            if stage == ROUTER_STAGE
                && string_at(invocation_state, "mode")
                    .unwrap_or_default()
                    .is_empty()
            {
                set_path(invocation_state, "mode", json!("resume_existing_spec"));
            }
            set_path(registry, "active_spec_id", json!(selected));
            touched.push("registry.yaml".to_string());
        }
        "active_roles" => {
            let selected_ids = string_list(value);
            set_path(request_context, "actors.reviewers", json!(selected_ids));
            if let Some(roles) = role_map.get_mut("roles").and_then(JsonValue::as_array_mut) {
                let selected = selected_ids
                    .iter()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>();
                for role in roles {
                    let role_id = role.get("id").and_then(JsonValue::as_str).unwrap_or("");
                    set_path(role, "enabled", json!(selected.contains(role_id)));
                }
            }
            touched.push("framing/request-context.yaml".to_string());
            touched.push("framing/role-map.yaml".to_string());
        }
        "review_cluster" => {
            let selected = value.as_str().unwrap_or_default().to_string();
            set_stage_focus(
                pipeline,
                "architecture",
                &[selected.clone()],
                &format!("Architecture review cluster: {}.", selected),
            );
            touched.push("pipeline-state.yaml".to_string());
        }
        "journey_focus" => {
            let selected = value.as_str().unwrap_or_default().to_string();
            set_stage_focus(
                pipeline,
                "journeys",
                &[selected.clone()],
                &format!("Journey focus: {}.", selected),
            );
            touched.push("pipeline-state.yaml".to_string());
        }
        "component_focus" => {
            let selected = value.as_str().unwrap_or_default().to_string();
            set_stage_focus(
                pipeline,
                "components",
                &[selected.clone()],
                &format!("Component focus: {}.", selected),
            );
            touched.push("pipeline-state.yaml".to_string());
        }
        "readiness_decision" => {
            set_path(
                implementation_spec,
                "implementation_gate.ready",
                json!(value.as_str() == Some("ready")),
            );
            touched.push("synthesis/implementation-spec.yaml".to_string());
        }
        _ => {
            if let Some(parameter_id) = answers_parameter.and_then(non_empty_string) {
                let stored = if parameter_id == "spec_id" {
                    json!(slugify_spec_id(value.as_str().unwrap_or_default()))
                } else {
                    value.clone()
                };
                set_path(
                    invocation_state,
                    &format!("parameters.{}", parameter_id),
                    stored.clone(),
                );
                if parameter_id == "spec_id" {
                    set_path(registry, "active_spec_id", stored);
                    touched.push("registry.yaml".to_string());
                }
            }
        }
    }

    Ok(touched)
}

fn set_stage_focus(pipeline: &mut JsonValue, kind: &str, ids: &[String], note: &str) {
    set_path(pipeline, "focus.kind", json!(kind));
    set_path(pipeline, "focus.ids", json!(ids));
    set_path(pipeline, "focus.note", json!(note));
}

fn stored_mode_for(context: &JsonValue, skill: &str, stage: &str) -> Option<String> {
    let invocation = nested_get(context, "invocation")?;
    if string_at(invocation, "skill").as_deref() == Some(skill)
        || string_at(invocation, "stage").as_deref() == Some(stage)
    {
        string_at(invocation, "mode")
    } else {
        None
    }
}

fn determine_stage(stage_override: Option<&str>, context: &JsonValue) -> String {
    stage_override
        .and_then(non_empty_string)
        .or_else(|| string_at(context, "pipeline.phase.current"))
        .unwrap_or_else(|| ROUTER_STAGE.to_string())
}

fn resolve_contract_value(
    param: &JsonValue,
    context: &JsonValue,
) -> (Option<JsonValue>, Option<String>) {
    for candidate in array_at(param, "resolve") {
        if let Some(path) = candidate.as_str() {
            if let Some(value) = nested_get(context, path) {
                if non_empty(Some(value)) {
                    return (Some(value.clone()), Some(path.to_string()));
                }
            }
        }
    }
    (None, None)
}

fn choice_options_for(choice: &JsonValue, context: &JsonValue) -> Vec<JsonValue> {
    if let Some(options) = choice.get("options").and_then(JsonValue::as_array) {
        return options.clone();
    }
    match choice
        .get("options_from")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
    {
        "registry.specs" => array_at(context, "registry.specs")
            .into_iter()
            .filter_map(|item| {
                let spec_id = item.get("spec_id").and_then(JsonValue::as_str)?;
                Some(json!({
                    "id": spec_id,
                    "label": item.get("title").and_then(JsonValue::as_str).unwrap_or(spec_id),
                    "description": item.get("summary").and_then(JsonValue::as_str).unwrap_or(""),
                }))
            })
            .collect(),
        "pipeline.focus.ids" => array_at(context, "pipeline.focus.ids")
            .into_iter()
            .filter_map(|item| {
                item.as_str().map(|value| {
                    json!({
                        "id": value,
                        "label": value,
                        "description": "",
                    })
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn choice_should_be_available(choice: &JsonValue, context: &JsonValue) -> bool {
    let show_when = choice
        .get("show_when")
        .and_then(JsonValue::as_str)
        .unwrap_or("always");
    let current_mode = string_at(context, "ux_mode").unwrap_or_default();
    if let Some(modes) = choice.get("show_in_modes").and_then(JsonValue::as_array) {
        if !modes
            .iter()
            .filter_map(JsonValue::as_str)
            .any(|mode| mode == current_mode)
        {
            return false;
        }
    }
    let specs = array_at(context, "registry.specs");
    let spec_id_source = string_at(context, "workspace.spec_id_source").unwrap_or_default();
    let spec_id = string_at(context, "workspace.spec_id").unwrap_or_default();
    let focus_ids = array_at(context, "pipeline.focus.ids");
    match show_when {
        "always" => true,
        "multiple_existing_specs_and_spec_not_explicit" => {
            specs.len() > 1
                && spec_id_source != "explicit_argument"
                && spec_id_source != "explicit_path"
        }
        "multiple_existing_specs_and_spec_missing" => specs.len() > 1 && spec_id.is_empty(),
        "existing_specs_available" => !specs.is_empty(),
        "focus_ids_available" => !focus_ids.is_empty(),
        "multiple_focus_ids" => focus_ids.len() > 1,
        "single_focus_id" => focus_ids.len() == 1,
        "spec_exists" => bool_at(context, "workspace.spec_exists"),
        _ => true,
    }
}

fn skill_contract(skill: Option<&str>, stage: Option<&str>) -> Result<JsonValue, CliError> {
    let contracts = ux_contracts()?;
    let skills = contracts
        .get("skills")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| CliError::new("ux_contract_invalid", "UX contract is missing skills."))?;
    let selected_skill = skill
        .map(str::to_string)
        .unwrap_or_else(|| stage_to_skill(stage.unwrap_or(ROUTER_STAGE)).to_string());
    skills.get(&selected_skill).cloned().ok_or_else(|| {
        CliError::new(
            "ux_contract_missing",
            format!("No UX contract registered for skill {:?}.", selected_skill),
        )
    })
}

fn stage_to_skill(stage: &str) -> &'static str {
    match stage.trim().to_lowercase().as_str() {
        "intake" => "spec-forge-intake",
        "architecture" => "spec-forge-architecture",
        "journeys" => "spec-forge-journeys",
        "components" => "spec-forge-components",
        "readiness" => "spec-forge-readiness",
        "implement" => "spec-forge-implement",
        _ => "spec-forge",
    }
}

fn sync_registry_from_pipeline(
    collection: &Path,
    spec_id: &str,
    pipeline: &JsonValue,
) -> Result<(), CliError> {
    let mut registry = load_registry(collection)?;
    set_path(
        &mut registry,
        "collection.target_path",
        json!(
            collection
                .parent()
                .unwrap_or(collection)
                .to_string_lossy()
                .to_string()
        ),
    );
    set_path(&mut registry, "active_spec_id", json!(spec_id));
    let title = string_at(pipeline, "request.title").unwrap_or_default();
    let summary = string_at(pipeline, "request.summary").unwrap_or_default();
    let current_stage =
        string_at(pipeline, "phase.current").unwrap_or_else(|| "intake".to_string());
    let status = string_at(pipeline, "phase.status").unwrap_or_else(|| "active".to_string());
    upsert_registry_spec(
        &mut registry,
        spec_id,
        &title,
        &summary,
        &current_stage,
        &status,
    );
    save_registry(collection, &registry)
}

fn upsert_registry_spec(
    registry: &mut JsonValue,
    spec_id: &str,
    title: &str,
    summary: &str,
    current_stage: &str,
    status: &str,
) {
    let updated_at = now_utc();
    let specs = ensure_path_array(registry, "specs");
    if let Some(entry) = specs
        .iter_mut()
        .find(|item| item.get("spec_id").and_then(JsonValue::as_str) == Some(spec_id))
    {
        set_path(entry, "title", json!(title));
        set_path(entry, "summary", json!(summary));
        set_path(
            entry,
            "workspace_path",
            json!(format!("{}/{}", SPECS_DIRNAME, spec_id)),
        );
        set_path(entry, "current_stage", json!(current_stage));
        set_path(entry, "status", json!(status));
        set_path(entry, "updated_at", json!(updated_at.clone()));
        if string_at(entry, "created_at")
            .unwrap_or_default()
            .is_empty()
        {
            set_path(entry, "created_at", json!(updated_at));
        }
    } else {
        specs.push(json!({
            "spec_id": spec_id,
            "title": title,
            "summary": summary,
            "workspace_path": format!("{}/{}", SPECS_DIRNAME, spec_id),
            "current_stage": current_stage,
            "status": status,
            "created_at": updated_at,
            "updated_at": updated_at,
        }));
    }
}

fn ensure_collection_dirs(collection: &Path) -> Result<(), CliError> {
    fs::create_dir_all(collection).map_err(|err| {
        CliError::new(
            "io_error",
            format!(
                "Failed to create directory {}: {}",
                collection.display(),
                err
            ),
        )
    })?;
    let specs = collection.join(SPECS_DIRNAME);
    fs::create_dir_all(&specs).map_err(|err| {
        CliError::new(
            "io_error",
            format!("Failed to create directory {}: {}", specs.display(), err),
        )
    })?;
    Ok(())
}

fn ensure_workspace_dirs(workspace: &Path) -> Result<(), CliError> {
    let mut paths = vec![workspace.to_path_buf()];
    for rel in [
        "framing",
        "architecture",
        "journeys",
        "components",
        "synthesis",
        "gates",
    ] {
        paths.push(workspace.join(rel));
    }
    for path in paths {
        fs::create_dir_all(&path).map_err(|err| {
            CliError::new(
                "io_error",
                format!("Failed to create directory {}: {}", path.display(), err),
            )
        })?;
    }
    Ok(())
}

fn normalize_target_dir(path_like: &str) -> Result<PathBuf, CliError> {
    let raw = absolute_path(path_like)?;
    let collection = collection_root(path_like)?;
    if file_name_is(&raw, WORKSPACE_DIRNAME) {
        return Ok(raw.parent().unwrap_or(&raw).to_path_buf());
    }
    if raw
        .parent()
        .is_some_and(|parent| file_name_is(parent, SPECS_DIRNAME))
        && raw
            .parent()
            .and_then(Path::parent)
            .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
    {
        return Ok(raw
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .unwrap_or(&raw)
            .to_path_buf());
    }
    if file_name_is(&raw, SPECS_DIRNAME)
        && raw
            .parent()
            .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
    {
        return Ok(raw
            .parent()
            .and_then(Path::parent)
            .unwrap_or(&raw)
            .to_path_buf());
    }
    if raw
        .parent()
        .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
    {
        return Ok(raw
            .parent()
            .and_then(Path::parent)
            .unwrap_or(&raw)
            .to_path_buf());
    }
    if collection.exists() && collection == raw {
        return Ok(raw.parent().unwrap_or(&raw).to_path_buf());
    }
    Ok(raw)
}

fn collection_root(path_like: &str) -> Result<PathBuf, CliError> {
    let raw = absolute_path(path_like)?;
    if raw
        .parent()
        .is_some_and(|parent| file_name_is(parent, SPECS_DIRNAME))
        && raw
            .parent()
            .and_then(Path::parent)
            .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
    {
        return Ok(raw.parent().and_then(Path::parent).unwrap().to_path_buf());
    }
    if file_name_is(&raw, SPECS_DIRNAME)
        && raw
            .parent()
            .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
    {
        return Ok(raw.parent().unwrap().to_path_buf());
    }
    if file_name_is(&raw, WORKSPACE_DIRNAME) {
        return Ok(raw);
    }
    Ok(raw.join(WORKSPACE_DIRNAME))
}

fn infer_spec_id(path_like: &str) -> Result<Option<String>, CliError> {
    let raw = absolute_path(path_like)?;
    Ok(
        if raw
            .parent()
            .is_some_and(|parent| file_name_is(parent, SPECS_DIRNAME))
            && raw
                .parent()
                .and_then(Path::parent)
                .is_some_and(|parent| file_name_is(parent, WORKSPACE_DIRNAME))
        {
            raw.file_name()
                .map(|name| name.to_string_lossy().to_string())
        } else {
            None
        },
    )
}

fn spec_workspace(collection: &Path, spec_id: &str) -> PathBuf {
    collection.join(SPECS_DIRNAME).join(spec_id)
}

fn slugify_spec_id(value: &str) -> String {
    let regex = Regex::new(r"[^a-zA-Z0-9]+").expect("regex");
    let lowered = value.trim().to_lowercase();
    let collapsed = regex.replace_all(lowered.as_str(), "-");
    let trimmed = collapsed.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "spec".to_string()
    } else {
        trimmed.chars().take(64).collect()
    }
}

fn allocate_spec_id(collection: &Path, requested: &str) -> Result<String, CliError> {
    let base = slugify_spec_id(requested);
    let mut candidate = base.clone();
    let mut counter = 2;
    while spec_workspace(collection, &candidate).exists() {
        candidate = format!("{}-{}", base, counter);
        counter += 1;
    }
    Ok(candidate)
}

fn approval_is_confirmed(data: &JsonValue) -> bool {
    data.get("approval")
        .and_then(|value| value.get("status"))
        .and_then(JsonValue::as_str)
        == Some("approved")
        && data
            .get("approval")
            .and_then(|value| value.get("confirmed_by_user"))
            .and_then(JsonValue::as_bool)
            == Some(true)
}

fn item_status(item: &JsonValue) -> String {
    item.get("review_status")
        .or_else(|| item.get("status"))
        .and_then(JsonValue::as_str)
        .unwrap_or("pending")
        .to_lowercase()
}

fn item_is_deferred(item: &JsonValue) -> bool {
    matches!(
        item_status(item).as_str(),
        "deferred" | "out_of_scope" | "cancelled"
    )
}

fn non_empty(value: Option<&JsonValue>) -> bool {
    match value {
        None | Some(JsonValue::Null) => false,
        Some(JsonValue::String(text)) => !text.trim().is_empty(),
        Some(JsonValue::Array(items)) => !items.is_empty(),
        Some(JsonValue::Object(map)) => !map.is_empty(),
        Some(_) => true,
    }
}

fn empty_object() -> JsonValue {
    JsonValue::Object(Map::new())
}

fn set_path(value: &mut JsonValue, dotted_path: &str, replacement: JsonValue) {
    let mut current = value;
    let parts = dotted_path.split('.').collect::<Vec<_>>();
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        let object = ensure_object(current);
        current = object
            .entry((*part).to_string())
            .or_insert_with(empty_object);
    }
    ensure_object(current).insert(parts.last().unwrap_or(&"").to_string(), replacement);
}

fn remove_path(value: &mut JsonValue, dotted_path: &str) {
    let parts = dotted_path.split('.').collect::<Vec<_>>();
    let mut current = value;
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        let Some(next) = current.get_mut(*part) else {
            return;
        };
        current = next;
    }
    if let Some(object) = current.as_object_mut() {
        if let Some(last) = parts.last() {
            object.remove(*last);
        }
    }
}

fn ensure_object(value: &mut JsonValue) -> &mut Map<String, JsonValue> {
    if !value.is_object() {
        *value = empty_object();
    }
    value.as_object_mut().expect("object")
}

fn ensure_path_array<'a>(value: &'a mut JsonValue, dotted_path: &str) -> &'a mut Vec<JsonValue> {
    let mut current = value;
    let parts = dotted_path.split('.').collect::<Vec<_>>();
    for part in parts.iter().take(parts.len().saturating_sub(1)) {
        let object = ensure_object(current);
        current = object
            .entry((*part).to_string())
            .or_insert_with(empty_object);
    }
    let object = ensure_object(current);
    let entry = object
        .entry(parts.last().unwrap_or(&"").to_string())
        .or_insert_with(|| JsonValue::Array(vec![]));
    if !entry.is_array() {
        *entry = JsonValue::Array(vec![]);
    }
    entry.as_array_mut().expect("array")
}

fn nested_get<'a>(value: &'a JsonValue, dotted_path: &str) -> Option<&'a JsonValue> {
    if dotted_path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    for part in dotted_path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn string_at(value: &JsonValue, dotted_path: &str) -> Option<String> {
    nested_get(value, dotted_path)?.as_str().map(str::to_string)
}

fn bool_at(value: &JsonValue, dotted_path: &str) -> bool {
    nested_get(value, dotted_path)
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
}

fn array_at(value: &JsonValue, dotted_path: &str) -> Vec<JsonValue> {
    if dotted_path.is_empty() {
        return value.as_array().cloned().unwrap_or_default();
    }
    nested_get(value, dotted_path)
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
}

fn ids_from_array(value: &JsonValue) -> Vec<String> {
    value
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            item.get("id")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .collect()
}

fn string_list(value: &JsonValue) -> Vec<String> {
    match value {
        JsonValue::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| item.to_string())
            })
            .collect(),
        _ => vec![
            value
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| value.to_string()),
        ],
    }
}

fn json_get_string(value: &JsonValue, dotted_path: &str) -> Option<String> {
    string_at(value, dotted_path).filter(|text| !text.trim().is_empty())
}

fn file_name_is(path: &Path, expected: &str) -> bool {
    path.file_name()
        .map(|name| name == expected)
        .unwrap_or(false)
}

fn absolute_path(path_like: &str) -> Result<PathBuf, CliError> {
    let path = Path::new(path_like);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io_error)?.join(path)
    };
    Ok(normalize_path(&joined))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn now_utc() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn io_error(err: std::io::Error) -> CliError {
    CliError::new("io_error", err.to_string())
}
