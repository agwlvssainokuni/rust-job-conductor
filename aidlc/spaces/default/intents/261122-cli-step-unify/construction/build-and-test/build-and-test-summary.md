# ビルドとテストのまとめ(Build and Test Summary): run と pipeline の重複をなくすリファクタリング

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,121 件が成功、失敗 0 件**(この intent の前は 1,118 件)。cli は 440 件から 443 件(既存は無変更)。
- 行カバレッジ: **97.99%**(下限 80%)。
- 前提: Rust 1.100 以上(`rustc 1.100.0`、MSRV と同じ)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`performance-test-instructions.md`、`security-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 流れ(cli) | 書き込みの順序、終了コード、リトライ、run ID、シグナル(既存、無変更)。削除の時機の違い(新規 3 本) | `flow_run.rs`、`flow_pipeline.rs`、`flow_retry.rs`、`flow_run_id.rs`、`signals_order.rs` ほか |
| 実バイナリ | 既存のまま | `binary_*.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上(プロジェクト全体) | 97.99%(9,465 行のうち未通過 190) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only` が終了コード 0 | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0(MSRV と同じ)で全テスト成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`21fef24` 以降) | `git diff 21fef24 HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-no-behaviour-change | BR2.1、SM2 | 既存のテストを変えずにすべて通る | cli の既存 440 件が無変更で成功。確認で古いコードと 1 行ずつ突き合わせ、違いなし | `git diff 21fef24..HEAD -- crates/cli/tests`(追加だけ)、確認の記録 | build-and-test | Met |
| T5-one-step | BR1.1、SM1 | 共通の手順が 1 か所 | `step.rs` に 4 つの関数。`run.rs` 100 行、`pipeline.rs` 143 行 | コード | build-and-test | Met |
| T6-docs | BR3.1、SM3 | 05 の §5.2 と §5.3 が共通の手順を指し、同じ手順を 2 回書かない | 新しい §5.1a に共通の手順、§5.2・§5.3 はそれを指す。02 も更新 | 確認の記録 | build-and-test | Met |
| T7-ci-os | `team.md`(他の OS は CI で確認) | ubuntu、macos、windows の `cargo test` が通る | 未実行(push の前) | 既存の `ci.yml` | (既存の CI) | N/A(push のあとで確かめる。OS に依存するコードは足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドとテストの準備: できている。
- 配信の準備: この作業の範囲の外。

## 既知の限界

- CI(Windows と ubuntu)は未実行(T7)。push 後に確認する。
- 版は 0.9.4 のまま(動きが変わらないので、リリースは要らない)。
