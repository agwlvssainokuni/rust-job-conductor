# 意図の記述(Intent Statement): 読まれない項目の削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 2: 読まれない項目の削除(動きを変えない。版は上げない)。core の FlowEdge.explicit(書かれるが本番で読まれない。target_position は位置だけ返す)、core の EnvOrigin と origin の項目(EffectiveEntry.origin、EnvNameEntry.origin。本番で読むのは Debug だけ。EnvNameEntry は名前だけに)、core の EnvFileContent.path と EnvEntry.line(本番で読まれない)、core の PipelineDefinition の format_version と description(本番で読まれない。型の検査は残す)を型から消し、テストは名前の並びと値などで確かめる。仕様書 rust の該当箇所を直す。判断が要るもの: language-agnostic の仕様書の「名前と出どころ」、「名前、値、行」、FlowEdge の explicit、定義の形の記述を直すか。範囲の外: converter の testing モジュール、cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。期待の文字列は変えない。変える・消すテストは検索で網羅し、消すものは理由を添えて設計で承認を取る)
- [Q2] `intent-capture-questions.md` Q2: A(言語に依らない仕様書も参照実装に合わせて直す。Q3 の結果、対象は名前の出どころ、`.env` の項目の行、辺の `explicit`)
- [Q3] `intent-capture-questions.md` Q3: B(定義の `format_version` と `description` は型に残す。範囲から外す)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- core の型に、書かれるが本番で読まれない項目が残っている(グラフの辺の `explicit`、環境変数の名前の出どころ `EnvOrigin` と `origin`、`.env` の内容の `path` と項目の `line`)。[desc] [Q3]
- 言語に依らない仕様書も、これらを内部のデータの形として書いている。[desc] [Q2]
- 解決したいのは、利用者から見える動きを変えずに、これらを型と仕様書から消すことである。[desc] [Q2]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読んで別の言語で作り直す実装者。[desc] [Q2]

## Success Metrics

- `FlowEdge.explicit`、`EnvOrigin` と `origin` の項目、`EnvFileContent.path`、`EnvEntry.line` が core の型から消え、本番とテストからの参照が 0 件になる。テストは名前の並び、値、位置など、残る項目で確かめる。[desc] [Q1]
- 仕様書: rust と言語に依らない仕様書の該当する記述(名前と出どころ、`.env` の項目の行、辺の `explicit`)が、参照実装と一致する。計画の表示が名前だけであること、重複キーの警告の行、図の描き方の記述は残る。確かめる手段: 設計の段で、直す仕様書の節と語句(`EnvOrigin`、`origin`、「出どころ」、`explicit`、`.env` の項目の「行」)を一覧にし、直した後にそれらの語句が該当する節に残らないことを検索で確かめる。[desc] [Q2] [Q1]
- 定義の `format_version` と `description` は型に残る(変えない)。[Q3]
- 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形、終わったと判断する基準(「参照が 0 件」)を一覧にし、実装の後に検索で確かめる。[Q1]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。版は上げない。[desc]
- 既存のテストの扱い: 書き方を変える・消すテストは認める。期待する出力の文字列は変えない。一覧は検索で網羅し、消すテストは理由を添えて、設計の段で承認を取る。[Q1]
  - 確かめる手段: 実装の前と後でテストの件数を記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。[Q1]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 6 回目のリファクタリングの調査(プロダクトコードの不要なコードの洗い出し)で挙がった候補を 4 つの intent に分けた、その 2 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: `FlowEdge.explicit`、`EnvOrigin` と `origin`、`EnvFileContent.path`、`EnvEntry.line` と、rust と言語に依らない仕様書の該当箇所。[desc] [Q2] [Q3]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 定義の `format_version` と `description` [Q3]、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分 [desc]。

## Assumptions & Open Questions

None.
