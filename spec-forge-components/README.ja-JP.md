# spec-forge-components

`spec-forge-components` は `P3 components` です。`P2 journeys` 承認後に、各 component を実装向け契約へ詳細化するときに使います。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-components` を使わせます。

- `P2 journeys` が承認済みである
- component index はあるが、component 詳細がまだ不足している
- 1 回に 1 つの component バッチずつ進めたい
- インターフェース、状態、依存関係、運用期待を明示したい

このステージは component 契約のためのものであり、依頼を最初から再発見する場ではありません。

## 前提条件

components を直接使う前に、次を確認してください。

- journeys が承認済みである
- in-scope な component 集合が定義済みである
- 必要なのが上位 architecture 議論ではなく、実装向け component 詳細である

ステージ選択に迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には components を使わないでください。

- 初期 intake framing
- 上位の解決方針決定
- journey フロー探索
- readiness の最終承認や成果記録

このステージは component 契約を詰める場所であり、依頼全体や architecture を気軽に開き直す場所ではありません。

## Agent から聞かれること

各 component について、Agent は次のような内容を確認します。

- 何を担当し、何を担当しないか
- 入力、出力、インターフェース、イベント
- 状態の所有者と source of truth
- 他システムや他 component への依存
- 検証ルール、順序制約、失敗時の振る舞い
- 性能、セキュリティ、アクセシビリティ、可観測性の期待
- テスト、ロールアウト、移行、flag の期待

議論は現在の component バッチに集中させる想定です。

## Plan mode

component 契約に必須の詳細が不足している場合、Agent は Plan mode で次の不足項目を集めてから先へ進むことがあります。

## YAML アーティファクトと Gate

components ステージでは次を維持します。

- `components/batches.yaml`
- 各 in-scope component に対応する `components/<component-id>.yaml`
- `P4 readiness` に進めるかを決める `P3 components` gate 結果

Agent は承認前に現在の YAML バッチを要約するべきです。

## あなたが承認すること

次の点が要約で明確になっていれば、その component バッチを承認できます。

- 各 component が何を担当するか
- 各 component が何を担当しないか
- ほかのシステム要素とどう連携するか
- 重要な状態、検証、失敗時の期待
- 重要なロールアウトやテスト期待

すべての in-scope component が承認されるまで、この流れを繰り返します。

## 次のステップ

components 承認後は [`spec-forge-readiness`](../spec-forge-readiness/README.ja-JP.md) の `P4 readiness` へ進みます。
