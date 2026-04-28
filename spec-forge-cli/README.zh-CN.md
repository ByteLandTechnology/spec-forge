# spec-forge-cli

`spec-forge-cli` 是 `spec-forge` YAML 工作流的权威 Rust runtime。

语言版本：

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 概览

- npm 包名：`@cli-forge-bin/spec-forge-cli`
- 源仓库：`ByteLandTechnology/spec-forge`
- Rust edition：`2024`
- 最低 Rust 版本：`1.85`
- 发布演练环境要求：Node `^22.14.0 || >=24.10.0`，npm `>=10`

当你需要初始化工作区、解析下一步交互、更新制品、记录批准、检查 gate、推进阶段或校验 UX 合同时，就使用这个 CLI。

## 安装

### 通过 npm 安装

```bash
npm install -g @cli-forge-bin/spec-forge-cli
```

npm 包会为以下已发布目标自动选择匹配的原生二进制：

- `darwin-arm64`
- `darwin-x64`
- `linux-arm64`
- `linux-x64`
- `win32-arm64`
- `win32-x64`

不需要 `postinstall` 下载。

### 克隆仓库后的备用安装

在已 checkout 的仓库中，从 `spec-forge-cli/` 目录运行这个脚本。

```bash
./scripts/install-current-release.sh
```

如果你已经 checkout 了仓库，并且想基于某个已发布 tag 进行本地安装而不经过 npm registry，可以使用这个脚本。

## 快速上手

以下示例命令从仓库根目录运行，这样 `--target .` 才会指向仓库工作区。

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write --format json
spec-forge-cli ux validate --target .
spec-forge-cli help gate check --format json
```

## 命令表

| 命令             | 作用                                                          |
| ---------------- | ------------------------------------------------------------- |
| `init`           | 创建 `.spec-forge/`、每个 spec 的 YAML 骨架和初始 gate 状态。 |
| `resolve`        | 根据当前工作区状态解析下一步需要的交互或 ready 状态。         |
| `apply`          | 持久化一个参数回答或一次选择结果。                            |
| `focus`          | 选择下一批要审阅的 journey 或 component。                     |
| `artifact get`   | 读取一个已解析 spec 工作区下的制品。                          |
| `artifact put`   | 通过 CLI 替换或创建一个制品。                                 |
| `artifact merge` | 通过 CLI 增量合并一个制品。                                   |
| `approve`        | 为制品写入明确的 approval 块。                                |
| `gate check`     | 基于持久化 YAML 状态评估阶段 gate。                           |
| `stage advance`  | 仅在当前 gate 通过后推进阶段。                                |
| `ux validate`    | 校验共享 UX 合同与 agent 元数据是否一致。                     |
| `help`           | 以人类可读或结构化形式查看帮助。                              |

## 常用操作

代表性命令：

```bash
spec-forge-cli apply --target . --spec-id demo --stage intake --parameter problem_goal --value '"Ship the workflow"'
spec-forge-cli focus --target . --spec-id demo --stage journeys --write
spec-forge-cli artifact get --target . --spec-id demo --file framing/request-context.yaml
spec-forge-cli artifact put --target . --spec-id demo --file journeys/journey-alpha.yaml --value '{}'
spec-forge-cli artifact merge --target . --spec-id demo --file framing/request-context.yaml --value '{problem:{goal:"Ship the native CLI"}}'
spec-forge-cli approve --target . --spec-id demo --file architecture/solution-outline.yaml --note 'Approved after review.'
spec-forge-cli gate check --target . --spec-id demo --stage intake --write
spec-forge-cli stage advance --target . --spec-id demo --stage intake
```

## 输出格式

结构化命令支持：

- `--format yaml`
- `--format json`
- `--format toml`

需要终端帮助时用 `--help`；需要机器可读的命令元数据时，用 `spec-forge-cli help <command> --format json`。

示例：

```bash
spec-forge-cli help gate check --format json
```

## 验证

以下验证命令从仓库根目录运行。

```bash
cargo test --manifest-path spec-forge-cli/Cargo.toml
cargo fmt --manifest-path spec-forge-cli/Cargo.toml --check
cargo clippy --manifest-path spec-forge-cli/Cargo.toml -- -D warnings
cargo package --manifest-path spec-forge-cli/Cargo.toml --offline
spec-forge-cli ux validate --target .
```

## 发布演练与恢复入口

只有在准备打包或验证发布路径时才需要 release harness。
本节的 release harness 命令都从 `spec-forge-cli/` 目录运行。

正式发布前，先做一次打包演练：

```bash
npm ci
npm run release:rehearse
```

首次生产发布前，先准备发布资产和 npm 前置条件：

```bash
npm run release:prepublish
```

恢复入口：

- 如果演练因为依赖或认证缺失而失败，修复环境后重新执行 `npm ci` 和 `npm run release:rehearse`。
- 如果 CI 发布在 tag 已经存在后卡住，请重新运行 `Release` GitHub Actions workflow，并填写必需的 `recover-version` 输入；如果需要复用原始构建产物，再填写可选的 `recover-run-id` 输入。
- 如果你需要从一个已经发布的 tag 重新做本地安装，请在 `spec-forge-cli/` 目录重新运行 `./scripts/install-current-release.sh`。

> [!NOTE]
> `release:rehearse` 只验证打包流程，不会创建 Git tag、GitHub Release 或真实 npm 发布。
