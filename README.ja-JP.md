# spec-forge

`spec-forge` は、曖昧な依頼を承認可能な実装仕様へ整理し、その後の実装結果まで YAML で追跡するワークフローです。agent による段階的な進行案内と、`.spec-forge/` に永続状態を保存する Rust CLI を組み合わせています。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## これは何か

- 依頼整理から実装報告までを扱う段階型ワークフロー
- 承認、gate、focus、成果物を YAML に保存するワークスペースモデル
- 状態変更の権威ランタイムである [`spec-forge-cli`](./spec-forge-cli/README.ja-JP.md)

## 使う場面

`spec-forge` は次のようなときに向いています。

- あいまいな要求を構造化されレビューしやすい spec にしたい
- 計画状態をチャットではなく版管理可能なファイルに残したい
- 複数段階の spec 作業を安全に中断・再開したい
- ステージ進行の前に明示的な承認を入れたい

コマンドライン操作だけが必要なら [`CLI ドキュメント`](./spec-forge-cli/README.ja-JP.md) を参照してください。

## クイックスタート

### Agent 入口

ワークスペースの初期化や再開、最初の未完了ステージの特定、その先の継続を agent に任せたい場合は `$spec-forge` を使うよう依頼します。

依頼例:

- 「`$spec-forge` を使って、この要求を承認可能な spec にしてください」
- 「このリポジトリで `$spec-forge` を使い、`checkout-redesign` を続けてください」
- 「`$spec-forge` を使って次の未完了ステージを見つけて進めてください」

### CLI 入口

以下の例はすべてリポジトリルートから実行します。

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write
spec-forge-cli ux validate --target .
```

> [!NOTE]
> agent スキルは対話を案内しますが、永続化されたワークフロー状態を変更する権威は `spec-forge-cli` です。

## ステージマップ

| ステージ            | 目的                                                |
| ------------------- | --------------------------------------------------- |
| `P0 / Intake`       | 依頼、関係者、スコープ、制約を整理します。          |
| `P1 / Architecture` | 全体方針と journey / component index を固めます。   |
| `P2 / Journeys`     | 対象 journey をレビューしやすい単位で詳細化します。 |
| `P3 / Components`   | 対象 component を実装向け契約へ落とし込みます。     |
| `P4 / Readiness`    | 承認済み内容を最終的な実装仕様へ統合します。        |
| `P5 / Implement`    | 実装状況、検証、ブロッカー、完了記録を残します。    |

## ワークスペースモデル

永続状態は `.spec-forge/` に保存されます。

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

主な成果物:

- `pipeline-state.yaml` はステージ、focus、UX handoff 状態を追跡します。
- `synthesis/implementation-spec.yaml` は実装向けの最終契約です。
- `synthesis/implementation-report.yaml` は実装状況と検証結果を記録します。
- `gates/<stage>.yaml` は各ステージの gate 判定結果を記録します。

## リポジトリ構成

以下のパスはすべてリポジトリルート基準です。

```text
README*.md                         プロジェクト入口ドキュメント
AGENTS.md                          このリポジトリ向け agent 指示
spec-forge/                        ルーター技能と共有 UX 契約/資産
spec-forge-intake/                 P0 ステージ技能
spec-forge-architecture/           P1 ステージ技能
spec-forge-journeys/               P2 ステージ技能
spec-forge-components/             P3 ステージ技能
spec-forge-readiness/              P4 ステージ技能
spec-forge-implement/              P5 ステージ技能
spec-forge-cli/                    Rust CLI、テスト、npm wrapper、リリーススクリプト
specs/001-spec-execution-stage/    AGENTS.md が参照するリポジトリ文脈
.spec-forge/specs/001-spec-execution-stage/
                                   このリポジトリ内の永続ワークフロー状態サンプル
```

## CLI 入口

[`CLI ドキュメント`](./spec-forge-cli/README.ja-JP.md) には次がまとまっています。

- インストールと対応プラットフォーム
- クイックスタートのコマンド列
- コマンド参照と構造化ヘルプ
- 出力形式と検証手順
- リリース事前演習と CI 復旧入力

## ドキュメントマップ

| 範囲             | ドキュメント                                                                                                                                                                                                                                                                                                                                                                                               | 用途                                                        |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| ワークフロー入口 | [`spec-forge` router ガイド](./spec-forge/README.ja-JP.md)、[`spec-forge/SKILL.md`](./spec-forge/SKILL.md)                                                                                                                                                                                                                                                                                                 | ワークフローの開始や再開、router の実行契約の確認。         |
| ステージガイド   | [`spec-forge-intake`](./spec-forge-intake/README.ja-JP.md)、[`spec-forge-architecture`](./spec-forge-architecture/README.ja-JP.md)、[`spec-forge-journeys`](./spec-forge-journeys/README.ja-JP.md)、[`spec-forge-components`](./spec-forge-components/README.ja-JP.md)、[`spec-forge-readiness`](./spec-forge-readiness/README.ja-JP.md)、[`spec-forge-implement`](./spec-forge-implement/README.ja-JP.md) | P0-P5 各ステージを使う場面と Agent から聞かれる内容の確認。 |
| ステージ契約     | [`intake`](./spec-forge-intake/SKILL.md)、[`architecture`](./spec-forge-architecture/SKILL.md)、[`journeys`](./spec-forge-journeys/SKILL.md)、[`components`](./spec-forge-components/SKILL.md)、[`readiness`](./spec-forge-readiness/SKILL.md)、[`implement`](./spec-forge-implement/SKILL.md)                                                                                                             | 各ステージの実行ルール、gate、必須入力、出力の確認。        |
| CLI パッケージ   | [`CLI ドキュメント`](./spec-forge-cli/README.ja-JP.md)、[`npm wrapper`](./spec-forge-cli/npm/main/README.md)、[`リリース記録`](./spec-forge-cli/CHANGELOG.md)                                                                                                                                                                                                                                              | CLI の導入、操作、パッケージング、配布内容の確認。          |
| CLI 契約         | [`spec-forge-cli/SKILL.md`](./spec-forge-cli/SKILL.md)                                                                                                                                                                                                                                                                                                                                                     | CLI コマンド面と構造化出力契約の確認。                      |
| リポジトリ文脈   | [`spec execution plan`](./specs/001-spec-execution-stage/plan.md)、[`AGENTS.md`](./AGENTS.md)                                                                                                                                                                                                                                                                                                              | リポジトリ単位の文脈と agent 指示の確認。                   |
