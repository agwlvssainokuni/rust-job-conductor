# コード生成のまとめ(Code Summary): 型と構造の整理・計算量・利用者から見える変更

計画 `code-generation-plan.md` の Step 1〜13 を実装した。基準は HEAD f1c9871(v0.9.6。計画のコミットは 1688684)。コードと仕様書のコミットは 37 件(段 1: 21、段 2: 9、段 3: 7)で、`functional-spec.md` §8 の並び 1〜37 と 1 対 1 に対応する。ほかに、この記録のコミットが 1 件。push、タグ、リリースはしていない。版は最後のコミットで 0.9.7 にした。

## コミットの一覧

| # | 段 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `e639fb5` | core・cli: ResumeInfo を列挙型にする(1a。T1・T2 の該当の行) |
| 2 | 1 | `29b10c0` | core・cli: PlanDisposition::WillRun に模擬の結果を持たせる(1b。T1・T2 の残り、T3、T4、T5 の :73) |
| 3 | 1 | `d6d3557` | core・cli: NextStepNote を列挙型にする(1e。T2 の残り) |
| 4 | 1 | `83bf6ec` | core・cli: GraphNode の節の種類を NodeKind にする(1c。T6〜T8) |
| 5 | 1 | `b30f0ba` | core・cli: 倍率と上限を Backoff にまとめる(1d。T9、T10 の RetryPolicy、T16、T26) |
| 6 | 1 | `bf7738c` | core・cli: RetryDecision::Retry に最後の終了コードと最大回数を持たせる(1f。T10 の比較) |
| 7 | 1 | `f046005` | core のテスト: リトライの判断の中身を確かめる(N1) |
| 8 | 1 | `90b46b1` | core・runtime: JobOutcome と NextStep の読まれない job_id を消す(1g。★ T17a、T19 の outcome・step、T20 の組み立て) |
| 9 | 1 | `b3a0a18` | core・runtime・cli: JobExecution の読まれない job_id と command を消す(1g。T5 の :91、T10〜T14、T17b、T18、T19 の exec、T20 の :113、T23) |
| 10 | 1 | `dd9a134` | runtime・cli: run_job(line, env, signal) にする(1g。T21、T22) |
| 11 | 1 | `5ad6b6a` | core: EnvFileError の理由をメソッド reason() にし、表示の書式を render_error にまとめる(1h。T15) |
| 12 | 1 | `fd6f167` | core のテスト: 誤りの理由と種類の文を固定の表で確かめる(N2) |
| 13 | 1 | `96b8f10` | cli: Checked は検査を終えた定義を base に持つ(1i) |
| 14 | 1 | `98b3864` | cli: 本番の準備の入口を prepare_session の 1 つにする(1i、D3) |
| 15 | 1 | `b3fdb1a` | cli: record_skip は SkipRecord を返す(1i) |
| 16 | 1 | `1336a50` | core・converter・runtime: 種類の文を label() の 1 か所に持つ(1j の前半、D10) |
| 17 | 1 | `a136b5f` | cli: messages.rs の関数の形を 3 つにそろえ、他のクレートの label() を使う(1j の後半) |
| 18 | 1 | `b88b4cc` | cli のテスト: 日本語の誤りの表示が各クレートの label() を使うことを確かめる(N3) |
| 19 | 1 | `ca924c1` | core: 外から使われない pub を狭め、根の再公開を名前の一覧にする(1k、D9) |
| 20 | 1 | `e827a63` | cli: 外から使われない pub を狭め、テスト用の関数に #[doc(hidden)] を付ける(1k、D9) |
| 21 | 1 | `347ac0e` | runtime: error と types のモジュールを非公開にし、再公開を名前の一覧にする(1k) |
| 22 | 2 | `22527bb` | core: 計画の模擬はステートを複製しない(2a、D8) |
| 23 | 2 | `950b2b8` | core のテスト: 計画の模擬の性能を確かめる(N4、N5) |
| 24 | 2 | `4a67337` | core: 図の重ね合わせを表で引く(2b) |
| 25 | 2 | `520bb52` | core: 辺の取り出しは遷移先の位置の表を 1 回作って引く(2c) |
| 26 | 2 | `c4f77ff` | core のテスト: 図の重ね合わせと辺の取り出しの性能を確かめる(N6、N7) |
| 27 | 2 | `47431cb` | core: .env の重複キーを位置の表と消した印で扱う(2d) |
| 28 | 2 | `9d3a5d7` | core のテスト: .env の重複キーの解析の性能を確かめる(N8) |
| 29 | 2 | `d4a76f7` | core: 計画の名前の要約は値を読まない merge_env_names で作る(2e) |
| 30 | 2 | `f2bcec8` | 仕様書: 性能の表に P1〜P5 とテストのファイルを足す |
| 31 | 3 | `5417632` | converter: 文法の誤りの理由を形式ごとの文にする(3a、D4、D5。★ T24) |
| 32 | 3 | `96a63d5` | converter のテスト: 文法の誤りの理由が形式ごとの文であることを確かめる(N10) |
| 33 | 3 | `4e9ee8b` | 仕様書: 変換の文法の誤りの理由を形式ごとの文にする(3a) |
| 34 | 3 | `e16b023` | runtime: .env の読み込みを定義ファイルと同じ順にする(3b) |
| 35 | 3 | `5339d38` | runtime のテスト: 読む権限のない大きな .env と名前付きパイプの .env を確かめる(N11、N12) |
| 36 | 3 | `a6f1762` | 仕様書: .env の読み込みの順を定義ファイルと同じにする(3b) |
| 37 | 3 | `7239aa2` | 版を 0.9.7 にする(ee9f409 と同じ 22 ファイル。★ T25) |

