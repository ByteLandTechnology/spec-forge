# spec-forge-readiness

`spec-forge-readiness` は `P4 readiness` です。`P3 components` 承認後に、前段の YAML アーティファクトを最終的な実装準備済み仕様へ統合するときに使います。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-readiness` を使わせます。

- `P3 components` が承認済みである
- 前段の判断を 1 つの実装準備ビューに統合したい
- 今すぐ build 作業を始めてよいか確認したい
- 実装前に未解決事項を明示したまま残したい

このステージは workflow が本当に `P5 implement` へ進めるかを決める場所です。

## 前提条件

readiness を直接使う前に、次を確認してください。

- components が承認済みである
- 前段アーティファクトが存在し、統合できる程度に安定している
- 最終スコープと受け入れ期待をレビューする準備ができている

ステージ選択に迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には readiness を使わないでください。

- 初期 intake や architecture の探索
- journey や component の詳細記述
- 実装作業そのもの
- 前進するためにブロッカーを隠すこと

このステージは正直な統合と gate レビューのためのもので、未解決事項を飛ばすための場所ではありません。

## Agent から聞かれること

readiness では、Agent は次のような内容を確認します。

- 最終スコープの確認
- 受け入れ条件や done の定義
- 実装をまだ止める未解決事項
- 後回しの判断が許容できるか
- 実装開始を承認してよいか

未解決事項は曖昧にせず、見えるまま扱うのが前提です。

## Plan mode

必須の readiness 項目が不足している場合、Agent は Plan mode で次の不足項目を集めてから実装準備済み仕様を確定することがあります。

## YAML アーティファクトと Gate

readiness ステージでは次を維持します。

- `synthesis/implementation-spec.yaml`
- `gates/readiness.yaml`

これは `P5 implement` に進めるかを決める `P4 readiness` gate です。Agent は承認前に、この実装準備済み YAML ビューを要約するべきです。

## あなたが承認すること

次の点が要約で明確なら readiness を承認できます。

- 最終的に実現するスコープ
- 受け入れ条件
- 残っているブロッカーや未解決事項
- 実装を本当に開始してよいか

ブロッカーが残るなら、readiness に留まるか、必要に応じて前段へ戻ってください。

## 次のステップ

readiness 承認後は [`spec-forge-implement`](../spec-forge-implement/README.ja-JP.md) の `P5 implement` へ進みます。
