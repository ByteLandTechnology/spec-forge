# spec-forge-journeys

`spec-forge-journeys` は `P2 journeys` です。`P1 architecture` 承認後に、journey をレビュー可能な YAML へバッチ単位で詳細化するときに使います。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-journeys` を使わせます。

- `P1 architecture` が承認済みである
- journey index はあるが、journey 詳細がまだ不足している
- 1 回に 1 つの journey バッチずつ進めたい
- フロー、状態、ルール、エッジケースを明示したい

このステージは journey の詳細化のためのものであり、システム全体像を作り直す場ではありません。

## 前提条件

journeys を直接使う前に、次を確認してください。

- architecture が承認済みである
- in-scope な journey 集合が定義済みである
- 必要なのが上位方針ではなく、詳細フロー成果物である

ステージ選択に迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には journeys を使わないでください。

- 初期 intake framing
- 上位 architecture の探索
- component 契約の詳細化
- readiness の最終承認や実装結果の記録

このステージは journey 振る舞いの詳細化が役割であり、システム全体像を気軽に開き直す場所ではありません。

## Agent から聞かれること

各 journey について、Agent は次のような内容を確認します。

- トリガーと前提条件
- メインフロー
- 代替フローや例外フロー
- loading、empty、success、error などの状態
- 業務ルールやエッジケース
- 関連する接点、システム、component
- 可観測性や受け入れ期待

議論は現在の journey バッチに集中させる想定です。

## Plan mode

journey に必須の詳細が不足している場合、Agent は Plan mode で次の不足項目を集めてからそのバッチを続けることがあります。

## YAML アーティファクトと Gate

journeys ステージでは次を維持します。

- `journeys/batches.yaml`
- 各 in-scope journey に対応する `journeys/<journey-id>.yaml`
- `P3 components` に進めるかを決める `P2 journeys` gate 結果

Agent は承認前に現在の YAML バッチを要約するべきです。

## あなたが承認すること

次の点が要約で正確に示されていれば、その journey バッチを承認できます。

- 各 journey が何を起点に始まるか
- メインフローと分岐フロー
- 重要な状態と失敗経路
- 重要なルールとエッジケース
- ほかのシステム要素とどうつながるか

すべての in-scope journey が承認されるまで、この流れを繰り返します。

## 次のステップ

journeys 承認後は [`spec-forge-components`](../spec-forge-components/README.ja-JP.md) の `P3 components` へ進みます。