各コミットの前に `cargo fmt --all -- --check`、`cargo build`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` を通した(macOS、rustc 1.100.0 = MSRV)。

## テストの件数の移り変わり(`cargo test --locked --workspace`、macOS)

| 時点 | core | runtime | converter | cli | 全体 | 見込み |
|------|------|---------|-----------|-----|------|--------|
| 基準(f1c9871) | 376 | 193 | 117 + 1 無視 | 460 | **1,146 通過、1 無視** | — |
| 段 1 の後(347ac0e) | 378 | 193 | 117 + 1 無視 | 461 | **1,149 通過、1 無視** | 1,149 |
| 段 2 の後(f2bcec8) | 383 | 193 | 117 + 1 無視 | 461 | **1,154 通過、1 無視** | 1,154 |
| 段 3 の後(7239aa2) | 383 | 195 | 118 + 1 無視 | 461 | **1,157 通過、1 無視** | 1,157 |

失敗はどの時点も 0。見込み(functional-spec §6)と一致した(+11 件。N9 は D6 で欠番)。

## テスト名の比較(`cargo test --locked --workspace -- --list 2>/dev/null | sort`)

- 基準 → 段 1: 追加 3(`リトライの判断は_最後の終了コードと最大回数を持つ`(N1)、`誤りの理由は種類の固定の文で_種類の文は変更前と同じ`(N2)、`日本語の誤りの表示は_各クレートの_label_を使う`(N3))。消えた名前は無い(★ T17a は 1 行の削除で、テスト `終了コードの分類と返す値` は残る)。
- 段 1 → 段 2: 追加 5(`p1_…`(N4)、`p2_…`(N5)、`p3_…`(N6)、`p4_…`(N7)、`p5_…`(N8))。消えた名前は無い。
- 段 2 → 段 3: 追加 3(`文法の誤りの理由は形式ごとの文で_種類の文と重ならない`(N10)、`権限のない_1_mib_超の_env_は_unreadable`(N11)、`名前付きパイプの_env_は開かずに_unreadable`(N12))。消えた名前は無い。
- 基準 → 最後: 名前の差は N1〜N8・N10〜N12 の 11 件の追加だけ(ほかは各ファイルの件数の行の変化だけ)。

## テストの変更の一覧(既存のテスト)

変えた既存のテストのファイルは、functional-spec §4 の 30 ファイルと同じ(core 統合 15・src の単体テスト 4、runtime 統合 6・src の単体テスト 1、converter 1、cli 3)。確かめる中身と期待する出力の文字列は変えていない(例外は ★ の 3 つ)。

| ID | 変えたこと |
|----|------------|
| T1 | `plan_pipeline.rs`: import を `ResumeInfo` に。`WillRun` を含む扱いの並びの比較 8 か所を `assert!(matches!(dispositions(&plan).as_slice(), [...]), "{:?}", …)` に。再開位置の 4 か所を `ResumeInfo` の 1 つの値の比較に。`.result` の 3 か所を `.disposition.result()` に |
| T2 | `plan_run.rs`: import。`D::WillRun(_)` の `matches!`。`ResumeInfo::Individual`。次の動きの 5 か所を `NextStepNote` の値・`matches!` に。`.result` の 2 か所を `.disposition.result()` に |
| T3 | `plan_assume.rs`: `.result.expect` 6 か所を `.disposition.result().expect` に |
| T4 | `plan_types.rs`: `disposition: PlanDisposition::WillRun(SimulatedResult { … })`、`result` を消し、`disposition.result()` |
| T5 | `plan_perf.rs`: `matches!(…, WillRun(_))`、`JobExecution::exited(0)` |
| T6〜T8 | `graph_overlay.rs`・`graph_types.rs`・`graph_drawable.rs`: `job_id()`・`is_end()`、節の組み立てを `kind: NodeKind::…` に |
| T9 | `definition_retry.rs`: `backoff: None`、`p.backoff.map(\|b\| …)` |
| T10 | `retry_flow.rs`: import に `Backoff`、補助 `exec`・`exited` の本体(引数は `_j`)、`retry(…)` を `wait_of(…)` に(5 か所)、組み立てから `&j` を外す、`RetryPolicy` の組み立て 2 か所、型の分解に `..` |
| T11〜T14 | `flow_sequences.rs`・`common/mod.rs`(引数は `_job_id`、説明を更新)・`graph_edges.rs`・`flow_perf.rs`: `JobExecution` の組み立てから `job` を外す |
| T15 | `env_file_parse.rs`: `err.reason` を `err.reason()` に |
| T16 | `types.rs` の単体テスト: `backoff: None`、`p.backoff = Some(Backoff { … })` |
| ★ T17a | `classify.rs` の単体テスト: `assert_eq!(o.job_id, "a")` の 1 行を消す(コミット 8) |
| T17b | `classify.rs` の単体テスト: 補助 `exec` の本体を `JobExecution::new(result)` に(コミット 9) |
| T18 | `next_step.rs` の単体テスト: 補助 `exec`(引数は `_job_id`)と 4 つの組み立て |
| T19 | `record.rs` の単体テスト: 補助 `exec`・`outcome`・`step`(引数は `_job`) |
| T20 | `nfr_content.rs`: `JobExecution::exited(3)`、`JobOutcome`・`NextStep` から `job_id` を消す |
| T21 | runtime の統合テスト 13 か所: `run_job(&j, &line, …)` を `run_job(&line, …)` に。`runner_basic.rs:108` は `j` が使われなくなったので `_j` にした(下の「計画からのずれ」) |
| T22 | `runner.rs` の単体テスト 3 か所: 同上。テストのモジュールに `use conductor_core::JobDefinition` を足した(本体の import を消したため) |
| T23 | cli `common/mod.rs`: 偽物の `run_job` の中の組み立てから `job` を外す。2 つ目の偽物は `job` が使われなくなったので引数を `_job` にした |
| ★ T24 | `convert_ac.rs:70`: 期待の文字列 `"文法の誤りです"` を `"YAML として読めません"` に(コミット 31) |
| ★ T25 | `binary_basic.rs:246`: `"conductor 0.9.6\n"` を `"conductor 0.9.7\n"` に(コミット 37) |
| T26 | `docs_retry.rs`: `retry.backoff.map(\|b\| …)` |

足したテスト: N1(`retry_flow.rs`)、N2(`env_types.rs`)、N3(`report_messages.rs`)、N4・N5(新規 `plan_simulate_perf.rs`)、N6・N7(新規 `graph_lookup_perf.rs`)、N8(新規 `env_parse_perf.rs`)、N10(`convert_ac.rs`)、N11・N12(`env_loader.rs`、`#[cfg(unix)]`)。

