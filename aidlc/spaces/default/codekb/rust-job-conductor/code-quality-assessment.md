# コードの品質の評価

## テスト

| 項目 | 状態 |
|------|------|
| 枠組み | Rust 標準の `#[test]` だけ。テスト名は日本語 |
| `#[test]` の数 | core 472、runtime 259、cli 613、converter 116(合計 1,460)。内訳は下の「テストの数」 |
| 統合テスト | core 36、runtime 16、cli 26、converter 2 のファイル(`tests/*.rs`)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル) |
| 流れのテスト | `cli/tests/flow_*.rs` が、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入して、終了コード・表示・ステートの書き込みの順序を確かめる。`flow_aux.rs` に `watch --follow` のテスト(完了・止まった・描き直し・追記・画面の消去・読めない・合図・1,000 回の繰り返し) |
| 実バイナリのテスト | `cli/tests/binary_*.rs`(`binary_log.rs` を含む)が、`conductor` とテスト専用の補助バイナリを動かす。シグナルは `binary_signal.rs`、`signals_real.rs`、`signals_order.rs` |
| 非機能のテスト | 性能(`*_perf.rs`、`nfr_performance.rs`、`nfr_perf.rs`)、安全性(`nfr_security.rs`)、頑健さ(`nfr_robust.rs`)、ソースの検査(`nfr_source.rs`)、内容(`nfr_content.rs`: ステートに許すのは開始と終了の時刻だけ) |
| 性能のテストの上限 | 「debug の見積もりの約 5 倍」で決めた固定の時間(`plan_simulate_perf.rs:39` の 150ms など)。5 回の中央値で比べる。v0.10.1 は上限を下げずに、計画の模擬の走査を減らした |
| 網羅のテスト | `CommandKind::ALL` などの配列で、種類の追加漏れを捕まえる |
| 参照出力との一致 | `core/tests/graph_layout_oracle.rs` が、`fixtures/graph_layout_oracle*.md` と配置を比べる |
| 文書との一致 | `cli/src/tests/docs.rs`(README と使い方の表)、`cli/tests/msrv_consistency.rs`、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)。`docs/spec/` を読むテストは無い |
| v0.10.1 で足したテスト | 内部の入口と公開の入口の判断と記録が一致する(core、`6104853`)、読む前と読んだ後の止める印(runtime、`37f79e3`)、ログの名前の表の空白を含む行(runtime、`5bab66d`)、デモの場面 4 の待ちの位置(cli、`38a1cb0`) |
| カバレッジ | CI の `coverage` で行 80% 以上を強制(テスト専用の補助バイナリは除外)。チームの方針(test-after、行カバレッジ 80%)と一致する |
| OS | CI の `test` で ubuntu・macos・windows。`msrv-test` で 1.100.0 |

評価: 良い。純粋な中核と差し替え口(時計を含む)の分離で、流れの失敗の経路(書き込みの失敗、シグナル、出力の失敗、時刻)まで偽の実装で試せる。揺れの源は、実時計と固定の時間の上限に頼るテスト。

### テストの数

数え方(2026-11-28、`55958fd`。読むだけのコマンドで数えた。テストは走らせていない):

- 統合テストのファイル: `crates/<crate>/tests/` の直下の `.rs`。補助のディレクトリ(`common/`、`graph_common/`、`bin_support/`、`signal_support/`、`fixtures/`)は含めない。
- `src/tests/` のファイル: `common/` と `mod.rs` を含む全部の `.rs`。cli は直下だけなら 10(`mod.rs` を含む)、テストを持つファイルは 9。converter は 4(`mod.rs` を含む)、テストを持つファイルは 3。
- `#[test]`: 行に `#[test]` だけがあるもの(`grep '^[[:space:]]*#\[test\][[:space:]]*$'`)。文字列の中の一致を含めても同じ値。`#[ignore]` つき 1 件(`converter/tests/nfr_security.rs:190`)を含む。補助のディレクトリと `src/bin/` には無い。

| 項目 | `team.md` の値 | 今の値 |
|------|----------------|--------|
| core の統合テストのファイル | 33 | 36 |
| runtime の統合テストのファイル | 14 | 16 |
| cli の統合テストのファイル | 25 | 26 |
| converter の統合テストのファイル | 2 | 2 |
| `crates/cli/src/tests/` のファイル | 11 | 12(直下だけなら 10) |
| `crates/converter/src/tests/` のファイル | 4 | 4 |
| `#[test]` の合計 | 1,170 | 1,460(core 472 = `tests/` 352 + `src/` 120、runtime 259 = 171 + 88、cli 613 = 408 + 205、converter 116 = 20 + 96) |

