# 意図の把握の質問(Intent Capture Questions): 読まれない項目の削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 2: 読まれない項目の削除(動きを変えない。版は上げない)。core の FlowEdge.explicit(書かれるが本番で読まれない。target_position は位置だけ返す)、core の EnvOrigin と origin の項目(EffectiveEntry.origin、EnvNameEntry.origin。本番で読むのは Debug だけ。EnvNameEntry は名前だけに)、core の EnvFileContent.path と EnvEntry.line(本番で読まれない)、core の PipelineDefinition の format_version と description(本番で読まれない。型の検査は残す)を型から消し、テストは名前の並びと値などで確かめる。仕様書 rust の該当箇所を直す。判断が要るもの: language-agnostic の仕様書の「名前と出どころ」、「名前、値、行」、FlowEdge の explicit、定義の形の記述を直すか。範囲の外: converter の testing モジュール、cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: 項目は 4 つで、1 項目 1 コミットで進められる見込みです。消す項目を読む・書くテストは、名前の並び、値、位置など、残る項目で確かめる書き方に直します。消す項目だけを確かめているテストの行は、項目と一緒に消えます。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。期待する出力の文字列は変えない。変える・消すテストは検索で網羅した一覧にし、消すものは理由を添えて設計の段で承認を取る
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 言語に依らない仕様書の記述

背景: 言語に依らない仕様書(`docs/spec/language-agnostic`)は、内部のデータの形として、環境変数の「名前と出どころ」(`EnvOrigin`、計画の `env_names` が `(name, origin)`)、`.env` の項目の「名前、値、行」、グラフの辺の `explicit`、定義のデータの `format_version` と `description` を書いています。どれも利用者から見える動き(出力、終了コード、ステートファイル)には出ません。前の intent では、シグナルの「すでに登録済み」を参照実装に合わせて直しました。

- A. 参照実装に合わせて直す(内部のデータの形から、読まれない項目を消す。定義ファイルのキーとしての `format_version`・`description` と、その検査の記述は残す)
- B. rust の仕様書だけ直し、言語に依らない仕様書は変えない(ほかの言語の実装には、項目を持つ自由を残す)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 定義の `format_version` と `description`

背景: 定義ファイルのキー `format_version`(必須。1 だけ受け付ける)と `description`(任意。文字列)は、読み取りで検査しますが、検査の後の定義のデータ(`PipelineDefinition`)の項目は本番で読まれません(`format_version` は常に 1)。項目を消しても、キーの検査(版の確認、型の誤り)と誤りの表示は変わりません。

- A. 型から消す。キーの検査は残す(推奨。読まれない項目をなくす)
- B. 残す(定義ファイルの形をそのまま写した型として、意味がある。範囲から外す)
- X. Other (please specify)

[Answer]: B
