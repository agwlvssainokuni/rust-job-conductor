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

## Target Verification Matrix の最終の判定

`build-and-test-summary.md` の表を参照。T-CI は **Not Met**(45908344996 の `coverage` の失敗)。他の 9 つは Met。
