# spec-forge-journeys

`spec-forge-journeys` is `P2 journeys`. Use it after approved architecture to refine journeys into reviewable YAML, one batch at a time.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-journeys` when:

- `P1 architecture` is already approved
- the journey index exists, but journey detail is still missing
- you want to work through one journey batch at a time
- flows, states, rules, and edge cases need to be made explicit

This stage is for detailed journey refinement, not for redefining the whole system shape.

## Prerequisites

Before using journeys directly, make sure:

- architecture is already approved
- the in-scope journey set is already defined
- you want detailed flow artifacts, not top-level direction

If stage selection is unclear, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use journeys for:

- initial intake framing
- top-level architecture discovery
- detailed component contracts
- final readiness approval or implementation reporting

This stage refines journey behavior. It is not where you should casually reopen system shape.

## What The Agent May Ask

For each journey, the Agent may ask about:

- triggers and preconditions
- the main flow
- alternate or exceptional flows
- loading, empty, success, and error states
- business rules and edge cases
- linked touchpoints, systems, or components
- observability or acceptance expectations

The Agent should keep the discussion focused on the current journey batch.

## Plan Mode

If a journey still has missing required detail, the Agent may use Plan mode to collect the next missing answer before continuing that batch.

## YAML Artifacts And Gate

The journeys stage persists:

- `journeys/batches.yaml`
- one `journeys/<journey-id>.yaml` file for each in-scope journey
- the `P2 journeys` gate result that determines whether `P3 components` can begin

The Agent should summarize the current YAML batch before asking for approval.

## What You Approve

Approve a journey batch when the summary reflects:

- what starts each journey
- the main and alternate flows
- key states and failure paths
- important rules and edge cases
- how the journeys connect to the rest of the system

Repeat this until all in-scope journeys are approved.

## Next Step

After journeys are approved, continue with [`../spec-forge-components/README.md`](../spec-forge-components/README.md) for `P3 components`.
