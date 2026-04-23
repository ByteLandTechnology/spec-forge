use serde_json::{Value as JsonValue, json};

#[derive(Clone, Copy)]
struct OptionDoc {
    name: &'static str,
    kind: &'static str,
    required: bool,
    default: &'static str,
    description: &'static str,
    values: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct ExampleDoc {
    command: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy)]
struct HelpDoc {
    name: &'static str,
    summary: &'static str,
    description: &'static str,
    synopsis: &'static [&'static str],
    options: &'static [OptionDoc],
    examples: &'static [ExampleDoc],
}

const EXIT_CODES: [(&str, &str); 2] = [
    ("0", "Successful execution or human-readable help output."),
    ("1", "Structured workflow, validation, or input error."),
];

const FORMATS: [&str; 3] = ["yaml (default)", "json", "toml"];

const RUNTIME_DIRECTORIES: [(&str, &str, &str); 4] = [
    (
        "config",
        "Package-local checked-in assets under assets/; no global user config file is read.",
        "Override behavior per invocation with --target, --skill, --stage, --mode, and --runtime-mode.",
    ),
    (
        "data",
        "<target>/.spec-forge/specs/<spec-id>/",
        "Spec artifacts are explicit project data selected by --target and --spec-id.",
    ),
    (
        "state",
        "<target>/.spec-forge/registry.yaml and <target>/.spec-forge/invocation-state.yaml",
        "Use resolve --write or apply to persist workflow state intentionally.",
    ),
    (
        "cache",
        "No persistent CLI cache; build and release caches stay in target/ or release-script temporary directories.",
        "Delete target/ or rerun commands to rebuild derived artifacts.",
    ),
];

const ACTIVE_CONTEXT: [(&str, &str); 4] = [
    (
        "inspect",
        "Run resolve or help with a structured format; outputs include workspace, spec_id, spec_id_source, skill, stage, and runtime_mode.",
    ),
    (
        "switch",
        "Use --spec-id for a per-invocation override, or apply a spec choice / init a workspace to persist registry.active_spec_id.",
    ),
    (
        "precedence",
        "Explicit --spec-id and command flags win for the current invocation and do not mutate persisted defaults unless --write or apply is used.",
    ),
    (
        "visibility",
        "Effective context remains visible in structured payload fields such as workspace.spec_id, workspace.spec_id_source, skill, stage, and runtime_mode.",
    ),
];

const HELP_ONLY_OPTION: [OptionDoc; 1] = [OptionDoc {
    name: "--help",
    kind: "bool",
    required: false,
    default: "false",
    description: "Render human-readable help for the current command path.",
    values: &[],
}];

const TARGET_OPTION: OptionDoc = OptionDoc {
    name: "--target",
    kind: "path",
    required: false,
    default: ".",
    description: "Project directory, .spec-forge collection root, or spec workspace path.",
    values: &[],
};

const SPEC_ID_OPTION: OptionDoc = OptionDoc {
    name: "--spec-id",
    kind: "string",
    required: false,
    default: "",
    description: "Optional explicit spec identifier when the target path is not already specific.",
    values: &[],
};

const FORMAT_OPTION: OptionDoc = OptionDoc {
    name: "--format",
    kind: "enum",
    required: false,
    default: "yaml",
    description: "Output format for structured output.",
    values: &["yaml", "json", "toml"],
};

const FILE_OPTION: OptionDoc = OptionDoc {
    name: "--file",
    kind: "path",
    required: true,
    default: "",
    description: "Relative artifact path under the resolved spec workspace, for example `framing/request-context.yaml`.",
    values: &[],
};

const VALUE_OPTION: OptionDoc = OptionDoc {
    name: "--value",
    kind: "string",
    required: true,
    default: "",
    description: "YAML value or object patch supplied inline as one argument.",
    values: &[],
};

