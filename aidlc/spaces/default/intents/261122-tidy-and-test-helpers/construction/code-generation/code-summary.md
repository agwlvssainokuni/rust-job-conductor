# コード生成のまとめ(Code Summary): 動きを変えない小さな整理とテストの補助

計画 `code-generation-plan.md` の Step 1〜10 を実装した。基準は HEAD 37210ca(コードは af018bc と同じ。v0.9.6)。コードと仕様書のコミットは 49 件(段 1: 5、段 2: 6、段 3: 8、段 4: 7、段 5: 5、段 6: 4、段 7: 13、最後: 1)。ほかに、この記録のコミットが 1 件。push、タグ、リリースはしていない。版は上げていない。

## コミットの一覧

| # | 段 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `5e5fa4d` | core: types.rs の生成元の説明を今の姿に直す(1a) |
| 2 | 1 | `a8936e0` | core: sanitize・escape_display の説明を直し、テストに注記を足す(1b、T40) |
| 3 | 1 | `cc95c42` | cli: args.rs の文法の表に 5 つのオプションの列を足す(1c) |
| 4 | 1 | `d9f4844` | 仕様書: 08 章の拒否の理由の 3 行を消し、05 章の OutOfOrder を直す(1d) |
| 5 | 1 | `043982c` | core・仕様書: 使われない Command::program_name とその単体テストを消す(1e、T38) |
| 6 | 2 | `54e3e1f` | cli: SKIPPED_LIST・NOTE_BY_* をまとめ、ALREADY_EXECUTED を置く(2a の前半) |
| 7 | 2 | `83b5703` | cli: RESUME_HERE・DETAIL_*・class_word・COMPLETED をまとめる(2a の後半) |
| 8 | 2 | `60f029e` | cli: error_prefix! で INTERNAL_ERROR・SIGNAL_SETUP_FAILED を組む(2a) |
| 9 | 2 | `c505e6f` | core: env_name_rule!・reserved_env_prefix! で環境変数の名前の文を組む(2b、2c) |
| 10 | 2 | `992e413` | core: ROOT_KEYS に VERSION_KEY を使い、間隔の文と定数を assert で結ぶ(2d、2e) |
| 11 | 2 | `6d92c53` | core: CONDUCTOR_RUN_ID と DEFAULT_RUN_ID_ENV の意味の違いを注記する(2f) |
| 12 | 3 | `24583d3` | runtime: 中立のモジュール temp_file に dir_or_current を移す(3a) |
| 13 | 3 | `519c464` | runtime: temp_name・create_temp・temp_prefix と単体テスト 4 件を temp_file に移す(3a、T22) |
| 14 | 3 | `5ec8299` | runtime: create_temp_beside と連番 1 つにまとめ、lib.rs の表と仕様書 02・06 章を直す(3a、D5) |
| 15 | 3 | `1870f05` | runtime のテスト: create_temp_beside の単体テスト(N4) |
| 16 | 3 | `0009734` | cli: push_line を 1 つにし、job_place で場所を組む(3b、3c) |
| 17 | 3 | `69f3e1e` | cli: reset_hint と StateContext::for_read で reset の案内の選び方を 1 か所に(3d) |
| 18 | 3 | `f4a745b` | cli: plan_end を消し、plan_text で模擬の最後の語を 1 回だけ選ぶ(3e) |
| 19 | 3 | `33ce5cf` | cli: graph_mermaid.rs の let-else を 1 段に(3f) |
| 20 | 4 | `a4bc38e` | core: decide の規則 5 のコードを消し、規則 6 に含まれることを注記する(4a) |
| 21 | 4 | `30f8b8e` | cli: lexopt の到達しない MissingValue の枝を消し、守りの枝を 1 つにまとめる(4b、D3) |
| 22 | 4 | `0d292d3` | cli: render の match を 1 つにし、到達しない枝を消す(4c) |
| 23 | 4 | `141268f` | cli: retry.rs の使われない既定値 0 を消し、1 つの let-else に(4d) |
| 24 | 4 | `89e3616` | core: classify_exit_code を切り出し、simulate が is_retry_target と共有する(4e、D4) |
| 25 | 4 | `a9ec60d` | core: retry_integer が u32 の範囲を受け取り、後始末の try_from を消す(4f) |
| 26 | 4 | `c69d8c5` | core: duration.rs の余分な n >= 1 を消す(4g) |
| 27 | 5 | `abecac5` | cli: CommandKind::ALL・from_name で args.rs のコマンド名の逆引きを置き換える(5a) |
| 28 | 5 | `13b5fb8` | cli のテスト: コマンドの名前の往復(N1) |
| 29 | 5 | `32097ff` | cli: 位置引数の扱いを Operand と網羅の match で決める(5b) |
| 30 | 5 | `cda19f0` | cli: 誤りの対象の直書きを Opt::subject と RUN_ID_OPTION にする(5c) |
| 31 | 5 | `bb4a874` | runtime: ParseFailure に原因の FailureCause を足し、reset --all の判定を理由の文で比べない(5d) |
| 32 | 6 | `df43986` | core: success と success_declared を SuccessCodes の 1 つのフィールドに(6a) |
| 33 | 6 | `3250f62` | core: build_jobs が所有権を受け取り、clone をやめる(6b) |
| 34 | 6 | `6a3cad4` | core: integer_value を置き、walk と version で共有する(6c) |
| 35 | 6 | `793c0d2` | core: depth.rs の 2 つの枝を walk_children で 1 つに(6d) |
| 36 | 7 | `ed3dee5` | cli のテスト: docs_support に README の読み取りと必須語句の検査をまとめる(T1〜T6) |
| 37 | 7 | `54bd29f` | cli のテスト: status_of・status_in を common にまとめる(T7〜T9、T20) |
| 38 | 7 | `cd2da99` | cli のテスト: signal_support にシグナルの送信をまとめる(T10〜T13) |
| 39 | 7 | `a7782dc` | cli のテスト: args_grammar.rs の MapEnv を common::FakeEnv::with に(T14) |
| 40 | 7 | `830a373` | cli のテスト: 偽物の Default を derive にし、使われない MemRuntime.writes を消す(T15、T16、T18) |
| 41 | 7 | `ca208be` | cli のテスト: MemRuntime は起動するときだけ台本を消費する(T19) |
| 42 | 7 | `52c3fbe` | cli のテスト: ScriptedRuntime に run_lines を足し、分割済みのコマンドの受け渡しを確かめる(T17、N2、N3) |
| 43 | 7 | `25d1acf` | runtime のテスト: without_run_id の包みと definition を common にまとめる(T21、T23〜T27) |
| 44 | 7 | `323f191` | core・runtime: consistency の単体テスト 4 件を core へ移し、仕様書 02・12 章を直す(T28、T29) |
| 45 | 7 | `97b78a5` | core のテスト: tests/common に SECRET・run・kinds・median をまとめる(T30 の一部、T31〜T35) |
| 46 | 7 | `d2102f1` | core のテスト: ステートの補助 state を tests/common にまとめる(T30 の一部、T36、T37) |
| 47 | 7 | `893ced4` | core・runtime: JobExecution の手組みを組み立ての関数にする(T39、T41〜T45、T25 の一部) |
| 48 | 7 | `3942abc` | converter: TargetFormat::opposite を足し、2 か所の反対の形式の求め方を置き換える(T46、N5) |
| 49 | 最後 | `d39420f` | 仕様書: 11 章のテストのファイルの表と、02 章の偽物の表を直す |

