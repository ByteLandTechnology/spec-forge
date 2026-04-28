# spec-forge-cli

`spec-forge-cli` is the authoritative Rust runtime for the `spec-forge` YAML workflow.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## At A Glance

- Package: `@cli-forge-bin/spec-forge-cli`
- Source repository: `ByteLandTechnology/spec-forge`
- Rust edition: `2024`
- Minimum Rust version: `1.85`
- Release harness requirements: Node `^22.14.0 || >=24.10.0`, npm `>=10`

Use this CLI when you need to initialize a workspace, resolve the next interaction, update artifacts, record approvals, check gates, advance stages, or validate the UX contract.

## Install

### From npm

```bash
npm install -g @cli-forge-bin/spec-forge-cli
```

The npm package selects a matching native binary for these published targets:

- `darwin-arm64`
- `darwin-x64`
- `linux-arm64`
- `linux-x64`
- `win32-arm64`
- `win32-x64`

No postinstall download is required.

### Clone-first fallback

Run this helper from `spec-forge-cli/` inside a checked-out repository clone.

```bash
./scripts/install-current-release.sh
```

Use the helper script when you already have the repository checked out and want to install the binary from a released tag without going through the npm registry.

## Quick Start

Run these examples from the repository root so `--target .` points at the repo workspace.

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write --format json
spec-forge-cli ux validate --target .
spec-forge-cli help gate check --format json
```

## Command Table

| Command          | Purpose                                                                                |
| ---------------- | -------------------------------------------------------------------------------------- |
| `init`           | Create `.spec-forge/`, per-spec YAML skeletons, and initial gate state.                |
| `resolve`        | Resolve the current workspace state into the next required interaction or ready state. |
| `apply`          | Persist one accepted parameter answer or choice selection.                             |
| `focus`          | Select the next journey or component batch to review.                                  |
| `artifact get`   | Read one artifact from the resolved spec workspace.                                    |
| `artifact put`   | Replace or create one artifact through the CLI.                                        |
| `artifact merge` | Patch one artifact incrementally through the CLI.                                      |
| `approve`        | Record an explicit approval block on an artifact.                                      |
| `gate check`     | Evaluate a stage gate from persisted YAML state.                                       |
| `stage advance`  | Promote a stage only after the current gate passes.                                    |
| `ux validate`    | Validate shared UX contracts and agent metadata alignment.                             |
| `help`           | Show command help in human-readable or structured form.                                |

## Common Operations

Representative commands:

```bash
spec-forge-cli apply --target . --spec-id demo --stage intake --parameter problem_goal --value '"Ship the workflow"'
spec-forge-cli focus --target . --spec-id demo --stage journeys --write
spec-forge-cli artifact get --target . --spec-id demo --file framing/request-context.yaml
spec-forge-cli artifact put --target . --spec-id demo --file journeys/journey-alpha.yaml --value '{}'
spec-forge-cli artifact merge --target . --spec-id demo --file framing/request-context.yaml --value '{problem:{goal:"Ship the native CLI"}}'
spec-forge-cli approve --target . --spec-id demo --file architecture/solution-outline.yaml --note 'Approved after review.'
spec-forge-cli gate check --target . --spec-id demo --stage intake --write
spec-forge-cli stage advance --target . --spec-id demo --stage intake
```

## Output Formats

Structured commands support:

- `--format yaml`
- `--format json`
- `--format toml`

Use `--help` for terminal help text, or `spec-forge-cli help <command> --format json` when you need machine-readable command metadata.

Example:

```bash
spec-forge-cli help gate check --format json
```

## Validation

Run these validation commands from the repository root.

```bash
cargo test --manifest-path spec-forge-cli/Cargo.toml
cargo fmt --manifest-path spec-forge-cli/Cargo.toml --check
cargo clippy --manifest-path spec-forge-cli/Cargo.toml -- -D warnings
cargo package --manifest-path spec-forge-cli/Cargo.toml --offline
spec-forge-cli ux validate --target .
```

## Release Rehearsal And Recovery

Use the release harness only when preparing packages or verifying the release path.
Run the release-harness commands in this section from `spec-forge-cli/`.

Run a dry packaging rehearsal before a real release:

```bash
npm ci
npm run release:rehearse
```

Prepare release assets and npm prerequisites before the first production publish:

```bash
npm run release:prepublish
```

Recovery entry points:

- If the rehearsal fails because dependencies or auth are missing, fix the environment and rerun `npm ci` and `npm run release:rehearse`.
- If a CI release is wedged after a tag already exists, rerun the `Release` GitHub Actions workflow with the required `recover-version` input and the optional `recover-run-id` input when you need to reuse the original build artifacts.
- If you need a local install from an already released tag, rerun `./scripts/install-current-release.sh` from `spec-forge-cli/`.

> [!NOTE]
> `release:rehearse` validates packaging without creating Git tags, GitHub Releases, or real npm publishes.
