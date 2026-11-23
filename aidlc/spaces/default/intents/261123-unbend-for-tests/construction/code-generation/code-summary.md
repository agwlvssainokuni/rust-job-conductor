# コード生成のまとめ(Code Summary): テストの都合で曲げられた本番のコードと、残りの整理

計画 `code-generation-plan.md` の Step 1〜14 を実装した。基準は HEAD ca9d1d5(コードの計画のコミット。コードは c5f1a3a と同じ)。コードと仕様書のコミットは 24 件(段 1: 13、段 2: 11)で、`functional-spec.md` §8 の並び 1〜25 と 1 対 1 に対応する(14 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。本番から、テストのためだけの要素をなくし、テストの側で代わりを持つ。本番に、テストのための要素を新しく足さない。

## コミットの一覧

| # | §8 | 段 | ハッシュ | 内容 |
|---|----|----|----------|------|
| 1 | 1 | 1 | `b82fc7f` | cli: 言語と色の判定の遅延を偽の Env の記録で確かめ、Reporter の確認用のメソッドを消す(1a。T22 の一部、T23) |
| 2 | 2 | 1 | `d95d964` | runtime・cli: reset --all の結果から消した数を消す(1b。T14、T16、T22 の一部、T27) |
| 3 | 3 | 1 | `f0dcd12` | core: 本番の呼び出しの無い graph::is_lane_drawable を消す(1c。T6、T7) |
| 4 | 4 | 1 | `95279bf` | core・cli: テストの便宜の LoadedEnvFiles::add を消す(1d。T5 の add_env_file、T8、T28) |
| 5 | 5 | 1 | `4e6c37a` | runtime: remove_stale_temps は今の時刻を関数の中で取る(1e。T18 の一部) |
| 6 | 6 | 1 | `67eb278` | core のテスト: 誤りの種類の一覧の写し tests/common/kinds.rs(1g。T3、T4、T5 の kinds、T1・T2 の ALL の部分) |
| 7 | 7 | 1 | `38ae90d` | converter のテスト: 誤りの種類の一覧の写し(1g。T20 の ALL の部分、T21) |
| 8 | 8 | 1 | `715f396` | cli のテスト: 5 つの一覧の写し(1g。T22 の kinds、T24)。§4.6 の args_grammar.rs の 1 件を消す |
| 9 | 9 | 1 | `ba4e1b0` | core・runtime・converter・cli: 本番の ALL 5 つを消す。§4.6 の src の単体テスト 2 件を消す |
| 10 | 10 | 1 | `b042bfe` | core・converter: code() を消し、型の説明に識別子の規則を書く(1f。★ T1、★ T2、T20 の残り) |
| 11 | 11 | 1 | `fbd770a` | cli: パニックのフックの差し替え口をなくし、本体を単体テストへ(1h。T29、§4.6) |
| 12 | 12 | 1 | `03e6fd6` | cli: シグナルの AlreadyRegistered と登録済みの印をなくす(1i。★ T30、T22 の一部、§4.6) |
| 13 | 13 | 1 | `1091276` | core: node_state_of を非公開に、flow_edges を crate の中に(1j。T9、T10、T11) |
| — | 14 | 1 | (コミットなし) | 段 1 の確かめ(完了の判定の検索と件数。下の記録) |
| 14 | 15 | 2 | `4bf3117` | cli・core・runtime・converter: 古くなった説明(2a-1、2a-3〜2a-12。コメントだけ) |
| 15 | 16 | 2 | `a8764bd` | 仕様書: .env の重複キーの警告は validate と graph が出す(2a-2。6 か所) |
| 16 | 17 | 2 | `4ad19ab` | core・runtime・cli: PipelineState の definition_key と job_order を型から消す(2b。T5 の state、T12、T13、T15、T17、T18 の一部、T19、T25) |
| 17 | 18 | 2 | `0939f39` | runtime・cli: CommandError の理由を CommandErrorKind::reason() に(2c。T26、T31) |
| 18 | 19 | 2 | `73f762d` | core: ExitCodeClass を消し、classify_exit_code が JobOutcome を返す(2d) |
| 19 | 20 | 2 | `6a31b0a` | core・cli: EXIT_* を classify.rs の中だけに、run_dry_run を非公開に(2e-1、2e-2) |
| 20 | 21 | 2 | `78d407c` | runtime: 常に真の条件と、テストだけの use の置き場所(2e-3、2e-4。T18 の一部) |
| 21 | 22 | 2 | `4e3eee5` | converter: convert の Path の包み直しをやめる(2e-5) |
| 22 | 23 | 2 | `d83ca20` | core・cli: ステートが無いときの判断を decide_from_state の 1 か所に(2f) |
| 23 | 24 | 2 | `7ea5380` | core のテスト: decide_from_state の単体テスト(N1) |
| 24 | 25 | 2 | `81ea255` | 仕様書: テストのファイルの表と、名前の変わったテストの参照を合わせる |

各コミットで `cargo build --locked --workspace`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` を、失敗で止まる確認のスクリプトで通した(整形は `cargo fmt --all`)。シグナルの Windows の写しを変えた 12 と、最後に、`--target x86_64-pc-windows-msvc` の clippy も通した。

## 件数の移り変わり(macOS、rustc 1.100.0、debug)

| 時点 | core 単体 | core 統合 | runtime 単体 | runtime 統合 | converter 単体 | converter 統合 | cli 単体 | cli 統合 | 合計 |
|------|-----------|-----------|--------------|--------------|----------------|----------------|----------|----------|------|
| 基準(ca9d1d5) | 113 | 271 | 70 | 128 | 53 | 64 + 1 無視 | 0 | 462 | **1,161 通過、1 無視** |
| 段 1 の後(1091276) | 112 | 271 | 70 | 128 | 52 | 64 + 1 無視 | 2 | 457 | **1,156 通過、1 無視**(見込みどおり) |
| 段 2 の後(81ea255) | 113 | 271 | 70 | 128 | 52 | 64 + 1 無視 | 2 | 457 | **1,157 通過、1 無視**(見込みどおり) |

コミットごとの合計: 1〜7 は 1,161、8 は 1,160(args_grammar の 1 件)、9・10 は 1,158(src の単体 2 件)、11 は 1,157(panic_hook の 1 件。2 件は移動)、12〜23 は 1,156(flow_run の 1 件)、24〜25 は 1,157(N1)。失敗は常に 0。

## テストの変更

### 変えたテスト(T 表)

変えたテストのファイルは、承認済みの T1〜T31 の範囲だけ(`git diff --stat ca9d1d5 HEAD -- 'crates/*/tests'` の 31 ファイルと、src の単体テストの `flow/record.rs`、`consistency.rs`、`state/writer.rs`、`state/reader.rs`、runtime `error.rs`、cli `panic_hook.rs`)。確かめる中身と期待する出力の文字列は変えていない。

- core: `env_types.rs`(★ T1、★ T2、T3)、`nfr_security.rs`(T4)、`common/mod.rs`・新しい `common/kinds.rs`(T5)、`graph_drawable.rs`(T6)、`graph_layout_oracle.rs`(T7)、`plan_types.rs`・`env_merge_perf.rs`・`plan_perf.rs`・`env_merge.rs`(T8)、`graph_edges.rs`(T9)、`graph_lookup_perf.rs`(T10)、`graph_types.rs`(T11)、src `flow/record.rs`(T12)、`consistency.rs`(T13)
- runtime: `run_id_state.rs`(T14、T15。`:313` の残したファイル 2 つの有無の確かめを足した)、`state_read.rs`(T16)、`state_write.rs`(T17)、src `state/writer.rs`(T18)、`state/reader.rs`(T19)、`error.rs`(T31)
- converter: `nfr_security.rs`(T20)、`common/mod.rs`・新しい `common/kinds.rs`(T21)
- cli: `common/mod.rs`・新しい `common/kinds.rs`(T22)、`report_messages.rs`(T23、T24、T25、T26)、`graph_report.rs`・`nfr_robust.rs`・`flow_pipeline.rs`・`plan_report.rs`(T25)、`flow_run_id.rs`(T27)、`plan_report.rs`(T28)、`src/panic_hook.rs` の単体テスト(T29)、`signals_real.rs`(★ T30)

### 名前を変えたテスト(★ 3 件)

| ID | 前 | 後 |
|----|----|----|
| T1 | `定義の誤りの種類は_33_種で識別子が重複せず表示が空でない` | `定義の誤りの種類は_33_種で表示が空でない` |
| T2 | `env_file_の誤りの種類は_13_種で識別子と理由が重複しない` | `env_file_の誤りの種類は_13_種で理由が重複しない` |
| T30 | `最初のシグナルの番号だけが残り_2_回目の登録は登録済みになる` | `最初のシグナルの番号だけが残る` |

### 置き場所を変えたテスト(2 件。名前は同じ)

cli の統合テスト `tests/panic_hook.rs` の `フックの本体は固定の英語の文を書き_70_で終わる`、`標準エラーに書けなくても_70_で終わる` → cli の単体テスト `panic_hook::tests::…`(T29)。`tests/panic_hook.rs` は消した。

### 消したテスト(§4.6 の 5 件)

| テスト | コミット |
|--------|----------|
| cli `tests/args_grammar.rs` の `使い方の誤りの全種類が網羅されている` | `715f396` |
| core `src/error.rs` の単体テスト `全種類に識別子と表示があり重複しない` | `ba4e1b0` |
| converter `src/error.rs` の単体テスト `表示は_どの種類でも入力の値を含まない` | `ba4e1b0` |
| cli `tests/panic_hook.rs` の `設定したフックは_パニックのとき終了の関数を_70_で呼ぶ`(フックの設定そのものは確かめられなくなる。仕様書 rust 10・11、language-agnostic 11 に記録) | `fbd770a` |
| cli `tests/flow_run.rs` の `すでに登録済みでも_流れは成功と同様に続ける` | `03e6fd6` |

### 足したテスト

N1: core `src/flow/decide.rs` の単体テスト `ステートが無いときは_記録が空で再開位置も_end_もないときと同じ判断`(`7ea5380`)。

### テスト名の一覧の比較(`cargo test --locked --workspace -- --list 2>/dev/null | sort`)

- 基準 → 段 1 の後: 差は、消した 5 件、★ の 3 件の名前の変更、`panic_hook` の 2 件の置き場所の変更(統合 → `panic_hook::tests::…`)だけ。
- 段 1 の後 → 段 2 の後: 差は、N1 の 1 件の追加だけ。
- 一覧のファイルの件数の行は、core 単体 113 → 112 → 113、converter 単体 53 → 52、cli 単体 0 → 2、cli の `panic_hook` の 3 件と `args_grammar` 61 → 60 の変化だけ。

### コンパイル時の確かめ(D1)

一覧の写しの並びを崩すと、`cargo test --no-run` が `evaluation panicked: assertion failed: definition_position(DEFINITION_ERROR_KINDS[i]) == i` で止まることを、core の写しの 2 つの要素を一時的に入れ替えて確かめ、すぐに戻した(コミットしていない)。

## 完了の判定の検索の結果

### 段 1(1091276 の時点。最後の木でも同じ結果)

| 検索 | 件数 |
|------|------|
| `grep -rn "language_resolved\|color_resolved" crates docs/spec/rust docs/spec/language-agnostic` | 0 |
| `grep -rn "\.removed\b\|removed:" crates` | 0 |
| `grep -rn "is_lane_drawable" crates docs/spec/rust docs/spec/language-agnostic` | 0 |
| `grep -rn "\.add(" crates` / `grep -n "pub fn add\b" crates/core/src/envfile/mod.rs` | 0 / 0 |
| `grep -n "remove_stale_temps(.*now\|now: SystemTime" crates/runtime/src/state/writer.rs` | 0 |
| `grep -rn "fn code(self)\|kind\.code()\|k\.code()\|Kind::[A-Za-z]*\.code()" crates` | 0 |
| `grep -rn "code()" docs/spec/rust docs/spec/language-agnostic \| grep -v "status.code()"` | 0 |
| `grep -rn "::ALL\b" crates \| grep -v "CommandKind::ALL\|Self::ALL"` | 0 |
| `grep -rn "pub const ALL" crates/*/src` | 1(cli `types.rs` の `CommandKind`) |
| `grep -rn "install_panic_hook_with\|on_panic" crates docs/spec/rust docs/spec/language-agnostic`、`ls crates/cli/tests/panic_hook.rs` | 0、無い |
| `grep -rn "AlreadyRegistered\|REGISTERED\|SignalMode::Already" crates docs/spec/rust docs/spec/language-agnostic` | 0 |
| `grep -rn "flow_edges\|node_state_of" crates \| grep -v "^crates/core/src/graph/"` | 0 |
| `grep -n "pub use edges::flow_edges\|node_state_of" crates/core/src/graph/mod.rs` | 0 |
| `grep -rn "doc(hidden)" crates/*/src` | 4(converter の `testing` の 1 件と、その説明の文 3 行) |

消した名前の参照は、本番のコード(src の `#[cfg(test)]` より前)だけでなく、テストを含めた `crates` の全体で 0 件。

### 段 2(81ea255 の時点)

| 検索 | 件数 |
|------|------|
| `grep -rn "validate\` だけ\|validate だけ" crates/*/src docs/spec/rust docs/spec/language-agnostic` | 0 |
| `grep -rn "自動の再試行はしない\|自動の再試行もしない" crates/*/src`、`grep -rn "再試行しない。" crates/cli/src crates/core/src` | 0、0 |
| runtime `lib.rs` の表の `bounded_read` の行 / converter `lib.rs` の表の `raw`・`scalar` の行 | 1 / 2 |
| `grep -rn "\.definition_key\|definition_key:\|\.job_order\|job_order:" crates` | 0 |
| `grep -rn "\"definition_key\"\|\"job_order\"" crates/runtime/src`(単体テストの `VALID` と `replace` を除く) | 6(書き出し 2、読み込みのキーの一覧 2、読み込みの検査 2。基準の c5f1a3a と同じ 6 か所。inventory の「4 か所」はキーの一覧を数えていなかった) |
| `grep -rn "pub reason: &'static str" crates/runtime/src/error.rs` | 1(`StateError`) |
| `grep -rn "CommandError::new" crates` | 0 |
| `grep -rn "ExitCodeClass" crates docs/spec` | 0 |
| `grep -rn "pub(crate) const EXIT_\|pub(crate) fn run_dry_run" crates/core/src crates/cli/src` | 0 |
| `grep -n "if bytes.is_empty() =>" crates/runtime/src/state/reader.rs` | 0 |
| `grep -n "^#\[cfg(test)\]" crates/runtime/src/state/writer.rs` | 1(`mod tests`) |
| `grep -n "Path::new(&request" crates/converter/src/lib.rs` | 0 |
| `grep -rn "fn decide(" crates/cli/src` | 0 |
| `grep -rn "None => (&\[\]\[..\], None, false)" crates` | 1(core `flow/decide.rs`) |

## ステートファイルの比較(2b)

`target/` の下の一時のディレクトリに、同じ定義(4 ジョブ。`a` は成功で `c` へ遷移して `b` をスキップ、`c` は `false` でエラー)を置き、2b の前(a8764bd のバイナリ)と後(4ad19ab のバイナリ)で、次を実行して作ったステートファイルを `cmp` で比べた。3 つとも 1 バイトも違わない(確かめたあと、作業のディレクトリは消した)。

| 実行 | ファイル | 大きさ | SHA-256 の先頭 | 結果 |
|------|----------|--------|----------------|------|
| `conductor pipeline p`(終了 1。再開位置 `c`) | `conductor.toml.state` | 402 バイト | `342f096564ec7ac9` | 同じ |
| `conductor run a`(終了 0。`b` を遷移でスキップ) | `conductor.toml.state` | 370 バイト | `8ece052fabdc9c4d` | 同じ |
| `conductor run --run-id r1 a` のあと `run --run-id r1 c`(終了 1) | `conductor.toml.r1.state` | 416 バイト | `6d7f09559fbaf61e` | 同じ |

## 性能のテスト P4(1j)

`graph_lookup_perf.rs` の P4(上限 600 ms。debug)の中央値: 変える前 117.2 ms(`flow_edges` を測る)、変えた後 121.3 ms・121.8 ms・132.3 ms(3 回。`build_graph(&def, None).edges` を測る)。上限の 1/3(200 ms)を下回る。

## 合格の条件

- `cargo fmt --all -- --check`: 通過
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 通過(Windows の対象 `x86_64-pc-windows-msvc` でも通過)
- `cargo test --locked --workspace`: 1,157 通過、1 無視、0 失敗
- `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`: 通過(行 98.12%、領域 97.81%、関数 98.31%)
- MSRV: ローカルの rustc が 1.100.0(= MSRV)で、上のすべてを通した
- `git diff ca9d1d5 HEAD -- '*Cargo.toml' Cargo.lock`: 空(依存も版も変えていない)
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)と README: 変更なし
- `unsafe`、src の `cfg(unix|windows)` は足していない。cli・runtime の `nfr_source.rs` が通る