const INIT_OPTIONS: [OptionDoc; 6] = [
    TARGET_OPTION,
    OptionDoc {
        name: "--spec-id",
        kind: "string",
        required: false,
        default: "",
        description: "Optional stable spec identifier. When omitted, the CLI derives one from the request title.",
        values: &[],
    },
    OptionDoc {
        name: "--request-title",
        kind: "string",
        required: false,
        default: "",
        description: "Short request title used to seed the workspace.",
        values: &[],
    },
    OptionDoc {
        name: "--request-summary",
        kind: "string",
        required: false,
        default: "",
        description: "Short request summary stored with the workspace.",
        values: &[],
    },
    OptionDoc {
        name: "--force",
        kind: "bool",
        required: false,
        default: "false",
        description: "Allow initialization into an existing workspace directory.",
        values: &[],
    },
    FORMAT_OPTION,
];

const RESOLVE_OPTIONS: [OptionDoc; 8] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    OptionDoc {
        name: "--skill",
        kind: "string",
        required: false,
        default: "",
        description: "Optional skill override such as spec-forge-intake.",
        values: &[],
    },
    OptionDoc {
        name: "--stage",
        kind: "enum",
        required: false,
        default: "",
        description: "Optional stage override used to resolve the next interaction.",
        values: &[
            "router",
            "intake",
            "architecture",
            "journeys",
            "components",
            "readiness",
            "implement",
        ],
    },
    OptionDoc {
        name: "--mode",
        kind: "string",
        required: false,
        default: "",
        description: "Optional invocation mode override from the UX contract.",
        values: &[],
    },
    OptionDoc {
        name: "--runtime-mode",
        kind: "enum",
        required: false,
        default: "default",
        description: "Current collaboration mode used when computing interactive requirements.",
        values: &["default", "plan"],
    },
    OptionDoc {
        name: "--write",
        kind: "bool",
        required: false,
        default: "false",
        description: "Persist the resolved UX payload into invocation or workspace YAML state.",
        values: &[],
    },
    FORMAT_OPTION,
];

const APPLY_OPTIONS: [OptionDoc; 10] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    OptionDoc {
        name: "--skill",
        kind: "string",
        required: false,
        default: "",
        description: "Optional skill override such as spec-forge-intake.",
        values: &[],
    },
    OptionDoc {
        name: "--stage",
        kind: "enum",
        required: false,
        default: "",
        description: "Optional stage override for the interaction being applied.",
        values: &[
            "router",
            "intake",
            "architecture",
            "journeys",
            "components",
            "readiness",
            "implement",
        ],
    },
    OptionDoc {
        name: "--mode",
        kind: "string",
        required: false,
        default: "",
        description: "Optional invocation mode override from the UX contract.",
        values: &[],
    },
    OptionDoc {
        name: "--runtime-mode",
        kind: "enum",
        required: false,
        default: "plan",
        description: "Current collaboration mode used when resolving the follow-up interaction.",
        values: &["default", "plan"],
    },
    OptionDoc {
        name: "--parameter",
        kind: "string",
        required: false,
        default: "",
        description: "Parameter identifier to answer. Exactly one of --parameter or --choice must be supplied.",
        values: &[],
    },
    OptionDoc {
        name: "--choice",
        kind: "string",
        required: false,
        default: "",
        description: "Choice identifier to answer. Exactly one of --parameter or --choice must be supplied.",
        values: &[],
    },
    OptionDoc {
        name: "--value",
        kind: "string",
        required: false,
        default: "",
        description: "Repeatable YAML-typed answer value. At least one value is required.",
        values: &[],
    },
    FORMAT_OPTION,
];

const FOCUS_OPTIONS: [OptionDoc; 6] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    OptionDoc {
        name: "--stage",
        kind: "enum",
        required: true,
        default: "",
        description: "Which review stage should select its next batch.",
        values: &["journeys", "components"],
    },
    OptionDoc {
        name: "--max-items",
        kind: "int",
        required: false,
        default: "1",
        description: "Maximum number of focus ids to return in the selected batch.",
        values: &[],
    },
    OptionDoc {
        name: "--write",
        kind: "bool",
        required: false,
        default: "false",
        description: "Persist the selected focus into pipeline and handoff YAML files.",
        values: &[],
    },
    FORMAT_OPTION,
];

