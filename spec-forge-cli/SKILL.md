---
name: spec-forge-cli
description: "Use when the task requires operating or validating the package-local `spec-forge-cli` Rust command surface for initializing workspaces, resolving interactions, mutating YAML artifacts, recording approvals, selecting focus batches, checking gates, advancing stages, or validating UX contracts."
---

# spec-forge-cli

## Description

`spec-forge-cli` is the package-local Rust CLI surface for the spec-forge
workflow family. It is the authoritative runtime for initializing workspaces,
resolving interactions, applying answers, inspecting and mutating persisted
YAML artifacts, recording approvals, selecting focus batches, checking gates,
advancing stages, validating UX contracts, and returning help in either
human-readable or structured formats.

## Prerequisites

- Rust 1.85 or newer for source builds.
- A checked-out spec-forge repository when running the development command.
- Node.js and npm only when using the release automation harness.

## Invocation

- Shipped binary: `spec-forge-cli`
- Dev invocation: `cargo run --`
- Release binary: `target/release/spec-forge-cli`

## Command Surface

- `init`
- `resolve`
- `apply`
- `focus`
- `artifact get`
- `artifact put`
- `artifact merge`
- `approve`
- `gate check`
- `stage advance`
- `ux validate`
- `help`

## Help Contract

- Non-leaf default behavior: human-readable help on stdout.
- `--help`: human-readable help on stdout.
- `help [COMMAND_PATH ...]`: structured help in `yaml`, `json`, or `toml`.
- Leaf missing required input: structured error on stderr with a nonzero exit code.

## Input

Inputs are command-specific flags and inline YAML values. Most commands accept
`--target` to resolve a project directory, `.spec-forge` collection, or spec
workspace, and `--spec-id` to explicitly select the active spec when workspace
state is ambiguous.

## Output

Structured commands emit YAML by default and support `--format json` and
`--format toml`. Human help is emitted only for `--help`, non-leaf commands
without a subcommand, and the top-level command without arguments.

## Errors

Leaf command failures return structured errors in the requested output format.
Errors include a stable `code` and human-readable `message`; validation and
workflow errors may include additional remediation context.

## Examples

```bash
spec-forge-cli init --target . --request-title "My Spec"
spec-forge-cli resolve --target . --runtime-mode plan --write --format json
spec-forge-cli artifact get --target . --spec-id demo --file framing/request-context.yaml
spec-forge-cli help gate check --format toml
```

## Package Scope

This package owns only the Rust CLI implementation and its package-local
documentation and tests. It does not redefine stage order, Plan-mode
requirements, approval-gate behavior, or the broader skill-family semantics
described by the stage `SKILL.md` files and UX contract.