- 増えたファイルは、core の `attempt_flow.rs`・`pipeline_activity.rs`・`time_format.rs`、runtime の `log_file.rs`・`runner_log.rs`、cli の `binary_log.rs`・`src/tests/graph_dot.rs`(すべて v0.10.0)。`team.md` の値は v0.10.0 の前の数と読める(推論)。
- 前回の知識ベース(v0.10.1 の intent、`4ebd8e7`)の値は 1,452(core 467、runtime 257、cli 612、converter 116)。v0.10.1 で +8。

## 静的検査と書式

- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が CI の `lint` で関門になる。設定ファイルは無く既定。
- 本体の `#[allow]` は `runtime/src/platform/unix.rs:20` の `#![allow(unsafe_code)]` 1 つだけ。テストの共通の補助(`tests/common/mod.rs` など 6 つ)に `#![allow(dead_code)]` がある。

## CI/CD

- マージの関門(`ci.yml`): `lint`、`build-check`(MSRV、C のビルドの不在)、`test`(3 OS)、`msrv-test`、`coverage`、`docs-check`。`startup-bench` は警告のみ。
- リリース(`release.yml`): タグが `main` の上にあり `crates/cli/Cargo.toml` の版と一致すること → `ci.yml` の再利用 → 4 対象のビルドと smoke test → 下書きのリリース。
- `audit.yml`: `cargo audit`(PR と毎週。警告のみ)。
- 変更は PR を使わず `main` に直接コミットする運用(`team.md`)のため、関門は `push` の CI で事後に効く。
- `docs/spec/` を検査するジョブは無い。

## 文書

- `README.md`(955 行)は、インストール、クイックスタート、使い方(`watch --follow` の節を含む)、定義の書き方(retry、env、`unset_env`、`log_dir`)、ステート(試行の時刻)、終了コード、保証、シグナル、graph、dry-run、init、convert、ロケールを書く。必須の語句は `docs-check` が検査。v0.10.1 で `watch --follow` の注意(`README.md:299`)と名前の置き換えの例を直した。
- 各モジュールの冒頭の `//!` が、責務、契約、規則の番号(BR/NFR/FR、v0.10.0 の Unit、v0.10.1 の BR)、守る性質を日本語で書く。
- `docs/spec/` に 7 版の再構築用の仕様(各 13 文書)。`rust/` と `language-agnostic/` は v0.10.0 の機能を書いていない(下の「仕様書と実装の差」)。

## 仕様書と実装の差

凡例: 「無」= 記述が無い、「古」= v0.9.x の形のまま書いてあり実装と食い違う、「誤」= 数や名前が実装と合わない。行は `docs/spec/rust/` の各ファイルの行(`55958fd`)。実装の側の場所は `api-documentation.md` に書いた。

### 機能ごと(`docs/spec/rust/`)

