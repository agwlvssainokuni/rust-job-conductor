# コード生成のまとめ(Code Summary): 残りの整理と読み込みの順序のそろえ

計画 `code-generation-plan.md` の Step 1〜13 を実装した。基準は HEAD 78f6040(v0.9.7。計画のコミット)。コードと仕様書のコミットは 45 件(段 1: 35、段 2: 10)。`functional-spec.md` §8 の並び 1〜46 のうち、**コミット 23(1g-4、`ConversionError.reason` を `&'static str` に)は進めなかった**(「計画からのずれ」の 1)。ほかに、この記録のコミットが 1 件。push、タグ、リリースはしていない。版は最後のコミットで 0.9.8 にした。

## コミットの一覧

| # | §8 | 段 | ハッシュ | 内容 |
|---|----|----|----------|------|
| 1 | 1 | 1 | `16e9edf` | runtime: 古くなった説明を直す(1a) |
| 2 | 2 | 1 | `d2a307d` | cli: 古くなった説明と守りの理由を直す(1a) |
| 3 | 3 | 1 | `67e1e1c` | core: apply_interrupt_in を消す(1b) |
| 4 | 4 | 1 | `68ba811` | core: EffectiveEnv の job_id と names() を消す(1b。T1a、★ T1b) |
| 5 | 5 | 1 | `5ff1064` | core のテスト: 補助の使わない引数を外す(1b。T2、T3) |
| 6 | 6 | 1 | `907a864` | core: 模擬の分類を classify_and_decide の結果から取る(1b) |
| 7 | 7 | 1 | `45f6e2f` | core: decide_before_run の常に真の条件を外す(1b) |
| 8 | 8 | 1 | `b519b4e` | core: 記録の hint を Option<usize> にそろえる(1c-4c・1c-4d) |
| 9 | 9 | 1 | `90aab54` | cli: interrupted_signal を消す(1b) |
| 10 | 10 | 1 | `7e1550d` | runtime: command_line を check_command に畳む(1b) |
| 11 | 11 | 1 | `9d252cd` | runtime: replace_via_temp(1c-1) |
| 12 | 12 | 1 | `2d4bfdf` | runtime: ステートの読み込みの段をなくす(1c-2 前半。T4 の parse_state) |
| 13 | 13 | 1 | `1e94d60` | runtime: bounded_read.rs(1c-2 後半。T4 の移動) |
| 14 | 14 | 1 | `4f50d9c` | core: build_retry(1c-3、D3) |
| 15 | 15 | 1 | `46274cf` | core・converter: ごく小さい整理(1c-4a・b・e) |
| 16 | 16 | 1 | `c42262e` | cli: Runtime::run_job から job を外す(1d。T15 の偽物) |
| 17 | 17 | 1 | `017c1bb` | core: flow の下位モジュールと EXIT_* を crate の中に(1e) |
| 18 | 18 | 1 | `b893c20` | core: plan・envfile・graph の下位モジュールと定数を crate の中に(1e。T5) |
| 19 | 19 | 1 | `309d89b` | runtime: pub(crate) を狭める(1f。10 個) |
| 20 | 20 | 1 | `3037c5c` | cli: escape_path を escape_full に(1g-1。T16、T17) |
| 21 | 21 | 1 | `7ff6033` | cli: run ID の有無を StateContext で渡す(1g-2。T18) |
| 22 | 22 | 1 | `8cc74a4` | converter: 行番号を Option<usize> に(1g-3) |
| — | 23 | 1 | (なし) | **進めなかった**(1g-4。下の「計画からのずれ」の 1) |
| 23 | 24 | 1 | `f2399c7` | core: env の表の大文字小文字の検査を HashSet に(1h-1) |
| 24 | 25 | 1 | `66f20fb` | core: 遷移の重複の確認を種類ごとの真偽値に(1h-2) |
| 25 | 26 | 1 | `c5ac260` | core: env_file_refs の重複の確認を HashSet に(1h-3) |
| 26 | 27 | 1 | `8581b0e` | core のテスト: N1、N2、N3 |
| 27 | 28 | 1 | `b66de7d` | cli: -v の図の辺を出どころのジョブで 1 回まとめる(1h-4) |
| 28 | 29 | 1 | `2059bbc` | cli: record_skip の土台を借りる(1h-5) |
| 29 | 30 | 1 | `4c4c9d3` | core のテスト: test_support(1i-1。T6) |
| 30 | 31 | 1 | `0329fea` | cli のテスト: for_language、run_with_fakes、state_error の共有(1i-2。T15 の残り、T20、T21、T24) |
| 31 | 32 | 1 | `f3327f3` | runtime のテスト: def・put・paths を common に(1i-3。T9〜T13) |
| 32 | 33 | 1 | `f372225` | runtime のテスト: run_job の名残を command_line に(1i-5。T7、T8、T13) |
| 33 | 34 | 1 | `a43bc35` | core のテスト: 深さの境界の重複した単体テストを消す(§4.6 の 1・2) |
| 34 | 35 | 1 | `9912269` | converter のテスト: 深さの重複したテストを消す(§4.6 の 3・4) |
| 35 | 36 | 1 | `aeaec20` | 仕様書: 性能の表とテストのファイルの表(rust 11、language-agnostic 11) |
| 36 | 37 | 2 | `b3a8a0b` | converter: 読み込みを形 B に(2a) |
| 37 | 38 | 2 | `b123710` | converter のテスト: N4 |
| 38 | 39 | 2 | `66bb9ba` | 仕様書: converter の取り込み(rust・language-agnostic 09、language-agnostic 11 の AC-212a) |
| 39 | 40 | 2 | `76aa4a3` | runtime: ステートの読み込みを形 B に(2b。★ T23。read_limited を非公開に) |
| 40 | 41 | 2 | `f07872e` | runtime のテスト: N5、N8 |
| 41 | 42 | 2 | `8627fad` | 仕様書: ステートの読み込み(rust・language-agnostic 06、rust 02・03・12) |
| 42 | 43 | 2 | `ffbe2c0` | runtime: 空白だけの先頭の語を Empty に(2c) |
| 43 | 44 | 2 | `a3dcfc5` | runtime・cli のテスト: N6、N7 |
| 44 | 45 | 2 | `5e12c92` | 仕様書: 空白だけのコマンド名(rust・language-agnostic 07、rust 12、language-agnostic 11 の AC-097a) |
| 45 | 46 | 2 | `5b34d7f` | 版を 0.9.8 にする(★ T22。7239aa2 と同じ 22 ファイル) |