コミットの並びは `functional-spec.md` §8 の 1〜49 と 1 対 1 に対応する(分けたりまとめたりしていない)。各コミットで `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --locked --workspace` を通してからコミットした(コミット 10 だけ、fmt の差を見落としてコミットしたため、直して `--amend` した。push 前の手元の作業)。

## 件数の移り変わり(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug)

| 時点 | 通過 | 無視 | 失敗 | core(単体+統合) | runtime | converter | cli | 基準からの差 | 内訳 |
|------|------|------|------|------------------|---------|-----------|-----|--------------|------|
| 基準 | 1,142 | 1 | 0 | 112+261 | 71+125 | 53+63 | 0+457 | — | inventory.md §0 と一致 |
| 段 1 のあと | **1,141** | 1 | 0 | 111+261 | 71+125 | 53+63 | 0+457 | −1 | T38 |
| 段 2 のあと | 1,141 | 1 | 0 | 111+261 | 71+125 | 53+63 | 0+457 | −1 | 変更なし |
| 段 3 のあと | **1,142** | 1 | 0 | 111+261 | 72+125 | 53+63 | 0+457 | +0 | N4(T22 の 4 件は移動) |
| 段 4 のあと | 1,142 | 1 | 0 | 111+261 | 72+125 | 53+63 | 0+457 | +0 | 変更なし |
| 段 5 のあと | **1,143** | 1 | 0 | 111+261 | 72+125 | 53+63 | 0+458 | +1 | N1 |
| 段 6 のあと | 1,143 | 1 | 0 | 111+261 | 72+125 | 53+63 | 0+458 | +1 | 変更なし |
| 段 7 のあと(最終) | **1,146** | 1 | 0 | 115+261 | 68+125 | 54+63 | 0+460 | **+4** | N2、N3、N5、T28/T29 の移動(runtime −4、core +4) |