const ARTIFACT_GET_OPTIONS: [OptionDoc; 4] =
    [TARGET_OPTION, SPEC_ID_OPTION, FILE_OPTION, FORMAT_OPTION];

const ARTIFACT_PUT_OPTIONS: [OptionDoc; 5] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    FILE_OPTION,
    VALUE_OPTION,
    FORMAT_OPTION,
];

const ARTIFACT_MERGE_OPTIONS: [OptionDoc; 5] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    FILE_OPTION,
    VALUE_OPTION,
    FORMAT_OPTION,
];

const APPROVE_OPTIONS: [OptionDoc; 5] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    FILE_OPTION,
    OptionDoc {
        name: "--note",
        kind: "string",
        required: false,
        default: "",
        description: "Optional approval note stored in approval.confirmation_note.",
        values: &[],
    },
    FORMAT_OPTION,
];

const GATE_CHECK_OPTIONS: [OptionDoc; 5] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    OptionDoc {
        name: "--stage",
        kind: "enum",
        required: true,
        default: "",
        description: "Which pipeline stage gate to evaluate.",
        values: &[
            "intake",
            "architecture",
            "journeys",
            "components",
            "readiness",
            "implement",
        ],
    },
    OptionDoc {
        name: "--write",
        kind: "bool",
        required: false,
        default: "false",
        description: "Persist the computed gate report to gates/<stage>.yaml.",
        values: &[],
    },
    FORMAT_OPTION,
];

const STAGE_ADVANCE_OPTIONS: [OptionDoc; 4] = [
    TARGET_OPTION,
    SPEC_ID_OPTION,
    OptionDoc {
        name: "--stage",
        kind: "enum",
        required: true,
        default: "",
        description: "Current stage to advance from after gate validation succeeds.",
        values: &[
            "intake",
            "architecture",
            "journeys",
            "components",
            "readiness",
            "implement",
        ],
    },
    FORMAT_OPTION,
];

const UX_VALIDATE_OPTIONS: [OptionDoc; 2] = [TARGET_OPTION, FORMAT_OPTION];

const HELP_OPTIONS: [OptionDoc; 2] = [
    OptionDoc {
        name: "COMMAND_PATH...",
        kind: "string",
        required: false,
        default: "",
        description: "Optional command path to describe, for example `gate check`.",
        values: &[],
    },
    FORMAT_OPTION,
];

const TOP_EXAMPLES: [ExampleDoc; 4] = [
    ExampleDoc {
        command: "spec-forge-cli init --target . --request-title \"Add implement stage\"",
        description: "Initialize a new .spec-forge workspace.",
    },
    ExampleDoc {
        command: "spec-forge-cli resolve --target . --skill spec-forge --stage router --write",
        description: "Resolve the next interaction for the router stage.",
    },
    ExampleDoc {
        command: "spec-forge-cli gate check --target . --spec-id demo --stage intake --write",
        description: "Evaluate a stage gate and optionally persist the report.",
    },
    ExampleDoc {
        command: "spec-forge-cli help gate check --format json",
        description: "Return structured help for a nested command path.",
    },
];

const INIT_EXAMPLES: [ExampleDoc; 2] = [
    ExampleDoc {
        command: "spec-forge-cli init --target . --request-title \"Spec execution stage\"",
        description: "Create a new workspace using the request title as the default slug source.",
    },
    ExampleDoc {
        command: "spec-forge-cli init --target . --spec-id spec-exec --request-title \"Spec execution stage\" --force",
        description: "Reinitialize a specific workspace path when it already exists.",
    },
];

const RESOLVE_EXAMPLES: [ExampleDoc; 2] = [
    ExampleDoc {
        command: "spec-forge-cli resolve --target . --skill spec-forge --stage router --write",
        description: "Resolve the router stage for the current project.",
    },
    ExampleDoc {
        command: "spec-forge-cli resolve --target . --spec-id demo --skill spec-forge-intake --stage intake --runtime-mode plan",
        description: "Resolve intake-stage interaction requirements for one spec.",
    },
];

