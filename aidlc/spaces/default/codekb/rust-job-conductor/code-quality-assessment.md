# コードの品質の評価

## テスト

| 項目 | 状態 |
|------|------|
| 枠組み | Rust 標準の `#[test]` だけ。テスト名は日本語 |
| 数 | `#[test]` の数: core 467、runtime 257、cli 612、converter 116(合計 1,452) |
| 統合テスト | core 36、runtime 16、cli 26、converter 2 のファイル(`tests/*.rs`)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`、`crates/converter/src/tests/` |
| 流れのテスト | `cli/tests/flow_*.rs`(10 ファイル)が、偽の `Runtime`/`Sink`/`Env`/`Signals` を注入して、終了コード・表示・ステートの書き込みの順序を確かめる。`flow_aux.rs` に `watch --follow` の U7 のテスト(完了・止まった・描き直し・追記・画面の消去・読めない・合図・1,000 回の繰り返し) |
| 実バイナリのテスト | `cli/tests/binary_*.rs`(8 ファイル。`binary_log.rs` を含む)が、`conductor` とテスト専用の補助バイナリを動かす。シグナルは `binary_signal.rs`、`signals_real.rs`、`signals_order.rs` |
| 非機能のテスト | 性能(`*_perf.rs`、`nfr_performance.rs`、`nfr_perf.rs`)、安全性(`nfr_security.rs`)、頑健さ(`nfr_robust.rs`)、ソースの検査(`nfr_source.rs`)、内容(`nfr_content.rs`: ステートに許すのは開始と終了の時刻だけ) |
| 性能のテストの上限 | 「debug の見積もりの約 5 倍」で決めた固定の時間(`plan_simulate_perf.rs` の 150ms、`definition_perf.rs` の 350ms・1,600ms・700ms、`graph_lookup_perf.rs` の 350ms・600ms など)。5 回の中央値で比べる。カバレッジの計測の下の遅さは見込んでいない |
| 網羅のテスト | `CommandKind::ALL` などの配列で、種類の追加漏れを捕まえる |
| 参照出力との一致 | `core/tests/graph_layout_oracle.rs` が、`fixtures/graph_layout_oracle*.md` と配置を比べる |
| 文書との一致 | `cli/src/tests/docs.rs`(README と使い方の表)、`cli/tests/msrv_consistency.rs`、デモのテスト(`unix_demo.rs`、`windows_demo.rs`) |
| デモのテスト | `unix_demo.rs` は `DEMO_PAUSE` を消し(`unix_demo.rs:44`)、標準出力をパイプで受けるため、端末での画面の消去と一時停止は検査しない |
| カバレッジ | CI の `coverage` で行 80% 以上を強制(テスト専用の補助バイナリは除外)。チームの方針(test-after、行カバレッジ 80%)と一致する |
| OS | CI の `test` で ubuntu・macos・windows。`msrv-test` で 1.100.0 |

評価: 良い。純粋な中核と差し替え口(時計を含む)の分離で、流れの失敗の経路(書き込みの失敗、シグナル、出力の失敗、時刻)まで偽の実装で試せる。揺れの源は、実時計と固定の時間の上限に頼るテスト。

## 静的検査と書式

- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が CI の `lint` で関門になる。設定ファイルは無く既定。
- 本体の `#[allow]` は `runtime/src/platform/unix.rs` の `#![allow(unsafe_code)]` 1 つだけ。テストの共通の補助(`tests/common/mod.rs` など 5 つ)に `#![allow(dead_code)]` がある。

## CI/CD

- マージの関門(`ci.yml`): `lint`、`build-check`(MSRV、C のビルドの不在)、`test`(3 OS)、`msrv-test`、`coverage`、`docs-check`。`startup-bench` は警告のみ。
- リリース(`release.yml`): タグが `main` の上にあり `crates/cli/Cargo.toml` の版と一致すること → `ci.yml` の再利用 → 4 対象のビルドと smoke test → 下書きのリリース。
- `audit.yml`: `cargo audit`(PR と毎週。警告のみ)。
- 変更は PR を使わず `main` に直接コミットする運用(`team.md`)のため、関門は `push` の CI で事後に効く。`coverage` のジョブの中でテストが 1 つでも落ちれば、カバレッジの値によらずジョブが落ちる。

## 文書

- `README.md`(953 行)は、インストール、クイックスタート、使い方(`watch --follow` の節を含む)、定義の書き方(retry、env、`unset_env`、`log_dir`)、ステート(試行の時刻)、終了コード、保証、シグナル、graph、dry-run、init、convert、ロケールを書く。必須の語句は `docs-check` が検査。
- 各モジュールの冒頭の `//!` が、責務、契約、規則の番号(BR/NFR/FR、v0.10.0 の Unit)、守る性質を日本語で書く。
- `docs/spec/` に 7 言語版の再構築用の仕様(各 13 文書)。

## 簡易評価(良い / 普通 / 弱い)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明瞭さ | 良い | Rust 標準の命名。型・関数の名前が責務を表し、文書が規則の番号と結ぶ |
| 関数の大きさ | 普通 | 多くは小さい。`walk`、`args` の解釈、`follow_watch`、`run_job_with_retry`、文言の表は長いが、単一の責務 |
| エラーの扱い | 良い | 失敗はすべて戻り値(パニックしない)。固定の文。ログの失敗も値(`LogOutcome`)で返す |
| テストの存在 | 良い | 主要な経路にすべてテストがある |
| 重複 | 普通 | 使い方の表が 3 か所(`args.rs`、`messages.rs`、README)。`watch --follow` とデモの説明が README・`demo/README.md`・デモのスクリプト(2 本)にある |
| 不要なコード | 良い | `TODO`/`FIXME`/`HACK` は無い |

