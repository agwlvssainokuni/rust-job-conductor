# コード生成のまとめ: v0.10.1 の整理・修正

出典: 承認済みの計画(`code-generation-plan.md`)、単体テストの手順(`unit-test-instructions.md`)、機能の設計(`functional-spec.md` の W1〜W6、`rules.md` の BR1.1〜BR7.1)、要件(`requirements.md` の FR1.1〜FR7.1、NFR1〜NFR8)。

## 変えたファイル

| 課題 | ファイル | 中身 | 規則 |
|---|---|---|---|
| F2 名前 | `crates/runtime/src/log.rs` | `safe_log_stem` で、最初の `.` より前の部分を `trim_matches(' ')`(ASCII の空白だけ)してから予約名と比べる。名前そのものは変えない。ドキュメントコメントの手順 3 を直した | BR2.1、FR2.1 |
| F2 名前 | `crates/runtime/tests/log_file.rs` | 名前の表に `console`、`con `、` con`、` con `、`con .x`、`nul .tar.gz` の 6 行を足した(`con` は既にあった) | BR2.2、FR2.2 |
| F2 名前 | `README.md` | 「名前の置き換え」に空白の例(`con ` → `_con_`、`nul .tar.gz` → `_nul .tar.gz`)を足した | BR2.2 |
| F2 印 | `crates/runtime/src/runner.rs` | `copy_stream` の繰り返しの初めに「止める」印の確かめを足した。読んだ後の確かめは変えない。`mod tests` に、`read` の回数を数える読み手 `CountingReader` と、2 つのテストを足した | BR2.3、BR2.4、FR2.3 |
| F5 | `demo/run-demo.sh` | 場面 4 の `watch --follow` の説明の 2 行の後、`cmd "$CONDUCTOR" watch --follow` の前に `pause` を足した | BR5.1 |
| F5 | `demo/README.md` | `DEMO_PAUSE` の説明に、場面 4 の `watch --follow` の前でも待つことを書き足した | BR5.4 |
| F5 | `crates/cli/tests/unix_demo.rs` | `DEMO_PAUSE=1` で動かし、場面 4 で待ちの表示が 1 回の `watch` の完了の行の後、`watch --follow` のコマンドの行の前に出ることを確かめるテストを足した | BR5.5 |
| F1 | `README.md` | 「ステートが無いとき」に、注意が当てはまらないことを書き足し、「エラーの後に続くジョブがあるときの注意」を、起きる使い方・起きること・端末 A・B の例・避け方に書き直した | BR1.1、BR1.2 |
| 文書のずれ | `crates/runtime/src/lib.rs` | 契約の表の `run_job` の行に `unset` を、`line`・`env`・`unset`・`signal`・`log` の順に書き足した | BR6.1 |
| 文書のずれ | `docs/spec/rust/` の 6 章、`crates/core/tests/env_types.rs`、`crates/cli/src/tests/report_messages.rs` | 「33 種」を「34 種」に直した(02 の 103 行には v0.10.0 の `UnsetEnvConflict` の追加も書いた。12 の 22 行の注記は「v0.9.0 の 3 種と v0.10.0 の 1 種を含む」にした) | BR6.2 |
| 版 | 4 クレートの `Cargo.toml`、`Cargo.lock`、`crates/cli/tests/binary_basic.rs`、`docs/spec/` の 16 ファイル | 版を 0.10.1 にした(e112426 と同じ 22 ファイル・37 行) | BR7.1 |

変えていないもの(確かめ済み): `crates/core/src/`(F3 を直さなかったため)、`crates/core/src/flow/activity.rs`(BR1.3)、`demo/windows/`(BR5.3)、`crates/core/tests/plan_simulate_perf.rs`(BR3.1)、`crates/cli/src/tests/docs.rs`(必須の語句「止まったと判定して終わることがあります」を書き直した注意に残したため、直す必要が無かった)。

## 主な判断

- **F2 の空白**: 要件の前提どおり、除くのは ASCII の空白(` `)だけ(`trim_matches(' ')`)。タブなどは手順 1 で既に `_` になる。
- **F2 のテスト**: 計画の「`read` が 0 回」のテストに加えて、読んでいる間に印が立ったときに読んだ分を書かずに終わるテスト(`br2_4_…`)を足した。読む前の確かめを足したことで、既存のテスト「止める印が立っていればどこにも書かずに終わる」は読む前の確かめで終わるようになり、読んだ後の確かめ(BR2.4)を単体テストで通る道が無くなったため。本番のコードにテスト専用の引数・`pub` は足していない。
- **F3**: 測定の結果、BR3.3 で原因でないと判定し、直さなかった(下の Step 5)。内部の入口は足していないので、名前は無い。
- **F1 の例**: 判定(`activity.rs`)では、エラーのステートのまま最初に描けば必ず「止まった」になる。例の「終わることがあります」は、端末 B の再開が先に `load` の試行を書いていれば続く、という条件を括弧で書き添えた。

