# spec-forge-intake

`spec-forge-intake` 是 `P0 intake`。当 Agent 需要先把粗糙需求整理成已批准的 framing YAML，再开始方案设计时，使用这个阶段。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-intake`：

- 需求仍然粗糙或有歧义
- 问题、目标、用户或范围还不清楚
- 在设计开始前需要先记录关键约束
- 需要先明确后续阶段的评审角色

只有当工作流已经处于 `P0 intake`，或者你明确只想停留在 intake 时，才建议直接使用这个子技能。

## 前置条件

直接使用 intake 前，请确认：

- 不需要由 router 决定正确阶段
- 不存在必须先批准的后续阶段制品
- 你已经准备好定义 framing、范围、约束和评审角色

如果不确定，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 intake 用于：

- 方案设计或 architecture 决策
- 按 journey 细化
- component 契约定义
- 实现就绪判断或交付结果记录

即使设计已经开始，intake 也可能需要返工，但这个技能不是跳到后续阶段的捷径。

## Agent 可能会问什么

在 intake 中，Agent 通常会问：

- 要解决什么问题
- 想达到什么结果
- 主要用户或干系人是谁
- 哪些内容在范围内，哪些不在范围内
- 关键约束、风险或不可妥协项是什么
- 哪些评审角色需要继续参与后续阶段

它应一次只追问下一个缺失答案，而不是一次发完整问卷。

## Plan mode

如果 intake 必填字段缺失，Agent 可能会使用 Plan mode 逐项收集。

## YAML 制品与 Gate

intake 阶段会维护：

- `framing/request-context.yaml`
- `framing/role-map.yaml`
- 决定是否可以进入 `P1 architecture` 的 `P0 intake` gate 结果

在请求批准前，Agent 应先总结这些 YAML 制品。

## 你要批准什么

当总结准确反映以下内容时，可以批准 intake：

- 正在解决什么问题
- 什么才算成功
- 这份 spec 服务于谁
- 重要边界和排除项是什么
- 哪些评审角色需要持续参与

如果仍然模糊，就继续停留在 intake，而不是往后推进。

## 下一步

intake 批准后，进入 [`spec-forge-architecture`](../spec-forge-architecture/README.zh-CN.md) 的 `P1 architecture`。
