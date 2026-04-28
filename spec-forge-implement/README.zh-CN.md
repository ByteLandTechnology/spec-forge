# spec-forge-implement

`spec-forge-implement` 是 `P5 implement`。它用于在 `P4 readiness` 已批准后，根据已批准的 spec 记录实现结果。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 什么时候使用

当出现以下情况时，让 Agent 使用 `$spec-forge-implement`：

- `P4 readiness` 已批准
- 实现工作正在按已批准 spec 推进
- 你希望工作流如实记录改动、验证和阻塞项
- 你需要正式的最终阶段记录，而不只是聊天总结

这个阶段在 readiness 批准后记录交付状态。

## 前置条件

直接使用 implement 前，请确认：

- readiness 已经批准
- implementation-ready spec 已存在
- 你要记录的是真实交付状态，而不是随意重开范围讨论

如果不确定阶段选择，请先看 [`router`](../spec-forge/README.zh-CN.md)。

## 不适用场景

不要把 implement 用于：

- 初始 intake、architecture、journey 或 component 设计
- 最终 readiness 汇总
- 不经过前序批准就重写范围
- 为了关闭工作流而掩盖验证失败

这个阶段负责诚实记录实现结果。

## Agent 可能会问什么

在 implement 中，Agent 可能会问：

- 实现是否已经完成
- 实际做了哪些改动
- 影响了哪些文件或区域
- 跑了哪些验证，结果如何
- 是否还有阻塞项或后续工作

Agent 应如实记录结果，包括验证失败或工作未完成的情况。

## Plan mode

如果实现报告必填字段缺失，Agent 可能会使用 Plan mode 先补齐当前缺失项，再完成最终实现报告。

## YAML 制品与 Gate

implement 阶段会维护：

- `synthesis/implementation-report.yaml`
- `gates/implement.yaml`

这里就是 `P5 implement` gate。在请求最终批准前，Agent 应先总结这些交付状态 YAML 制品。

## 你要批准什么

当总结清楚记录以下内容时，可以批准 implement：

- 工作是否完成
- 实际改动范围是什么
- 做了哪些验证
- 还剩哪些阻塞项、风险或后续事项

工作流不应为了关闭阶段而掩盖失败检查或未解决阻塞项。

## 下一步

如果 implement 阶段通过，整个工作流即可关闭为完成状态。
