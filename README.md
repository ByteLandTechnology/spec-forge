# spec-forge

`spec-forge` is a YAML-first workflow for turning a rough request into an approved implementation spec and an auditable implementation report. It combines agent-guided stage progression with a Rust CLI that persists durable state under `.spec-forge/`.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## What It Is

- A staged workflow that moves from request framing to implementation reporting.
- A persistent workspace model that keeps approvals, gates, focus, and artifacts in YAML.
- A CLI runtime, [`spec-forge-cli`](./spec-forge-cli/README.md), that is the source of truth for workspace mutations.

## When To Use It

Use `spec-forge` when you need to:

- turn a vague feature request into a structured, reviewable spec
- persist planning state in versionable files instead of chat memory
- pause and resume multi-stage spec work safely
- require explicit approvals before stage advancement

If you only need the command-line surface, start with [`spec-forge-cli/README.md`](./spec-forge-cli/README.md).

## Quick Start

### Agent entry

Ask your agent to use `$spec-forge` when you want it to initialize or resume a workspace, resolve the earliest incomplete stage, and continue from there.

Example prompts:

- "Use `$spec-forge` to turn this request into an approved spec."
- "Use `$spec-forge` in this repo and continue `checkout-redesign`."
- "Use `$spec-forge` to find the next incomplete stage and keep going."

### CLI entry

Run these examples from the repository root.

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write
spec-forge-cli ux validate --target .
```

> [!NOTE]
> Agent skills guide the conversation, but `spec-forge-cli` is the authoritative runtime for changing persisted workflow state.

## Stage Map

| Stage               | Purpose                                                             |
| ------------------- | ------------------------------------------------------------------- |
| `P0 / Intake`       | Frame the request, stakeholders, scope, and constraints.            |
| `P1 / Architecture` | Lock the solution outline plus the journey and component indexes.   |
| `P2 / Journeys`     | Refine in-scope journeys in reviewable batches.                     |
| `P3 / Components`   | Refine in-scope components into implementation-facing contracts.    |
| `P4 / Readiness`    | Consolidate approved work into the final implementation spec.       |
| `P5 / Implement`    | Record delivery status, validations, blockers, and closure details. |

## Workspace Model

Persistent workflow state lives under `.spec-forge/`:

```text
.spec-forge/
├── registry.yaml
└── specs/
    └── <spec-id>/
        ├── pipeline-state.yaml
        ├── handoff.yaml
        ├── framing/
        ├── architecture/
        ├── journeys/
        ├── components/
        ├── synthesis/
        └── gates/
```

Key artifacts:

- `pipeline-state.yaml` tracks stage, focus, and UX handoff state.
- `synthesis/implementation-spec.yaml` is the implementation-ready contract.
- `synthesis/implementation-report.yaml` records delivery status and validations.
- `gates/<stage>.yaml` records gate evaluation output per stage.

## Repo Layout

Paths below are relative to the repository root.

```text
README*.md                         Project entry docs
AGENTS.md                          Agent instructions for this repo
spec-forge/                        Router skill and shared UX contracts/assets
spec-forge-intake/                 P0 stage skill
spec-forge-architecture/           P1 stage skill
spec-forge-journeys/               P2 stage skill
spec-forge-components/             P3 stage skill
spec-forge-readiness/              P4 stage skill
spec-forge-implement/              P5 stage skill
spec-forge-cli/                    Rust CLI, tests, npm wrapper, release scripts
specs/001-spec-execution-stage/    Repo-level context referenced by AGENTS.md
.spec-forge/specs/001-spec-execution-stage/
                                   Example persisted workflow state for this repo
```

## CLI Entry Point

Use [`spec-forge-cli/README.md`](./spec-forge-cli/README.md) for:

- installation and platform support
- quick-start command sequences
- command reference and structured help
- output formats and validation commands
- release rehearsal and CI recovery workflow inputs

## Documentation Map

| Area            | Documents                                                                                                                                                                                                                                                                                                                                                              | Use for                                                                          |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Workflow entry  | [`spec-forge` router guide](./spec-forge/README.md), [`spec-forge/SKILL.md`](./spec-forge/SKILL.md)                                                                                                                                                                                                                                                                    | Start or resume the workflow and inspect the executable router contract.         |
| Stage guides    | [`spec-forge-intake`](./spec-forge-intake/README.md), [`spec-forge-architecture`](./spec-forge-architecture/README.md), [`spec-forge-journeys`](./spec-forge-journeys/README.md), [`spec-forge-components`](./spec-forge-components/README.md), [`spec-forge-readiness`](./spec-forge-readiness/README.md), [`spec-forge-implement`](./spec-forge-implement/README.md) | Understand when to use each P0-P5 stage and what the Agent will ask.             |
| Stage contracts | [`intake`](./spec-forge-intake/SKILL.md), [`architecture`](./spec-forge-architecture/SKILL.md), [`journeys`](./spec-forge-journeys/SKILL.md), [`components`](./spec-forge-components/SKILL.md), [`readiness`](./spec-forge-readiness/SKILL.md), [`implement`](./spec-forge-implement/SKILL.md)                                                                         | Review the executable rules, gates, required inputs, and outputs for each stage. |
| CLI package     | [`spec-forge-cli`](./spec-forge-cli/README.md), [`npm wrapper`](./spec-forge-cli/npm/main/README.md), [`release record`](./spec-forge-cli/CHANGELOG.md)                                                                                                                                                                                                                | Install, operate, package, or audit the CLI distribution.                        |
| CLI contract    | [`spec-forge-cli/SKILL.md`](./spec-forge-cli/SKILL.md)                                                                                                                                                                                                                                                                                                                 | Inspect the CLI command surface and structured output contract.                  |
| Repo context    | [`spec execution plan`](./specs/001-spec-execution-stage/plan.md), [`AGENTS.md`](./AGENTS.md)                                                                                                                                                                                                                                                                          | Read repository-level context and agent instructions.                            |