## 件数の移り変わり(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug)

| 時点 | core | runtime | converter | cli | 全体 |
|------|------|---------|-----------|-----|------|
| 基準(78f6040) | 383(単体 115、統合 268) | 195(68、127) | 118 + 1 無視(54、64 + 1) | 461 | **1,157 通過、1 無視** |
| 段 1 の後(aeaec20) | 384(113、271) | 195(68、127) | 116 + 1 無視(53、63 + 1) | 461 | **1,156 通過、1 無視**(見込み 1,156) |
| 段 2 の後(5b34d7f) | 384(113、271) | 198(70、128) | 117 + 1 無視(53、64 + 1) | 462 | **1,161 通過、1 無視、0 失敗**(見込み 1,161) |

途中: N1〜N3 の追加で 1,160、§4.6 の削除で 1,158 → 1,156、N4 で 1,157、N5・N8 で 1,159、N6・N7 で 1,161。各コミットの前に fmt・build・clippy(`-D warnings`)・全テストを通した。

## テストの変更(承認済みの T 表)

| ID | 変えた所 | コミット |
|----|----------|----------|
| T1a | core `tests/env_merge.rs` の `env.names()` をファイルの中の補助 `origins` に | `68ba811` |
| ★ T1b | 同 `assert_eq!(e3.job_id, "a")` を `assert_eq!(get(&e3, "CONDUCTOR_JOB_ID"), Some("a"))` に | `68ba811` |
| T2 | core `src/flow/record.rs`・`next_step.rs` の単体テストの補助 `exec`・`outcome`・`step` の使わない引数(呼ぶ所 37 か所) | `5ff1064` |
| T3 | core `tests/retry_flow.rs` の `exec()`・`exited(code)`(呼ぶ所 16 か所) | `5ff1064` |
| T4 | runtime `src/state/reader.rs` の補助 `parse_state` の本体、単体テスト `上限_1_つぶんだけ読む` を `bounded_read.rs` へ移動(名前と中身は同じ) | `2d4bfdf`、`1e94d60` |
| T5 | core `tests/graph_types.rs` の `graph::overlay::node_state_of` を `graph::node_state_of` に | `b893c20` |
| T6 | core の src の単体テストの補助 7 ファイル(types、consistency、flow の resume・decide・next_step・classify・record)を `test_support` と構造体の更新の書き方で | `4c4c9d3` |
| T7 | runtime `src/runner.rs` の単体テストの `job`・`line` を `line(command)` 1 つに | `f372225` |
| T8 | runtime `tests/runner_basic.rs`・`runner_signal.rs`・`runner_env.rs`・`nfr_robust.rs` の `job(..).command` を `command_line(..)` に。`runner_basic.rs` の `_j` を消し、`'` の確認は `check_command(&Command::from("'"))` | `f372225` |
| T9〜T12 | runtime `tests/state_read_raw.rs`・`run_id_state.rs` の `def`・`put`、`state_read.rs`・`state_write.rs` の `paths` を `common` に | `f3327f3` |
| T13 | runtime `tests/common/mod.rs` に `def`・`put`・`paths`・`command_line` を足す | `f3327f3`、`f372225` |
| T15 | cli `tests/common/mod.rs`: 偽物の `run_job` から `job` を外し `job_id_of(env)`(D1)、`state_error` の理由を `"reason"`(D12)、`run_os`・`run_mem` は `run_with_fakes`、`FakeEnv::for_language` | `c42262e`、`0329fea` |
| T16・T17 | cli `tests/escape_table.rs`・`graph_long_id.rs` の `escape_path` を `escape_full` に | `3037c5c` |
| T18 | cli `tests/report_messages.rs` の `with_run_id: false` を `context: StateContext::Read` に | `7ff6033` |
| T19 | **進めなかった**(1g-4 を進めなかったため不要) | — |
| T20 | cli `tests/report_messages.rs` の `env_for`・`state_error` を 1 行の包みに | `0329fea` |
| T21 | cli `tests/flow_init.rs` の `run_with` を `for_language` と `run_with_fakes` に | `0329fea` |
| ★ T22 | cli `tests/binary_basic.rs` の `"conductor 0.9.7\n"` を `"conductor 0.9.8\n"` に | `5b34d7f` |
| ★ T23 | runtime `tests/state_read.rs` の `終わりのない入力でも上限で止まる` を、期待 `Unreadable`、名前 `文字デバイスへのリンクのステートは開かずに_unreadable` に | `76aa4a3` |
| T24 | cli `tests/nfr_robust.rs` の `utf8_でない引数でパニックしない` の `Deps` の組み立てを `run_with_fakes` に | `0329fea` |

