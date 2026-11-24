# 意図の把握の質問(Intent Capture Questions): cli のライブラリの公開範囲を狭める

## Sources

- [desc] Initial description: "6 回目の調査の intent 4: cli のライブラリの公開範囲を狭める(動きを変えない。版は上げない)。cli の lib.rs が公開しているもの(pub mod exit・report・run_id、parse_args、CliCommand などの型、report の render_text・render_mermaid・escape_label・render_template・render_plan・Reporter・DisplayEvent・ResultDetail・StateContext、run_id::resolve_run_id、exit の定数など)のうち、main と依存の注入の継ぎ目(run_cli、run_cli_with、Deps、Runtime・Sink・Env・Signals のトレイト、StdSink・SystemEnv・SystemRuntime・SystemSignals、install_panic_hook と、それらの型の引数に要るもの)以外は、テストのためだけに公開している。これらを crate の中に狭め、それに頼るテスト(args_grammar.rs、report_messages.rs、graph_report.rs、plan_report.rs、init_template.rs、exit_codes.rs、escape_table.rs、run_id_arg.rs、init_types.rs、docs_*.rs など)を、src の単体テストへ移すか、公開の入口 run_cli_with で確かめる形に直す。判断が要るもの: テストの行き先、進め方(規模が大きい)。仕様書 rust の該当箇所を直す"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] テストの行き先

背景: 公開をやめる関数や型を直接呼ぶテストは、cli の 9 ファイル(約 3,800 行。引数の解析、表示の文、graph の text と Mermaid、計画の表示、init のひな形、終了コード、エスケープの表、run ID の引数など)と、`docs_*.rs` の一部です。前の intent(converter)では、同じ状況で src の単体テストへ移しました。cli の src は `nfr_source.rs` が `unsafe`・ネットワーク・`panic!(` などを検査しているので、移す先の形は設計で決めます。

- A. src の単体テストへ移す(今の確かめ方と期待の値をそのまま残す。移すのは機械的。converter と同じ形)
- B. 公開の入口 `run_cli_with`(偽の `Runtime`・`Sink`・`Env`・`Signals`)で確かめる形に書き直す(利用者に見える出力だけで確かめる。期待の値は出力の文に書き直すことになり、書き直しの量が多い)
- C. 組み合わせる(出力の文として確かめられるもの(表示の文、終了コード)は `run_cli_with` で、細かい単位(引数の解析の各規則、エスケープの表、graph の配置など)は単体テストへ)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 進め方

背景: 移す・直すテストが多く、コミットも 20 前後になる見込みです。前の intent と同じく、設計とコードの計画は 1 回で承認できますが、実装の途中で止めて確かめることもできます。

- A. 前回と同じ。設計とコードの計画は 1 回で承認し、実装は小さなコミットで進め、結果は最後にまとめて見せる。確かめる中身と期待の値は変えない。移す・変える・消すテストは検索で網羅し、消すものは理由を添えて設計の段で承認を取る
- B. 2 つの intent に分ける(たとえば、引数と型の公開を狭める段と、表示の公開を狭める段)
- C. 1 つの intent のまま、実装の途中(段の区切り)で止まって結果を見せる
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] 公開に残すもの

背景: `main` が使うのは `run_cli`、`StdSink`、`SystemEnv`、`install_panic_hook` の 4 つです。依存の注入の継ぎ目(`run_cli_with`、`Deps`、4 つのトレイトと `SystemRuntime`・`SystemSignals`)は、前の intent で設計上の継ぎ目として残すと決めています。継ぎ目のトレイトの引数と戻り値に出てくる型(`CliCommand` などの解析結果ではなく、偽物を書くのに要る型)は、公開に残す必要があります。

- A. `main` が使う 4 つ、依存の注入の継ぎ目、その継ぎ目を使うのに要る型だけを公開に残し、それ以外はすべて crate の中に狭める(推奨)
- B. A に加えて、終了コードの定数(`exit`)も公開に残す(ライブラリとして使う人が終了コードを名前で比べられる)
- X. Other (please specify)

[Answer]: A
