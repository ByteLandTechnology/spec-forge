# spec-forge Router

Use `spec-forge` as the default entry point when you want an Agent to start or resume the spec-forge workflow safely.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge` when you want it to:

- start a new workflow without guessing the correct stage
- resume an existing workflow from the earliest incomplete stage
- gather missing invocation details step by step
- keep the P0-P5 order and approval gates enforced

This router is the normal entry point for the skill family.

## Prerequisites

The router works best when you can provide:

- the target repo or working directory
- the `spec_id`, if one already exists
- whether the Agent should create a new spec or continue an existing one

You do not need to know the next child stage in advance.

## Do Not Use It For

Do not use the router when:

- you only want command-line usage details
- you already know the exact child stage and all earlier stages are already approved
- you want to bypass a stage gate or skip approval

For CLI usage, go to [`../spec-forge-cli/README.md`](../spec-forge-cli/README.md).

## What The Agent May Ask

The router may ask for:

- the target repo or working directory
- the `spec_id`
- whether to create a new spec or continue an existing one
- the next missing required invocation field

If several specs exist, name the intended `spec_id`.

## Plan Mode

When required invocation details are missing, the router may use Plan mode to collect them one answer at a time. It should ask only for the next missing field before routing onward.

## YAML Artifacts And Gate

The router does not do deep stage work itself. It prepares or updates workflow state such as:

- `.spec-forge/registry.yaml`
- `.spec-forge/specs/<spec-id>/pipeline-state.yaml`
- `.spec-forge/specs/<spec-id>/handoff.yaml`

The router does not normally own the main stage gate. It routes to the correct child stage that owns the next gate and approval.

## What You Approve

At router time, confirm that:

- the Agent chose the correct `spec_id`
- the routed stage matches the current workflow state
- unresolved blockers remain visible in handoff state

Main artifact approval usually happens in the child stage, not in the router itself.

## Next Step

After routing, continue in the child stage the Agent selects:

- [`../spec-forge-intake/README.md`](../spec-forge-intake/README.md) for `P0 intake`
- [`../spec-forge-architecture/README.md`](../spec-forge-architecture/README.md) for `P1 architecture`
- [`../spec-forge-journeys/README.md`](../spec-forge-journeys/README.md) for `P2 journeys`
- [`../spec-forge-components/README.md`](../spec-forge-components/README.md) for `P3 components`
- [`../spec-forge-readiness/README.md`](../spec-forge-readiness/README.md) for `P4 readiness`
- [`../spec-forge-implement/README.md`](../spec-forge-implement/README.md) for `P5 implement`

If you are unsure, start with the router and let the Agent decide the next stage.
