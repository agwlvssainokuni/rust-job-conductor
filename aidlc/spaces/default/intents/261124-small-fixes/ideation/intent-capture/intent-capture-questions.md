# 意図の把握の質問(Intent Capture Questions): 小さな直し

## Sources

- [desc] Initial description: "7 回目の調査の intent A: 小さな直し(動きを変えない。版は上げない)。(1) 古い説明: core の plan/mod.rs:18 と types.rs:440 が非公開の decide_before_run を指す(decide_from_state に)。(2) 仕様書 rust 02: resolve_run_id の引数の型(Option<&str> ではなく Option<&RunIdArg>)、公開の一覧に consistency が無い、42 行目・105 行目・137 行目の版の注記の混在を「今の形」と「版の履歴」に分ける。(3) テストの重複の削除: core の plan_perf.rs の「終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる」は plan_simulate_perf.rs の P2 と同じ場面なので消し、残る 1 件を plan_simulate_perf.rs に移す。cli の src/tests/init_types.rs の 3 件のうち args_grammar.rs と重複する 2 件を消す。cli の signals_order.rs は signals_real.rs が包含する性質(要判断)。(4) cli の tests/nfr_source.rs の #[path の確かめに include!( なども足す。範囲の外(別の intent): テストのファイルの統合、補助の写しの解消、init のひな形の原本の重複、本番の小さなモジュールの統合"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方と既存のテストの扱い

背景: 項目は小さく、1 項目 1 コミットで進められる見込みです。消すテストは、同じ場面や同じ性質をほかのテストが確かめているものだけです。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。消すテストは、同じことを確かめて残るテストを添えて設計の段で承認を取る
- B. 段ごとに止まり、結果を見せて、続けるかを確かめる
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] cli の `signals_order.rs`(INT のあとに HUP を送ると INT の番号 2 が残る)

背景: `signals_real.rs` は、TERM(15)が届いたあとに INT と HUP を送って 15 が残ることを確かめていて、「後から来たシグナルが最初の番号を上書きしない」という同じ性質を含みます。シグナルの登録はプロセスで 1 回だけなので、どちらも 1 ファイル 1 件で、実プロセスのシグナルを使います。消すと、テストの実行ファイルが 1 つ減ります。残すと、番号が小さいシグナル(2 のあとの 1)の組み合わせも確かめ続けられます。

- A. 消す(同じ性質を `signals_real.rs` が確かめる)
- B. 残す(番号の組み合わせが違うので、確かめとして意味がある)
- X. Other (please specify)

[Answer]: B

## Q3. [プロダクト] cli のソースの検査で止める書き方

背景: 前の intent で、製品のコードから `src/tests/` を読む道を塞ぐため、`src/tests/` の外に `#[path` が無いことを確かめるようにしました。ただ、`include!(` で別のファイルを取り込む道は止めていません。今の製品のコードには、`include!`・`include_str!`・`include_bytes!` はどれもありません。

- A. `include!(`・`include_str!(`・`include_bytes!(` をすべて止める(製品がテストのファイルやデータを読む道をすべて塞ぐ)
- B. `include!(` だけを止める(コードを取り込む道だけ塞ぐ。将来、製品がデータのファイルを埋め込む余地を残す)
- X. Other (please specify)

[Answer]: A
