# spec-forge-implement

`spec-forge-implement` is `P5 implement`. Use it after approved readiness to record implementation results against the approved spec.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-implement` when:

- `P4 readiness` is already approved
- implementation work is being carried out from the approved spec
- you want the workflow to keep an honest record of changes, validations, and blockers
- you need a formal final-stage record instead of a chat-only summary

This stage records delivery status after readiness approval.

## Prerequisites

Before using implement directly, make sure:

- readiness is already approved
- the implementation-ready spec exists
- you want to record actual delivery status, not reopen scope casually

If stage selection is unclear, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use implement for:

- initial intake, architecture, journey, or component design
- final readiness synthesis
- rewriting scope without going back through earlier approvals
- hiding failed validation just to close the workflow

This stage is for honest reporting of implementation outcomes.

## What The Agent May Ask

During implement, the Agent may ask for:

- whether implementation is complete
- what changed
- which files or areas were affected
- what validations ran and how they ended
- whether blockers or follow-up work remain

The Agent should record outcomes honestly, including failed validation or incomplete work.

## Plan Mode

If required report fields are missing, the Agent may use Plan mode to collect the next missing answer before finalizing the implementation report.

## YAML Artifacts And Gate

The implement stage persists:

- `synthesis/implementation-report.yaml`
- `gates/implement.yaml`

This is the `P5 implement` gate. The Agent should summarize the delivery-state YAML artifacts before asking for final approval.

## What You Approve

Approve implement when the summary clearly records:

- whether the work is complete
- the real scope of changes
- what was validated
- any remaining blockers, risks, or follow-up items

The workflow should not hide failed checks or unresolved blockers just to close the stage.

## Next Step

If the implement stage passes, the workflow can close as complete.
