# spec-forge-journeys

`spec-forge-journeys` 是 `P2 journeys`。它用于在 `P1 architecture` 已批准后，按批次把 journey 细化成可审阅的 YAML。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-journeys`：

- `P1 architecture` 已批准
- journey index 已存在，但 journey 细节还不完整
- 你希望一次处理一个 journey 批次
- 需要把流程、状态、规则和边界情况明确写出来

这个阶段用于细化 journey，不是重新定义整个系统轮廓。

## 前置条件

直接使用 journeys 前，请确认：

- architecture 已经批准
- in-scope 的 journey 集合已经确定
- 你需要的是详细流程制品，而不是顶层方向讨论

如果不确定阶段选择，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 journeys 用于：

- 初始 intake framing
- 顶层 architecture 探索
- 详细 component 契约
- 最终 readiness 批准或实现结果记录

这个阶段负责细化 journey 行为，不适合随意重新打开系统轮廓讨论。

## Agent 可能会问什么

针对每条 journey，Agent 可能会问：

- 触发条件和前置条件
- 主流程
- 分支流程或异常流程
- loading、empty、success、error 等状态
- 业务规则和边界情况
- 关联的触点、系统或组件
- 可观测性或验收预期

Agent 应把讨论聚焦在当前 journey 批次。

## Plan mode

如果某条 journey 还缺少必填细节，Agent 可能会使用 Plan mode 先补齐当前缺失项，再继续这个批次。

## YAML 制品与 Gate

journeys 阶段会维护：

- `journeys/batches.yaml`
- 每个 in-scope journey 对应一个 `journeys/<journey-id>.yaml`
- 决定是否可以进入 `P3 components` 的 `P2 journeys` gate 结果

在请求批准前，Agent 应先总结当前这批 YAML 制品。

## 你要批准什么

当总结准确反映以下内容时，可以批准某个 journey 批次：

- 每条 journey 如何开始
- 主流程和分支流程
- 关键状态和失败路径
- 重要规则和边界情况
- 这些 journey 如何连接到系统其他部分

重复这个过程，直到所有 in-scope journey 都已批准。

## 下一步

journeys 批准后，进入 [`spec-forge-components`](../spec-forge-components/README.zh-CN.md) 的 `P3 components`。
