---
name: spec-forge-journeys
description: "Use when you need to refine user journeys or process flows step by step in small review batches, including states, alternate paths, edge cases, and acceptance detail."
---

# Spec Forge Journeys

Use this stage to refine the behavior of each major journey in progressive
review batches.

This stage answers: what happens step by step, which states and alternate paths
exist, what rules apply, and which code touchpoints the journey implies.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before journey questions.
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

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-journeys --stage journeys --write`

If the runtime is not Plan mode, stop and ask the user to switch before
collecting inputs or showing any choice dialog.

Use chat summaries as the approval surface; YAML remains the persisted contract
for the stage.

## Entry Gate

| #   | Check                               | Source             |
| --- | ----------------------------------- | ------------------ |
| 1   | Architecture artifacts exist        | Architecture stage |
| 2   | Architecture artifacts are approved | Architecture stage |

## Required Inputs

- Approved architecture artifacts
- The current journey index and batch policy

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-journeys --stage journeys --write`
   Confirm chat language and file language before stage-specific journey
   questions.
   Then collect only the next missing parameter, one step at a time, and record
   every accepted answer or choice with
   `spec-forge-cli apply ...`.
2. Select the next review batch with:
   `spec-forge-cli focus --target <target-dir> --spec-id <spec-id> --stage journeys --max-items 1 --write`
3. Focus on one journey at a time unless the batch contains a tiny pair of
   tightly coupled sibling flows.
4. For each selected journey, create or update
   `.spec-forge/specs/<spec-id>/journeys/<journey-id>.yaml`.
   Use `spec-forge-cli artifact get` to inspect the current artifact when
   needed, then persist detail with `spec-forge-cli artifact merge` for
   incremental patches or `spec-forge-cli artifact put` for a full replacement.
5. Record the relevant `discussion_roles` for that journey. Common roles are:
   - `product-manager`
   - `ux-architect`
   - `qa-lead`
   - `tech-lead`
6. Fill the journey artifact progressively:
   - trigger, actors, and preconditions
   - main flow
   - alternate flows
   - loading, empty, error, and success states
   - business rules and edge cases
   - linked components
   - code touchpoints and public contracts
   - observability expectations
7. Keep review light:
   - no more than 3 tightly related questions in a turn
   - no more than one primary journey per review turn
   - defer component internals to the components stage
8. Update `.spec-forge/specs/<spec-id>/journeys/batches.yaml` so each batch reflects real review
   status, using `spec-forge-cli artifact merge --file journeys/batches.yaml`
   or `spec-forge-cli artifact put --file journeys/batches.yaml`.
9. Present a concise journey summary in chat for each finished journey and ask
   for explicit approval there. Summarize the batch status in chat before
   approving the batch file as well. Do not ask the user to inspect YAML files
   directly.
   Use a choice dialog when the runtime supports it; otherwise fall back to a
   compact numbered menu.
   When the user approves the chat summary, record it with:
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file journeys/<journey-id>.yaml`
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file journeys/batches.yaml`
10. Run:

- `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage journeys --write`
- `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage journeys`

## Outputs

- `.spec-forge/specs/<spec-id>/journeys/batches.yaml`
- `.spec-forge/specs/<spec-id>/journeys/<journey-id>.yaml`
- `.spec-forge/specs/<spec-id>/gates/journeys.yaml`

## Exit Gate

| #   | Check                                                   |
| --- | ------------------------------------------------------- |
| 1   | Journey batches exist                                   |
| 2   | Every in-scope journey has a YAML detail artifact       |
| 3   | Every in-scope journey detail artifact is user-approved |
| 4   | Journey batch file is user-approved                     |

## Guardrails

- Do not skip a journey because it feels obvious.
- Do not jump into component internals except for identifying touchpoints.
- Do not review too many flows at once.
- Do not leave open high-risk journey questions hidden inside prose.
- Do not collect missing required parameters in one large batch.
- Do not continue in non-Plan mode.
- Do not ask the user to inspect YAML files directly. Summarize journey content
  in chat instead.

## Next Step

Continue with
[`../spec-forge-components/SKILL.md`](../spec-forge-components/SKILL.md).