## 性能の直す前と後の時間(debug、5 回の中央値、手元の macOS(Apple M2))

| テスト | 測る処理 | 直す前 | 直した後 | 上限 | 直す前の測定の木 |
|--------|----------|--------|----------|------|------------------|
| P1(N4) | 計画、1,000 個がすべて実行 | **95.1 ms** | **22.1 ms** | 150 ms | 347ac0e(2a の前) |
| P2(N5) | 計画、`@end` のあとの 999 個のスキップ | **104.0 ms** | **21.7 ms** | 150 ms | 347ac0e |
| P3(N6) | `build_graph` + 重ね合わせ、100 回 | **369.1 ms**(上限超え) | 2b の後 **72.2 ms**、2c の後 **96.1〜99.7 ms** | 350 ms | 950b2b8(2b・2c の前) |
| P4(N7) | `flow_edges`、遷移 1,996 個、100 回 | **689.3 ms**(上限超え) | **118.0〜121.4 ms** | 600 ms | 950b2b8 |
| P5(N8) | `parse_env_file`、10,000 行の逆順の重複 | **102.6 ms**(上限超え) | **16.1〜18.4 ms** | 100 ms | c4f77ff(2d の前) |
| 2e(一時の測定。テストは足さない) | 計画、ジョブ 100 個、`.env` 10,000 項目 | **605.8 ms** | **472.0 ms** | — | 9d3a5d7(2e の前) |