const APPLY_EXAMPLES: [ExampleDoc; 2] = [
    ExampleDoc {
        command: "spec-forge-cli apply --target . --stage intake --parameter problem_goal --value \"Ship the implement stage\"",
        description: "Apply one parameter answer to workspace state.",
    },
    ExampleDoc {
        command: "spec-forge-cli apply --target . --stage intake --choice active_roles --value product-manager --value tech-lead",
        description: "Apply a multi-select choice response.",
    },
];

const FOCUS_EXAMPLES: [ExampleDoc; 2] = [
    ExampleDoc {
        command: "spec-forge-cli focus --target . --spec-id demo --stage journeys --write",
        description: "Choose the next journey batch and persist the selected focus.",
    },
    ExampleDoc {
        command: "spec-forge-cli focus --target . --spec-id demo --stage components --max-items 2",
        description: "Preview a component batch without mutating workspace state.",
    },
];

const ARTIFACT_GET_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli artifact get --target . --spec-id demo --file framing/request-context.yaml",
    description: "Read one workspace artifact through the CLI instead of opening the YAML file directly.",
}];

const ARTIFACT_PUT_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli artifact put --target . --spec-id demo --file journeys/journey-alpha.yaml --value '{approval:{status: approved, confirmed_by_user: true}}'",
    description: "Replace or create a workspace artifact from one inline YAML value.",
}];

const ARTIFACT_MERGE_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli artifact merge --target . --spec-id demo --file framing/request-context.yaml --value '{problem:{goal:\"Ship the native CLI\"}}'",
    description: "Deep-merge an inline YAML patch into an existing artifact file.",
}];

const APPROVE_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli approve --target . --spec-id demo --file architecture/solution-outline.yaml --note 'Approved after review.'",
    description: "Mark an artifact approved without editing the YAML by hand.",
}];

const GATE_CHECK_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli gate check --target . --spec-id demo --stage readiness --write",
    description: "Evaluate one workflow stage gate.",
}];

const STAGE_ADVANCE_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli stage advance --target . --spec-id demo --stage intake",
    description: "Advance a stage after the current gate passes.",
}];

const UX_VALIDATE_EXAMPLES: [ExampleDoc; 1] = [ExampleDoc {
    command: "spec-forge-cli ux validate --target .",
    description: "Validate the shared and per-stage UX contracts in the current repo.",
}];

const HELP_EXAMPLES: [ExampleDoc; 2] = [
    ExampleDoc {
        command: "spec-forge-cli help",
        description: "Return structured help for the top-level command.",
    },
    ExampleDoc {
        command: "spec-forge-cli help stage advance --format json",
        description: "Return structured help for a nested command path.",
    },
];

pub fn human_help(path: &[String]) -> Option<String> {
    let doc = find_doc(path)?;
    Some(render_human_help(doc))
}

pub fn structured_help(path: &[String]) -> Option<JsonValue> {
    let doc = find_doc(path)?;
    Some(json!({
        "command_path": path,
        "name": doc.name,
        "summary": doc.summary,
        "description": doc.description,
        "synopsis": doc.synopsis,
        "options": doc.options.iter().map(|option| {
            json!({
                "name": option.name,
                "type": option.kind,
                "required": option.required,
                "default": option.default,
                "description": option.description,
                "values": option.values,
            })
        }).collect::<Vec<_>>(),
        "formats": FORMATS,
        "runtime_directories": RUNTIME_DIRECTORIES.iter().map(|(kind, default, notes)| {
            json!({
                "kind": kind,
                "default": default,
                "notes": notes,
            })
        }).collect::<Vec<_>>(),
        "active_context": ACTIVE_CONTEXT.iter().map(|(capability, description)| {
            json!({
                "capability": capability,
                "description": description,
            })
        }).collect::<Vec<_>>(),
        "examples": doc.examples.iter().map(|example| {
            json!({
                "command": example.command,
                "description": example.description,
            })
        }).collect::<Vec<_>>(),
        "exit_codes": EXIT_CODES.iter().map(|(code, meaning)| {
            json!({
                "code": code,
                "meaning": meaning,
            })
        }).collect::<Vec<_>>(),
    }))
}

