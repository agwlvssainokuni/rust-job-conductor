# テストの結果: v0.10.1

出典: この段階で走らせたコマンドの出力、CI の実行(45906490030、45908344996)、`unit-test-instructions.md`、`build-instructions.md`。

## ビルド

| コマンド | 結果 |
|---|---|
| `cargo build --workspace --locked` | 成功(exit 0) |
| `cargo fmt --all -- --check` | 成功(exit 0) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 成功(exit 0) |

## 単体のコマンド(`unit-test-instructions.md`。各 1 回)

| コマンド | 結果 |
|---|---|
| `cargo test -p conductor-runtime --test log_file --test runner_log --locked` | exit 0。log_file 11 passed、runner_log 8 passed |
| `cargo test -p conductor-runtime --lib --locked -- runner::tests` | exit 0。17 passed |
| `cargo test -p conductor-core --lib --locked -- flow::` | exit 0。51 passed |
| `cargo test -p conductor-core --test plan_pipeline --test plan_run --test plan_assume --test plan_simulate_perf --test flow_sequences --test flow_perf --test retry_flow --test attempt_flow --locked` | exit 0。8 + 1 + 9 + 5 + 10 + 5 + 3 + 17 passed |
| `cargo test -p conductor-cli --test unix_demo --locked` | exit 0。4 passed |
| `cargo test -p conductor-cli --lib --locked -- tests::docs tests::report_messages` | exit 0。67 passed |
| `cargo test -p conductor-core --test env_types --locked` | exit 0。6 passed |

## 全体のテスト(手元の macOS)

- `cargo test --workspace --locked`: exit 0。1,432 passed、0 failed、1 ignored(前からのもの)。
- 依存: `Cargo.lock` の package 46(v0.10.0 と同じ)。

## カバレッジ(T-COV)

- コマンド: `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only`(手元の macOS)
- exit 0。TOTAL の行カバレッジ 98.17%(10,522 行のうち 193 行が未通過)。領域 97.85%、関数 98.15%。

## CI(T-CI)

| 実行 | コミット | 結果 |
|---|---|---|
| 45906490030 | 7582709(作業の途中) | `lint` が失敗(rustfmt の形の差。後の c222b2a で直した)。`coverage` を含む他のジョブは成功 |
| 45908344996 | 0567f45(この段階の対象) | `coverage` が失敗。他の 11 のジョブ(`lint`、`build-check` ×2、`test` ×3、`msrv-test`、`docs-check`、`startup-bench` ×3)は成功 |

### 45908344996 の `coverage` の失敗

- 落ちたテスト: `conductor-core` の `crates/core/tests/plan_simulate_perf.rs` の 2 つ。
  - `p1_千個のジョブがすべて実行する計画は_150ms_以内`: 中央値 161.33ms(各回 207.93、216.74、159.58、161.33、137.06ms)。`plan_simulate_perf.rs:105` の assert。
  - `p2_終点のあとの_999_個のスキップの計画は_150ms_以内`: 中央値 154.96ms(各回 206.68、194.48、154.35、154.30、154.96ms)。`plan_simulate_perf.rs:136` の assert。
- 同じファイルの `千個のジョブがすべて実行する計画は_1秒以内に作れる` は通った。他のテストはすべて通った。
- OS: ubuntu-24.04(GitHub Actions の `coverage` のジョブ。`cargo llvm-cov` の下)。

## 原因不明の失敗(F4 の手順による記録)

`build-and-test-summary.md` の手順に従って記録する。

| 項目 | 値 |
|---|---|
| 実行の番号 | GitHub Actions 45908344996、job `coverage` |
| 落ちたテスト | `conductor-core` `tests/plan_simulate_perf.rs` の `p1_千個のジョブがすべて実行する計画は_150ms_以内`、`p2_終点のあとの_999_個のスキップの計画は_150ms_以内` |
| 失敗の出力 | 「中央値 161.327178ms が上限 150ms を超えた」(105 行)、「中央値 154.961394ms が上限 150ms を超えた」(136 行) |
| OS | ubuntu-24.04(GitHub Actions)、`cargo llvm-cov` の下 |
| コミット | 0567f459cc1568493dbdda41f6655b5001e52052 |
| 同じコミットでの再実行 | していない(結果を待たずに判断を仰ぐ。下の「診断」) |

- v0.10.0 の CI(45660860712、45713211039)と同じテストで、これで 3 回目の記録になる。「同じテストが 2 回以上記録されたら、次の intent の課題にする」の条件を満たしており、原因不明ではなくなった(下の診断)。

## 診断