見込み(1,141 → 1,142 → 1,143 → 1,146)と、`functional-spec.md` §6 のクレートごとの見込み(core 376、runtime 193、converter 117 + 1 無視、cli 460)と一致する。ファイルごと: `args_grammar.rs` 61、`flow_run.rs` 53、`flow_retry.rs` 19(§6 と同じ)。

## テスト名の比較(`cargo test --locked --workspace -- --list 2>/dev/null | sort`)

基準と最終の一覧の差は、次だけだった(各段の前後でも比べ、差は該当する段にだけ出た)。

- 消えた名前: `types::tests::プログラム名は_line_では先頭の語_argv_では先頭の要素`(T38)。
- 道が変わった名前(移動): `state::writer::tests::{一時ファイルの名前の形, 名前が重なったら連番を進める, 名前が_16_回とも重なれば失敗, 作成のそれ以外の失敗はすぐに返す}` → `temp_file::tests::…`(T22)。runtime の `state::consistency::tests::{一致すれば_ok_で_パスの違いは見ない, パイプライン名の違い, ジョブidの追加_削除_改名, 並びだけの違い}` → core の `consistency::tests::…`(T28、T29)。
- 足した名前: `コマンドの名前から種類を引ける_往復_重なりなし_知らない名前は_none`(N1)、`分割済みのコマンドを分け直さずに起動へ渡す`(N2)、`リトライのたびに同じ分割済みのコマンドを渡す`(N3)、`temp_file::tests::対象の隣に接頭辞つきの一時ファイルを作り_続けて呼ぶと別の名前`(N4)、`types::tests::反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る`(N5)。
- ほかの差は、テストのバイナリごとの件数の行(`N tests, 0 benchmarks`)だけ。

## テストの変更の一覧(承認済みの T1〜T46)

