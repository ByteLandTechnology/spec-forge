---
name: spec-forge-readiness
description: "Use when you need to consolidate prior stage artifacts into an implementation-ready spec, confirm completeness, record open assumptions, and decide whether the work is ready to build."
---

# Spec Forge Readiness

Use this stage to consolidate the approved stage artifacts into a final
implementation-ready spec contract.

This stage answers: is the spec complete enough to build, what remains deferred,
what acceptance criteria are binding, and are any blockers still unresolved.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before readiness questions.
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
- `spec-forge-cli gate check`
- `spec-forge-cli stage advance`

## Required UX Contract

Read [`../spec-forge/assets/contracts/ux-contracts.yaml`](../spec-forge/assets/contracts/ux-contracts.yaml)
before collecting parameters.

At the start of the stage, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-readiness --stage readiness --write`

If the runtime is not Plan mode, stop and ask the user to switch before
collecting inputs or showing any choice dialog.

Use chat summaries as the approval surface; YAML remains the persisted contract
for the stage.

## Entry Gate

| #   | Check                            | Source           |
| --- | -------------------------------- | ---------------- |
| 1   | Component artifacts exist        | Components stage |
| 2   | Component artifacts are approved | Components stage |

## Required Inputs

- Approved intake artifacts
- Approved architecture artifacts
- Approved journey artifacts
- Approved component artifacts

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-readiness --stage readiness --write`
   Confirm chat language and file language before stage-specific readiness
   questions.
   Then collect only the next missing parameter, one step at a time, and record
   every accepted answer or choice with
   `spec-forge-cli apply ...`.
2. Update `.spec-forge/specs/<spec-id>/synthesis/implementation-spec.yaml`.
   Use `spec-forge-cli artifact merge --file synthesis/implementation-spec.yaml`
   for incremental patches and `spec-forge-cli artifact put --file synthesis/implementation-spec.yaml`
   when replacing the full contract.
3. Consolidate the stage artifacts into one implementation contract:
   - final objective and scope
   - final journey list
   - final component list
   - interface and data-model contracts
   - cross-cutting concerns
   - feature flags, migrations, and rollout notes
   - acceptance criteria
   - deferred items
   - unresolved items
4. Treat unresolved items honestly. If something is not decided, record it as a
   blocker instead of pretending the spec is complete.
5. Set `implementation_gate.ready` to `true` only when:
   - the final scope is explicit
   - acceptance criteria are explicit
   - unresolved items are empty
   - the user confirms the spec is implementation-ready
6. Present a concise readiness summary in chat and ask for explicit approval
   there. Do not ask the user to inspect the YAML file directly.
   Use a choice dialog when the runtime supports it; otherwise fall back to a
   compact numbered menu.
   When the user approves the chat summary, record it with:
   - `spec-forge-cli approve --target <target-dir> --spec-id <spec-id> --file synthesis/implementation-spec.yaml`
7. Run:
   - `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage readiness --write`
   - `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage readiness`

## Outputs

- `.spec-forge/specs/<spec-id>/synthesis/implementation-spec.yaml`
- `.spec-forge/specs/<spec-id>/gates/readiness.yaml`

## Exit Gate

| #   | Check                                      |
| --- | ------------------------------------------ |
| 1   | Final scope is listed                      |
| 2   | Acceptance criteria are listed             |
| 3   | Unresolved items are empty                 |
| 4   | `implementation_gate.ready` is true        |
| 5   | Final implementation spec is user-approved |

## Guardrails

- Do not hide uncertainty to force completion.
- Do not reopen broad design exploration unless a real blocker requires it.
- Do not mark the implementation gate ready without explicit confirmation.
- Do not emit the final deliverable as Markdown. The contract lives in YAML.
- Do not collect missing required parameters in one large batch.
- Do not continue in non-Plan mode.
- Do not ask the user to inspect YAML files directly. Summarize readiness
  content in chat instead.
- Do not leave approval implicit. Persist it with `spec-forge-cli approve`.

## Next Step

After readiness passes, continue with
[`../spec-forge-implement/SKILL.md`](../spec-forge-implement/SKILL.md) so the
pipeline records implementation results before completion.