### 消したテスト(§4.6。D10)

| 消したテスト | 同じことを確かめて残るテスト | コミット |
|--------------|-------------------------------|----------|
| core `definition::depth::tests::配列の深さ_32_は受理し_33_は拒否` | core `definition_errors.rs` の `深さ_32_は受け入れ_33_と_1000_は_syntax`、`深さの上限と巨大な入れ子`(行番号は単体テスト `深い入力でも行番号を返す`) | `a43bc35` |
| core `definition::depth::tests::インラインテーブルの深さ_32_は受理し_33_は拒否` | 同上(使われなくなる補助 `inline_tables` も消した) | `a43bc35` |
| converter `yaml_read::tests::とても深い入力は_異常終了せずに失敗する` | converter `read_resolve.rs` の `極端に深い入力は_異常終了せずに文法の誤りで失敗する`。消す前に、単体テストだけにあった閉じない流れのマップ(`a: {a: …`)と末尾の値の無いブロックの配列(`- - …`)の 1000・100,000 段を、統合テストの入口(`common::yaml`。読み取りと展開)で一時に試し、4 つとも `syntax` で、合わせて約 2.2 ms で返ることを確かめた(一時のテストは消した) | `9912269` |
| converter `読み取りの深さの上限_32_と_33`(統合) | converter `yaml_read::tests::深さ32は受理し_33は文法の誤り` | `9912269` |

