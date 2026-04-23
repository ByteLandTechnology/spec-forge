#![allow(clippy::result_large_err)]

use clap::ValueEnum;
use serde::Serialize;
use serde_json::{Value as JsonValue, json};
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Yaml,
    Json,
    Toml,
}

impl OutputFormat {
    pub fn from_argv(argv: &[String]) -> Self {
        let mut iter = argv.iter().peekable();
        while let Some(arg) = iter.next() {
            if let Some(value) = arg.strip_prefix("--format=") {
                return Self::from_str_lossy(value);
            }
            if arg == "--format"
                && let Some(value) = iter.peek()
            {
                return Self::from_str_lossy(value);
            }
        }
        Self::Yaml
    }

    pub fn from_str_lossy(value: &str) -> Self {
        match value {
            "json" => Self::Json,
            "toml" => Self::Toml,
            _ => Self::Yaml,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CliError {
    pub code: String,
    pub message: String,
    pub details: Option<JsonValue>,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            exit_code: 1,
        }
    }

    pub fn with_details(mut self, details: JsonValue) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = exit_code;
        self
    }
}

pub fn emit_stdout<T: Serialize>(format: OutputFormat, value: &T) -> Result<(), CliError> {
    let content = serialize(format, value)?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(content.as_bytes())
        .map_err(|err| io_error("stdout_write_failed", err))?;
    if !content.ends_with('\n') {
        stdout
            .write_all(b"\n")
            .map_err(|err| io_error("stdout_write_failed", err))?;
    }
    Ok(())
}

pub fn emit_stderr_error(format: OutputFormat, err: &CliError) -> Result<(), CliError> {
    let payload = json!({
        "error": {
            "code": err.code,
            "message": err.message,
            "details": err.details,
        }
    });
    let content = serialize(format, &payload)?;
    let mut stderr = io::stderr().lock();
    stderr
        .write_all(content.as_bytes())
        .map_err(|write_err| io_error("stderr_write_failed", write_err))?;
    if !content.ends_with('\n') {
        stderr
            .write_all(b"\n")
            .map_err(|write_err| io_error("stderr_write_failed", write_err))?;
    }
    Ok(())
}

fn serialize<T: Serialize>(format: OutputFormat, value: &T) -> Result<String, CliError> {
    match format {
        OutputFormat::Yaml => serde_yaml::to_string(value)
            .map_err(|err| serialization_error("yaml_serialization_failed", err.to_string())),
        OutputFormat::Json => serde_json::to_string_pretty(value)
            .map_err(|err| serialization_error("json_serialization_failed", err.to_string())),
        OutputFormat::Toml => toml::to_string_pretty(value)
            .map_err(|err| serialization_error("toml_serialization_failed", err.to_string())),
    }
}

pub fn io_error(code: &str, err: io::Error) -> CliError {
    CliError::new(code, err.to_string())
}

pub fn serialization_error(code: &str, message: String) -> CliError {
    CliError::new(code, message)
}
