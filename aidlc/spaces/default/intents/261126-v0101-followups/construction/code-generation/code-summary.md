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

## やり直し 1(Build and Test の Loop-back 1。Step 13)

CI 45908344996 の `coverage`(ubuntu、`cargo llvm-cov` の下)で `plan_simulate_perf.rs` の P1・P2 が上限 150ms を超えたため、人が「Retry with fix」を選び、Step 5 の「直さない」の判定を覆して、Step 6 の直しを行った(`build-and-test/test-results.md` の Loop-Back Log)。上の Step 1〜12 の記録は、そのまま残す。

### 変えたファイル

| ファイル | 中身 | 規則 |
|---|---|---|
| `crates/core/src/flow/mod.rs` | 目印の型 `SimulationMarks`(`pub(crate)`。フィールドは `flow` の外から見えない)と、目印を進める関数(`predecessors_settled`、`first_pending`、`has_error`、`note_status_change`)を足した。位置で引く `record_at`(`status_at` はこれを呼ぶ形にした)と、手がかりの位置を確かめて違えば `job_index` で探す `position_of` を足した。`mod tests` に R-06 のテストを 5 つ足した | BR3.4 |
| `crates/core/src/flow/decide.rs` | 判断の本体を `decide_with`(規則 1〜7)に分け、公開の側(`decide_before_run` → `decide_from_state`)は位置を `job_index` で、目印を `SimulationMarks::new()` で求めて本体を呼ぶ形にした。模擬の内部の入口 `decide_at` を足した | BR3.4 |
| `crates/core/src/flow/next_step.rs` | 本体の `next_step` は位置を引数で受け取る形にした。公開の `classify_and_decide` は `job_index` で求めて渡し、模擬の内部の入口 `classify_and_decide_at` は `position_of` で求めて渡す | BR3.4 |
| `crates/core/src/flow/record.rs` | `apply_outcome_in`・`apply_skip_in` に目印を足した。状態を変えるたびに `error` の数を直し、`apply_skip_in` の「エラーがあるか」と「最初の pending」は目印から決める。公開の `apply_outcome`・`apply_skip` は新しい目印を渡す | BR3.4 |
| `crates/core/src/plan/simulate.rs` | `plan_pipeline` が位置と目印を持ち回り、`decide_at`・`classify_and_decide_at`・目印つきの記録の関数を呼ぶ。判断と記録の規則は書き写していない。`plan_run` は今までどおり公開の入口を呼ぶ | BR3.4 |

変えていないもの(確かめ済み): `job_index`(`types.rs`)・`record_of`(`flow/mod.rs`)の中身、公開の API(`lib.rs` の再公開)、`plan_simulate_perf.rs`(BR3.1)、`next_step.rs` の `pending_ids`(R-05 の表で効かないと分かったため)、`resolve_transition` の遷移先の `job_index`(遷移のあるジョブでだけ呼ばれ、R-05 の走査の対象でない)。

### 内部の入口と目印の名前

- 内部の入口: `flow::decide_at(job_id, index, definition, state, marks)`、`flow::classify_and_decide_at(job, index, execution, job_states, definition, run_mode)`(どちらも `pub(crate)`)。
- 目印: `flow::SimulationMarks`(設計の `PlanSimulationLookup` の目印にあたる。位置は別の引数 `index` で渡す)。作るのは `SimulationMarks::new()`。

### 走査ごとの直し方(R-05 の表の「直すときに使う値」)

