---
name: spec-forge
description: "Router for the spec-forge skill family: initialize a multi-spec `.spec-forge/` YAML collection, detect the earliest incomplete phase for the targeted `spec_id`, enforce hard stage gates, and route broad-to-fine, role-based spec refinement from problem framing through readiness and implementation."
---

# Spec Forge Router

Use this parent skill as the stable entry point for the full `spec-forge`
workflow.

This skill does not perform deep spec discussion itself. Its job is to
initialize the YAML workspace, inspect stage state, enforce gates, and route to
the correct child skill.

## Shared Operating Rules

- This skill family runs only in Plan mode. If the runtime is not Plan mode,
  stop and tell the user to switch before doing any routing or collection.
- At the beginning of the session, confirm both the chat language and the file
  language. Use the user's current chat language and English file artifacts only
  as defaults until the user confirms or overrides them.
- Decide details through chat. YAML remains the persisted source of truth, but
  do not ask the user to inspect files unless they explicitly request raw file
  contents.
- When a stage needs approval, use a chat summary as the review surface, get
  the user's confirmation there, and then persist the approval into YAML.

## Required Shared CLI Contract

The family relies on these shared CLI commands:

- `spec-forge-cli init`
- `spec-forge-cli resolve`
- `spec-forge-cli apply`
- `spec-forge-cli artifact get`
- `spec-forge-cli artifact put`
- `spec-forge-cli artifact merge`
- `spec-forge-cli approve`
- `spec-forge-cli gate check`
- `spec-forge-cli focus`
- `spec-forge-cli stage advance`
- `spec-forge-cli ux validate`

Child skills must name the exact command they use. Do not replace these shared
commands with ad hoc one-off files when the native CLI already supports the
step.

## Required Shared UX Contract

The family also relies on a machine-readable UX contract at
[`./assets/contracts/ux-contracts.yaml`](./assets/contracts/ux-contracts.yaml).

Use that contract to drive:

- parameter display in Agent UI
- step-by-step missing-parameter collection
- choice-dialog definitions
- Plan-mode requirements for all runtime execution

Before doing any routing work, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge --stage router --write`

Then use `spec-forge-cli apply ...` to write each collected
parameter answer or choice selection back into YAML before resolving the next
interaction.

Treat Plan mode, the UX contract, and the persisted YAML state as the shared
runtime contract for the whole family.

Use the artifact commands for artifact-level reads and writes that are not
covered by the current parameter or choice:

- `spec-forge-cli artifact get` to inspect one YAML artifact through the CLI
  when the assistant needs internal context
- `spec-forge-cli artifact put` to replace one artifact completely
- `spec-forge-cli artifact merge` to patch one artifact incrementally
- `spec-forge-cli approve` to record an explicit `approval` block in YAML

## Pipeline

| Phase | Skill                                                            | Purpose                                                         | Primary YAML outputs                                                                                         |
| ----- | ---------------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| P0    | [`spec-forge-intake`](../spec-forge-intake/SKILL.md)             | Frame the request, scope, roles, and constraints                | `framing/request-context.yaml`, `framing/role-map.yaml`                                                      |
| P1    | [`spec-forge-architecture`](../spec-forge-architecture/SKILL.md) | Lock the overall solution shape before detail work              | `architecture/solution-outline.yaml`, `architecture/journey-index.yaml`, `architecture/component-index.yaml` |
| P2    | [`spec-forge-journeys`](../spec-forge-journeys/SKILL.md)         | Refine user journeys and step-by-step flows in small batches    | `journeys/batches.yaml`, `journeys/<journey-id>.yaml`                                                        |
| P3    | [`spec-forge-components`](../spec-forge-components/SKILL.md)     | Refine each component and code-level contract in small batches  | `components/batches.yaml`, `components/<component-id>.yaml`                                                  |
| P4    | [`spec-forge-readiness`](../spec-forge-readiness/SKILL.md)       | Validate completeness and consolidate the final spec            | `synthesis/implementation-spec.yaml`, `gates/readiness.yaml`                                                 |
| P5    | [`spec-forge-implement`](../spec-forge-implement/SKILL.md)       | Execute the approved implementation and record delivery results | `synthesis/implementation-report.yaml`, `gates/implement.yaml`                                               |

## Shared Workspace Layout

All phases work inside the target project under this structure:

```text
<target>/.spec-forge/
├── registry.yaml
└── specs/
    └── <spec-id>/
        ├── pipeline-state.yaml
        ├── handoff.yaml
        ├── framing/
        │   ├── request-context.yaml
        │   └── role-map.yaml
        ├── architecture/
        │   ├── solution-outline.yaml
        │   ├── journey-index.yaml
        │   └── component-index.yaml
        ├── journeys/
        │   ├── batches.yaml
        │   └── <journey-id>.yaml
        ├── components/
        │   ├── batches.yaml
        │   └── <component-id>.yaml
        ├── synthesis/
        │   ├── implementation-spec.yaml
        │   └── implementation-report.yaml
        └── gates/
            ├── intake.yaml
            ├── architecture.yaml
            ├── journeys.yaml
            ├── components.yaml
            ├── readiness.yaml
            └── implement.yaml
