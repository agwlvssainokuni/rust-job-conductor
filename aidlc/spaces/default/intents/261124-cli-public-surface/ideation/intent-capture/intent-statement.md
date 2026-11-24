# 意図の記述(Intent Statement): cli のライブラリの公開範囲を狭める

## Sources

- [desc] Initial description: "6 回目の調査の intent 4: cli のライブラリの公開範囲を狭める(動きを変えない。版は上げない)。cli の lib.rs が公開しているもの(pub mod exit・report・run_id、parse_args、CliCommand などの型、report の render_text・render_mermaid・escape_label・render_template・render_plan・Reporter・DisplayEvent・ResultDetail・StateContext、run_id::resolve_run_id、exit の定数など)のうち、main と依存の注入の継ぎ目(run_cli、run_cli_with、Deps、Runtime・Sink・Env・Signals のトレイト、StdSink・SystemEnv・SystemRuntime・SystemSignals、install_panic_hook と、それらの型の引数に要るもの)以外は、テストのためだけに公開している。これらを crate の中に狭め、それに頼るテスト(args_grammar.rs、report_messages.rs、graph_report.rs、plan_report.rs、init_template.rs、exit_codes.rs、escape_table.rs、run_id_arg.rs、init_types.rs、docs_*.rs など)を、src の単体テストへ移すか、公開の入口 run_cli_with で確かめる形に直す。判断が要るもの: テストの行き先、進め方(規模が大きい)。仕様書 rust の該当箇所を直す"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(公開をやめるものを直接呼ぶテストは、src の単体テストへ移す。確かめ方と期待の値をそのまま残す)
- [Q2] `intent-capture-questions.md` Q2: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。確かめる中身と期待の値は変えない。移す・変える・消すテストは検索で網羅し、消すものは理由を添えて設計で承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目を使うのに要る型だけを公開に残し、それ以外は crate の中に狭める)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- cli のライブラリは、`main` と依存の注入の継ぎ目に要るもののほかに、引数の解析、表示、run ID、終了コードなどを、テストのためだけに公開している。[desc]
- 解決したいのは、利用者から見える動きを変えずに、公開範囲を `main` と継ぎ目に要るものだけに狭めることである。[desc] [Q3]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- cli のライブラリの公開範囲が、`main` が使う 4 つ(`run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook`)、依存の注入の継ぎ目(`run_cli_with`、`Deps`、`Runtime`・`Sink`・`Env`・`Signals` のトレイト、`SystemRuntime`・`SystemSignals`)、継ぎ目を使うのに要る型だけになる。それ以外の公開は crate の中に狭まる。[Q3]
  - 確かめる手段: 設計の段で、今の公開の項目を一覧にし、項目ごとに「残す(理由)」か「狭める」かを書く。実装の後、`lib.rs` の `pub` の項目を一覧と比べ、狭めた名前を `tests/` から参照する所が 0 件であることを検索で確かめる。[Q2] [Q3]
- 狭めた項目を直接呼ぶテストは src の単体テストへ移り、今の確かめ方と期待の値をそのまま残す。移す先の形は、cli の src のソースの検査(`nfr_source.rs`)と矛盾しない形にする。[Q1]
  - 「継ぎ目を使うのに要る型」の基準: 公開に残すトレイトと関数(`run_cli_with`、`Deps`、`Runtime`・`Sink`・`Env`・`Signals`)の引数・戻り値・項目に出てくる cli の型で、偽物を書くのに要るもの。設計の段で、型ごとにこの基準に当たるかを一覧にする。[Q3]
- 仕様書 rust の公開範囲とテストの場所の記述が実装と合う。確かめる手段: 狭めた名前を公開として書く記述が仕様書に残らないことと、移したテストのファイルを今の場所(`tests/`)として書く記述が残らないことを、検索で確かめる。[desc] [Q1]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。版は上げない。[desc]
- 既存のテストの扱い: 移す・書き方を変える・消すテストは認める。確かめる中身と期待の値は変えない。一覧は検索で網羅し、消すテストは理由を添えて、設計の段で承認を取る。移すテストの名前を変える必要があるかどうかも設計の段で一覧にし、変えるものは承認を取る。[Q2]
  - 確かめる手段: 実装の前と後でテストの件数を記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。[Q2]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 6 回目のリファクタリングの調査(プロダクトコードの不要なコードの洗い出し)で挙がった候補を 4 つの intent に分けた、その 4 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: cli の `lib.rs` の公開範囲と、狭めた項目に頼るテスト、仕様書 rust の該当箇所。[desc] [Q1] [Q3]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q2] [memory:M1]
- 範囲の外: core・runtime・converter の公開範囲。依存の注入の継ぎ目そのものの形。[desc] [Q3]

## Assumptions & Open Questions

None.