| 走査 | 直し方 | 目印が正しくない入力のとき |
|---|---|---|
| `job_index`(`decide.rs`、`next_step.rs`) | 模擬が渡す位置 `index` を `position_of` で確かめて使う(O(1)) | 位置のジョブが違えば `job_index` で探す(定義になければ、今までどおり順序外・停止) |
| `record_of`(`decide.rs`) | 位置で引く `record_at`(O(1)) | 記録の位置が違う・記録が欠けるときは `record_of` で探す(`status_at` と同じ形) |
| 規則 6 の `job[..index]` | 目印の「済んだ前置き」(`0..settled_prefix` はすべて済んでいる)から続けて走査する(計画の全体で O(n)) | 状態は `status_at` で引く。前置きは、`flow` の記録の関数が済んだジョブを済んでいない状態に戻さない(実行するのは pending か error のジョブだけ)ので、記録を変えても正しいまま |
| `apply_skip_in` の `any(Error)` | 目印の `error` の数(要るときに 1 回だけ数え、記録の関数が状態を変えるたびに直す) | 数は定義に無い記録も含めて数える(`any` と同じ範囲) |
| `apply_skip_in` の `first_pending` | 目印の「pending の無い前置き」から続けて走査する(計画の全体で O(n)) | 状態は `status_at` で引く(記録なしは pending)。記録の関数はどのジョブも pending に戻さないので、前置きは正しいまま |

公開の入口は、呼ぶたびに新しい目印を作る(前置きは 0、数は未計数)ので、走査は今までと同じになる。

### 直す前と後の測定(使い捨てのテスト)

- 測り方は Step 5 と同じ(`crates/core/tests/tmp_f3_measure.rs` を Write で作り、P1・P2 を各 5 回作って中央値を取る)。測り終えて `rm` で消し、`git status` で残っていないことを確かめた(コミットしていない)。手元の macOS(Apple Silicon)。
- 直す前は、この作業の初め(5405854 の前、152c4a1 の後の木)に測り直した値。Step 5 の表とほぼ同じ。

計画を作る時間の中央値:

| ジョブの数 | 直す前 debug P1 / P2 | 直す前 llvm-cov P1 / P2 | 直した後 debug P1 / P2 | 直した後 llvm-cov P1 / P2(1 回目) | 直した後 llvm-cov P1(2・3・4 回目) | 直した後 llvm-cov P2(2・3・4 回目) |
|---|---|---|---|---|---|---|
| 250 | 4.24 / 2.85ms | 4.39 / 2.81ms | 0.91 / 0.74ms | 0.92 / 0.75ms | 2.42 / 1.77 / 2.09ms | 1.31 / 1.25 / 1.26ms |
| 500 | 6.87 / 5.95ms | 7.22 / 6.42ms | 1.79 / 1.52ms | 1.80 / 1.48ms | 2.40 / 2.31 / 2.40ms | 1.74 / 1.71 / 1.73ms |
| 1,000 | 21.56 / 21.11ms | 23.65 / 23.06ms | 3.60 / 2.89ms | 3.56 / 2.98ms | 3.53 / 3.54 / 3.54ms | 2.75 / 2.77 / 2.77ms |
| 2,000 | 74.61 / 75.09ms | 84.20 / 83.86ms | 7.02 / 5.73ms | 7.17 / 5.85ms | 6.72 / 6.67 / 6.69ms | 5.47 / 5.51 / 5.42ms |

2 倍ごとの伸び(llvm-cov の下):

| 段 | 直す前 P1 / P2 | 直した後 P1(1〜4 回目) | 直した後 P2(1〜4 回目) |
|---|---|---|---|
| 250 → 500 | 1.64 / 2.29 倍 | 1.97 / 0.99 / 1.31 / 1.15 倍 | 1.97 / 1.33 / 1.37 / 1.38 倍 |
| 500 → 1,000 | 3.27 / 3.59 倍 | 1.98 / 1.47 / 1.53 / 1.47 倍 | 2.01 / 1.58 / 1.62 / 1.60 倍 |
| 1,000 → 2,000 | 3.56 / 3.64 倍 | 2.01 / 1.90 / 1.88 / 1.89 倍 | 1.96 / 1.99 / 1.99 / 1.96 倍 |