```

All user-facing deliverables in the workflow must be YAML. Do not emit
Markdown artifacts as the stage outputs.

## Workflow

1. Resolve invocation UX before routing:
   - load `./assets/contracts/ux-contracts.yaml`
   - run `spec-forge-cli resolve --target <target-dir> --skill spec-forge --stage router --write`
   - if the runtime is not Plan mode, stop and instruct the user to switch
     before continuing
   - confirm the chat language and file language before any stage-specific
     discussion
   - if it reports choices and the Agent UI supports dialogs, present those dialogs before continuing
   - if it reports a missing required parameter, collect only the next missing
     parameter rather than asking for everything at once
   - after every accepted answer or dialog selection, run
     `spec-forge-cli apply ...` so the next prompt comes from
     YAML state instead of chat memory
2. Determine the target project directory and the target `spec_id`.
3. If `.spec-forge/` does not exist, or if the target `spec_id` does not
   exist, run:
   `spec-forge-cli init --target <target-dir> --spec-id <spec-id> --request-title ...`
4. Read `.spec-forge/registry.yaml` to identify the active or requested spec.
5. Read `.spec-forge/specs/<spec-id>/pipeline-state.yaml`.
6. Route to the earliest incomplete stage:
   - Missing or unapproved intake artifacts -> Intake
   - Intake approved, architecture incomplete -> Architecture
   - Architecture approved, journeys incomplete -> Journeys
   - Journeys approved, components incomplete -> Components
   - Components approved, final synthesis incomplete -> Readiness
   - Readiness approved, implementation report incomplete -> Implement
7. Before handing off, update `.spec-forge/specs/<spec-id>/handoff.yaml` with
   the next stage, reason, and blockers. Prefer the built-in handoff updates
   from `resolve`, `focus`, `gate check`, and `stage advance`; if a manual
   handoff patch is still needed, use
   `spec-forge-cli artifact merge --file handoff.yaml` instead of raw file
   editing.
8. At every phase boundary, run:
   - `spec-forge-cli gate check --target <workspace-or-target> --spec-id <spec-id> --stage <stage> --write`
   - `spec-forge-cli stage advance --target <workspace-or-target> --spec-id <spec-id> --stage <stage>`
9. Never skip a failed gate and never promote a stage without explicit user
   approval recorded in the YAML artifacts, preferably with
   `spec-forge-cli approve`.

When multiple specs coexist in one project, never guess the intended spec when
the user has named one. Use the explicit `spec_id`, or a direct path under
`.spec-forge/specs/<spec-id>/`. Only fall back to the active spec in
`registry.yaml` when the user has not identified a specific spec.

## Entry Gate

| #   | Check               | Source |
| --- | ------------------- | ------ |
| 1   | User request exists | User   |

## Outputs

- `<target>/.spec-forge/registry.yaml`
- `<target>/.spec-forge/specs/<spec-id>/pipeline-state.yaml`
- `<target>/.spec-forge/specs/<spec-id>/handoff.yaml`
- Explicit routing to the correct child skill

## Exit Gate

| #   | Check                                                   |
| --- | ------------------------------------------------------- |
| 1   | `.spec-forge/` collection exists                        |
| 2   | Target `spec_id` resolved correctly                     |
| 3   | Earliest incomplete phase identified correctly          |
| 4   | `handoff.yaml` updated inside the target spec workspace |
| 5   | No phase with a failed gate was skipped                 |

## Strict Guardrails

- Do not perform implementation work in this router.
- Do not perform stage-specific spec work here. Route only.
- Do not bypass the phase order.
- Do not allow free-form Markdown deliverables to replace the required YAML
  artifacts.
- Do not collect missing required parameters in bulk when the UX contract says
  to collect them step by step.
- Do not continue in non-Plan mode. Stop and tell the user to switch to Plan
  mode first.
- Do not skip the initial chat-language and file-language confirmation.
- Do not ask the user to review YAML files directly. Summarize the relevant
  stage content in chat instead.
- Do not overload the user with full-system detail in one turn. The family must
  stay broad-to-fine and batch detail progressively.
- Do not let a later stage start while an earlier stage still has unresolved
  high-risk questions or missing approvals.

## Next Step

Route to one of:

- [`../spec-forge-intake/SKILL.md`](../spec-forge-intake/SKILL.md)
- [`../spec-forge-architecture/SKILL.md`](../spec-forge-architecture/SKILL.md)
- [`../spec-forge-journeys/SKILL.md`](../spec-forge-journeys/SKILL.md)
- [`../spec-forge-components/SKILL.md`](../spec-forge-components/SKILL.md)
- [`../spec-forge-readiness/SKILL.md`](../spec-forge-readiness/SKILL.md)
- [`../spec-forge-implement/SKILL.md`](../spec-forge-implement/SKILL.md)
