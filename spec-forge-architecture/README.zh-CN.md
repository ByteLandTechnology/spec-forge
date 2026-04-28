# spec-forge-architecture

`spec-forge-architecture` 是 `P1 architecture`。它用于在 `P0 intake` 已批准后，先锁定整体方案轮廓，再开始更细的阶段工作。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-architecture`：

- `P0 intake` 已批准
- 整体方案方向还需要定义
- 顶层 journey 集合还没有确定
- 顶层 component 集合还没有确定

这个阶段应先于 `P2 journeys` 和 `P3 components`。

## 前置条件

直接使用 architecture 前，请确认：

- intake framing 已经批准
- 当前已经准备好讨论方案轮廓，而不是继续澄清问题
- 你要确定的是顶层结构，而不是底层实现细节

如果不确定阶段选择，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 architecture 用于：

- 初始问题 framing
- 细化某条 journey 的流程
- 细化某个 component 契约
- 最终实现就绪判断或交付结果记录

这个阶段负责定义系统轮廓，不是一次性写完所有 flow 或 component 细节的地方。

## Agent 可能会问什么

在 architecture 中，Agent 可能会问：

- 倾向采用什么整体方案
- 关键边界和依赖是什么
- 顶层的用户或系统 journey 有哪些
- 顶层的组件或子系统有哪些
- 哪些风险或决策会影响后续细化

目标是先确定系统轮廓，而不是讨论全部实现细节。

## Plan mode

如果 architecture 输入不完整，Agent 可能会使用 Plan mode 逐项补齐，并把讨论维持在合适抽象层级。

## YAML 制品与 Gate

architecture 阶段会维护：

- `architecture/solution-outline.yaml`
- `architecture/journey-index.yaml`
- `architecture/component-index.yaml`
- 决定是否可以进入详细阶段工作的 `P1 architecture` gate 结果

在请求批准前，Agent 应先总结这些 YAML 制品。

## 你要批准什么

当总结清楚表达以下内容时，可以批准 architecture：

- 预期的方案方向
- 关键边界和主要假设
- 顶层完整的 in-scope journey 集合
- 顶层完整的 in-scope component 集合

如果整体形态还没有稳定，就不要继续往后推进。

## 下一步

architecture 批准后，进入 [`spec-forge-journeys`](../spec-forge-journeys/README.zh-CN.md) 的 `P2 journeys`。