### 足したテスト(N1〜N8)

| ID | 場所 | コミット | 直す前の木 |
|----|------|----------|------------|
| N1 | core `tests/definition_perf.rs` `p1_一万項目の_pipeline_env_の定義の読み込みは_350ms_以内` | `8581b0e` | 中身は通り、時間の上限で落ちた |
| N2 | 同 `p2_一つのジョブに重複した遷移が四万個の定義の読み込みは_1600ms_以内` | `8581b0e` | 同上 |
| N3 | core `tests/env_refs_perf.rs` `p3_一万個の_env_file_の記述の一覧は_40ms_以内` | `8581b0e` | 同上 |
| N4 | converter `tests/convert_ac.rs` `名前付きパイプの入力は開かずに_unreadable`(UNIX) | `b123710` | 5 秒で返らず落ちた(Timeout) |
| N5 | runtime `tests/state_read.rs` `名前付きパイプと権限のない大きなステートは開かずに_unreadable`(UNIX) | `f07872e` | `read_state` が 5 秒で返らず落ちた。権限の部分だけを分けた一時のテストは直す前も通った |
| N6 | runtime `src/split.rs` 単体 `空白だけの先頭の語はコマンド名がない` | `a3dcfc5` | `'  '` が `Ok` で落ちた |
| N7 | cli `tests/flow_validate.rs` `空白だけのコマンド名は_64_で_ジョブを起動しない` | `a3dcfc5` | `validate` が 0 で落ちた(64 を期待) |
| N8 | runtime `src/bounded_read.rs` 単体 `終わりのない入力でも上限_1_バイトで読み止めて_too_large` | `f07872e` | 直す前(段 2 の前)も通った |

直す前の木は、`git worktree add --detach target/lf/before aeaec20`(段 1 の後、段 2 の前)に、ファイルのツールでテストを書き足して走らせ、確かめたあと `git checkout -- .` で戻し、worktree を消した(作業の木は stash・reset で触っていない)。★ T23 の直す前の期待(`TooLarge`)は、直す前の木で通り、直した後は同じ入力が `Unreadable` になる(名前を変えたテストがそれを確かめて通る)。

## 性能の直す前と後(debug、5 回の中央値、macOS)

| 測定 | 直す前(2 回) | 直した後(2 回) | 上限 | 備考 |
|------|----------------|------------------|------|------|
| P1(N1)`[pipeline.env]` 10,000 項目、118,977 バイト | 645 / 617 ms | 41.0 / 41.0 ms | 350 ms | 見積もり 72 ms |
| P2(N2)遷移 20,000 + 20,000(1 行に 1 つ)、1,000,146 バイト | 2,032 / 2,035 ms | 237 / 241 ms | 1,600 ms | 見積もり 324 ms。入力は inventory の 960,120 バイトの形(1 行)と違い、誤りの位置の順を行で確かめるため 1 行に 1 つにした |
| P3(N3)`env_file_refs` 10,000 + 10,000 | 624 / 616 ms | 10.1 / 6.7 ms | 40 ms | 見積もり 7 ms |
| M4 `-v` の図(ジョブ 1,000、辺 1,998、`render_text` 1 回) | 19.8 ms | 6.8 / 6.6 ms | (テストなし。D8) | 出力の長さは同じ 93,417 バイト。一時のテストで測り、消した |
| M5 `record_skip` の土台(999 回。複製・`apply_skip`・比較の部分。書き込みを除く) | 162.8 ms(複製する形) | 97.5 ms(借りる形) | (テストなし。D8) | `record_skip` は crate の中なので、同じ処理を一時のテストで 2 つの形に分けて測り、消した |

