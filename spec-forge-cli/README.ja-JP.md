# spec-forge-cli

`spec-forge-cli` は `spec-forge` YAML ワークフローを操作する権威 Rust ランタイムです。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## 概要

- パッケージ名: `@cli-forge-bin/spec-forge-cli`
- ソースリポジトリ: `ByteLandTechnology/spec-forge`
- Rust edition: `2024`
- 最低 Rust バージョン: `1.85`
- リリース演習の実行要件: Node `^22.14.0 || >=24.10.0`, npm `>=10`

ワークスペース初期化、次の対話解決、成果物更新、承認記録、gate 判定、ステージ進行、UX 契約検証が必要なときに使います。

## インストール

### npm から

```bash
npm install -g @cli-forge-bin/spec-forge-cli
```

npm パッケージは次の公開済みターゲット向けネイティブバイナリを自動選択します。

- `darwin-arm64`
- `darwin-x64`
- `linux-arm64`
- `linux-x64`
- `win32-arm64`
- `win32-x64`

`postinstall` による追加ダウンロードは不要です。

### リポジトリを取得済みの場合の代替

チェックアウト済みリポジトリ内の `spec-forge-cli/` ディレクトリから実行します。

```bash
./scripts/install-current-release.sh
```

すでにこのリポジトリをチェックアウトしていて、npm registry を経由せずにリリース済みタグからローカル導入したい場合の補助手段です。

## クイックスタート

以下の例はリポジトリルートから実行し、`--target .` がリポジトリのワークスペースを指すようにします。

```bash
spec-forge-cli init --target . --spec-id demo --request-title "Demo Spec"
spec-forge-cli resolve --target . --spec-id demo --skill spec-forge --stage router --write --format json
spec-forge-cli ux validate --target .
spec-forge-cli help gate check --format json
```

## コマンド一覧

| コマンド         | 役割                                                                      |
| ---------------- | ------------------------------------------------------------------------- |
| `init`           | `.spec-forge/`、spec ごとの YAML 骨格、初期 gate 状態を作成します。       |
| `resolve`        | 現在のワークスペース状態から次に必要な対話または ready 状態を解決します。 |
| `apply`          | 1 件の回答または選択を永続化します。                                      |
| `focus`          | 次に扱う journey / component バッチを選びます。                           |
| `artifact get`   | 解決済み spec ワークスペースから 1 つの成果物を読みます。                 |
| `artifact put`   | CLI 経由で 1 つの成果物を置き換えまたは作成します。                       |
| `artifact merge` | CLI 経由で 1 つの成果物を増分更新します。                                 |
| `approve`        | 成果物に明示的な approval ブロックを記録します。                          |
| `gate check`     | 永続化された YAML からステージ gate を評価します。                        |
| `stage advance`  | 現在の gate が通った後にだけステージを進めます。                          |
| `ux validate`    | 共有 UX 契約と agent メタデータの整合性を検証します。                     |
| `help`           | 人向けまたは構造化形式でヘルプを表示します。                              |

## よく使う操作

代表的なコマンド:

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

## 出力形式

構造化コマンドは次をサポートします。

- `--format yaml`
- `--format json`
- `--format toml`

端末向けヘルプは `--help`、機械可読のコマンド情報は `spec-forge-cli help <command> --format json` を使います。

例:

```bash
spec-forge-cli help gate check --format json
```

## 検証

以下の検証コマンドはリポジトリルートから実行します。

```bash
cargo test --manifest-path spec-forge-cli/Cargo.toml
cargo fmt --manifest-path spec-forge-cli/Cargo.toml --check
cargo clippy --manifest-path spec-forge-cli/Cargo.toml -- -D warnings
cargo package --manifest-path spec-forge-cli/Cargo.toml --offline
spec-forge-cli ux validate --target .
```

## リリース事前演習と復旧入口

release harness は、パッケージ準備やリリース経路の確認時だけ使います。
この節の release harness コマンドは `spec-forge-cli/` ディレクトリから実行します。

本番リリース前にパッケージングを演習するには:

```bash
npm ci
npm run release:rehearse
```

初回の本番公開前に、リリース資産と npm 前提条件を整えるには:

```bash
npm run release:prepublish
```

復旧入口:

- 依存関係不足や認証不足で演習が失敗した場合は、環境を修正して `npm ci` と `npm run release:rehearse` を再実行します。
- 既存 tag の後で CI リリースが止まった場合は、`Release` GitHub Actions workflow を再実行し、必須入力 `recover-version` を指定します。元のビルド成果物を再利用する必要がある場合は、任意入力 `recover-run-id` も指定します。
- 既に公開済みのタグからローカル導入をやり直したい場合は、`spec-forge-cli/` ディレクトリで `./scripts/install-current-release.sh` を再実行します。

> [!NOTE]
> `release:rehearse` はパッケージング確認専用であり、Git tag、GitHub Release、実際の npm publish は作成しません。
