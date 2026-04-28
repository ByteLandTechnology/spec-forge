# spec-forge-components

`spec-forge-components` is `P3 components`. Use it after approved journeys to refine each component into an implementation-facing contract.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-components` when:

- `P2 journeys` is already approved
- the component index exists, but components still need detail
- you want to work through one component batch at a time
- interfaces, state, dependencies, or operational expectations need to be made explicit

This stage is for component contracts, not for rediscovering the request from scratch.

## Prerequisites

Before using components directly, make sure:

- journeys are already approved
- the in-scope component set is already defined
- you want implementation-facing component detail, not top-level architecture discussion

If stage selection is unclear, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use components for:

- initial intake framing
- top-level solution shape decisions
- journey-flow discovery
- final readiness approval or delivery reporting

This stage refines component contracts. It is not where you should reopen the full request or redesign architecture casually.

## What The Agent May Ask

For each component, the Agent may ask about:

- responsibilities and non-responsibilities
- inputs, outputs, interfaces, or events
- state ownership and source of truth
- dependencies on other systems or components
- validation, ordering, or failure behavior
- performance, security, accessibility, or observability expectations
- testing, rollout, migration, or flagging expectations

The Agent should stay focused on the current component batch.

## Plan Mode

If a component contract still has missing required detail, the Agent may use Plan mode to collect the next missing answer before continuing.

## YAML Artifacts And Gate

The components stage persists:

- `components/batches.yaml`
- one `components/<component-id>.yaml` file for each in-scope component
- the `P3 components` gate result that determines whether `P4 readiness` can begin

The Agent should summarize the current YAML batch before asking for approval.

## What You Approve

Approve a component batch when the summary clearly covers:

- what each component owns
- what it does not own
- how it interacts with other parts of the system
- key state, validation, and failure expectations
- important rollout or test expectations

Repeat this until all in-scope components are approved.

## Next Step

After components are approved, continue with [`../spec-forge-readiness/README.md`](../spec-forge-readiness/README.md) for `P4 readiness`.