- 2〜4 回目の 250・500 は、1 回目より遅く、伸びが 2 倍を下回った。小さい数では、1 回の時間がジョブの数によらない分(初めの割り当てなど)に左右されるためと見る。どの回も、伸びは 2.5 倍以下。
- 1,000 ジョブは、llvm-cov の下で 23.65ms → 3.53〜3.56ms(約 6.6 分の 1)、P2 は 23.06ms → 2.75〜2.98ms(約 8 分の 1)。

### 判定(BR3.5)

- **合格**。llvm-cov の下で 1,000 ジョブが P1 3.56ms・P2 2.98ms(1 回目。上限 75ms)、2 倍ごとの伸びはどの回・どの段でも 2.5 倍以下(最大 2.01 倍)。直しは戻していない(BR3.6 の戻しの枝には進まない)。
- 参考: CI の ubuntu の llvm-cov の下は手元より約 6〜9 倍遅かった(`test-results.md` の診断)。同じ比で見積もると、1,000 ジョブは約 20〜35ms で、`plan_simulate_perf.rs` の上限 150ms に収まる見込み。CI の `coverage` が通ることは、Build and Test で、人の push の後に確かめる。

### テストの結果

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-core --lib --locked -- flow::` | 56 passed(5 増。R-06 のテスト) |
| `cargo test -p conductor-core --test plan_pipeline --test plan_run --test plan_assume --test plan_simulate_perf --test flow_sequences --test flow_perf --test retry_flow --test attempt_flow --locked` | attempt_flow 8、flow_perf 1、flow_sequences 9、plan_assume 5、plan_pipeline 10、plan_run 5、plan_simulate_perf 3、retry_flow 17 passed(どのテストも変えていない) |
| `cargo fmt --all -- --check` | 通った |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通った |
| `cargo test --workspace --locked` | 1,437 passed、0 failed、1 ignored(前からのもの) |

- R-06 のテスト(`flow::tests`)は、定義の順に全ジョブを、公開の入口(`decide_from_state`・`classify_and_decide`・`apply_skip`・`apply_outcome`)と内部の入口(`decide_at`・`classify_and_decide_at`・目印つきの `apply_skip_in`・`apply_outcome_in`)の 2 通りで進め、各ジョブで判断・分類・次の動き・記録が一致することを確かめる。入力は、記録が逆順のステート、記録の欠け(一部、すべて)、再開位置と error の記録(遷移で error のジョブをスキップに変える)、途中のエラーの後の順序外、`end_reached`、位置の手がかりの違い(別のジョブ、範囲の外、定義にないジョブ)。拒否やエラーの後も止めずに進める。
- テストが効くことを確かめるため、一時的に `note_status_change` の減らす側を外して走らせ、「再開位置とエラーの記録があるステート」のテストが落ちることを見てから、元に戻した(コミットには入っていない)。

### コミット

| コミット | 件名 |
|---|---|
| 5405854 | core: 計画の模擬に位置と目印を渡す内部の入口を足す |
| 6104853 | core: 内部の入口と公開の入口の判断と記録が一致するテストを足す |

この記録(`code-summary.md`、`traceability.json`)は、最後の `aidlc:` のコミットに入れる。計画のファイル、監査の記録、承認の質問のファイルは、呼び出し側に任せ、コミットしていない。

### 計画からのずれ

- この直しのコミットは、版の更新(152c4a1、0.10.1)の後に入った。タグはまだ付けておらず、版は変えていない(BR7.1 の「すべてを片づけてから版を上げる」からのずれ。計画のやり直し 1 に書かれたとおり)。
- `simulate_job` の引数が 7 つになるのを避けるため、仮定した終了コードと「指定したか」を 1 つの組 `(i32, bool)` にまとめた(動きは同じ)。
- `traceability.json` の BR3.4 を `crates/core/src/flow/mod.rs`、BR3.5 を `crates/core/src/plan/simulate.rs` を指す OK にした。BR3.6 は Deferred のまま(戻しはしていない。Build and Test で扱う)。
