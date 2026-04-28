# spec-forge-implement

`spec-forge-implement` は `P5 implement` です。`P4 readiness` 承認後に、承認済み spec に対する実装結果を記録するときに使います。

言語版:

- [English](./README.md)
- [简体中文](./README.zh-CN.md)
- [日本語](./README.ja-JP.md)

## いつ使うか

次のような場合は Agent に `$spec-forge-implement` を使わせます。

- `P4 readiness` が承認済みである
- 承認済み spec に沿って実装作業を進めている
- 変更内容、検証、ブロッカーを正直に記録したい
- チャットだけでなく正式な最終ステージ記録が必要である

このステージは readiness 承認後の成果状態を記録します。

## 前提条件

implement を直接使う前に、次を確認してください。

- readiness が承認済みである
- implementation-ready spec が存在する
- 記録したいのが実際の成果状態であり、気軽なスコープ再開ではない

ステージ選択に迷う場合は [`router`](../spec-forge/README.ja-JP.md) から始めてください。

## 使わない場面

次の目的には implement を使わないでください。

- 初期 intake、architecture、journey、component の設計
- readiness の最終統合
- 以前の承認を経ずにスコープを書き換えること
- workflow を閉じるために検証失敗を隠すこと

このステージは実装結果を正直に報告するためのものです。

## Agent から聞かれること

implement では、Agent は次のような内容を確認します。

- 実装が完了したか
- 何を変更したか
- どのファイルや領域に影響したか
- どの検証を実行し、結果がどうだったか
- ブロッカーや後続作業が残っているか

失敗した検証や未完了の作業も含め、結果を正直に残す前提です。

## Plan mode

実装レポートの必須項目が不足している場合、Agent は Plan mode で次の不足項目を集めてから最終レポートを確定することがあります。

## YAML アーティファクトと Gate

implement ステージでは次を維持します。

- `synthesis/implementation-report.yaml`
- `gates/implement.yaml`

これは `P5 implement` gate です。Agent は最終承認前に、これらの成果状態 YAML アーティファクトを要約するべきです。

## あなたが承認すること

次の点が要約で明確に記録されていれば implement を承認できます。

- 作業が完了しているか
- 実際の変更範囲は何か
- どの検証を行ったか
- 残るブロッカー、リスク、後続事項は何か

ステージを閉じるために、失敗したチェックや未解決ブロッカーを隠してはいけません。

## 次のステップ

implement ステージが通れば、workflow は完了として閉じられます。
