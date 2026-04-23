---
name: spec-forge-implement
description: "Use when you need to execute an approved implementation spec, record what changed, capture validations, and track whether any blockers still prevent completion."
---

# Spec Forge Implement

Use this stage after readiness passes to perform the implementation work and
record its outcome in the per-spec implementation report.

This stage answers: what changed, whether implementation is complete, which
validations ran, and whether any blockers remain.

## Shared Operating Rules

- This skill runs only in Plan mode. If the runtime is not Plan mode, stop and
  tell the user to switch before continuing.
- Confirm both chat language and file language before implementation work or
  implementation-report updates.
- Keep decisions and status updates in chat. YAML remains the persisted source
  of truth, but the user should not need to inspect files directly.
- Report implementation progress and completion from chat summaries rather than
  file review.

## Required Shared CLI Commands

- `spec-forge-cli resolve`
- `spec-forge-cli artifact get`
- `spec-forge-cli artifact put`
- `spec-forge-cli artifact merge`
- `spec-forge-cli gate check`
- `spec-forge-cli stage advance`

## Required UX Contract

Read [`../spec-forge/assets/contracts/ux-contracts.yaml`](../spec-forge/assets/contracts/ux-contracts.yaml)
before collecting parameters.

At the start of the stage, run:

`spec-forge-cli resolve --target <target-dir> --skill spec-forge-implement --stage implement --write`

If the runtime is not Plan mode, stop and ask the user to switch before
continuing.

Use chat summaries as the progress surface; the implement stage still closes on
the persisted report plus gate evaluation rather than a new approval gate.

## Entry Gate

| #   | Check                                 | Source          |
| --- | ------------------------------------- | --------------- |
| 1   | Readiness gate passed                 | Readiness stage |
| 2   | Final implementation spec is approved | Readiness stage |

## Required Inputs

- Approved `synthesis/implementation-spec.yaml`
- Passed `gates/readiness.yaml`

## Workflow

1. Resolve invocation UX first with:
   `spec-forge-cli resolve --target <target-dir> --skill spec-forge-implement --stage implement --write`
   Confirm chat language and file language before implementation work begins.
2. Perform the implementation work against the target codebase.
3. Update `.spec-forge/specs/<spec-id>/synthesis/implementation-report.yaml`.
   Prefer `spec-forge-cli artifact merge --file synthesis/implementation-report.yaml`
   for progressive updates during implementation and
   `spec-forge-cli artifact put --file synthesis/implementation-report.yaml`
   when writing the final report payload in one pass.
4. Record at least:
   - whether implementation is completed
   - a short summary of changes
   - changed files
   - validations that ran and their status
   - any remaining blockers
5. Keep blockers explicit. Do not mark implementation completed while blockers
   remain or while a validation is failing.
6. Run:
   - `spec-forge-cli gate check --target <target-dir> --spec-id <spec-id> --stage implement --write`
   - `spec-forge-cli stage advance --target <target-dir> --spec-id <spec-id> --stage implement`

## Outputs

- `.spec-forge/specs/<spec-id>/synthesis/implementation-report.yaml`
- `.spec-forge/specs/<spec-id>/gates/implement.yaml`

## Exit Gate

| #   | Check                                        |
| --- | -------------------------------------------- |
| 1   | Readiness is approved and marked ready       |
| 2   | `implementation.completed` is true           |
| 3   | At least one validation is recorded          |
| 4   | No validation has status `failed` or `error` |
| 5   | No blockers remain                           |

## Guardrails

- Do not skip the readiness gate.
- Do not mark implementation completed while blockers remain.
- Do not omit validation results from the implementation report.
- Do not close the pipeline when a validation has failed.
- Do not ask the user to inspect YAML files directly. Summarize implementation
  progress and results in chat instead.
- Do not introduce a separate approval gate for the implementation report unless
  the workflow contract changes; the implement stage closes on report contents
  plus gate evaluation.

## Next Step

If the implement gate passes, the pipeline can advance to complete.
