# 意図の記述(Intent Statement): テストだけが使う本番のコードの削除

## Sources

- [desc] Initial description: "6 回目の調査の intent 1: テストだけが使う本番のコードの削除(小。動きを変えない。版は上げない)。削除: core の Command の From<&str>・From<String>、cli の RunIdArg::as_str、runtime の InterruptSignal の is_raised・PartialEq・Eq・Default、本番で使われない Default(core の JobStatus、runtime の Durability、core の StateOverlay、cli の Language)、core の sanitize(escape_display を包むだけ)、core の EnvFileError::reason()。公開範囲を狭める: core の decide_before_run(テストは decide_from_state に)、core の flow::classify・MAX_RUN_ID_LEN・is_valid_env_name・is_reserved_env_name・JobDefinition::effective_success_codes の公開、cli の report::escape の escape_display の再公開、cli の lib.rs の pub mod 4 つ(args、deps、panic_hook、types)と根の GraphOptions・DryRunOptions の再公開。判断が要るもの: core の RunIdError の 4 種を 1 つに、runtime が core の関数と型を再公開している部分、誤りの種類の Display と Error、本番で使われない Hash、ライブラリの誤りの型の Display(残す推奨)。テストは偽物の側か公開の入口で代わりを持つ。範囲の外(別の intent): 読まれない項目(FlowEdge.explicit、EnvOrigin、EnvFileContent.path と EnvEntry.line、format_version と description)、converter の testing モジュール、cli のライブラリの公開範囲の大きい部分"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。期待の文字列は変えない。変える・消すテストは検索で網羅し、消すものは理由を添えて設計で承認を取る)
- [Q2] `intent-capture-questions.md` Q2: A(`RunIdError` を種類のない誤りの型 1 つにまとめる。テストは 4 つの場合の入力が通らないことを確かめる)
- [Q3] `intent-capture-questions.md` Q3: A(runtime の core の再公開をやめ、使う所を `conductor_core` からの `use` に直す)
- [Q4] `intent-capture-questions.md` Q4: A(誤りの種類の `Display`・`Error` と、本番で使われない `Hash` をすべて消す。テストは `label()` などで確かめる)
- [Q5] `intent-capture-questions.md` Q5: A(ライブラリの誤りの型の `Display` と `Error` は残す)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- 本番のコードに、テストからしか使われない関数・実装・公開範囲が残っている(初期の記述の「削除」と「公開範囲を狭める」の項目)。[desc]
- 同じものへの入口が 2 つある所(runtime による core の再公開)と、種類を誰も見ない誤りの型(`RunIdError` の 4 種)がある。[desc] [Q2] [Q3]
- 解決したいのは、利用者から見える動きを変えずに、これらを本番のコードから消すことである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- 初期の記述の「削除」の項目のそれぞれが本番のコードから消え、「公開範囲を狭める」の項目のそれぞれが外から見えなくなる。テストは偽物の側か公開の入口で代わりを持つ。[desc] [Q1]
  - `RunIdError` は種類のない誤りの型 1 つになり、テストは 4 つの場合の入力が通らないことを確かめる。[Q2]
  - runtime は core の関数と型(`check_consistency`、`apply_interrupt`・`apply_outcome`・`apply_skip`、`initial_state`、`ConsistencyResult`、`MismatchKind`)を再公開しない。使う所は `conductor_core` から使う。終わりの基準は、`conductor_runtime::` を通したこれらの名前の参照が 0 件になること。[Q3]
  - 誤りの種類の `Display`・`Error`(core の `DefinitionErrorKind`・`EnvFileErrorKind` の `Display` と `Error`、converter の `ConversionErrorKind` の `Error`)と、本番で使われない `Hash`(converter の `TargetFormat`・`ConversionErrorKind`、core の `EnvFileErrorKind`・`DefinitionErrorKind`)がなくなる。[Q4]
  - ライブラリの誤りの型の `Display` と `Error` は残る。[Q5]
- 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形、終わったと判断する基準(たとえば「本番とテストからの参照が 0 件」)を一覧にし、実装の後に検索で確かめる。[Q1]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。版は上げない。[desc]
- 既存のテストの扱い: 書き方を変える・消すテストは認める。期待する出力の文字列は変えない。一覧は検索で網羅し、消すテストは理由を添えて、設計の段で承認を取る。[Q1]
  - 確かめる手段: 実装の前と後でテストの件数を記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。[Q1]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 6 回目のリファクタリングの調査(プロダクトコードの不要なコードの洗い出し)で挙がった候補を 4 つの intent に分けた、その 1 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の「削除」「公開範囲を狭める」と、判断の要るもののうち Q2〜Q4 で消すと決めたもの。[desc] [Q2] [Q3] [Q4]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 初期の記述の「範囲の外」に挙げたもの(読まれない項目、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分)と、ライブラリの誤りの型の `Display`。[desc] [Q5]

## Assumptions & Open Questions

None.