## テストの結果(この段階の最後)

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-runtime --test log_file --test runner_log --locked` | log_file 11 passed、runner_log 8 passed |
| `cargo test -p conductor-runtime --lib --locked -- runner::tests` | 17 passed(2 増) |
| `cargo test -p conductor-cli --test unix_demo --locked` | 4 passed(1 増) |
| `cargo test -p conductor-cli --lib --locked -- tests::docs tests::report_messages` | 67 passed |
| `cargo test -p conductor-core --test env_types --locked` | 6 passed |
| `cargo fmt --all -- --check` | 通った(1 度、足したテストの形の差を Edit で直した。コミット c222b2a) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通った |
| `cargo test --workspace --locked`(Step 4) | 1,431 passed、0 failed |
| `cargo test --workspace --locked`(Step 10・Step 11) | 1,432 passed、0 failed |

F3 を直さなかったため、`flow::` と計画・流れ・性能のテストのコマンドは、Step 1 と全体のテストの中でだけ走らせた(どちらも通った)。

## この段階で走らせなかったテスト

- 行カバレッジ(`cargo llvm-cov --workspace … --fail-under-lines 80`): Build and Test が全体で測る(手順のとおり)。
- Windows の動き(`windows_demo.rs`、Windows のログの名前)と ubuntu、最小の Rust(1.100)の CI: push の後の CI で確かめる。push は人が行う。

## 計画からのずれ

- Step 3 で、計画のテスト 1 つに加えて、読んだ後の確かめのテストを 1 つ足した(上の「主な判断」)。
- Step 6 は、Step 5 の判定が「原因でない」だったため飛ばした(計画の定めどおり)。そのチェックボックスは付けていない。
- Step 10 の `cargo fmt` の差を直すコミット(c222b2a)を 1 つ足した。

## コミット

| コミット | 件名 |
|---|---|
| acf1938 | runtime: ログの名前の予約名を前後の空白を除いて比べる |
| 5bab66d | runtime: ログの名前の表に空白を含む行を足す |
| b4d41d0 | docs: README の名前の置き換えに空白の例を足す |
| 1e1995f | runtime: 出力を読む前にも止める印を見る |
| 37f79e3 | runtime: 読む前と読んだ後の止める印のテストを足す |
| d58f872 | docs: デモの場面 4 で watch --follow の前に待つ |
| 38a1cb0 | cli: デモの場面 4 の待ちの位置のテストを足す |
| 7582709 | docs: README の watch --follow の注意を起きる使い方に限って書き直す |
| a018b42 | runtime: 契約の表の run_job の行に unset の引数を書き足す |
| 134a9cf | docs: DefinitionErrorKind の数を 34 種に直す |
| c222b2a | cli: デモの場面 4 の待ちのテストを rustfmt の形にする |
| 152c4a1 | 版を 0.10.1 にする |

この記録(`code-summary.md`、`traceability.json`)は、最後の `aidlc:` のコミットに入れる。計画のファイル(チェックボックスのほかに、承認の時の変更が作業の木に残っている)、監査の記録、承認の質問のファイルは、呼び出し側に任せ、コミットしていない。

## Step 1: 変更の前のテスト(`unit-test-instructions.md` のコマンド)

変更の前の作業の木(ea3e1d1)で、手順のコマンドをそのまま走らせた。どれも通り、書き方の誤りは無かった。

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-runtime --test log_file --test runner_log --locked` | log_file 11 passed、runner_log 8 passed |
| `cargo test -p conductor-runtime --lib --locked -- runner::tests` | 15 passed(71 filtered out) |
| `cargo test -p conductor-core --lib --locked -- flow::` | 51 passed(64 filtered out) |
| `cargo test -p conductor-core --test plan_pipeline --test plan_run --test plan_assume --test plan_simulate_perf --test flow_sequences --test flow_perf --test retry_flow --test attempt_flow --locked` | attempt_flow 8、flow_perf 1、flow_sequences 9、plan_assume 5、plan_pipeline 10、plan_run 5、plan_simulate_perf 3、retry_flow 17 passed |
| `cargo test -p conductor-cli --test unix_demo --locked` | 3 passed |
| `cargo test -p conductor-cli --lib --locked -- tests::docs tests::report_messages` | 67 passed(138 filtered out) |
| `cargo test -p conductor-core --test env_types --locked` | 6 passed |

## Step 5: F3 の測定と判定(BR3.2、BR3.3、R-05)

### 測り方

