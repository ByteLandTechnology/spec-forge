# spec-forge-readiness

`spec-forge-readiness` 是 `P4 readiness`。它用于在 `P3 components` 已批准后，把前面阶段的 YAML 制品汇总成最终可实施规格。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-readiness`：

- `P3 components` 已批准
- 需要把前序阶段结论汇总成一个实现就绪视图
- 你想确认现在是否可以开始构建
- 在实现开始前，仍需把未决事项明确保留下来

这个阶段决定工作流是否真的可以进入 `P5 implement`。

## 前置条件

直接使用 readiness 前，请确认：

- components 已经批准
- 前序阶段制品已经存在，且稳定到可以汇总
- 你已经准备好审查最终范围和验收预期

如果不确定阶段选择，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 readiness 用于：

- 初始 intake 或 architecture 探索
- 详细 journey 或 component 编写
- 直接执行实现工作
- 为了推进而掩盖阻塞项

这个阶段负责诚实汇总和 gate 审查，不是跳过未解决问题的地方。

## Agent 可能会问什么

在 readiness 中，Agent 可能会问：

- 最终范围是否确认
- 验收标准或完成定义是什么
- 还有哪些未决事项仍阻塞实现
- 某些延后决策是否可以接受
- 现在是否已经可以批准进入实现

Agent 应把未解决问题明确保留下来，而不是模糊带过。

## Plan mode

如果 readiness 必填字段缺失，Agent 可能会使用 Plan mode 先补齐当前缺失项，再完成最终实现就绪规格。

## YAML 制品与 Gate

readiness 阶段会维护：

- `synthesis/implementation-spec.yaml`
- `gates/readiness.yaml`

这里就是决定是否可以进入 `P5 implement` 的 `P4 readiness` gate。在请求批准前，Agent 应先总结这个实现就绪 YAML 视图。

## 你要批准什么

当总结清楚说明以下内容时，可以批准 readiness：

- 最终计划范围
- 验收标准
- 剩余阻塞项或未决问题
- 是否真的已经可以开始实现

如果仍有阻塞项，就继续停留在 readiness，或者回到更早阶段，而不是强行推进。

## 下一步

readiness 批准后，进入 [`spec-forge-implement`](../spec-forge-implement/README.zh-CN.md) 的 `P5 implement`。