## 計画からのずれ

1. **inventory に無かった仕様書の記載も合わせた**(仕様書だけ。テストは変えていない)。
   - 1g: rust `06-state-file.md:448`、`10-errors-and-exit-codes.md:121`・`:141` の `ALL: [...]` の記載(`ba4e1b0`)。
   - 1i: rust `02-architecture.md` の流れの図(`登録済みは続行`)と `FakeSignals` の行、`05-execution-flow.md:263`、`10-errors-and-exit-codes.md:32`(`03e6fd6`)。
   - 2f: rust `05-execution-flow.md` の 4.1 の擬似コード(`decide` → `decide_from_state`)、5.2・5.3・16 章の擬似コード、`02-architecture.md` のモジュールの表(`step.rs`、`flow/decide.rs`)(`d83ca20`)。
2. **language-agnostic の「すでに登録済み」の契約の文は、計画の範囲では変えなかった**(のちにユーザーの決定で直した。下の箇条)。inventory は「language-agnostic には記載が無い」としていたが、英語の `AlreadyRegistered` ではなく日本語の「すでに登録済み」で、`02-architecture.md:89`・`:147`、`05-execution-flow.md:284`、`07-process-and-signals.md:323`・`:326`・`:365`・`:460`、`10-errors-and-exit-codes.md:32`・`:102`・`:253`、`12-rebuild-guide.md:216` にある。これは言語によらない契約(10 章 102 行は「実装が複数回呼ぶ構造のときだけ関係する」と書く)で、承認済みの範囲に無いため変えなかった。参照実装は登録をプロセスで 1 回だけ呼ぶので、この条項には当たらない。テストの名前の欄(11 章の AC-285 の ★ 名前の変更、AC-299 の消したテスト、AC-348 の消したテスト)だけは §8 の 25 として合わせた(`81ea255`)。
   - **ユーザーの決定(2026-11-23)で足した**: `6dac72e` で、language-agnostic の 02・05・07・10・12 章の「すでに登録済み」の文を「シグナルの監視の登録はプロセスで 1 回だけ(2 回登録したときの動きは契約の外)」に直し、シグナルの準備の誤りを「失敗」(70)の 1 種にした。11 章の AC-285 は 1 回の登録に合わせた。場面の無くなる AC-299 は、AC の番号が連番で範囲(12 章の「AC-280〜301」)から参照されているため、行を消さずに番号を残して「(該当なし)」にした(版を上げないので「v0.9.8 までは」の版の注記は付けない)。凍結した仕様書は変えていない。