- 直す前の時間は、各テストのファイルを直す前の木に一時的に置いて走らせた(作業の木は stash や reset で戻していない。テストのファイルは、直すコミットの間だけ `target/tpv/` に退避し、テストのコミットで戻した)。直す前の木では、P3・P4・P5 は時間の上限で落ち、結果の中身の確かめはすべて通った(仕様書 §5.2 の「手元でわずかに超える」のとおり)。
- 2e は M3b と同じ場面の一時のテスト(`crates/core/tests/tmp_measure_2e.rs`)で測り、測ったあと消した(コミットしていない)。
- P3 は 2c のあとに 72 → 約 97 ms に増えた。`build_graph` が呼ぶ `flow_edges` が、遷移の無い定義でも位置の表を毎回作るため。上限の中で、直す前(369 ms)の約 1/4。

## 段 3 の N11・N12 の前後

- N11(読む権限のない 1 MiB 超の `.env`): 直す前の木(3b の前の 4e9ee8b)では `left: [(None, TooLarge)] right: [(None, Unreadable)]` で**落ちた**。直した後(e16b023)は通る。
- N12(書き手のいない名前付きパイプの `.env`): 直す前の木でも直した後でも**通る**(5 秒以内に `Unreadable`)。形 B でも「開かずに返る」が保たれることを確かめた。

## §7 の検索の結果(最後の HEAD 7239aa2 で実行)

