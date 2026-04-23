---
name: spec-forge-components
description: "Use when you need to define component-level contracts in reviewable batches, including responsibilities, interfaces, state, data models, and implementation constraints."
---

# Spec Forge Components

Use this stage to convert the approved journeys and component index into
component-level contracts that are detailed enough for implementation.

This stage answers: what each component owns, what interfaces it exposes, what
state and data it manages, and what code-level constraints must hold.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before component questions.
- Keep decisions in chat. YAML remains the persisted source of truth, but the
  user should not need to inspect files directly.
- Approvals happen from chat summaries, then get persisted into YAML.

## Required Shared CLI Commands

- `spec-forge-cli resolve`
- `spec-forge-cli apply`
- `spec-forge-cli artifact get`
- `spec-forge-cli artifact put`
- `spec-forge-cli artifact merge`
- `spec-forge-cli approve`
- `spec-forge-cli focus`
- `spec-forge-cli gate check`
- `spec-forge-cli stage advance`

## Required UX Contract

Read [`../spec-forge/assets/contracts/ux-contracts.yaml`](../spec-forge/assets/contracts/ux-contracts.yaml)
before collecting parameters.

At the start of the stage, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-components --stage components --write`

If the runtime is not Plan mode, stop and ask the user to switch before
collecting inputs or showing any choice dialog.

Use chat summaries as the approval surface; YAML remains the persisted contract
for the stage.

## Entry Gate

| #   | Check                          | Source         |
| --- | ------------------------------ | -------------- |
| 1   | Journey artifacts exist        | Journeys stage |
| 2   | Journey artifacts are approved | Journeys stage |

## Required Inputs

- Approved journey artifacts
- Approved component index
- The current component batch plan

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-components --stage components --write`
   Confirm chat language and file language before stage-specific component
   questions.
   Then collect only the next missing parameter, one step at a time, and record
   every accepted answer or choice with
   `spec-forge-cli apply ...`.
2. Select the next review batch with:
   `spec-forge-cli focus --target <target-dir> --spec-id <spec-id> --stage components --max-items 1 --write`
3. Focus on one component at a time unless the batch contains a tiny,
   inseparable pair.
4. For each selected component, create or update
   `.spec-forge/specs/<spec-id>/components/<component-id>.yaml`.
   Use `spec-forge-cli artifact get` to inspect the current artifact when
   needed, then persist detail with `spec-forge-cli artifact merge` for
   incremental patches or `spec-forge-cli artifact put` for a full replacement.
5. Record the relevant `discussion_roles` for that component. Choose roles that
   fit the component, such as:
   - `tech-lead`
   - `frontend-engineer`
   - `backend-engineer`
   - `data-engineer`
   - `security-reviewer`
   - `qa-lead`
   - `sre-operator`
6. Fill the component artifact progressively:
   - purpose, responsibilities, and non-responsibilities
   - inputs, outputs, and events
   - source of truth, persisted state, and derived state
   - dependencies
   - algorithms, validation rules, and error handling
   - concurrency or ordering constraints when relevant
   - performance, security, accessibility, and observability requirements
   - feature flags, migrations, and testing expectations
   - links back to journeys
7. Keep the review progressive:
   - no more than 3 tightly related questions in a turn
   - prefer the next code-level question that most reduces ambiguity
   - do not review unrelated components together
8. Update `.spec-forge/specs/<spec-id>/components/batches.yaml` as work progresses,
   using `spec-forge-cli artifact merge --file components/batches.yaml` or
   `spec-forge-cli artifact put --file components/batches.yaml`.
9. Present a concise component summary in chat for each finished component and
   ask for explicit approval there. Summarize the batch status in chat before
   approving the batch file as well. Do not ask the user to inspect YAML files
   directly.
   Use a choice dialog when the runtime supports it; otherwise fall back to a
   compact numbered menu.
   When the user approves the chat summary, record it with:
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file components/<component-id>.yaml`
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file components/batches.yaml`
10. Run:

- `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage components --write`
- `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage components`

## Outputs

- `.spec-forge/specs/<spec-id>/components/batches.yaml`
- `.spec-forge/specs/<spec-id>/components/<component-id>.yaml`
- `.spec-forge/specs/<spec-id>/gates/components.yaml`

## Exit Gate

| #   | Check                                                     |
| --- | --------------------------------------------------------- |
| 1   | Component batches exist                                   |
| 2   | Every in-scope component has a YAML detail artifact       |
| 3   | Every in-scope component detail artifact is user-approved |
| 4   | Component batch file is user-approved                     |

## Guardrails

- Do not modify implementation code in this stage.
- Do not accept hand-wavy code detail such as "handle errors somehow."
- Do not hide migrations, feature flags, or testing expectations.
- Do not force the user to review multiple complex components in one pass.
- Do not collect missing required parameters in one large batch.
- Do not continue in non-Plan mode.
- Do not ask the user to inspect YAML files directly. Summarize component
  content in chat instead.

## Next Step

Continue with
[`../spec-forge-readiness/SKILL.md`](../spec-forge-readiness/SKILL.md).
