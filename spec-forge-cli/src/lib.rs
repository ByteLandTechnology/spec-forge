mod help;
mod output;
mod workflow;

use clap::{Args, Parser, Subcommand};
pub use output::{CliError, OutputFormat};
use output::{emit_stderr_error, emit_stdout};
use workflow::CommandOutcome;

#[derive(Parser, Debug)]
#[command(
    name = "spec-forge-cli",
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init(InitArgs),
    Resolve(ResolveArgs),
    Apply(ApplyArgs),
    Focus(FocusArgs),
    Artifact(ArtifactArgs),
    Approve(ApproveArgs),
    Gate(GateArgs),
    Stage(StageArgs),
    Ux(UxArgs),
    Help(HelpArgs),
}

#[derive(Args, Debug)]
struct InitArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, default_value = "")]
    request_title: String,
    #[arg(long, default_value = "")]
    request_summary: String,
    #[arg(long, default_value_t = false)]
    force: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum RuntimeMode {
    Default,
    Plan,
}

impl RuntimeMode {
    fn as_str(&self) -> &'static str {
        match self {
            RuntimeMode::Default => "default",
            RuntimeMode::Plan => "plan",
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum WorkflowStage {
    Router,
    Intake,
    Architecture,
    Journeys,
    Components,
    Readiness,
    Implement,
}

impl WorkflowStage {
    fn as_str(&self) -> &'static str {
        match self {
            WorkflowStage::Router => "router",
            WorkflowStage::Intake => "intake",
            WorkflowStage::Architecture => "architecture",
            WorkflowStage::Journeys => "journeys",
            WorkflowStage::Components => "components",
            WorkflowStage::Readiness => "readiness",
            WorkflowStage::Implement => "implement",
        }
    }
}

#[derive(Args, Debug)]
struct ResolveArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, default_value = "")]
    skill: String,
    #[arg(long)]
    stage: Option<WorkflowStage>,
    #[arg(long, default_value = "")]
    mode: String,
    #[arg(long, value_enum, default_value_t = RuntimeMode::Default)]
    runtime_mode: RuntimeMode,
    #[arg(long, default_value_t = false)]
    write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ApplyArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, default_value = "")]
    skill: String,
    #[arg(long)]
    stage: Option<WorkflowStage>,
    #[arg(long, default_value = "")]
    mode: String,
    #[arg(long, value_enum, default_value_t = RuntimeMode::Plan)]
    runtime_mode: RuntimeMode,
    #[arg(long, default_value = "")]
    parameter: String,
    #[arg(long, default_value = "")]
    choice: String,
    #[arg(long = "value", required = false)]
    value: Vec<String>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum FocusStage {
    Journeys,
    Components,
}

impl FocusStage {
    fn as_str(&self) -> &'static str {
        match self {
            FocusStage::Journeys => "journeys",
            FocusStage::Components => "components",
        }
    }
}

#[derive(Args, Debug)]
struct FocusArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, value_enum)]
    stage: FocusStage,
    #[arg(long, default_value_t = 1)]
    max_items: i64,
    #[arg(long, default_value_t = false)]
    write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ArtifactArgs {
    #[command(subcommand)]
    command: Option<ArtifactCommand>,
}

#[derive(Subcommand, Debug)]
enum ArtifactCommand {
    Get(ArtifactGetArgs),
    Put(ArtifactPutArgs),
    Merge(ArtifactMergeArgs),
}

#[derive(Args, Debug)]
struct ArtifactGetArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long)]
    file: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ArtifactPutArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long)]
    file: String,
    #[arg(long)]
    value: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ArtifactMergeArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long)]
    file: String,
    #[arg(long)]
    value: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct ApproveArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long)]
    file: String,
    #[arg(long, default_value = "")]
    note: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct GateArgs {
    #[command(subcommand)]
    command: Option<GateCommand>,
}