| 検索 | 結果 |
|------|------|
| `grep -rn "ResumeKind\|resume\.kind\|resume\.job_id" crates docs/spec/rust` | 0 件 |
| `grep -rn "is_end:" crates` | 0 件 |
| `grep -rn "multiplier: Option\|max_interval_secs: Option" crates` | 1 件: `definition/walk/retry.rs:41`(読み取りの層の `RawRetry` の `multiplier: Option<u32>`。読み取った生の値で、`RetryPolicy` ではない。前からあり、変えない) |
| `grep -rn "(policy.multiplier" crates` | 0 件 |
| `grep -rn "note\.\(stopped\|completed\|end_reached\)" crates` | 0 件 |
| `grep -n "ExecutionResult::Exited" crates/cli/src/retry.rs` | 0 件 |
| `grep -rn "pub job_id" crates/core/src/types.rs` | 1 件(`JobState`) |
| `grep -rn "command: job.command.clone()" crates` | 0 件 |
| `grep -rn "pub reason: String" crates/core/src/envfile` | 0 件 |
| `grep -rn "fn prepare_run\|fn prepare_pipeline\|definition_path:" crates/cli/src` | `fn prepare_*` は 0 件。`definition_path:` は 15 件で、すべて関数の引数の名前(`deps.rs` の `Runtime` の `load_env_files(definition_path: &Path, …)` など。前からある)。`Checked` のフィールドの `definition_path` は無い |
| `grep -rn "Result<(PipelineState, bool)" crates/cli/src` | 0 件 |
| `grep -n "to_string()" crates/cli/src/report/messages.rs` | 3 件(`plan_next_step` の値を埋める形 F3 の中だけ)。F4 の 4 か所は無い |
| `grep -rn '"ステートファイルを読めません"' crates` | 1 件(runtime の `error.rs`) |
| `grep -n "fn plan_disposition\|fn class_word" …/messages.rs` | どちらも `-> Text` |
| `grep -rn "\.result\b" crates/core/src/plan crates/cli/src/report/plan_text.rs` | 2 件(`entry.disposition.result()` の読み方の補助だけ) |
| `grep -rn "result: Option<SimulatedResult>" crates` | 0 件 |
| `grep -rn '{file}:{line}: …' crates/core/src` | `render_error` の 1 か所 |
| 1k の検索(inventory §8) | core の狭めた名前は外から 0 件(`\.add(` 13 件と `definition::intake` 3 件は狭めていないもの)。cli の狭めた名前は tests・bin・main から 0 件(`language_resolved`・`color_resolved` は 1 ファイル。狭めない)。`conductor_runtime::(error\|types)::` は 0 件 |
| `grep -n "state.clone()" crates/core/src/plan/simulate.rs` | 1 件(出発点) |
| `grep -n "\.position(\|\.remove(" crates/core/src/envfile/parse.rs` | 0 件 |
| `grep -n "\.find(" crates/core/src/graph/overlay.rs` | 0 件 |
| `grep -n "job_index" crates/core/src/graph/edges.rs crates/core/src/flow/next_step.rs` | コードは `next_step.rs` の `classify_and_decide` の経路(`next_step` の 2 か所)と単体テストの補助だけ。ほかは説明の文 |
| `grep -rn 'reason::SYNTAX\|"文法の誤りです"' crates/converter/src` | 1 件(種類の文 `ConversionErrorKind::label` の `Syntax`) |
| 「v0.9.6 までは…」の注記 | rust 03(1)・09(6)、language-agnostic 03(1)・09(7)にある |
| `grep -rn "0\.9\.6" crates Cargo.lock docs/spec/rust docs/spec/language-agnostic` | 54 件で、すべて「v0.9.6 までは…」の注記 |
| `cargo run -q -p conductor-cli --bin conductor -- --version` | `conductor 0.9.7` |
| `Cargo.lock` の差分(f1c9871 から) | 版の 4 行だけ(`Cargo.toml` × 4 も版の行だけ) |
| src に足した `unsafe`・`cfg(unix\|windows)`・`.unwrap()`・`.expect(`・`panic!(` など | 0 件(`git diff f1c9871 -- 'crates/*/src'` の追加行) |
| `crates/converter/Cargo.toml` の `conductor-core` | 0 件 |