## 守る性質(コードで確認したもの)

- `unsafe` は runtime の `platform/unix.rs` の `libc::kill` 1 か所。core・converter・cli は `#![forbid(unsafe_code)]`。
- 本体の `cfg(unix)`/`cfg(windows)` は `runtime/src/platform/` と `cli/src/platform/` の中だけ(`clear_screen_supported` と `color_supported` は `cfg!(unix)`)。
- ネットワークのコードとライブラリは無い。
- エラーの表示に入力の値を含めない。ステートに書くのは開始と終了の時刻だけで、command・環境変数・出力は書かない。ログのファイルにはジョブの出力だけを書く。
- 入力の大きさの上限(定義・`.env`・ステート・変換の入力は 1 MiB、入れ子 32 段、`.env` は 10,000 項目、変換の展開は 100,000 ノードと 16 MiB、ログの名前の stem は 100 バイト)。

## 技術的負債と、この intent への影響

| 項目 | 場所 | 影響 |
|------|------|------|
| F1: 「止まった」の判定が、エラーで止まったステートと再開の直前を見分けない | `crates/core/src/flow/activity.rs:49-63`、`crates/cli/src/aux_commands.rs:161-165`、`README.md:295`・`README.md:299` | `error` のジョブがあるステートで `watch --follow` を始めると、再開のジョブが試行を書く(`step.rs:231-238`)前の最初の 1 回で 0 で終わる。今の README は「次のジョブの試行が始まるまでの短い間」とだけ書き、パイプラインを始める前から見ておける(`README.md:292`)という説明と並ぶため、どの使い方で起きるかが分かりにくい。言い方を直すのが F1。判定(`pipeline_activity`)の変更は F1 の範囲の外(README の書き直し) |
| F2: Windows の予約名の比較が前後の空白を除かない | `crates/runtime/src/log.rs:88-94`、`crates/runtime/tests/log_file.rs:49-80`、`README.md:625` | `"CON .x"` などの ID は `_` が付かない。直すと、空白を含む ID のログの名前が v0.10.0 と変わる。名前の表のテストと README の「名前の置き換え」を揃える必要がある。OS の分岐は足さない(どの OS でも同じ名前の規則) |
| F2: 「止める」印を読み取りの後にだけ見る | `crates/runtime/src/runner.rs:313-322`、`runner.rs:228-231` | 待ち切れて印を立てた後も、スレッドは次の `read` が返るまで残り、返った分を捨てて終わる。印を読み取りの前にも見ると、立った後に新しく読みに行かない。単体テスト `止める印が立っていればどこにも書かずに終わる`(`runner.rs:609-616`)は、印を先に立ててから呼ぶ形 |
| F3: 性能のテストの上限がカバレッジの計測を見込まない | `crates/core/tests/plan_simulate_perf.rs:39`、`ci.yml` の `coverage` | 150ms は「debug の見積もり約 31ms の約 5 倍」。計測つきの build では遅くなり、上限に近づく(推論)。計画の模擬は、ジョブごとに `decide_from_state` の `job_index`・`record_of`、`apply_skip_in` のエラーの確認と最初の `pending` の探索(`crates/core/src/flow/record.rs:197-212`)で前から探すため、1,000 ジョブで二乗に近い比較になる(推論)。上限は下げない方針 |
| F4: u6 で 1 度だけ出た原因不明の失敗 | コードからは特定できない | 実時計を挟んで `before <= 値 <= after` を見るテスト(`crates/cli/tests/binary_basic.rs:414-417` など)や、時間の上限を持つテストが候補になりうる(推論)。再び起きたら、ジョブの名前・OS・ログを残す |
| F5: デモの場面 4 で画面が消える | `demo/run-demo.sh:94-96`、`crates/cli/src/report/mod.rs:262-268`、`crates/cli/src/platform/mod.rs` の `clear_screen_supported` | UNIX 系の端末では `watch --follow` が 1 回目から `ESC[H ESC[2J` を出し、場面 4 の前半(再開、`watch` の表)を消す。`DEMO_PAUSE=1` で見せているときに目立つ。Windows 版(`demo/windows/run-demo.cmd:98-100`)は画面を消さない。デモのテストは端末を使わないため検査しない |
| 仕様書の古さ | `docs/spec/rust/00-README.md` ほか | v0.10.0 を名乗るが、`follow`・`log_dir`・`unset_env` の記述は数か所だけ。誤りの種類を 33 種と書く(実装は 34 種) |
| runtime の契約の表の古さ | `crates/runtime/src/lib.rs` | `run_job` の説明に消す名前(`unset`)が無い。呼び出しの順序の節は「初期ステートを `Fast` で書く」で、v0.10.0 の「試行を始めて `Fast` で書く」とずれる |
| 使い方の表の重複 | `args.rs`、`report/messages.rs`、`README.md` | オプションを変えるたびに 3 か所の更新が要る |
| `team.md` のテストの数が古い | `aidlc/spaces/default/memory/team.md` の Testing Posture | 書かれた数(33・14・25・2 のファイル、1,170)は今の数(36・16・26・2、1,452)と合わない |