pub fn help_path_from_flag(argv: &[String]) -> Option<Vec<String>> {
    let help_index = argv.iter().position(|arg| arg == "--help")?;
    let mut path = Vec::new();
    for token in argv.iter().skip(1).take(help_index.saturating_sub(1)) {
        if token.starts_with('-') {
            break;
        }
        path.push(token.clone());
    }
    Some(path)
}

fn render_human_help(doc: HelpDoc) -> String {
    let mut output = String::new();
    output.push_str("NAME\n");
    output.push_str(&format!("    {} - {}\n\n", doc.name, doc.summary));

    output.push_str("SYNOPSIS\n");
    for synopsis in doc.synopsis {
        output.push_str(&format!("    {}\n", synopsis));
    }
    output.push('\n');

    output.push_str("DESCRIPTION\n");
    output.push_str(&format!("    {}\n\n", doc.description));

    output.push_str("OPTIONS\n");
    for option in doc.options {
        output.push_str(&format!(
            "    {} [{}{}] {}\n",
            option.name,
            option.kind,
            if option.required { ", required" } else { "" },
            option.description
        ));
        if !option.default.is_empty() {
            output.push_str(&format!("        default: {}\n", option.default));
        }
        if !option.values.is_empty() {
            output.push_str(&format!("        values: {}\n", option.values.join(", ")));
        }
    }
    output.push('\n');

    output.push_str("FORMATS\n");
    for format in FORMATS {
        output.push_str(&format!("    {}\n", format));
    }
    output.push('\n');

    output.push_str("EXAMPLES\n");
    for example in doc.examples {
        output.push_str(&format!("    {}\n", example.command));
        output.push_str(&format!("        {}\n", example.description));
    }
    output.push('\n');

    output.push_str("EXIT CODES\n");
    for (code, meaning) in EXIT_CODES {
        output.push_str(&format!("    {}  {}\n", code, meaning));
    }
    output
}