| 機能(版) | 記述 | 主な場所 |
|------------|------|----------|
| `watch --follow` / `--interval`(v0.10.0) | 無 | 04:20(構文)、04:33-47(オプションの表)、04:174-176(`watch`)、08:497・533(`--help`) |
| `[pipeline] log_dir`(v0.10.0) | 無。逆に「出力を保存しない・読まない」と書く | 01:64、07:206 |
| `unset_env`(v0.10.0) | 無。逆に「取り除かない」と書く | 00:75、01:64、05:634、07:205、12:182 |
| `[pipeline.retry]`・`retry = false`(v0.10.0) | 無 | 03:131-138、03:175、03:230-279、05:452-561 |
| `env_file` の表 `{ path, required }`(v0.10.0) | 無。型は文字列か文字列の配列のまま | 02:80・82、03:138・174・318-328・588、05:703-716 |
| `graph --format dot`(v0.10.0) | 無。`text|mermaid` のまま | 01:30、04:23・41・74・108・209、05:727-847、08:500・515・536・550・1203 |
| ステートの `[[job.attempt]]`(v0.10.0) | 無 | 06:51-58(ジョブのキーは 4 個)、06:60-67、06:69-160、06:167-181 |
| 書き込みの時機(試行ごとに `Fast`)(v0.10.0) | 古 | 04:158、05:275-307、05:531、06:171 |
| `watch` の所要時間の列(v0.10.0) | 無。3 列のまま | 08:393-437 |
| 時刻(`Timestamp`、`clock::now`)(v0.10.0) | 無 | 02:50-72、02:290-341 |
| `run_job` の形 | 古: `(line, env, signal) -> JobExecution` | 02:121、02:266、05:37・521・697、07:181-188、12:49・106 |
| 定義の誤り 34 種 | 誤: 見出しは 34 種、表は 33 行(`unsetEnvConflict` が無い) | 03:523-561、10:128 |
| `.env` の誤り 14 種 | 誤: 13 種 | 02:106・298、03:9・391・563、08:314-358、10:12・121、11:213・291、12:107・161・321 |
| ステートの誤りの理由(試行の 3 つ) | 無 | 10:142-170 |
| ログの表示(3 つの文) | 無 | 08 の 4 章 |
| v0.10.1 の内部の入口 | 無 | 05:869-900 |
| デモの場面 15 と場面 4 の待ち | 古: 場面 0〜14 | 00:31、11:441-516 |
| 再構築の段 | 古: 段 0〜13(v0.10.0 の段が無い) | 00:32、12:5-142 |

### 章ごとの要約

| 章 | 要約 |
|----|------|
| 00 | 版は 0.10.1 だが「v0.9.0 の機能を含む」とだけ書く。用語「環境変数の層」が「取り除かない」。目次の 11 章・12 章の要約が古い |
| 01 | ユースケースと非目標が v0.9.x のまま(ログの保存と `unset` を非目標に挙げる) |
| 02 | 公開の関数・型の表、`Runtime` の trait(`now` が無い)、モジュールの表(`time.rs`、`flow/attempt.rs`、`flow/activity.rs`、`log.rs`、`clock.rs`、`graph_dot.rs` が無い)、シーケンスが v0.9.x のまま。34 種だけは正しい |
| 03 | `[pipeline]` 6 キー・`[[job]]` 8 キーのまま(実装は 9・9)。v0.10.0 のキーの節、理由の文、検査の順序、データモデル、例が無い。9.3 の表が 33 行 |
| 04 | 構文・オプションの表・`watch`・`graph` が v0.9.x のまま。書き込みの時機が古い |
| 05 | 試行、既定のリトライ、`unset_env`、`required = false`、DOT、`watch --follow` の判定、内部の入口が無い。531 行が v0.10.0 と逆 |
| 06 | 最上位のキーは正しい。`[[job.attempt]]`、試行の矛盾の条件、v0.9.9 のステートが 65 になること、試行の `Fast` の書き込みが無い |
| 07 | `run_job` の形、環境の引き継ぎ(`env_remove`)、標準出力のパイプと読み取りのスレッド、`watch --follow` の待ちが無い |
| 08 | `watch` の表が 3 列、`--help` の全文が古い、DOT の例・ログの文・`.env` の 14 種目が無い |
| 09 | 差なし(変換は v0.10.0 で変わっていない) |
| 10 | 0・64・65・127 の意味に v0.10.0 の場合が無い。誤りの数が 33・13。ステートの理由の表が古い |
| 11 | テストの一覧に v0.10.0 の 7 ファイルが無い。デモが場面 0〜14 |
| 12 | v0.10.0 の段が無い。落とし穴(182)が「足すだけ」 |

### `docs/spec/language-agnostic/`(軽く見た範囲)

- 「v0.10.1 までの挙動」を名乗る(00:3・10、01:3、02:3)が、v0.10.0 の機能の語の一致は 13 文書で 0 件。
- 誤りの数は 33 種と 13 種(00:24、03:499、03:688、10:147、12:38、12:248)。rust 版の「34 種」の直し(`134a9cf`)はこの版に及んでいない。
- 受け入れ基準(AC 番号。例 11:924 の `AC-465`)の体系を持つため、v0.10.0 を書き足すと AC の追加も要る(推論)。
- ほかの 5 版(csharp、go、powershell、python、typescript)は冒頭が v0.8.0・v0.9.0 で、v0.10.0 の機能を持たない。