#[derive(Subcommand, Debug)]
enum GateCommand {
    Check(GateCheckArgs),
}

#[derive(Args, Debug)]
struct GateCheckArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, value_enum)]
    stage: GateStage,
    #[arg(long, default_value_t = false)]
    write: bool,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum GateStage {
    Intake,
    Architecture,
    Journeys,
    Components,
    Readiness,
    Implement,
}

impl GateStage {
    fn as_str(&self) -> &'static str {
        match self {
            GateStage::Intake => "intake",
            GateStage::Architecture => "architecture",
            GateStage::Journeys => "journeys",
            GateStage::Components => "components",
            GateStage::Readiness => "readiness",
            GateStage::Implement => "implement",
        }
    }
}

#[derive(Args, Debug)]
struct StageArgs {
    #[command(subcommand)]
    command: Option<StageCommand>,
}

#[derive(Subcommand, Debug)]
enum StageCommand {
    Advance(StageAdvanceArgs),
}

#[derive(Args, Debug)]
struct StageAdvanceArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, default_value = "")]
    spec_id: String,
    #[arg(long, value_enum)]
    stage: GateStage,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct UxArgs {
    #[command(subcommand)]
    command: Option<UxCommand>,
}

#[derive(Subcommand, Debug)]
enum UxCommand {
    Validate(UxValidateArgs),
}

#[derive(Args, Debug)]
struct UxValidateArgs {
    #[arg(long, default_value = ".")]
    target: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args, Debug)]
