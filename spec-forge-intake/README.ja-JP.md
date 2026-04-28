# spec-forge-intake

`spec-forge-intake` は `P0 intake` です。Agent が粗い依頼を、設計開始前の承認済み framing YAML に変える必要があるときに使います。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-intake` を使わせます。

- 依頼がまだ粗い、または曖昧である
- 問題、目標、利用者、スコープがまだ明確でない
- 設計に入る前に重要な制約を記録したい
- 後続ステージのレビュー役割を先に決めたい

この子スキルを直接使うのは、workflow がすでに `P0 intake` にある場合か、意図的に intake の中だけで進めたい場合に限ります。

## 前提条件

intake を直接使う前に、次を確認してください。

- 正しいステージ判定のために router を使う必要がない
- 先に承認すべき後続ステージの成果物がない
- framing、スコープ、制約、レビュー役割を定義する準備ができている

迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には intake を使わないでください。

- 解決方針や architecture の設計
- journey ごとの詳細化
- component 契約の定義
- 実装準備の最終確認や実装結果の記録

設計が始まった後でも intake を見直すことはありますが、このスキルは後続ステージへ飛ぶ近道ではありません。

## Agent から聞かれること

intake では、Agent は通常次のような内容を確認します。

- 解決したい問題
- 目指す結果
- 主な利用者や関係者
- スコープ内とスコープ外の項目
- 重要な制約、リスク、譲れない条件
- 後続でも関与すべきレビュー役割

一度に全部ではなく、次に不足している答えだけを聞く想定です。

## Plan mode

必須の intake 項目が不足している場合、Agent は Plan mode で 1 つずつ集めることがあります。

## YAML アーティファクトと Gate

intake ステージでは次を維持します。

- `framing/request-context.yaml`
- `framing/role-map.yaml`
- `P1 architecture` に進めるかを判断する `P0 intake` gate 結果

Agent は承認前に、これらの YAML アーティファクトを要約するべきです。

## あなたが承認すること

次の点が正確に整理されていれば intake を承認できます。

- 何の問題を解くのか
- 何を成功とするのか
- 誰のための spec なのか
- 重要な境界や除外事項は何か
- どのレビュー役割が今後も関与すべきか

まだ曖昧なら、先へ進まず intake を続けてください。

## 次のステップ

intake 承認後は [`spec-forge-architecture`](../spec-forge-architecture/README.ja-JP.md) の `P1 architecture` へ進みます。