3. **runtime のテストの補助のコメントの古い参照**: `crates/runtime/tests/flow_with_state.rs:140` の補助 `decide` の説明が「cli の `step::decide`」(2f で消した関数)を指していた。T 表に無いテストのファイルなので最初は変えなかったが、**ユーザーの決定(2026-11-23)で** `e47877a` として、コメントの 1 行だけを「core の `flow::decide_from_state` と同じ」に直した(テストの中身は変えていない)。
   - 追加の 2 コミットでも、build、clippy(`-D warnings`)、全テスト(1,157 通過、1 無視)を通した。
4. **2f の `plan_run`**: 判断は `decide_from_state` にまとめたが、模擬の分類(`simulate_job` → `classify_and_decide`)に渡す状態の一覧が要るため、Run の枝で `active_state(input).map_or(&[], |s| s.job_states.as_slice())` を取り出す(判断の `match` の重複ではない)。
5. **T14 の `:297`**: `assert_eq!(report.removed, 1)` を消すと `report` が使われなくなるので、`let report =` を外して呼び出しだけにした。
6. **T1・T2**: 消した確かめ(種類の数と重複)の代わりに、写しの型とコンパイル時の確かめが保つことを、テストの中の 1 行のコメントで書いた。
7. **1c**: `is_lane_drawable` の説明(「対応する形」の意味)を `layout_graph` の説明に移した。
8. **2a-2**: 「`validate` と `graph` が表示する」の参照を、03 章の 2 か所は「04 の 7.5、7.7」にした(`graph` の節が 7.7)。
9. コミットの数は 24(§8 の「24 コミットほど」のとおり。14 はコミットなし)。分けたりまとめたりはしていない。

指示に反した点は無い(push・タグ・リリース・版の更新なし。lifecycle の aidlc コマンドは実行していない。ファイルの編集はファイルのツールと `cargo fmt --all` だけ。git のソースは読んでいない。プロジェクトの外にファイルを作っていない。作業ファイルは `target/unbend-cg/` に置き、終わったあと消した。作業の前からある audit の `.md` はコミットしていない)。
