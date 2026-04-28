# spec-forge Router

`spec-forge` router は、Agent が spec-forge workflow を安全に開始または再開するための標準入口です。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のようなときは Agent に `$spec-forge` を使わせます。

- 正しいステージを推測せずに新しい workflow を始めたい
- 既存 workflow を最も早い未完了ステージから再開したい
- 不足している呼び出し情報を 1 つずつ埋めたい
- `P0-P5` の順序と approval gate を守って進めたい

この router が skill family の通常入口です。

## 前提条件

次の情報を出せると router が動きやすくなります。

- 対象リポジトリまたは作業ディレクトリ
- 既存であれば `spec_id`
- 新規 spec を作るか、既存 spec を続けるか

次にどの子ステージへ進むかを事前に知っている必要はありません。

## 使わない場面

次の目的には router を使わないでください。

- コマンドラインの使い方だけを知りたい
- 進むべき子ステージが明確で、前段ステージがすべて承認済みである
- ステージ gate や承認を飛ばしたい

CLI の使い方は [`CLI ドキュメント`](../spec-forge-cli/README.ja-JP.md) を参照してください。

## Agent から聞かれること

router は次のような内容を確認することがあります。

- 対象リポジトリまたは作業ディレクトリ
- `spec_id`
- 新規 spec を作るか、既存 spec を続けるか
- 現在不足している次の必須呼び出し項目

複数の spec がある場合は、対象の `spec_id` を明示してください。

## Plan mode

必要な呼び出し情報が不足している場合、router は Plan mode で 1 件ずつ集めることがあります。毎回、次に不足している項目だけを聞いてから先へ進む想定です。

## YAML アーティファクトと Gate

router 自体は深いステージ作業を行いません。代わりに次の workflow 状態を準備または更新します。

- `.spec-forge/registry.yaml`
- `.spec-forge/specs/<spec-id>/pipeline-state.yaml`
- `.spec-forge/specs/<spec-id>/handoff.yaml`

router は通常、主要な stage gate を持ちません。次の gate と approval は、router が遷移させる子ステージが担当します。

## あなたが承認すること

router の段階では、次を確認してください。

- Agent が正しい `spec_id` を選んだか
- ルーティング先が現在の workflow 状態に合っているか
- 未解決ブロッカーが handoff 状態で見えるままか

主要アーティファクトの承認は通常、router 自体ではなく子ステージで行われます。

## 次のステップ

ルーティング後は、Agent が選んだ子ステージへ進みます。

- [`spec-forge-intake`](../spec-forge-intake/README.ja-JP.md) は `P0 intake`
- [`spec-forge-architecture`](../spec-forge-architecture/README.ja-JP.md) は `P1 architecture`
- [`spec-forge-journeys`](../spec-forge-journeys/README.ja-JP.md) は `P2 journeys`
- [`spec-forge-components`](../spec-forge-components/README.ja-JP.md) は `P3 components`
- [`spec-forge-readiness`](../spec-forge-readiness/README.ja-JP.md) は `P4 readiness`
- [`spec-forge-implement`](../spec-forge-implement/README.ja-JP.md) は `P5 implement`

迷う場合は router から始め、次の判断を Agent に任せてください。
