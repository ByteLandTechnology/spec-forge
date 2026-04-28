# spec-forge-architecture

`spec-forge-architecture` は `P1 architecture` です。`P0 intake` 承認後に使い、詳細ステージへ入る前に解決方針の全体像を固めます。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-architecture` を使わせます。

- `P0 intake` が承認済みである
- 全体的な解決方針をまだ決める必要がある
- 上位レベルの journey 集合がまだ定まっていない
- 上位レベルの component 集合がまだ定まっていない

このステージは `P2 journeys` や `P3 components` より前に行います。

## 前提条件

architecture を直接使う前に、次を確認してください。

- intake framing が承認済みである
- 問題発見ではなく、解決方針の議論に入る準備ができている
- 欲しいのが上位構造であり、低レベル実装詳細ではない

ステージ選択に迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には architecture を使わないでください。

- 初期の問題 framing
- journey フローの詳細化
- component 契約の詳細化
- 実装準備の最終確認や成果記録

このステージはシステムの形を定める場所であり、すべての flow や component 詳細を書き切る場所ではありません。

## Agent から聞かれること

architecture では、Agent は次のような内容を確認します。

- 望ましい解決方針
- 主要な境界と依存関係
- 上位レベルのユーザーまたはシステム journey
- 上位レベルの component またはサブシステム
- 後続の詳細化に影響するリスクや判断

目的はシステムの輪郭を固めることであり、全実装詳細を決めることではありません。

## Plan mode

architecture の入力が不足している場合、Agent は Plan mode で次の不足項目を集めつつ、議論を適切な抽象度に保ちます。

## YAML アーティファクトと Gate

architecture ステージでは次を維持します。

- `architecture/solution-outline.yaml`
- `architecture/journey-index.yaml`
- `architecture/component-index.yaml`
- 詳細ステージへ進めるかを決める `P1 architecture` gate 結果

Agent は承認前に、これらの YAML アーティファクトを要約するべきです。

## あなたが承認すること

次の点が明確であれば architecture を承認できます。

- 意図している解決方針
- 主要な境界と前提
- 上位レベルでの完全な in-scope journey 集合
- 上位レベルでの完全な in-scope component 集合

全体像がまだ不安定なら、先へ進まないでください。

## 次のステップ

architecture 承認後は [`spec-forge-journeys`](../spec-forge-journeys/README.ja-JP.md) の `P2 journeys` へ進みます。