fn find_doc(path: &[String]) -> Option<HelpDoc> {
    match path {
        [] => Some(HelpDoc {
            name: "spec-forge-cli",
            summary: "Native Rust CLI for the spec-forge YAML workflow.",
            description: "Use the top-level command as the stable front door for initializing workspaces, resolving interactions, applying answers, reading and updating artifacts, recording approvals, selecting focus batches, evaluating gates, advancing stages, validating UX contracts, and reading help for any command path.",
            synopsis: &[
                "spec-forge-cli <COMMAND> [OPTIONS]",
                "spec-forge-cli help [COMMAND_PATH ...] [--format yaml|json|toml]",
                "spec-forge-cli --help",
            ],
            options: &HELP_ONLY_OPTION,
            examples: &TOP_EXAMPLES,
        }),
        [command] if command == "init" => Some(HelpDoc {
            name: "spec-forge-cli init",
            summary: "Initialize a .spec-forge workspace and seed the per-spec YAML collection.",
            description: "Create the .spec-forge collection, write the per-spec YAML skeleton, initialize gate reports, and point the handoff toward intake.",
            synopsis: &[
                "spec-forge-cli init [--target PATH] [--spec-id ID] [--request-title TEXT] [--request-summary TEXT] [--force] [--format yaml|json|toml]",
            ],
            options: &INIT_OPTIONS,
            examples: &INIT_EXAMPLES,
        }),
        [command] if command == "resolve" => Some(HelpDoc {
            name: "spec-forge-cli resolve",
            summary: "Resolve the next workflow interaction from workspace state and the UX contracts.",
            description: "Inspect the current invocation and workspace state, resolve the active UX contract, and report the next required interaction or ready state.",
            synopsis: &[
                "spec-forge-cli resolve [--target PATH] [--spec-id ID] [--skill SKILL] [--stage STAGE] [--mode MODE] [--runtime-mode default|plan] [--write] [--format yaml|json|toml]",
            ],
            options: &RESOLVE_OPTIONS,
            examples: &RESOLVE_EXAMPLES,
        }),
        [command] if command == "apply" => Some(HelpDoc {
            name: "spec-forge-cli apply",
            summary: "Apply one parameter answer or choice selection and resolve the next interaction.",
            description: "Write one user answer into invocation or workspace YAML state, then immediately recompute the next interaction payload so the workflow remains state-driven instead of chat-memory-driven.",
            synopsis: &[
                "spec-forge-cli apply [--target PATH] [--spec-id ID] [--skill SKILL] [--stage STAGE] [--mode MODE] [--runtime-mode default|plan] (--parameter ID | --choice ID) --value YAML [--value YAML ...] [--format yaml|json|toml]",
            ],
            options: &APPLY_OPTIONS,
            examples: &APPLY_EXAMPLES,
        }),
        [command] if command == "focus" => Some(HelpDoc {
            name: "spec-forge-cli focus",
            summary: "Select the next journey or component batch to review from the current pipeline indexes.",
            description: "Choose the next review-sized batch for journeys or components, either from declared batches or from the next pending index items.",
            synopsis: &[
                "spec-forge-cli focus [--target PATH] [--spec-id ID] --stage journeys|components [--max-items N] [--write] [--format yaml|json|toml]",
            ],
            options: &FOCUS_OPTIONS,
            examples: &FOCUS_EXAMPLES,
        }),
        [command] if command == "artifact" => Some(HelpDoc {
            name: "spec-forge-cli artifact",
            summary: "Non-leaf command group for direct workspace-artifact inspection and mutation.",
            description: "Use `artifact get`, `artifact put`, or `artifact merge` when the workflow needs direct YAML artifact access through the CLI instead of manual file editing.",
            synopsis: &[
                "spec-forge-cli artifact get [OPTIONS]",
                "spec-forge-cli artifact put [OPTIONS]",
                "spec-forge-cli artifact merge [OPTIONS]",
                "spec-forge-cli artifact --help",
            ],
            options: &HELP_ONLY_OPTION,
            examples: &ARTIFACT_GET_EXAMPLES,
        }),
        [first, second] if first == "artifact" && second == "get" => Some(HelpDoc {
            name: "spec-forge-cli artifact get",
            summary: "Read one artifact from the resolved spec workspace.",
            description: "Resolve the target spec workspace, load one relative artifact path, and emit the parsed YAML value in the selected output format.",
            synopsis: &[
                "spec-forge-cli artifact get [--target PATH] [--spec-id ID] --file RELATIVE_PATH [--format yaml|json|toml]",
            ],
            options: &ARTIFACT_GET_OPTIONS,
            examples: &ARTIFACT_GET_EXAMPLES,
        }),
        [first, second] if first == "artifact" && second == "put" => Some(HelpDoc {
            name: "spec-forge-cli artifact put",
            summary: "Replace or create one artifact in the resolved spec workspace.",
            description: "Resolve the target spec workspace, parse one inline YAML value, and write it as the complete contents of the selected artifact file.",
            synopsis: &[
                "spec-forge-cli artifact put [--target PATH] [--spec-id ID] --file RELATIVE_PATH --value YAML [--format yaml|json|toml]",
            ],
            options: &ARTIFACT_PUT_OPTIONS,
            examples: &ARTIFACT_PUT_EXAMPLES,
        }),
        [first, second] if first == "artifact" && second == "merge" => Some(HelpDoc {
            name: "spec-forge-cli artifact merge",
            summary: "Deep-merge an inline YAML patch into one artifact file.",
            description: "Resolve the target spec workspace, parse one inline YAML object patch, and merge it into the selected artifact. Missing files start from an empty object.",
            synopsis: &[
                "spec-forge-cli artifact merge [--target PATH] [--spec-id ID] --file RELATIVE_PATH --value YAML [--format yaml|json|toml]",
            ],
            options: &ARTIFACT_MERGE_OPTIONS,
            examples: &ARTIFACT_MERGE_EXAMPLES,
        }),
        [command] if command == "approve" => Some(HelpDoc {
            name: "spec-forge-cli approve",
            summary: "Mark one artifact approved through a standard approval block.",
            description: "Resolve the target spec workspace, load or create the selected artifact file, and write approval.status=approved plus the confirmation metadata expected by the stage gates.",
            synopsis: &[
                "spec-forge-cli approve [--target PATH] [--spec-id ID] --file RELATIVE_PATH [--note TEXT] [--format yaml|json|toml]",
            ],
            options: &APPROVE_OPTIONS,
            examples: &APPROVE_EXAMPLES,
        }),
        [command] if command == "gate" => Some(HelpDoc {
            name: "spec-forge-cli gate",
            summary: "Non-leaf command group for stage-gate operations.",
            description: "Use `gate check` to evaluate a workflow stage gate and optionally persist the report.",
            synopsis: &[
                "spec-forge-cli gate check [OPTIONS]",
                "spec-forge-cli gate --help",
            ],
            options: &HELP_ONLY_OPTION,
            examples: &GATE_CHECK_EXAMPLES,
        }),
        [first, second] if first == "gate" && second == "check" => Some(HelpDoc {
            name: "spec-forge-cli gate check",
            summary: "Evaluate a stage gate and optionally write the gate report into the workspace.",
            description: "Load the current workspace artifacts, evaluate the selected stage gate, and emit the report in the chosen output format.",
            synopsis: &[
                "spec-forge-cli gate check [--target PATH] [--spec-id ID] --stage intake|architecture|journeys|components|readiness|implement [--write] [--format yaml|json|toml]",
            ],
            options: &GATE_CHECK_OPTIONS,
            examples: &GATE_CHECK_EXAMPLES,
        }),
        [command] if command == "stage" => Some(HelpDoc {
            name: "spec-forge-cli stage",
            summary: "Non-leaf command group for workflow stage transitions.",
            description: "Use `stage advance` to persist the current gate report and promote the pipeline to the next valid stage.",
            synopsis: &[
                "spec-forge-cli stage advance [OPTIONS]",
                "spec-forge-cli stage --help",
            ],
            options: &HELP_ONLY_OPTION,
            examples: &STAGE_ADVANCE_EXAMPLES,
        }),
        [first, second] if first == "stage" && second == "advance" => Some(HelpDoc {
            name: "spec-forge-cli stage advance",
            summary: "Advance the workflow to the next stage after confirming that the current stage gate passes.",
            description: "Check the current gate, update pipeline and handoff state, and mark the next stage ready when the current stage has passed.",
            synopsis: &[
                "spec-forge-cli stage advance [--target PATH] [--spec-id ID] --stage intake|architecture|journeys|components|readiness|implement [--format yaml|json|toml]",
            ],
            options: &STAGE_ADVANCE_OPTIONS,
            examples: &STAGE_ADVANCE_EXAMPLES,
        }),
        [command] if command == "ux" => Some(HelpDoc {
            name: "spec-forge-cli ux",
            summary: "Non-leaf command group for UX-contract tooling.",
            description: "Use `ux validate` to validate the shared and per-stage UX contracts used by the spec-forge workflow family.",
            synopsis: &[
                "spec-forge-cli ux validate [OPTIONS]",
                "spec-forge-cli ux --help",
            ],
            options: &HELP_ONLY_OPTION,
            examples: &UX_VALIDATE_EXAMPLES,
        }),
        [first, second] if first == "ux" && second == "validate" => Some(HelpDoc {
            name: "spec-forge-cli ux validate",
            summary: "Validate the shared and per-stage UX contracts that drive spec-forge interaction collection.",
            description: "Read the checked-in UX contract files from the target repo, verify required top-level keys, and emit any structural problems.",
            synopsis: &["spec-forge-cli ux validate [--target PATH] [--format yaml|json|toml]"],
            options: &UX_VALIDATE_OPTIONS,
            examples: &UX_VALIDATE_EXAMPLES,
        }),
        [command] if command == "help" => Some(HelpDoc {
            name: "spec-forge-cli help",
            summary: "Return structured help for the top level or a specific command path.",
            description: "Emit machine-readable help in yaml, json, or toml for the top-level command or for a specific nested command path.",
            synopsis: &["spec-forge-cli help [COMMAND_PATH ...] [--format yaml|json|toml]"],
            options: &HELP_OPTIONS,
            examples: &HELP_EXAMPLES,
        }),
        _ => None,
    }
}
