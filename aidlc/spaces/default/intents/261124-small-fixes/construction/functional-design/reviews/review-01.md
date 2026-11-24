## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | functional-spec.md:92-110(§2.4 の検査の形) | 空白を除いた文字列の部分一致なので、`my_include!` のような別のマクロ名や、ブロックコメント `/* */` と文字列リテラルの中の語も当たる(誤検出の向き)。一方、`#[cfg_attr(..., path = "...")]` の形は `#[path` に当たらず見逃す。今の製品コードには、どの語も 0 件だと実物で確かめた(`grep include` と `#[path` が `crates/cli/src` の `src/tests/` の外で 0 件)ので、今回は通る。誤検出は安全側で、検査の目的(止める)には合う。 | 設計は変えなくてよい。§2.4 かコード生成の記録に、「誤検出は安全側、`cfg_attr` の `path` は範囲の外」と 1 行書いて、既知の限界にする。 | New |
| R-02 | Minor | functional-spec.md:136-141(§4.1 の T1) | T1 の本体と P2 を並べて確かめた。定義の文字列(`j0` に `@end` への遷移、残りは `echo i`)、`initial_state` から `classify_and_decide`、`apply_outcome` へのステートの作り方、`state.end_reached && !state.completed`、4 つの確かめ(件数、先頭が `AlreadyExecuted`、残りが `SkippedAfterEnd`、`EndReached`)は、一字違わず同じ。時間の上限は 1 回で 1 秒に対し、P2 は 5 回の中央値で 150 ms なので、P2 の方が厳しい。消してよい。ただし T1 の `run_id: None` と P2 の `measure` の `run_id: None` も同じ。違いは計測の回数だけ。 | 変更は要らない。承認の判断材料として、この比べ方が正しいことを記録する。 | New |
| R-03 | Minor | functional-spec.md:171-177(§4.6 の T3a、T3b) | 実物を比べた。残る `args_grammar` の `init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` は、`init` だけの引数の取り出しが `InitOptions::default()` と等しいことと、出力先、名前、`force` の 3 つの値を確かめる。T3a(既定の 3 つの値)と T3b(`CliCommand::new(Init).init_options()` が既定と等しい)は、`args.rs:488` が `CliCommand::new(kind)` から作るので、本番の経路を通して同じことを確かめている。同じ確かめが残るので消してよい。T3b だけは、引数の解釈を通さない直接の呼び出しが無くなる点が弱くなる(`CliCommand::new` の既定値の直接の確認)。 | 変更は要らない。承認のときに、この弱くなる点(`new` の直接の確認が無くなるが、引数の解釈の経路で同じ値を確かめる)を 1 行で見せる。 | New |
| R-04 | info | functional-spec.md:62-70(§2.3 の T2)、inventory.md | `plan_perf.rs` の `#[test]` から関数の終わりまでは 30〜79 行目、T1 は前の空行(80 行目)を含む 81〜134 行目、`init_types.rs` の T3a は 21〜30 行目で、行番号は実物と一致する。`common::add_env_file` は `common` にあり、移す先は `mod common;` を持つので、import の追加だけで足りる。`plan_perf.rs` の module の説明にある「U3、BR6.3、NFR1、plan-state-unify の BR2.3」は、移す先の説明の括弧にそのまま写す方針で、規則の番号の対応は保たれる。 | 変更は要らない。 | New |
| R-05 | info | functional-spec.md:20-24(§2.1 と D5) | 実物で `decide_before_run` を検索した。コードでは `plan/mod.rs:18` と `types.rs:440`(C1、C2)が、公開のものとして書く説明で、残りは `flow/decide.rs` の中(非公開の本体の説明とテスト)。仕様書 rust 05 の 341 行目と 850 行目が、呼ぶ側の関数として書く 2 か所で、D5 の直しの範囲と一致する。言語に依らない仕様書 05 の 339 行目の同じ書き方は、言語に依らない側が「関数の名前は仮の名前」と明記している(`00-README.md:113`)ので、直さなくてよい。 | 変更は要らない。見落としは無かった。 | New |
| R-06 | info | functional-spec.md:40-61(§2.2)、functional-spec.md:42 の元の段落 | 実物の確認: `run_id.rs:37-41` の `resolve_run_id` は `Option<&RunIdArg>` で、仕様書 02 の 146 行目の `Option<&str>` と食い違う(S1 は正しい)。core の `lib.rs` の `pub mod` は `consistency`、`definition`、`envfile`、`error`、`flow`、`graph`、`plan`、`run_id`、`types` で、仕様書 02 の公開モジュールの一覧には `consistency` だけが無い(S2、S3 は正しい)。runtime の `lib.rs` は `mod` が 11 個ですべて非公開で、D2 の「11 個」と一致する。版の注記は、42 行目に 3 つ(`v0.9.6 までは`、`v0.9.0 以降は`、`v0.9.7 までは`)、105 行目と 137 行目に設計の数のとおり。§2.2 の `awk` は節の見出しで止まり、2 章の各節の履歴のあとに ` - ` で始まる別の行が無いので、7 行になる見込みは妥当。 | 変更は要らない。 | New |
| R-07 | info | functional-spec.md:56-60(§2.2 の分け方) | 42 行目の「v0.9.0 以降は、さらに ... を再公開する」は、注記であると同時に、今の形の再公開の一覧の一部(`envfile::` と `plan::` の再公開)でもある。「今の形」から抜くときに、この 2 つの一覧を今の形の本文に残さないと、仕様が欠ける。 | コード生成で S4 を書くとき、`envfile::{...}` と `plan::{...}` の一覧は今の形の再公開の一覧に統合し、「v0.9.0 以降」の言い回しだけを版の履歴に移す、と記録して確かめる。 | New |
| R-08 | info | functional-spec.md:238-258(§8 のコミットの並び) | 10 件のコミット。各コミットでビルドが通る順になっている。コミット 7 で `plan_perf.rs` から T1 を消すと、`Path`、`JobExecution`、`RunMode`、`apply_outcome`、`classify_and_decide`、`initial_state` の import が未使用になるが、同じコミットで消す方針なので clippy は通る。コミット 9 で `CliCommand`、`CommandKind`、`InitOptions` の import を `init_types.rs` から外すのも同様。件数の見込み(1,152、1,151、1,151、1,149)と合計(1,149、core 統合 269、cli 単体 162)は引き算として合っている。テストの表の直し(rust 11 の 115 行目の性能の表、215 行目の core の統合テストの表、217 行目の `plan_simulate_perf.rs` の行、302 行目の `init_types.rs` の行、言語に依らない 11 の 979 行目)は inventory の S8〜S12 に挙がっていて、実物の `plan_perf` を指す行(115、215、979)と対応し、漏れは見つからなかった。 | 変更は要らない。コミット 8 のときに、217 行目の `plan_simulate_perf.rs` の行に、移したテスト(env と env_file を持つ 1,000 ジョブ)を足すことを S9 か S10 で確実に行う。 | New |
| R-09 | info | functional-spec.md:9-15(§1 の制約) | 本番の処理の変更は、説明の行(C1〜C4)だけ。依存の追加、`unsafe`、製品の `cfg` の追加、凍結した仕様書(go、python、typescript、csharp、powershell)の変更、版の更新は、設計のどこにも無い。言語に依らない仕様書 11 の変更は S12 の 1 か所で、凍結の対象に入っていない(D3 の決定のとおり)。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run`)に触れる直しは無い。 | 変更は要らない。 | New |
| R-10 | info | functional-spec.md:288-296(§11 の SM4)、traceability.json | SM4 の文は `include!(` のように括弧つきで書いているが、D4 で `(` `[` `{` のどれでも止める形に広げた。SM4 の文と BR4.1 の範囲に食い違いは無い(広い方が SM4 を満たす)が、文字は一致しない。 | 変更は要らない。必要なら、承認のときに「SM4 の括弧つきの書き方は、D4 で広げた形に含まれる」と 1 行添える。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| grep decide_before_run(crates、docs/spec/rust、docs/spec/language-agnostic) | コードは `plan/mod.rs:18`、`types.rs:440`、`flow/decide.rs` の中だけ。rust 05 の 341、850 を含む | C1、C2、D5 の範囲と一致 |
| grep include と path(`crates/cli/src` の `src/tests/` の外) | 0 件 | T4 は今の製品のコードで通る |
| grep plan_perf、init_types(`aidlc` を除く) | rust 11 の 115、215、302 行目と、言語に依らない 11 の 979 行目 | S8〜S12 と一致。ほかに参照なし |
| 行番号の確認(`plan_perf.rs`、`init_types.rs`) | 30〜79、81〜134、21〜30 | 設計の行番号と一致 |
| cargo test | 実行していない(設計の段で、コードは変わっていない。HEAD 6ce7d45 の 1,152 通過は依頼文のとおり) | 件数は引き算で確かめた。実装後に §7 で確かめる |

### Summary

設計は、実物と突き合わせて、消す 3 件のテストが残るテストと同じことを確かめていること、移す 1 件の本体が変わらないこと、古い説明と仕様書の直しの一覧が実物と合うこと、本番の処理を変えないことを確かめた。critical と major は無く、nfr_source の検査の限界(誤検出は安全側、`cfg_attr` の `path` は範囲の外)の記録と、仕様書 02 の 42 行目の分け方で再公開の一覧を欠けさせないことの 2 点を、実装で気をつければ足りる。READY。
