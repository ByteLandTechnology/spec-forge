---
name: spec-forge-architecture
description: "Use when you need to turn approved framing into a top-level solution shape, journey list, and component list before refining detailed flows or implementation contracts."
---

# Spec Forge Architecture

Use this stage to convert the approved framing contract into a top-level
solution shape and decomposition plan.

This stage answers: what is the overall approach, what journeys matter, what
components exist, and which roles must challenge the design before detail work
starts.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before architecture questions.
- Keep decisions in chat. YAML remains the persisted source of truth, but the
  user should not need to inspect files directly.
- Approvals happen from chat summaries, then get persisted into YAML.

## Required Shared CLI Commands

- `spec-forge-cli resolve`
- `spec-forge-cli apply`
- `spec-forge-cli artifact get`
- `spec-forge-cli artifact merge`
- `spec-forge-cli approve`
- `spec-forge-cli gate check`
- `spec-forge-cli stage advance`

## Required UX Contract

Read [`../spec-forge/assets/contracts/ux-contracts.yaml`](../spec-forge/assets/contracts/ux-contracts.yaml)
before collecting parameters.

At the start of the stage, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-architecture --stage architecture --write`

If the runtime is not Plan mode, stop and ask the user to switch before
collecting inputs or showing any choice dialog.

Use chat summaries as the approval surface; YAML remains the persisted contract
for the stage.

## Entry Gate

| #   | Check                         | Source       |
| --- | ----------------------------- | ------------ |
| 1   | Intake artifacts exist        | Intake stage |
| 2   | Intake artifacts are approved | Intake stage |

## Required Inputs

- Approved `.spec-forge/specs/<spec-id>/framing/request-context.yaml`
- Approved `.spec-forge/specs/<spec-id>/framing/role-map.yaml`

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-architecture --stage architecture --write`
   Confirm chat language and file language before stage-specific architecture
   questions.
   Then collect only the next missing parameter, one step at a time, and record
   every accepted answer or choice with
   `spec-forge-cli apply ...`.
2. Read the framing artifacts before asking new questions.
3. Update `.spec-forge/specs/<spec-id>/architecture/solution-outline.yaml`.
   Prefer `spec-forge-cli artifact merge --file architecture/solution-outline.yaml`
   for artifact-level writes beyond the current `apply` parameter or choice.
4. Update `.spec-forge/specs/<spec-id>/architecture/journey-index.yaml`.
   Prefer `spec-forge-cli artifact merge --file architecture/journey-index.yaml`
   for artifact-level writes beyond the current `apply` parameter or choice.
5. Update `.spec-forge/specs/<spec-id>/architecture/component-index.yaml`.
   Prefer `spec-forge-cli artifact merge --file architecture/component-index.yaml`
   for artifact-level writes beyond the current `apply` parameter or choice.
6. Work broad-to-fine:
   - lock the overall solution summary
   - define owned boundaries and external dependencies
   - list top-level journeys
   - list top-level components
7. Use role-based review on the solution outline. At minimum, capture concerns
   from:
   - `product-manager`
   - `ux-architect`
   - `tech-lead`
   - `qa-lead`
8. Keep discussion progressive:
   - ask only one major design cluster at a time
   - do not dive into full step detail yet
   - do not dive into full component internals yet
9. In `journey-index.yaml`, keep each item concise and reviewable. Each journey
   item should eventually include:
   - `id`
   - `title`
   - `summary`
   - `priority`
   - `status`
   - `discussion_roles`
10. In `component-index.yaml`, keep each item concise and reviewable. Each
    component item should eventually include:

- `id`
- `title`
- `kind`
- `summary`
- `priority`
- `status`
- `discussion_roles`

11. Present a concise architecture summary in chat and ask for explicit approval
    there. Do not ask the user to inspect the YAML files directly.
    Use a choice dialog when the runtime supports it; otherwise fall back to a
    compact numbered menu.
    When the user approves the chat summary, record it with:

- `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file architecture/solution-outline.yaml`
- `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file architecture/journey-index.yaml`
- `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file architecture/component-index.yaml`

12. Run:

- `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage architecture --write`
- `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage architecture`

## Outputs

- `.spec-forge/specs/<spec-id>/architecture/solution-outline.yaml`
- `.spec-forge/specs/<spec-id>/architecture/journey-index.yaml`
- `.spec-forge/specs/<spec-id>/architecture/component-index.yaml`
- `.spec-forge/specs/<spec-id>/gates/architecture.yaml`

## Exit Gate

| #   | Check                                              |
| --- | -------------------------------------------------- |
| 1   | Solution summary is explicit                       |
| 2   | Journey index contains at least one journey        |
| 3   | Component index contains at least one component    |
| 4   | All three architecture artifacts are user-approved |

## Guardrails

- Do not skip straight to per-file or per-function detail.
- Do not allow architecture to pass with a vague solution summary.
- Do not let the journey index and component index become giant prose blobs.
- Do not let one role dominate the review. Record cross-role concerns.
- Do not collect missing required parameters in one large batch.
- Do not continue in non-Plan mode.
- Do not ask the user to inspect YAML files directly. Summarize architecture
  content in chat instead.

## Next Step

Continue with
[`../spec-forge-journeys/SKILL.md`](../spec-forge-journeys/SKILL.md).
