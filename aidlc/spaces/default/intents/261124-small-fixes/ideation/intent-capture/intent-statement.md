# 意図の記述(Intent Statement): 小さな直し

## Sources

- [desc] Initial description: "7 回目の調査の intent A: 小さな直し(動きを変えない。版は上げない)。(1) 古い説明: core の plan/mod.rs:18 と types.rs:440 が非公開の decide_before_run を指す(decide_from_state に)。(2) 仕様書 rust 02: resolve_run_id の引数の型(Option<&str> ではなく Option<&RunIdArg>)、公開の一覧に consistency が無い、42 行目・105 行目・137 行目の版の注記の混在を「今の形」と「版の履歴」に分ける。(3) テストの重複の削除: core の plan_perf.rs の「終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる」は plan_simulate_perf.rs の P2 と同じ場面なので消し、残る 1 件を plan_simulate_perf.rs に移す。cli の src/tests/init_types.rs の 3 件のうち args_grammar.rs と重複する 2 件を消す。cli の signals_order.rs は signals_real.rs が包含する性質(要判断)。(4) cli の tests/nfr_source.rs の #[path の確かめに include!( なども足す。範囲の外(別の intent): テストのファイルの統合、補助の写しの解消、init のひな形の原本の重複、本番の小さなモジュールの統合"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(設計とコードの計画は 1 回で承認し、小さなコミットで進め、結果は最後にまとめて見せる。消すテストは、同じことを確かめて残るテストを添えて設計で承認を取る)
- [Q2] `intent-capture-questions.md` Q2: B(cli の `signals_order.rs` は残す)
- [Q3] `intent-capture-questions.md` Q3: A(cli のソースの検査で、製品のコードの `include!(`・`include_str!(`・`include_bytes!(` をすべて止める)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- core の説明 2 か所が、今は使わない非公開の `decide_before_run` を指している。[desc]
- 仕様書 rust 02 に、実装と食い違う型の記述と、公開の一覧の欠け、版の注記の混在がある。[desc]
- 同じ場面や同じことを確かめるテストが重なっている(core の性能のテスト 1 件、cli の init の型のテスト 2 件)。[desc] [Q2]
- cli のソースの検査は、製品のコードが `include!` などで別のファイルを読む道を止めていない。[desc] [Q3]
- 解決したいのは、利用者から見える動きを変えずに、これらを直すことである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- 古い説明: core の `plan/mod.rs` と `types.rs` の説明に `decide_before_run` が残らない(検索で 0 件。非公開の `flow/decide.rs` の中は除く)。[desc]
- 仕様書 rust 02: `resolve_run_id` の引数の型が実装と一致し、公開の一覧に `consistency` がある。版の注記は「今の形」と「版の履歴」に分かれ、注記の版の並びが時系列になる。確かめる手段: 実装のシグネチャと仕様書の行を比べる。版の注記は、設計の段で直す段落と、直した後の「今の形」の段落に「v0.」で始まる版の注記が残らないことを確かめる検索を決める。[desc]
- テストの重複: core の `plan_perf.rs` の該当の 1 件を消し、残る 1 件を `plan_simulate_perf.rs` へ移して `plan_perf.rs` が無くなる。cli の `init_types.rs` の重複の 2 件を消す。`signals_order.rs` は残す。消すテストは、同じことを確かめて残るテストを添えて設計の段で承認を取る。[desc] [Q1] [Q2]
- ソースの検査: cli の `nfr_source.rs` が、`src/tests/` の外の製品のコードで `#[path`、`include!(`、`include_str!(`、`include_bytes!(` を見つけたら失敗する。今の製品のコードでは通る。[Q3]
- 確かめる手段: 実装の前と後でテストの件数を記録し、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること(ファイルを移すだけのテストは件数に数えない)。テスト名の差が、消したテストと、パスだけが変わる移したテストだけであること。[Q1]
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)は変わらない。版は上げない。[desc]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 7 回目のリファクタリングの調査(ファイルの統合の観点を加えた洗い出し)の候補を 3 つの intent に分けた、その 1 つ目。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (1)〜(4)。ただし `signals_order.rs` は残す。[desc] [Q2]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: テストのファイルの統合、補助の写しの解消、init のひな形の原本の重複、本番の小さなモジュールの統合(別の intent)。[desc]

## Assumptions & Open Questions

None.