| ID | ファイル | 変えたこと | コミット |
|----|----------|------------|----------|
| T1〜T5 | cli `docs_dry_run.rs`、`docs_env.rs`、`docs_graph.rs`、`docs_init.rs`、`docs_retry.rs` | `readme()`・見出しの後のブロックの取り出しの定義を消し、`docs_support` の `readme`・`assert_readme_has`・`assert_docs_check_words`・`block_after_heading` を使う。`docs_retry.rs` の docs-check の 7 語は `assert_readme_has(&text, &DOCS_CHECK_WORDS)`(文「README に {word} がない」を保つ) | 36 |
| T6 | cli `docs_support/mod.rs`(新規) | 上の補助。必須語句の失敗の文は 5 ファイルとも今と同じ。`expect` の文は「見出し `<見出し>` がある」「例がある」になる(失敗のときだけ。既定で進めるもの) | 36 |
| T7、T8 | cli `flow_run.rs`、`flow_pipeline.rs` | `status_of` の定義を消し、`common` のものを使う(呼ぶ側は同じ) | 37 |
| T9 ★ | cli `flow_retry.rs` | `status_of` を消し、5 か所を `status_in(&state, "a")` に | 37 |
| T10〜T13 | cli `binary_signal.rs`、`signals_order.rs`、`signals_real.rs`、`signal_support/mod.rs`(新規) | `send` の定義を消し、`signal_support::send` / `send_self` を使う | 38 |
| T14 | cli `args_grammar.rs` | `MapEnv`・`env_of` を消し、`mod common;` と `common::FakeEnv::with(env)` | 39 |
| T15、T16、T18 | cli `common/mod.rs` | `WriteFailure` と `ScriptedRuntime` に `#[derive(Default)]`(手書きの `impl Default` を消す)、`MemRuntime.writes` を消す | 40 |
| T19 ★ | cli `common/mod.rs` | `MemRuntime::run_job` は合図を先に見て、起動するときだけ台本を消費する | 41 |
| T17 | cli `common/mod.rs` | `RunLine` の別名、`ScriptedRuntime.run_lines`、`run_job` の `line` を記録 | 42 |
| T20 | cli `common/mod.rs` | `status_in`、`status_of` を足す | 37 |
| T21 | runtime `tests/common/mod.rs` | `definition(name, jobs)` と `without_run_id { read_state, reset_state, initial_state, write_state }` を足す | 43 |
| T22 | runtime `src/state/writer.rs` → `src/temp_file.rs` | 単体テスト 4 件を名前と中身を変えずに移す | 13 |
| T23〜T27 | runtime `state_read.rs`、`state_write.rs`、`nfr_content.rs`、`nfr_performance.rs`、`flow_with_state.rs` | run ID なしの包みを消し `common::without_run_id::…` を使う。`nfr_content.rs`・`nfr_performance.rs` の `definition()` の本体を `common::definition(…)` に | 43 |
| T25(続き) | runtime `nfr_content.rs` | `JobExecution` の手組みを `JobExecution::exited(&d.job[0], 3)` に | 47 |
| T28 | runtime `src/state/consistency.rs`(削除)、`src/state/mod.rs`、`src/lib.rs` | ファイルとモジュールの宣言、表の行を消し、説明を直す | 44 |
| T29 | core `src/consistency.rs` | runtime の単体テスト 4 件と補助を末尾に移す(`use crate::…`) | 44 |
| T30 | core `tests/common/mod.rs`(新規) | `SECRET`、`run`、`kinds`、`median`(45)、`state`(46)、`job`、`exited`(47) | 45〜47 |
| T31、T32 | core `definition_errors.rs`、`definition_retry.rs` | `run`・`kinds` の定義を消し `common` のものを使う | 45 |
| T33 ★ | core `env_definition.rs` | `SECRET`・`kinds` を `common` のものに(`parse`・`ok` は残す) | 45 |
| T34 | core `env_file_parse.rs`、`env_merge.rs`、`env_types.rs`、`plan_types.rs` | `const SECRET` を消し `common::SECRET` | 45 |
| T35 | core `env_merge_perf.rs`、`flow_perf.rs` | `median` を `common` のものに | 45 |
| T36 | core `plan_run.rs` | `state_with` を消し、4 か所を `state(&[…], None, false)` に | 46 |
| T37 ★ | core `plan_pipeline.rs`、`graph_overlay.rs` | `state` の定義を消し `common::state` を使う | 46 |
| T38 | core `src/types.rs` | テスト `プログラム名は_line_では先頭の語_argv_では先頭の要素` を消す(件数 −1) | 5 |
| T39 | core `flow_sequences.rs` | 補助 `exited` を消し `common::exited`、`NotStarted` の手組みを `JobExecution::not_started(job, 15)` に | 47 |
| T40 | core `src/error.rs` | テスト `エスケープ後も_64_文字を超えない` に注記を 1 行足す(名前は同じ) | 2 |
| T41 | core `retry_flow.rs` | 補助 2 つの本体と 4 つの `..exec(&j)` を組み立ての関数に | 47 |
| T42 | core `graph_edges.rs` | 補助 `exec` を消し `JobExecution::exited(job, code)` | 47 |
| T43〜T45 | core `src/flow/classify.rs`、`next_step.rs`、`record.rs` | 単体テストの補助 `exec` の本体と `next_step.rs` の 4 つの手組みを組み立ての関数に | 47 |
| T46 | converter `convert_roundtrip.rs` | `let to = from.opposite();` | 48 |