- 使い捨てのテスト `crates/core/tests/tmp_f3_measure.rs` を Write で作り、測り終えてから `rm` で消した(コミットしていない。`git status` で残っていないことを確かめた)。
- 計画は `plan_simulate_perf.rs` と同じ形。P1 はステートなしで全ジョブが実行する計画、P2 は `j0` の成功で `@end` に着いた後(`end_reached`)のステートからの計画。各ジョブの数で 5 回作り、中央値を取った。
- debug は `cargo test -p conductor-core --test tmp_f3_measure --locked -- --nocapture`、カバレッジの計測の下は `cargo llvm-cov -p conductor-core --test tmp_f3_measure --locked --summary-only -- --nocapture`。手元の macOS(Apple Silicon)。
- 走査ごとの効き目(R-05)は、本番の走査と同じ比較を測定のテストの中に写して、別に測った(本番のコードは変えていない)。

### 計画を作る時間の中央値

| ジョブの数 | debug P1 | debug P2 | llvm-cov P1(1 回目 / 2 回目 / 3 回目) | llvm-cov P2(1 回目 / 2 回目 / 3 回目) |
|---|---|---|---|---|
| 250 | 3.34ms | 2.73ms | 4.52 / 4.27 / 3.72ms | 2.95 / 2.86 / 2.86ms |
| 500 | 6.46ms | 6.18ms | 6.89 / 6.90 / 6.72ms | 6.54 / 6.56 / 6.47ms |
| 1,000 | 21.97ms | 21.26ms | 25.56 / 23.84 / 23.72ms | 23.38 / 24.00 / 23.31ms |
| 2,000 | 77.75ms | 75.38ms | 84.99 / 85.86 / 85.05ms | 84.66 / 87.92 / 88.60ms |

### 2 倍ごとの伸び(llvm-cov の下、1 回目)

| 段 | P1 | P2 |
|---|---|---|
| 250 → 500 | 1.52 倍 | 2.22 倍 |
| 500 → 1,000 | 3.71 倍 | 3.57 倍 |
| 1,000 → 2,000 | 3.33 倍 | 3.62 倍 |

2 回目・3 回目も、250 → 500 は P1 1.62・1.81 倍、P2 2.29・2.26 倍で、3 倍に届かなかった。

### 走査ごとの時間(llvm-cov の下、1 回目、全ジョブ分の合計)

| 走査 | 場所 | 250 | 500 | 1,000 | 2,000 | 直すときに使う値 | 判断 |
|---|---|---|---|---|---|---|---|
| `job_index`(1 回分) | `decide.rs`、`next_step.rs`(P1 では 2 回、P2 では 1 回) | 0.25ms | 0.75ms | 3.12ms | 12.58ms | ジョブの位置(定義の順に進む添え字) | 直さない |
| `record_of` | `decide.rs` | 0.24ms | 0.72ms | 2.83ms | 11.15ms | ジョブの位置(`status_at` で引く) | 直さない |
| 規則 6 の `job[..index]` | `decide.rs` | 0.84ms | 2.48ms | 9.90ms | 39.14ms | 最初の済んでいないジョブの位置 | 直さない |
| `apply_skip_in` の `any(Error)` | `record.rs` | 0.28ms | 0.80ms | 3.29ms | 13.07ms | エラーがあるか | 直さない |
| `apply_skip_in` の `first_pending` | `record.rs` | 0.99ms | 3.32ms | 12.91ms | 49.37ms | 最初の pending の位置 | 直さない |
| `next_step.rs` の `pending_ids` | `next_step.rs` | ― | ― | ― | ― | (P1 は既定の遷移で呼ばれず、P2 も `j0` の 1 回だけで O(n)) | 直さない |

- 1,000 ジョブでは、P1 の写した走査の合計(`job_index` 2 回、`record_of`、規則 6)が約 19ms で計画の 25.6ms の大半、P2 の合計(`job_index`、`record_of`、`any(Error)`、`first_pending`)が約 22ms で計画の 23.4ms の大半を占める。500 ジョブ以上では、これらの走査が二乗で伸びている。
- 250 → 500 の段では、走査より、ジョブの数に比例する分(項目を作る `entry` の割り当てなど)が勝ち、伸びが 2 倍前後にとどまる。

### 判定(BR3.3)

- **原因でない**と判定した。理由: llvm-cov の下で、250 → 500 の段の伸びが P1 1.52〜1.81 倍、P2 2.22〜2.29 倍で、BR3.3 の「どの段でも 3 倍以上」を満たさない(500 → 1,000 と 1,000 → 2,000 は 3 倍を超えた)。
- そのため Step 6(直し)は飛ばした。コードもテストも変えていない(BR3.6)。どの走査も直していない。
- 参考: 手元の llvm-cov の下で、1,000 ジョブの計画は約 24ms で、`plan_simulate_perf.rs` の上限 150ms の 6 分の 1 ほど。CI で上限を超えた原因は、この測定からは分からない(F4 の記録の手順に回る)。
- この表と理由は、Build and Test で `build-and-test-summary.md` にも書く(BR3.6)。