struct HelpArgs {
    #[arg()]
    command_path: Vec<String>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

pub fn run() -> i32 {
    let argv = std::env::args().collect::<Vec<_>>();
    let fallback_format = OutputFormat::from_argv(&argv);

    if let Some(path) = help::help_path_from_flag(&argv) {
        return match help::human_help(&path) {
            Some(rendered) => {
                print!("{}", rendered);
                if !rendered.ends_with('\n') {
                    println!();
                }
                0
            }
            None => {
                let err = CliError::new(
                    "help_path_unknown",
                    format!("Unknown help command path: {}", path.join(" ")),
                );
                let _ = emit_stderr_error(fallback_format, &err);
                err.exit_code
            }
        };
    }

    match Cli::try_parse_from(&argv) {
        Ok(cli) => dispatch(cli),
        Err(err) => {
            let cli_error = CliError::new("argument_parse_failed", err.to_string());
            let _ = emit_stderr_error(fallback_format, &cli_error);
            cli_error.exit_code
        }
    }
}

fn dispatch(cli: Cli) -> i32 {
    match cli.command {
        None => match help::human_help(&[]) {
            Some(rendered) => {
                print!("{}", rendered);
                if !rendered.ends_with('\n') {
                    println!();
                }
                0
            }
            None => 1,
        },
        Some(Command::Help(args)) => emit_result(
            args.format,
            match help::structured_help(&args.command_path) {
                Some(payload) => Ok(CommandOutcome {
                    payload,
                    exit_code: 0,
                }),
                None => Err(CliError::new(
                    "help_path_unknown",
                    format!("Unknown help command path: {}", args.command_path.join(" ")),
                )),
            },
        ),
        Some(Command::Init(args)) => emit_result(
            args.format,
            workflow::init_command(
                &args.target,
                opt(&args.spec_id),
                opt(&args.request_title),
                opt(&args.request_summary),
                args.force,
            ),
        ),
        Some(Command::Resolve(args)) => emit_result(
            args.format,
            workflow::resolve_command(
                &args.target,
                opt(&args.spec_id),
                opt(&args.skill),
                args.stage.as_ref().map(WorkflowStage::as_str),
                opt(&args.mode),
                args.runtime_mode.as_str(),
                args.write,
            ),
        ),
        Some(Command::Apply(args)) => emit_result(
            args.format,
            workflow::apply_command(
                &args.target,
                opt(&args.spec_id),
                opt(&args.skill),
                args.stage.as_ref().map(WorkflowStage::as_str),
                opt(&args.mode),
                args.runtime_mode.as_str(),
                opt(&args.parameter),
                opt(&args.choice),
                &args.value,
            ),
        ),
        Some(Command::Focus(args)) => emit_result(
            args.format,
            workflow::focus_command(
                &args.target,
                opt(&args.spec_id),
                args.stage.as_str(),
                args.max_items,
                args.write,
            ),
        ),
        Some(Command::Artifact(ArtifactArgs { command: None })) => {
            match help::human_help(&["artifact".to_string()]) {
                Some(rendered) => {
                    print!("{}", rendered);
                    if !rendered.ends_with('\n') {
                        println!();
                    }
                    0
                }
                None => 1,
            }
        }
        Some(Command::Artifact(ArtifactArgs {
            command: Some(ArtifactCommand::Get(args)),
        })) => emit_result(
            args.format,
            workflow::artifact_get_command(&args.target, opt(&args.spec_id), &args.file),
        ),
        Some(Command::Artifact(ArtifactArgs {
            command: Some(ArtifactCommand::Put(args)),
        })) => emit_result(
            args.format,
            workflow::artifact_put_command(
                &args.target,
                opt(&args.spec_id),
                &args.file,
                &args.value,
            ),
        ),
        Some(Command::Artifact(ArtifactArgs {
            command: Some(ArtifactCommand::Merge(args)),
        })) => emit_result(
            args.format,
            workflow::artifact_merge_command(
                &args.target,
                opt(&args.spec_id),
                &args.file,
                &args.value,
            ),
        ),
        Some(Command::Approve(args)) => emit_result(
            args.format,
            workflow::approve_command(
                &args.target,
                opt(&args.spec_id),
                &args.file,
                opt(&args.note),
            ),
        ),
        Some(Command::Gate(GateArgs { command: None })) => {
            match help::human_help(&["gate".to_string()]) {
                Some(rendered) => {
                    print!("{}", rendered);
                    if !rendered.ends_with('\n') {
                        println!();
                    }
                    0
                }
                None => 1,
            }
        }
        Some(Command::Gate(GateArgs {
            command: Some(GateCommand::Check(args)),
        })) => emit_result(
            args.format,
            workflow::gate_check_command(
                &args.target,
                opt(&args.spec_id),
                args.stage.as_str(),
                args.write,
            ),
        ),
        Some(Command::Stage(StageArgs { command: None })) => {
            match help::human_help(&["stage".to_string()]) {
                Some(rendered) => {
                    print!("{}", rendered);
                    if !rendered.ends_with('\n') {
                        println!();
                    }
                    0
                }
                None => 1,
            }
        }
        Some(Command::Stage(StageArgs {
            command: Some(StageCommand::Advance(args)),
        })) => emit_result(
            args.format,
            workflow::stage_advance_command(&args.target, opt(&args.spec_id), args.stage.as_str()),
        ),
        Some(Command::Ux(UxArgs { command: None })) => {
            match help::human_help(&["ux".to_string()]) {
                Some(rendered) => {
                    print!("{}", rendered);
                    if !rendered.ends_with('\n') {
                        println!();
                    }
                    0
                }
                None => 1,
            }
        }
        Some(Command::Ux(UxArgs {
            command: Some(UxCommand::Validate(args)),
        })) => emit_result(args.format, workflow::validate_ux_command(&args.target)),
    }
}

fn emit_result(format: OutputFormat, result: Result<CommandOutcome, CliError>) -> i32 {
    match result {
        Ok(outcome) => {
            if let Err(err) = emit_stdout(format, &outcome.payload) {
                let _ = emit_stderr_error(format, &err);
                err.exit_code
            } else {
                outcome.exit_code
            }
        }
        Err(err) => {
            let _ = emit_stderr_error(format, &err);
            err.exit_code
        }
    }
}

fn opt(value: &str) -> Option<&str> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}