テストの名前、件数(移動を含む)、確かめる中身、期待する出力の文字列は変えていない(T38 の 1 件の削除を除く)。T1〜T46 の外の既存のテストを変える必要は出なかった。

## 確かめ方の検索の結果(`functional-spec.md` §7。最後のコミットのあと)

| 検索 | 結果 |
|------|------|
| `grep -n "SKIPPED_LIST\|NOTE_BY_\|…\|fn plan_end" -r crates/cli/src` | 0 件 |
| `grep -rn '"conductor: error: ' crates/cli/src` | 1 件(`messages.rs` の `error_prefix!` の定義) |
| `grep -rn "英字または" crates/core/src` | 1 件(`run_id.rs` の `env_name_rule!` の定義) |
| `grep -rn 'CONDUCTOR_` で' crates/core/src` | 2 件。どちらもドキュメントコメント(`envfile/mod.rs:59` の `is_reserved_env_name`、`error.rs:102` の `ReservedEnvName`)。誤りの文の直書きは 0 件 |
| `grep -rn "static SEQUENCE" crates/runtime/src` | 1 件(`temp_file.rs`) |
| `grep -rn "crate::state::\(dir_or_current\|writer::\)" crates/runtime/src/file_create.rs crates/runtime/src/platform` | 0 件 |
| `grep -rn "fn push_line" crates/cli/src` | 1 件 |
| `grep -rn "job '{}'" crates/cli/src` | 1 件(`job_place`) |
| `nothing_started`(decide.rs)、`MissingValue { .. }`(args.rs)、`=> String::new()`(report/mod.rs)、`_ => 0`(retry.rs)、`unwrap_or(Classification::Error)`(core/src)、`try_from(m)`(walk/retry.rs) | すべて 0 件 |
| `grep -n '"run" =>' crates/cli/src/args.rs` | 0 件 |
| `grep -rn '"--\(to\|assume\|format\|run-id\)"' crates/cli/src` | 1 件(`types.rs` の `RUN_ID_OPTION` の定義) |
| `grep -rn "reason == reason::" crates/runtime/src` | 0 件 |
| `grep -rn "success_declared" crates/core/src` | 0 件 |
| `grep -rn "from_str_radix" crates/core/src/definition` | 1 件(`integer_value`) |
| `grep -n "for word in" crates/cli/tests/docs_*.rs` | 1 件(`docs_graph.rs` の行ごとの検査。範囲外) |
| `grep -rn "fn readme\|fn status_of\|fn send\|struct MapEnv\|fn median\|const SECRET\|fn kinds" crates/*/tests` | 共通の場所に 1 か所ずつ(`docs_support::readme`、cli `common::status_of`、`signal_support::send`/`send_self`、core `common::{SECRET, kinds, median}`)。ほかに当たるのは、名前が `readme_` で始まるテスト関数、範囲外の cli 4 ファイルと runtime `env_loader.rs` の `SECRET`、値の違う `nfr_content.rs`・converter `nfr_security.rs` の `SECRET`、形の違う runtime `env_loader.rs` の `kinds` |
| `grep -rn "JobExecution {" crates` | 構造体の値の手組みは 0 件。当たるのは型の定義・`impl` の行と、戻り値の型 `-> JobExecution {` の行だけ(`grep -v -- "-> "` で 2 件: `types.rs:206`、`:232`) |
| `grep -rn "fn read_state(def" crates/runtime/tests` | 1 件(`common::without_run_id`) |
| `ls crates/runtime/src/state/consistency.rs` | 無い |
| `grep -rn "program_name" crates docs/spec/rust` | 0 件 |
| `git diff --stat af018bc -- Cargo.lock Cargo.toml crates/*/Cargo.toml` | 差分なし(新しい依存なし。converter の依存も同じ) |
| 足した行の `unsafe`・`cfg(unix)`・`cfg(windows)` | `signal_support/mod.rs` のドキュメントコメントの 1 行だけ(コードには足していない) |

