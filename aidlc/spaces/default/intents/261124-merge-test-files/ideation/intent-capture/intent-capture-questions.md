# 意図の把握の質問(Intent Capture Questions): テストのファイルの統合

## Sources

- [desc] Initial description: "7 回目の調査の intent B: テストのファイルの統合(動きを変えない。版は上げない)。(1) cli の README のテスト(tests/docs_{dry_run,env,graph,init,retry}.rs と tests/docs_support/mod.rs、src/tests/docs_{dry_run,graph,init}.rs と src/tests/docs_support.rs、計 10 ファイル)をまとめる。(2) converter の tests/convert_roundtrip.rs(1 件)を tests/convert_ac.rs へ、src/tests/convert_ac.rs(2 件)を src/tests/convert_roundtrip.rs へ、tests/common(kinds だけ)を tests/nfr_security.rs へ。(3) cli の src/tests/init_types.rs(残る 1 件)と src/tests/run_id_arg.rs(1 件)を src/tests/args_grammar.rs へ(run-id の節の並びも直す)、src/tests/graph_long_id.rs を src/tests/graph_report.rs へ。(4) runtime の tests/state_read_raw.rs を tests/state_read.rs へ、tests/runner_env.rs を tests/runner_basic.rs へ。(5) cli の tests/common と src/tests/common の補助の写し(FakeEnv、FakeSink、state_error)の二重管理の解消。(6) init のひな形の原本(日本語と英語)が tests/flow_init.rs と src/tests/init_template.rs の 2 か所にある重複。判断が要るもの: README のテストの置き場所、補助の写しの解消の仕方、ひな形の原本の扱い。範囲の外: 本番の小さなモジュールの統合(intent C)、core の補助と性能のテストの統合(見送り)、nfr_perf.rs・msrv_consistency.rs・platform・converter の src の直下のファイル"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: テストのファイルを移してまとめる作業で、テストの名前と中身は変えない見込みです(変わるのはファイルとパス)。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。テストの名前・確かめの行・期待の値は変えない。名前を変える・消すテストがあれば、理由を添えて設計の段で承認を取る
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] cli の README のテスト(10 ファイル、613 行)の置き場所

背景: README の例と必須の語句を確かめるテストが、統合テスト(`tests/docs_*.rs` 5 つと補助)と単体テスト(`src/tests/docs_*.rs` 3 つと補助)に分かれています。分かれたのは前の intent で「狭めた内部の関数を使うかどうか」で分けたためで、README の確かめそのものには関係しません。統合テストの 5 つは cli の公開の入口を使わず、README と core と `tempfile` だけを使います。補助の `readme` は 2 か所に写しがあります。

- A. すべてを単体テスト `crates/cli/src/tests/docs.rs` 1 つ(約 480 行)にまとめる(補助の写しが 1 つになり、統合テストの実行ファイルが 5 つ減る)
- B. 2 つの階層は保ち、`tests/docs.rs`(約 300 行)と `src/tests/docs.rs`(約 220 行)の 2 ファイルにまとめる(補助の写しは残る)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] cli のテストの補助の写し(FakeEnv、FakeSink、state_error)

背景: 統合テストの `tests/common/mod.rs` と単体テストの `src/tests/common/mod.rs` に、同じ中身の写しがあります。違いは `Language` の型だけです。すでに食い違いが出始めています(統合テストでは使わない `color_read` が写しに残っている)。

- A. 1 つのファイル(例 `tests/common/fakes.rs`)にして、統合テストは `mod` で、単体テストは `#[path = "..."]` で読む(写しが無くなる。`#[path` は `src/tests/` の中なので、製品のコードの検査の外。やや珍しい書き方)
- B. 写しは保ち、2 つが同じであることを文字で比べるテストを足す(差が出たら失敗する。書き方はふつう)
- C. 今のまま(範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] init のひな形の原本(日本語と英語、各 約 30 行)

背景: 本番の `init` が書き出すひな形と同じ文を、テストが「原本」として 2 か所(統合テスト `tests/flow_init.rs` と単体テスト `src/tests/init_template.rs`)に持っています。日本語版は README にもあり、README のテストが一致を確かめています。

- A. テスト用のデータのファイル(例 `crates/cli/tests/fixtures/init_ja.toml`、`init_en.toml`)にして、両方のテストから `include_str!` で読む(converter の `tests/fixtures/` と同じ形。`include_str!` はテストの側なので、製品のコードの検査の外)
- B. 今のまま(範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] cli の `run_id_arg.rs`(1 件)の行き先

背景: `src/tests/args_grammar.rs`(1,133 行)の `--run-id` の節(約 200 行)に、`resolve_run_id` の見出しの中に別の性質のテスト 2 件が紛れ込んでいます。

- A. `args_grammar.rs` の `--run-id` の節へ移し、紛れ込んだ 2 件を正しい節へ並べ直す(ファイルが 1 つ減る)
- B. `--run-id` の節(約 200 行)と `run_id_arg.rs` を `src/tests/run_id.rs`(約 240 行)に切り出す(`args_grammar.rs` が軽くなる。ファイルの数は変わらない)
- X. Other (please specify)

[Answer]: A
