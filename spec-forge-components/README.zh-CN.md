# spec-forge-components

`spec-forge-components` 是 `P3 components`。它用于在 `P2 journeys` 已批准后，把每个 component 细化成面向实现的契约。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-components`：

- `P2 journeys` 已批准
- component index 已存在，但 component 细节还不完整
- 你希望一次处理一个 component 批次
- 需要把接口、状态、依赖或运行期预期明确写出来

这个阶段用于定义 component 契约，不是重新从头澄清需求。

## 前置条件

直接使用 components 前，请确认：

- journeys 已经批准
- in-scope 的 component 集合已经确定
- 你需要的是面向实现的 component 细节，而不是顶层 architecture 讨论

如果不确定阶段选择，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 components 用于：

- 初始 intake framing
- 顶层方案轮廓决策
- journey 流程探索
- 最终 readiness 批准或交付结果记录

这个阶段负责细化 component 契约，不适合随意重新打开完整需求或整体 architecture。

## Agent 可能会问什么

针对每个 component，Agent 可能会问：

- 它负责什么，不负责什么
- 有哪些输入、输出、接口或事件
- 状态归谁管理，source of truth 是什么
- 依赖哪些系统或其他组件
- 校验规则、顺序要求或失败行为是什么
- 对性能、安全、可访问性或可观测性有什么预期
- 对测试、发布、迁移或 flag 有什么要求

Agent 应把讨论聚焦在当前 component 批次。

## Plan mode

如果某个 component 契约还缺少必填细节，Agent 可能会使用 Plan mode 先补齐当前缺失项，再继续推进。

## YAML 制品与 Gate

components 阶段会维护：

- `components/batches.yaml`
- 每个 in-scope component 对应一个 `components/<component-id>.yaml`
- 决定是否可以进入 `P4 readiness` 的 `P3 components` gate 结果

在请求批准前，Agent 应先总结当前这批 YAML 制品。

## 你要批准什么

当总结清楚覆盖以下内容时，可以批准某个 component 批次：

- 每个 component 负责什么
- 每个 component 不负责什么
- 它如何与系统其他部分协作
- 关键状态、校验和失败预期
- 重要的发布或测试预期

重复这个过程，直到所有 in-scope component 都已批准。

## 下一步

components 批准后，进入 [`spec-forge-readiness`](../spec-forge-readiness/README.zh-CN.md) 的 `P4 readiness`。
