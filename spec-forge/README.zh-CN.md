# spec-forge Router

当你希望 Agent 安全地启动或恢复 spec-forge 工作流时，`spec-forge` router 是默认入口。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当你希望 Agent：

- 在不猜测阶段的前提下启动一个新工作流
- 从最早未完成的阶段恢复已有工作流
- 逐步补齐缺失的调用信息
- 严格遵守 `P0-P5` 顺序和 approval gate

这时应使用 `$spec-forge`。

## 前置条件

router 最适合在你能提供以下信息时使用：

- 目标仓库或工作目录
- 已存在时对应的 `spec_id`
- 是新建 spec，还是继续已有 spec

你不需要事先知道下一步应该进入哪个子阶段。

## 不适用场景

以下情况不要使用 router：

- 你只是想看命令行用法
- 你已经明确知道要进哪个子阶段，且前序阶段都已批准
- 你想绕过某个阶段 gate 或跳过批准

CLI 用法请查看 [`CLI 文档`](../spec-forge-cli/README.zh-CN.md)。

## Agent 可能会问什么

router 可能会向你确认：

- 目标仓库或工作目录
- `spec_id`
- 是新建 spec，还是继续已有 spec
- 当前缺失的下一个必填调用字段

如果项目里有多个 spec，最好明确指出目标 `spec_id`。

## Plan mode

当调用所需信息不完整时，router 可能使用 Plan mode 逐项收集。它应该一次只问下一个缺失字段，再继续路由。

## YAML 制品与 Gate

router 不负责深入执行具体阶段。它会准备或更新如下工作流状态：

- `.spec-forge/registry.yaml`
- `.spec-forge/specs/<spec-id>/pipeline-state.yaml`
- `.spec-forge/specs/<spec-id>/handoff.yaml`

router 通常不拥有主要阶段 gate。真正的 gate 和批准由它路由到的子阶段负责。

## 你要批准什么

在 router 这一步，主要确认：

- Agent 选中的 `spec_id` 是否正确
- 路由到的阶段是否符合当前工作流状态
- 未解决阻塞项是否仍清楚保留在 handoff 状态中

主要制品的批准通常发生在子阶段，而不是 router 自身。

## 下一步

路由完成后，进入 Agent 选定的子阶段：

- [`spec-forge-intake`](../spec-forge-intake/README.zh-CN.md) 对应 `P0 intake`
- [`spec-forge-architecture`](../spec-forge-architecture/README.zh-CN.md) 对应 `P1 architecture`
- [`spec-forge-journeys`](../spec-forge-journeys/README.zh-CN.md) 对应 `P2 journeys`
- [`spec-forge-components`](../spec-forge-components/README.zh-CN.md) 对应 `P3 components`
- [`spec-forge-readiness`](../spec-forge-readiness/README.zh-CN.md) 对应 `P4 readiness`
- [`spec-forge-implement`](../spec-forge-implement/README.zh-CN.md) 对应 `P5 implement`

如果你不确定，就从 router 开始，让 Agent 来判断下一步。