## 簡易評価(良い / 普通 / 弱い)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | 良い | Rust 標準の命名。型・関数の名前が責務を表し、文書が規則の番号と結ぶ |
| 関数の大きさ | 普通 | 多くは小さい。`walk`、`args` の解釈、`follow_watch`、`run_job_with_retry`、文言の表は長いが、単一の責務 |
| エラーの扱い | 良い | 失敗はすべて戻り値(パニックしない)。固定の文。ログの失敗も値(`LogOutcome`)で返す |
| テストの存在 | 良い | 主要な経路にすべてテストがある |
| 重複 | 普通 | 使い方の表が 3 か所(`args.rs`、`messages.rs`、README)、仕様書の `--help` の全文を含めると 4 か所。`watch --follow` とデモの説明が README・`demo/README.md`・デモのスクリプト(2 本)にある |
| 不要なコード | 良い | `TODO`/`FIXME`/`HACK` は無い |
| 仕様書の追従 | 弱い | v0.10.0 の機能が `docs/spec/rust/`・`language-agnostic/` に無い。検査も無い |

## 守る性質(コードで確認したもの)

- `unsafe` は runtime の `platform/unix.rs` の `libc::kill` 1 か所。core・converter・cli は `#![forbid(unsafe_code)]`。
- 本体の `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ(`clear_screen_supported` と `color_supported` は `cfg!(unix)`)。
- ネットワークのコードとライブラリは無い。
- エラーの表示に入力の値を含めない。ステートに書くのは開始と終了の時刻だけで、command・環境変数・出力は書かない。ログのファイルにはジョブの出力だけを書く。
- 入力の大きさの上限(定義・`.env`・ステート・変換の入力は 1 MiB、入れ子 32 段、`.env` は 10,000 項目、変換の展開は 100,000 ノードと 16 MiB、ログの名前の stem は 100 バイト)。

## 技術的負債と、この intent への影響

| 項目 | 場所 | 影響 |
|------|------|------|
| 仕様書が v0.10.0 の機能を書いていない | `docs/spec/rust/`(上の表)、`docs/spec/language-agnostic/` | この intent の主対象。直す範囲(rust 版だけか、language-agnostic も含めるか)を要件で決める。追記は 12 文書に及ぶため、章ごと・機能ごとの小さなコミットに分ける必要がある(`team.md` の Way of Working)。09 は直さなくてよい |
| 仕様書の数の誤り | 03:523-561(33 行)、`.env` 13 種の 12 か所、10:142-170 | 機能の追記と同じ節で直る。数の一致を見るテストは無いため、人の確かめに頼る |
| 仕様書の「足すだけ」「保存しない」 | 00:75、01:64、05:634、07:205-206、12:182 | 追記に加えて、今の文の書き換え(削除)が要る。書き足すだけでは矛盾が残る |
| `team.md` のテストの数が古い | `aidlc/spaces/default/memory/team.md` の Testing Posture | 上の「テストの数」の値に直す。`src/tests/` の数え方(全部か直下か)を人に確かめる。`team.md` の書き換えは規則の書き込みの道具の作法による |
| runtime の「呼び出しの順序」が古い | `crates/runtime/src/lib.rs:72-82` | 「初期ステートを `Fast` で書く」の形で、試行の `Fast` の書き込みが無い。コードの文書で、仕様書の範囲の外。直すかは要件で決める |
| 使い方の表の重複 | `args.rs`、`report/messages.rs`、`README.md`、`docs/spec/rust/08-output-and-i18n.md:485-563` | 仕様書の `--help` の全文を直すときは、`messages.rs:78-136` を写す |
| 版を 0.10.2 にする | `crates/*/Cargo.toml`、`Cargo.lock`、`crates/cli/tests/binary_basic.rs`(`--version` の期待値)、`docs/spec/rust/` の版の表記(00:3・10、03:1、04:1・250・256、09:1、11:23-28・542、12:13・272)、`docs/spec/language-agnostic/` の版の表記 | 版の更新は「版を X にする」の 1 コミット(`team.md`)。v0.10.1 の `152c4a1` は、4 つの `Cargo.toml`、`Cargo.lock`、`binary_basic.rs`、`rust/` 6 文書と `language-agnostic/` 10 文書の版の表記を 1 コミットで変えた |
| `watch --follow` の「止まった」の判定 | `crates/core/src/flow/activity.rs:38-63` | 判定は変えず、README の注意で補っている。仕様書に書くときも、同じ注意が要る |
