---
name: spec-forge-intake
description: "Use when you need to turn a rough request into a clear spec framing by defining the problem, stakeholders, scope boundaries, and non-negotiable constraints before solution design begins."
---

# Spec Forge Intake

Use this stage to turn a rough request into a stable framing contract.

This stage answers: what problem are we solving, for whom, inside what scope,
under what constraints, and through which discussion roles.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before collecting stage-specific
  details. Default to the current chat language and English file artifacts only
  if the user has no preference.
- Keep decisions in chat. YAML remains the persisted source of truth, but the
  user should not need to inspect files to move the stage forward.
- Approvals happen from chat summaries. Summarize the current intake state in
  chat, get confirmation there, and then record approval in YAML.

## Required Shared CLI Commands

- `spec-forge-cli resolve`
- `spec-forge-cli apply`
- `spec-forge-cli artifact get`
- `spec-forge-cli artifact merge`
- `spec-forge-cli approve`
- `spec-forge-cli init`
- `spec-forge-cli gate check`
- `spec-forge-cli stage advance`

## Required UX Contract

Read [`../spec-forge/assets/contracts/ux-contracts.yaml`](../spec-forge/assets/contracts/ux-contracts.yaml)
before collecting parameters.

At the start of the stage, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-intake --stage intake --write`

If the runtime is not Plan mode, stop and ask the user to switch before
collecting inputs or showing any choice dialog.

Use chat summaries as the approval surface; YAML remains the persisted contract
for the stage.

## Entry Gate

| #   | Check                                       | Source            |
| --- | ------------------------------------------- | ----------------- |
| 1   | Target project directory is known           | User or workspace |
| 2   | `.spec-forge/` exists or can be initialized | Filesystem        |

## Required Inputs

- The user request
- The target project directory
- Enough context to describe the goal, users, scope, and major constraints

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-intake --stage intake --write`
   Confirm chat language and file language before stage-specific intake
   questions.
   Then collect only the next missing parameter, one step at a time, and record
   every accepted answer or choice with
   `spec-forge-cli apply ...`.
2. If `.spec-forge/` or the target `spec_id` is missing, run:
   `spec-forge-cli init --target <target-dir> --spec-id <spec-id> --request-title ... --request-summary ...`
3. Update `.spec-forge/specs/<spec-id>/framing/request-context.yaml`.
   Prefer `spec-forge-cli artifact merge --file framing/request-context.yaml`
   for artifact-level writes that are not covered by the current `apply`
   parameter or choice.
4. Update `.spec-forge/specs/<spec-id>/framing/role-map.yaml`.
   Prefer `spec-forge-cli artifact merge --file framing/role-map.yaml`
   for artifact-level writes that are not covered by the current `apply`
   parameter or choice.
5. Choose the active discussion roles for this spec. Always keep at least:
   - `product-manager`
   - `ux-architect`
   - `tech-lead`
   - `qa-lead`
6. Stay broad. Ask only the highest-leverage framing questions:
   - problem and goal
   - users and reviewers
   - in-scope and out-of-scope boundaries
   - product, technical, and delivery constraints
7. Keep each turn lightweight:
   - no more than 3 tightly related questions
   - no solution deep dive yet
   - no per-component or per-file review yet
8. After each meaningful user answer, update the YAML artifacts instead of
   holding the state only in conversation. Prefer
   `spec-forge-cli apply ...` over manual edits
   for normal parameter and choice collection, and use
   `spec-forge-cli artifact merge ...` when refining artifact fields outside the
   declared UX parameters.
9. Present a concise intake summary in chat and ask for explicit approval there.
   Do not ask the user to open or review `request-context.yaml` or
   `role-map.yaml`.
   Use a choice dialog when the runtime supports it; otherwise fall back to a
   compact numbered menu.
   When the user approves the chat summary, record it with:
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file framing/request-context.yaml`
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file framing/role-map.yaml`
     If the user is not ready to approve, keep the artifact explicit with
     `spec-forge-cli artifact merge ...` and an `approval.status` such as
     `pending`.
10. Run:

- `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage intake --write`
- `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage intake`

## Outputs

- `.spec-forge/specs/<spec-id>/framing/request-context.yaml`
- `.spec-forge/specs/<spec-id>/framing/role-map.yaml`
- `.spec-forge/specs/<spec-id>/gates/intake.yaml`

## Exit Gate

| #   | Check                                        |
| --- | -------------------------------------------- |
| 1   | Problem statement is explicit                |
| 2   | Goal is explicit                             |
| 3   | In-scope list exists                         |
| 4   | Primary users are named                      |
| 5   | Active role map exists                       |
| 6   | Both intake YAML artifacts are user-approved |

## Guardrails

- Do not design the final solution here.
- Do not start journey or component enumeration here.
- Do not discuss detailed code structure here beyond capturing hard constraints.
- Do not overwhelm the user with a giant checklist in one message.
- Do not collect missing required parameters in one large batch.
- Do not continue in non-Plan mode.
- Do not ask the user to inspect YAML files directly. Summarize intake content
  in chat instead.
- Do not proceed if approval is ambiguous. Record `approval.status: pending` until
  the user confirms, using `spec-forge-cli artifact merge ...` when needed.

## Next Step

Continue with
[`../spec-forge-architecture/SKILL.md`](../spec-forge-architecture/SKILL.md).
