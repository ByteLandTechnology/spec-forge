# spec-forge

`spec-forge` 是一个以 YAML 为中心的工作流，用来把模糊需求逐步沉淀为可批准的实施规格，并继续记录可审计的实现结果。它把 agent 引导式阶段流程与一个在 `.spec-forge/` 下持久化状态的 Rust CLI 结合起来。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 它是什么

- 一个从需求梳理推进到实现报告的分阶段工作流。
- 一个把批准记录、gate、focus 和阶段制品持久化到 YAML 的工作区模型。
- 一个名为 [`spec-forge-cli`](./spec-forge-cli/README.zh-CN.md) 的 CLI runtime，也是工作区状态变更的事实来源。

## 什么时候使用

当你需要下面这些能力时，适合使用 `spec-forge`：

- 把模糊需求整理成结构化、可审阅的 spec
- 把规划状态保存为可版本化文件，而不是只留在聊天记录里
- 安全地暂停并恢复多阶段 spec 工作
- 在阶段推进前要求明确批准

如果你只需要命令行入口，请直接查看 [`CLI 文档`](./spec-forge-cli/README.zh-CN.md)。

## 快速上手

### Agent 入口

当你希望 agent 帮你初始化或恢复工作区、找出最早未完成阶段并继续推进时，直接要求它使用 `$spec-forge`。

示例提示语：

- “请使用 `$spec-forge`，把这个需求整理成可批准的 spec。”
- “请在这个仓库里使用 `$spec-forge`，继续 `checkout-redesign`。”
- “请使用 `$spec-forge` 找出下一个未完成阶段并继续。”

### CLI 入口

以下示例命令都从仓库根目录运行。

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write
spec-forge-cli ux validate --target .
```

> [!NOTE]
> Agent 技能负责引导交互，但真正修改持久化工作流状态的权威 runtime 是 `spec-forge-cli`。

## 阶段地图

| 阶段                | 作用                                          |
| ------------------- | --------------------------------------------- |
| `P0 / Intake`       | 明确需求、角色、范围和约束。                  |
| `P1 / Architecture` | 锁定方案轮廓，以及 journey / component 索引。 |
| `P2 / Journeys`     | 按可审阅批次细化 in-scope journey。           |
| `P3 / Components`   | 把 in-scope component 细化成面向实现的契约。  |
| `P4 / Readiness`    | 汇总已批准内容，形成最终实施规格。            |
| `P5 / Implement`    | 记录交付状态、验证结果、阻塞项和收尾信息。    |

## 工作区模型

持久化状态位于 `.spec-forge/`：

```text
.spec-forge/
├── registry.yaml
└── specs/
    └── <spec-id>/
        ├── pipeline-state.yaml
        ├── handoff.yaml
        ├── framing/
        ├── architecture/
        ├── journeys/
        ├── components/
        ├── synthesis/
        └── gates/
```

关键制品：

- `pipeline-state.yaml` 跟踪阶段、focus 和 UX handoff 状态。
- `synthesis/implementation-spec.yaml` 是面向实现的最终契约。
- `synthesis/implementation-report.yaml` 记录交付状态与验证结果。
- `gates/<stage>.yaml` 记录各阶段的 gate 检查输出。

## 仓库结构

以下路径都相对于仓库根目录。

```text
README*.md                         项目入口文档
AGENTS.md                          本仓库的 Agent 说明
spec-forge/                        Router 技能与共享 UX 合同/资源
spec-forge-intake/                 P0 阶段技能
spec-forge-architecture/           P1 阶段技能
spec-forge-journeys/               P2 阶段技能
spec-forge-components/             P3 阶段技能
spec-forge-readiness/              P4 阶段技能
spec-forge-implement/              P5 阶段技能
spec-forge-cli/                    Rust CLI、测试、npm wrapper、发布脚本
specs/001-spec-execution-stage/    AGENTS.md 引用的仓库级上下文
.spec-forge/specs/001-spec-execution-stage/
                                   本仓库中的示例持久化工作流状态
```

## CLI 入口

[`CLI 文档`](./spec-forge-cli/README.zh-CN.md) 包含：

- 安装与平台支持
- 快速上手命令序列
- 命令参考与结构化帮助
- 输出格式与验证命令
- 发布演练与 CI 恢复输入项

## 文档地图

| 范围       | 文档                                                                                                                                                                                                                                                                                                                                                                                                       | 用途                                             |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| 工作流入口 | [`spec-forge` router 指南](./spec-forge/README.zh-CN.md)、[`spec-forge/SKILL.md`](./spec-forge/SKILL.md)                                                                                                                                                                                                                                                                                                   | 启动或恢复工作流，并查看 router 的可执行契约。   |
| 阶段指南   | [`spec-forge-intake`](./spec-forge-intake/README.zh-CN.md)、[`spec-forge-architecture`](./spec-forge-architecture/README.zh-CN.md)、[`spec-forge-journeys`](./spec-forge-journeys/README.zh-CN.md)、[`spec-forge-components`](./spec-forge-components/README.zh-CN.md)、[`spec-forge-readiness`](./spec-forge-readiness/README.zh-CN.md)、[`spec-forge-implement`](./spec-forge-implement/README.zh-CN.md) | 了解 P0-P5 各阶段何时使用，以及 Agent 会问什么。 |
| 阶段契约   | [`intake`](./spec-forge-intake/SKILL.md)、[`architecture`](./spec-forge-architecture/SKILL.md)、[`journeys`](./spec-forge-journeys/SKILL.md)、[`components`](./spec-forge-components/SKILL.md)、[`readiness`](./spec-forge-readiness/SKILL.md)、[`implement`](./spec-forge-implement/SKILL.md)                                                                                                             | 查看每个阶段的执行规则、gate、必填输入和输出。   |
| CLI 包     | [`CLI 文档`](./spec-forge-cli/README.zh-CN.md)、[`npm wrapper`](./spec-forge-cli/npm/main/README.md)、[`发布记录`](./spec-forge-cli/CHANGELOG.md)                                                                                                                                                                                                                                                          | 安装、操作、打包或审计 CLI 分发。                |
| CLI 契约   | [`spec-forge-cli/SKILL.md`](./spec-forge-cli/SKILL.md)                                                                                                                                                                                                                                                                                                                                                     | 查看 CLI 命令面和结构化输出契约。                |
| 仓库上下文 | [`spec execution plan`](./specs/001-spec-execution-stage/plan.md)、[`AGENTS.md`](./AGENTS.md)                                                                                                                                                                                                                                                                                                              | 阅读仓库级上下文和 agent 指令。                  |