- 手元の macOS では、llvm-cov の下で 1,000 ジョブの計画が約 24ms。CI の ubuntu の llvm-cov の下では、同じテストが 137〜217ms で、**約 6〜9 倍遅い**。
- コード生成の Step 5 の測定では、500 → 1,000 と 1,000 → 2,000 の段で、二乗に近い伸び(3.3〜3.7 倍)が出ていた。250 → 500 の段は、ジョブの数に比例する分が勝って 3 倍に届かず、BR3.3 の「どの段でも 3 倍以上」を満たさなかったため、直さないと判定した。
- 遅い機械(CI の llvm-cov の下)では、二乗で伸びる走査(`job_index`、`record_of`、規則 6 の前のジョブの走査、`apply_skip_in` の `any(Error)` と `first_pending`。1,000 ジョブで計画の時間の大半)が、上限 150ms を超えさせている。**根は、コード生成で選んだ「直さない」という判断(BR3.3 の基準を速い手元の機械で当てたこと)にある**。この段階の設定(テストの設定、ビルドの手順)の問題ではない。
- 上限を下げる・テストを外す・`coverage` のジョブから外すことは、要件(NFR8、FR3.1)と規則(BR3.1)で禁じられており、直し方にならない。

## Loop-Back Log

### Loop-back 1 — 2026-11-26T23:10:00Z

- **Diagnosis**: CI 45908344996 の `coverage`(ubuntu-24.04、`cargo llvm-cov` の下)で、`plan_simulate_perf.rs` の P1(中央値 161.33ms)と P2(154.96ms)が上限 150ms を超えた。手元の macOS より約 6〜9 倍遅い機械で、計画の模擬の二乗で伸びる走査(`job_index`、`record_of`、規則 6 の前のジョブの走査、`apply_skip_in` の `any(Error)` と `first_pending`)が上限を超えさせている。
- **Root-cause stage**: code-generation(Step 5 の BR3.3 の判定で「直さない」とし、Step 6 を飛ばした判断)。人が「Retry with fix」を選び、この判定を覆して直すことを決めた。
- **Planned fix**: 承認済みの設計(functional-spec.md の W3 の手順 2〜7、rules.md の BR3.4〜BR3.6、Q2 A)と計画の Step 6 のとおり、`crates/core/src/flow/` に、位置と目印(R-05 で走査ごとに決める値)を受け取る `pub(crate)` の内部の入口と目印を進める関数を足し、`crates/core/src/plan/simulate.rs` から呼ぶ。判断と記録の規則は flow の 1 か所のまま。R-06 の判断の一致のテストを足す。`job_index`・`record_of` そのもの、実行の流れの動き、計画の結果、`plan_simulate_perf.rs` は変えない。合否は BR3.5(llvm-cov の下で 1,000 ジョブが 75ms 以下、2 倍ごとの伸び 2.5 倍以下)に加え、push の後の CI の `coverage` が通ること。
- **Estimated impact**: 手間は core の `flow`(`decide.rs`・`next_step.rs`・`record.rs`・`mod.rs`)と `plan/simulate.rs` とテストの変更(開発の担当への 1 回の依頼の程度)。費用は CI の実行の分のみ。危険は中(実行と模擬が共有する判断の経路に手が入る)。判断の一致のテストと全体のテストで押さえる。

## やり直し 1 の後の実行(2026-11-26、コミット 3a0b491)

コード生成のやり直し 1(5405854、6104853、3862d2d)の後に、この段階を Modify で再び実行した。

### 手元(macOS)

| コマンド | 結果 |
|---|---|
| `cargo build --workspace --locked` | exit 0 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test -p conductor-runtime --test log_file --test runner_log --locked` | exit 0。19 passed |
| `cargo test -p conductor-runtime --lib --locked -- runner::tests` | exit 0。17 passed |
| `cargo test -p conductor-core --lib --locked -- flow::` | exit 0。56 passed(R-06 の 5 つを含む) |
| `cargo test -p conductor-core --test plan_pipeline --test plan_run --test plan_assume --test plan_simulate_perf --test flow_sequences --test flow_perf --test retry_flow --test attempt_flow --locked` | exit 0。58 passed |
| `cargo test -p conductor-cli --test unix_demo --locked` | exit 0。4 passed |
| `cargo test -p conductor-cli --lib --locked -- tests::docs tests::report_messages` | exit 0。67 passed |
| `cargo test -p conductor-core --test env_types --locked` | exit 0。6 passed |
| `cargo test --workspace --locked` | exit 0。1,437 passed、0 failed、1 ignored(前からのもの) |
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | exit 0。行カバレッジ 98.22%(10,823 行のうち 193 行が未通過)、領域 97.92%、関数 98.27% |
| 依存 | `Cargo.lock` の package 46(変わらず) |

### CI 45912018052(コミット 3a0b491)

- 12 のジョブがすべて成功(`lint`、`build-check` ×2、`test`(ubuntu-24.04、macos-latest、windows-2022)、`msrv-test`、`coverage`、`docs-check`、`startup-bench` ×3)。
- `coverage` の中で、`plan_simulate_perf.rs` の `p1_千個のジョブがすべて実行する計画は_150ms_以内`・`p2_終点のあとの_999_個のスキップの計画は_150ms_以内` は ok(テストのファイル全体が約 0.07 秒で終わった)。CI の行カバレッジの TOTAL は 98.22%。

## Target Verification Matrix の最終の判定

`build-and-test-summary.md` の表を参照。最初の実行では T-CI が **Not Met**(45908344996 の `coverage` の失敗)だった。やり直し 1 の後の実行で、T-CI を含む 10 の目標すべてが Met。