## 合格の条件

- `cargo fmt --all -- --check`: 通った。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通った。
- `cargo test --locked --workspace`: 1,146 通過、1 無視、0 失敗(rustc 1.100.0。MSRV と同じ版)。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行 98.14%、関数 98.20%、領域 97.76%。
- 文言の定数をまとめたところ(2a、2b、2c)は、出力の検査のテストがそのまま通ったことに加え、組み立てた文が元の文字列のままバイナリに入っていることを `grep -a` で確かめた(`conductor: error: internal error`、`conductor: error: failed to set up signal handling`、環境変数の名前の 3 つの文、予約名の 2 つの文)。

## 計画からのずれ

1. **`SuccessCodes::Valid` は位置を持たない**(6a)。設計は `Valid(Vec<i32>, usize)`(値と位置)だったが、位置はどこからも読まれず、`clippy -D warnings`(dead_code)で止まった(今の `success` の位置も読まれていなかった)。`Valid(Vec<i32>)` にした。動きは同じ。
2. **位置引数の扱いの関数の名前**(5b)。設計の `fn operand(kind)` は、同じ関数の中の変数 `operand` と紛れるため `operand_rule(kind)` にした。
3. **`map_lex_error` の名前**(4b)。`UnexpectedValue` だけを扱う関数にしたので、`map_unexpected_value(option: &str)` に改めた。
4. **`render` の言語の判定**(4c)。言語を必要な枝だけで判定した結果、`Graph`・`Plan`・`InitTemplate` の表示と、スキップしたジョブの無い `Transition` の表示でも、ロケールを読まなくなった(今までは、`SignalSetupFailed`・`Version` 以外はすべて読んでいた)。出力は 1 文字も変わらない。`report_messages.rs` の遅延の検査はそのまま通る。
5. **T6 の `expect` の文**。承認済みの見出しの文に加えて、`docs_retry.rs` の「TOML の例がある」が共通の「例がある」になった(失敗のときだけ出る文。合否と期待の文字列は変わらない)。
6. **`writer.rs` の import**(T22)。移したテストのあとも残る単体テスト(`古い一時ファイルだけを消す`)が `OpenOptions` を使うため、非テストのコードで使わなくなった `OpenOptions` を、ファイルの先頭で `#[cfg(test)] use std::fs::OpenOptions;` にした(同じファイルの `JobStatus` の前例と同じ形。テストの中身は変えていない)。
7. **仕様書の追加の直し**。§8 に挙がっていない次の行も、名前と置き場所を合わせるために直した: 02 章の `ctx.rs` の行の `read_context` と `report/mod.rs` の行(3d、コミット 17)、05 章の `simulate_job` の説明(4e、コミット 24)、02 章の `TargetFormat` の行に `opposite()`(コミット 48)。
8. **コミット 10 の `--amend`**。fmt の差を見落としてコミットしたため、直して同じコミットに含めた(push 前)。

補足(レビューの指摘による記録): T9 の `status_in` は、ジョブがステートに無いとき `Pending` を返さず `expect` で失敗する(旧 `status_of` は `Pending` を返していた)。確かめ方が厳しくなる方向の変化で、承認済みの ★ T9 のとおり。対象のジョブはいつもステートにあるので、結果は変わらない。

## 範囲の外として残したもの(設計どおり)

- cli 4 ファイルと runtime `env_loader.rs` の `SECRET`(クレートをまたいだ `tests` の共有はできない)、`docs_graph.rs` の行ごとの検査と ```` ```text ```` の取り出し、`writer.rs` の `create_temp(dir, …)` に渡す `dir` を `dir_or_current` に通すこと、`lexopt::Error` の守りの枝そのもの。
- Linux・Windows での確かめ(ローカルは macOS だけ。CI の `test` で確かめる)。
