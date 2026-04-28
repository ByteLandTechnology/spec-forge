# spec-forge-readiness

`spec-forge-readiness` is `P4 readiness`. Use it after approved components to consolidate earlier YAML artifacts into the final implementation-ready spec.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-readiness` when:

- `P3 components` is already approved
- earlier-stage decisions need to be consolidated into one implementation-ready view
- you want to confirm whether build work can start
- unresolved items need to stay explicit before implementation begins

This is the stage that decides whether the workflow is actually ready to enter `P5 implement`.

## Prerequisites

Before using readiness directly, make sure:

- components are already approved
- earlier-stage artifacts exist and are stable enough to synthesize
- you are ready to review final scope and acceptance expectations

If stage selection is unclear, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use readiness for:

- initial intake or architecture discovery
- detailed journey or component authoring
- implementation execution itself
- hiding blockers just to move forward

This stage is for honest synthesis and gate review, not for skipping unresolved work.

## What The Agent May Ask

During readiness, the Agent may ask for:

- confirmation of final scope
- acceptance criteria or definition of done
- unresolved items that still block implementation
- whether any deferred decisions are acceptable
- whether the spec is ready for implementation approval

The Agent should keep unresolved items explicit instead of smoothing them over.

## Plan Mode

If required readiness fields are missing, the Agent may use Plan mode to collect the next missing answer before finalizing the implementation-ready spec.

## YAML Artifacts And Gate

The readiness stage persists:

- `synthesis/implementation-spec.yaml`
- `gates/readiness.yaml`

This is the `P4 readiness` gate that determines whether `P5 implement` can begin. The Agent should summarize the implementation-ready YAML view before asking for approval.

## What You Approve

Approve readiness when the summary clearly shows:

- the final intended scope
- the acceptance criteria
- any remaining blockers or unresolved items
- whether implementation is actually ready to begin

If blockers remain, stay in readiness or return to an earlier stage instead of forcing implementation forward.

## Next Step

After readiness is approved, continue with [`../spec-forge-implement/README.md`](../spec-forge-implement/README.md) for `P5 implement`.