## テスト名の比較(`cargo test --locked --workspace -- --list | sort`)

- 基準 → 段 1 の後: 消えたもの 4(§4.6 の 4 つ)、足したもの 3(N1〜N3)、置き場所の変わったもの 1(`state::reader::tests::上限_1_つぶんだけ読む` → `bounded_read::tests::上限_1_つぶんだけ読む`)。ほかの差は件数の行だけ。
- 段 1 → 段 2 の後: 足したもの 5(N4〜N8)、名前の変わったもの 1(★ T23: `終わりのない入力でも上限で止まる` → `文字デバイスへのリンクのステートは開かずに_unreadable`)。ほかの差は件数の行だけ。
- 上のほかの名前の差は無い(予定どおり)。

## §7 の検索の結果(5b34d7f)

| 検索 | 結果 |
|------|------|
| `grep -n "JobDefinition" crates/runtime/src/lib.rs` | 0 件 |
| `grep -n "U3" crates/cli/src/report/messages.rs` | 1 件(`:620` の `--dry-run` の計画の節の見出し「(U3。BR3.1〜…)」。1a-6 の対象の「最終の確定は U3」(`:207`)は消した。残る 1 件は Unit の名前で、1a の対象外) |
| `grep -rn "直接組み立てて" crates/cli/src` | 0 件 |
| `grep -rn "apply_interrupt_in\|fn interrupted_signal\|fn command_line" crates docs/spec` | 2 件(runtime `tests/common/mod.rs` の補助 `command_line`(T13 で計画どおり足したもの)、cli `src/report/mod.rs` の非公開の表示の関数 `command_line`(前からある別のもの))。消す対象の 3 つは 0 件。`grep -rn "fn command_line" crates/runtime/src` は 0 件 |
| `grep -rn "\.names()" crates/core` | 0 件 |
| `grep -rn "pub(crate) fn classify_exit_code" crates` | 0 件 |
| `grep -rn "fs::rename" crates/runtime/src` | 1 件(`temp_file.rs`) |
| `grep -rn "read_to_end" crates/runtime/src` | 1 件(`bounded_read.rs`) |
| `grep -rn "fn read_state_file_raw\|fn parse_state_for\b" crates/runtime/src` | 0 件 |
| `grep -rn "job: &JobDefinition" crates/cli/src/deps.rs crates/cli/tests/common/mod.rs` | 0 件 |
| `grep -rn "escape_path" crates` | 0 件(仕様書 rust 08 の旧名の注記 1 件だけ) |
| `grep -rn "with_run_id" crates/cli` | 0 件 |
| `grep -n "line: 0\|pub reason: String" crates/converter/src` | 1 件(`error.rs:107` の `pub reason: String`。1g-4 を進めなかったため残る)。`line: 0` は 0 件 |
| `grep -n "seen.iter().any\|out.iter().any\|out.contains" crates/core/src/definition/walk crates/core/src/envfile/mod.rs -r` | 0 件 |
| `grep -n "filter(\|e\| e.from_job" crates/cli/src/report/graph_text.rs` | 0 件 |
| `grep -n "\.cloned()" crates/cli/src/step.rs` | 0 件 |
| `grep -c "warning_exit_codes:" …` | `types.rs` 1(製品の型の定義)、`test_support.rs` 1、`classify.rs`・`next_step.rs` 各 1(構造体の更新の書き方で警告コードだけを上書きする補助)、ほか 0。単体テストの手組みは無い |
| `grep -rn "check_command(&j.command)" crates/runtime` | 0 件 |
| `grep -rn "0\.9\.7" crates Cargo.lock` | 0 件 |
| 仕様書の `0.9.7` | 「v0.9.7 以降」「v0.9.7 までは」の注記だけ。「v0.9.7 までは」は rust 02・06・07・08・09・12、language-agnostic 06・07・09・11 |
| `cargo run -q -p conductor-cli --bin conductor -- --version` | `conductor 0.9.8` |
| `Cargo.lock` の差分(基準から) | 版の 4 行だけ |

