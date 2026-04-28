# spec-forge-architecture

`spec-forge-architecture` is `P1 architecture`. Use it after approved intake to lock the high-level solution shape before detailed stage work begins.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-architecture` when:

- `P0 intake` is already approved
- the overall solution direction still needs to be defined
- the top-level journey set still needs to be identified
- the top-level component set still needs to be identified

This stage should happen before `P2 journeys` or `P3 components`.

## Prerequisites

Before using architecture directly, make sure:

- intake framing is already approved
- the workflow is ready to discuss solution shape instead of problem discovery
- you want top-level structure, not low-level implementation detail

If you are unsure about stage choice, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use architecture for:

- initial problem framing
- detailed journey flows
- detailed component contracts
- final implementation readiness or delivery reporting

This stage defines the shape of the system. It is not the place to fully specify every flow or component yet.

## What The Agent May Ask

During architecture, the Agent may ask for:

- the preferred solution shape
- major boundaries and dependencies
- the top-level user or system journeys
- the top-level components or subsystems
- risks or decisions that affect later detail work

The goal is to settle the shape of the system, not every implementation detail.

## Plan Mode

If architecture inputs are incomplete, the Agent may use Plan mode to collect the next missing answer while keeping the discussion at the right level of abstraction.

## YAML Artifacts And Gate

The architecture stage persists:

- `architecture/solution-outline.yaml`
- `architecture/journey-index.yaml`
- `architecture/component-index.yaml`
- the `P1 architecture` gate result that determines whether detailed stage work can begin

The Agent should summarize these YAML artifacts before asking for approval.

## What You Approve

Approve architecture when the summary clearly captures:

- the intended solution direction
- the major boundaries and assumptions
- the full in-scope journey set at a top level
- the full in-scope component set at a top level

If the overall shape is still unsettled, do not move on yet.

## Next Step

After architecture is approved, continue with [`../spec-forge-journeys/README.md`](../spec-forge-journeys/README.md) for `P2 journeys`.
