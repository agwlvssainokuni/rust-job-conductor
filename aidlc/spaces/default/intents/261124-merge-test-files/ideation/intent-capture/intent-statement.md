# 意図の記述(Intent Statement): テストのファイルの統合

## Sources

- [desc] Initial description: "7 回目の調査の intent B: テストのファイルの統合(動きを変えない。版は上げない)。(1) cli の README のテスト(tests/docs_{dry_run,env,graph,init,retry}.rs と tests/docs_support/mod.rs、src/tests/docs_{dry_run,graph,init}.rs と src/tests/docs_support.rs、計 10 ファイル)をまとめる。(2) converter の tests/convert_roundtrip.rs(1 件)を tests/convert_ac.rs へ、src/tests/convert_ac.rs(2 件)を src/tests/convert_roundtrip.rs へ、tests/common(kinds だけ)を tests/nfr_security.rs へ。(3) cli の src/tests/init_types.rs(残る 1 件)と src/tests/run_id_arg.rs(1 件)を src/tests/args_grammar.rs へ(run-id の節の並びも直す)、src/tests/graph_long_id.rs を src/tests/graph_report.rs へ。(4) runtime の tests/state_read_raw.rs を tests/state_read.rs へ、tests/runner_env.rs を tests/runner_basic.rs へ。(5) cli の tests/common と src/tests/common の補助の写し(FakeEnv、FakeSink、state_error)の二重管理の解消。(6) init のひな形の原本(日本語と英語)が tests/flow_init.rs と src/tests/init_template.rs の 2 か所にある重複。判断が要るもの: README のテストの置き場所、補助の写しの解消の仕方、ひな形の原本の扱い。範囲の外: 本番の小さなモジュールの統合(intent C)、core の補助と性能のテストの統合(見送り)、nfr_perf.rs・msrv_consistency.rs・platform・converter の src の直下のファイル"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。テストの名前・確かめの行・期待の値は変えない。名前を変える・消すテストがあれば、理由を添えて設計で承認を取る)
- [Q2] `intent-capture-questions.md` Q2: A(cli の README のテストを、単体テスト `src/tests/docs.rs` 1 つにまとめる)
- [Q3] `intent-capture-questions.md` Q3: A(cli のテストの補助を 1 つのファイルにし、統合テストは `mod`、単体テストは `#[path = "..."]` で読む)
- [Q4] `intent-capture-questions.md` Q4: A(init のひな形の原本をテスト用のデータのファイルにし、両方のテストから `include_str!` で読む)
- [Q5] `intent-capture-questions.md` Q5: A(`run_id_arg.rs` を `args_grammar.rs` の `--run-id` の節へ移し、紛れ込んだ 2 件を正しい節へ並べ直す)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- 同じ対象を確かめるテストが、小さなファイルに分かれている(cli の README のテスト 10 ファイル、converter・cli・runtime の 1〜5 件の小さなファイル)。[desc]
- cli のテストの補助(FakeEnv、FakeSink、state_error)と init のひな形の原本が、統合テストと単体テストに写しで 2 か所ずつあり、食い違いが出始めている。[desc] [Q3] [Q4]
- 解決したいのは、テストの名前と確かめる中身を変えずに、ファイルをまとめ、写しを 1 つにすることである。[desc] [Q1]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- cli の README のテストは `crates/cli/src/tests/docs.rs` 1 つになり、補助の `readme` の写しは 1 つになる。`tests/docs_*.rs` と `tests/docs_support/`、`src/tests/docs_*.rs` と `src/tests/docs_support.rs` は無くなる。統合テストだった 5 つは単体テストへ階層が変わる(README と core を読むだけで、cli の公開の入口を使わないので、確かめる中身は変わらない)。[Q2]
- converter・cli・runtime の初期の記述の (2)〜(4) のファイルは、まとめ先へ移って無くなる。`run_id_arg.rs` は `args_grammar.rs` の `--run-id` の節へ移り、紛れ込んだ 2 件は正しい節へ並ぶ。[desc] [Q5]
- cli のテストの補助の中身は 1 つのファイルにだけあり、統合テストと単体テストの両方がそれを読む。[Q3]
- init のひな形の原本はテスト用のデータのファイルにだけあり、両方のテストがそれを読む。[Q4]
- テストの名前・確かめの行・期待の値は変えない。テスト名の差は、名前の最後の部分で比べて 0 件(変わるのはパスだけ)。名前を変える・消すテストがあれば、理由を添えて設計の段で承認を取る。[Q1]
  - 確かめる手段: 実装の前(基準は実装を始める前の HEAD の `cargo test --locked --workspace` の通過の件数)と後でテストの件数を記録し、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。テスト名は `cargo test --locked --workspace -- --list` の名前の最後の部分で前後を比べる。[Q1]
- 製品のコードの検査(cli の `nfr_source.rs`、converter の `nfr_security.rs`、runtime の `nfr_source.rs`)のテストの数と中身を変えずに通る。[desc]
- 仕様書 rust と言語に依らない仕様書の、テストのファイルの表と根拠の欄が実装と合う。確かめる手段: 無くなったファイルの名前が仕様書に残らないことを検索で確かめる。[desc]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。本番のコードは変えない。版は上げない。[desc]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 7 回目のリファクタリングの調査(ファイルの統合の観点を加えた洗い出し)の候補を 3 つの intent に分けた、その 2 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (1)〜(6)。[desc] [Q2] [Q3] [Q4] [Q5]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 本番の小さなモジュールの統合(intent C)、core の補助と性能のテストの統合、`nfr_perf.rs`・`msrv_consistency.rs`・`platform`・converter の src の直下のファイル。[desc]

## Assumptions & Open Questions

None.