## 合格の条件

- `cargo fmt --all -- --check`: 通過。
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通過。
- `cargo test --locked --workspace`: 1,161 通過、1 無視、0 失敗。
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 行 98.13%(9,442 行中 177 行が未実行)、関数 98.32%、領域 97.80%。
- MSRV: 手元の rustc は 1.100.0(MSRV と同じ)。
- 新しい依存なし。`unsafe` と src の `cfg(unix|windows)` は足していない(core・converter の src のテストの `cfg(unix)` も増やしていない。名前付きパイプのテストは `tests/` に置いた)。runtime と cli の `nfr_source.rs` が通る。converter の `Cargo.toml` に core は無い。

## 計画からのずれ

1. **1g-4(§8 のコミット 23。`ConversionError.reason` を `&'static str` に)を進めなかった**。型を変えると、承認済みの T 表に無い既存のテスト `crates/converter/tests/nfr_security.rs:328`(`assert!(shown.contains(&e.reason), "{shown}")`)が `clippy::needless_borrows_for_generic_args`(`-D warnings`)で止まる。設計(inventory 1g-4)は「`shown.contains(&e.reason)` はそのまま通る」としていたが、コンパイルは通るものの clippy で止まる。T1〜T24 の外のテストを変えないと進めないので、指示どおりこの項目は進めず(作業中の変更は捨てた)、ほかの項目は続けた。そのため T19 も行っていない。続けるには、`nfr_security.rs:328` の `&e.reason` を `e.reason` にする変更の承認が要る。
2. コミット 8(1c-4c)で `record.rs` の製品のコードの import から `Classification` を外したため(`JobStatus::from(classification)` にすると使われなくなる)、同じファイルの単体テストの `use crate::types::JobDefinition;` を `use crate::types::{Classification, JobDefinition};` にした(T2 の対象のファイルの import の 1 行。確かめる中身は変えていない)。なお、このコミットは最初、clippy の失敗を見落として作り、直してから `--amend` で作り直した(`b519b4e`。push 前)。
3. 1a-2: inventory の目標は「v0.9.6 までは `run_job(job, line, env, signal)` で…」を足す、だったが、指示「コードのコメントに版の番号を書かない」に従い、「以前の `run_job(job, line, env, signal)` の `job` は使われなかった」と版の番号なしで書いた。
4. コミット 11 で `create_temp_beside` も非公開にした(inventory 1f の「1c-1 のあとは `temp_file.rs` の中だけになるので、そのとき非公開にする」に従った)。
5. コミット 17 で、`plan::simulate` が `crate::flow::retry::is_retry_target` を下位モジュールのパスで使っていたので、`flow/mod.rs` に `pub(crate) use retry::is_retry_target;` を足した。
6. 1c-4b と同じ形の失敗の写しのクロージャを、2a の converter の `read_input` にも使った(パスのメタデータと開くときの失敗)。Windows でディレクトリを開いたときの特別扱いは、形 B の 1 の段で同じ結果になるので消した。
7. 段 2 の仕様書の受け入れ基準には、新しい行 `AC-212a`(名前付きパイプの converter)と `AC-097a`(空白だけの先頭の語)を足した。性能の表と新しい AC の行には「v0.9.8 以降」と書いた。
8. 性能 P2 の入力は、誤りの位置の順を行番号で確かめるため 1 行に 1 つの遷移にした(1,000,146 バイト。1 MiB 以内)。
9. 計画の Step 6 のチェックボックスは、1g-4 が残るので付けていない。
