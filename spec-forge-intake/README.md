# spec-forge-intake

`spec-forge-intake` is `P0 intake`. Use it when an Agent needs to turn a rough request into approved framing YAML before solution design begins.

Language versions:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## When To Use It

Ask the Agent to use `$spec-forge-intake` when:

- the request is still rough or ambiguous
- the problem, goal, users, or scope are not yet clear
- major constraints need to be captured before design starts
- review roles need to be defined for later stages

Use this child skill directly only when the workflow is already in `P0 intake` or you intentionally want to stay inside intake.

## Prerequisites

Before using intake directly, make sure:

- the router is not needed to decide the correct stage
- no later-stage artifact needs to be approved first
- you are ready to define framing, scope, constraints, and reviewers

If you are unsure, start with [`../spec-forge/README.md`](../spec-forge/README.md).

## Do Not Use It For

Do not use intake for:

- solution design or architecture decisions
- journey-by-journey detail work
- component contracts
- implementation readiness or delivery reporting

If design has already started, intake may still need revision, but this skill is not a shortcut to later stages.

## What The Agent May Ask

During intake, the Agent will usually ask for:

- the problem statement
- the desired outcome
- the primary users or stakeholders
- what is in scope and out of scope
- major constraints, risks, or non-negotiables
- the review roles that should stay active later

It should ask for the next missing answer, not a full questionnaire at once.

## Plan Mode

If required intake fields are missing, the Agent may use Plan mode to collect them one by one.

## YAML Artifacts And Gate

The intake stage persists:

- `framing/request-context.yaml`
- `framing/role-map.yaml`
- the `P0 intake` gate result that determines whether `P1 architecture` can begin

The Agent should summarize these YAML artifacts before asking for approval.

## What You Approve

Approve intake when the summary accurately reflects:

- what problem is being solved
- what success looks like
- who the spec is for
- what boundaries or exclusions matter
- which review roles should remain involved

If anything is still vague, keep working in intake instead of moving forward.

## Next Step

After intake is approved, continue with [`../spec-forge-architecture/README.md`](../spec-forge-architecture/README.md) for `P1 architecture`.