## 合格の条件

- `cargo fmt --all -- --check`: 通る。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通る。
- `cargo test --locked --workspace`: 1,157 通過、1 無視、0 失敗。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 通る(行 **98.10%**、関数 98.07%、領域 97.75%)。
- MSRV: 手元の rustc が 1.100.0(MSRV と同じ)で、上のすべてが通る。
- Linux・Windows は CI で確かめる(ローカルは macOS だけ)。

## 計画からのずれ

1. **整形に `cargo fmt --all` を使った**。内容の変更はすべてファイルのツール(Edit/Write)で行い、`rustfmt` の整形(行の折り返し)だけはコマンドで当てた(プロジェクトの整形の道具で、CI の `lint` と同じ)。また、**N1 のテストの追加(コミット 7)だけは、シェルのヒアドキュメント(`cat >>`)でファイルの末尾に書き足した**(手順の誤り。以後はファイルのツールで行った。内容は計画のとおり)。
2. **N2 はコミット 12 では `to_string()` を固定の表と比べ、コミット 16 で `label()` と比べる形に替えた**。計画の並びでは `label()` がコミット 16 で入るため。表は `label()` へ移す前に変更前の文を固定し、移したあとも 1 文字も違わないことを同じ表で確かめた。
3. **`apply_outcome_in` の手がかりは `Option<usize>`** にした(`apply_skip_in` は `usize`)。公開の `apply_outcome` は `None` で、スキップの並びの探索も含めてすべて先頭から(今と同じ「最初に一致したもの」)。模擬は `Some(位置)` で、スキップの並びを直前に見つけた位置の次から探す。
4. **`classify.rs` の内部の補助 `outcome` から `job` の引数を外した**(`JobOutcome.job_id` が無くなり使われないため)。
5. **`plan_next_step` はエスケープの関数を受け取る形** `plan_next_step(lang, note, escape)` にした(網羅の `match` の `NextJob` の枝の中でエスケープするため。表示は同じ)。
6. **`check_job`・`check_pipeline` を `pub(crate)` にした**(`run.rs`・`pipeline.rs` から `prepare_session` に渡すため)。
7. **T21・T22・T23 のファイルで、組み立ての変更に伴う最小の変更を 3 つ足した**: `runner_basic.rs:108` の `let j` を `let _j` に(T21 の `:111` で `j` が使われなくなったため)、`runner.rs` の単体テストのモジュールに `use conductor_core::JobDefinition`(本体の import が不要になったため)、cli `common/mod.rs` の 2 つ目の偽物の `run_job` の引数を `_job` に(T23 で使われなくなったため。§4 の冒頭の「使わなくなった引数は `_` で残す」と同じ扱い)。確かめる中身は変えていない。
8. **仕様書**: §8 の行のほか、rust 02 の再公開の一覧(core・cli・runtime)、`resolve_transition` の行、`StateErrorKind::label` の公開、rust 05 の模擬の擬似コード(`apply_*_in`)、rust 08 の 4.3・4.5・4.8 と `.env` の誤りの節の関数の名前も直した。version のコミット(37)に、rust 11 の P1〜P5 の注記の 1 語(「v0.9.6 まで。」→「v0.9.6 までは、」。検索の注記の形にそろえる)を含めた。
9. **§7 の検索で 0 件にならないもの 2 つ**は、上の表の説明のとおり(読み取りの層の `RawRetry`、`Runtime` の関数の引数の名前)。どちらも前からあり、目的(`RetryPolicy` の 2 つの `Option`、`Checked` の二重のフィールド)は消えている。
10. 直す前の性能の測定は、計画の「直すコミットの 1 つ前」の木で行った。P1・P2 は 2a の前(347ac0e)、P3・P4 は 2b・2c の前(950b2b8)、P5 は 2d の前(c4f77ff)、2e は 9d3a5d7。
