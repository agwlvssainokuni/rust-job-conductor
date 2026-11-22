# ビルドとテストのまとめ(Build and Test Summary): `--dry-run` の模擬の重複をなくすリファクタリング

## 全体の状況

- ビルド: 成功。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: 全体で **1,123 件が成功、失敗 0 件**(この intent の前は 1,121 件)。
- 行カバレッジ: **97.96%**(下限 80%)。
- 前提: Rust 1.100 以上(`rustc 1.100.0`、MSRV と同じ)、`cargo-llvm-cov`。新しい依存なし。
- 詳細: `build-instructions.md`、`integration-test-instructions.md`、`performance-test-instructions.md`、`security-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類(Minimal)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体(core) | 更新の関数(移動、無変更) | `crates/core/src/flow/record.rs` |
| 模擬(core) | 計画(既存)、承認した手書きのステートの形(新規)、性能(新規) | `plan_pipeline.rs`、`plan_run.rs`、`plan_perf.rs` |
| 流れ(runtime、cli) | ステートの書き込み、`--dry-run`(既存、無変更) | `flow_with_state.rs`、`flow_*.rs` |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract、`team.md`、`project.md` の ALWAYS | 行カバレッジ 80% 以上 | 97.96%(9,447 行のうち未通過 193) | `cargo llvm-cov ... --fail-under-lines 80 --summary-only` が終了コード 0 | build-and-test | Met |
| T2-quality | `project.md` の ALWAYS(`fmt`、`clippy -D warnings`、MSRV 1.100) | 差分なし、警告なし、MSRV で通る | 差分なし、警告なし。rustc 1.100.0 で全テスト成功 | `cargo fmt --check`、`cargo clippy -D warnings`、`rustc --version` | build-and-test | Met |
| T3-no-new-deps | `project.md` の NEVER | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行(`acbd187` 以降) | `git diff acbd187 HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` | build-and-test | Met |
| T4-no-behaviour-change | BR2.1、BR2.2、SM3 | 既存のテストが変えずに通り、`--dry-run` の出力が変わらない(承認した 1 形を除く) | 既存のテストが無変更で成功。90 万例の比べで違い 0 件 | 開発担当の一時のテスト、確認の記録 | build-and-test | Met |
| T5-one-place | BR1.1〜BR1.3、SM1、SM2 | 更新の関数が core の 1 か所。模擬の写しがない | `flow/record.rs` に 4 つの関数。`apply_result` は削除 | コード、確認の記録 | build-and-test | Met |
| T6-perf | BR2.3 | 終点の後の 1,000 個近いスキップの計画が 1 秒以内 | 約 0.1 秒 | `plan_perf.rs` | build-and-test | Met |
| T7-docs | BR3.1、SM4 | 05 §3.3〜§3.5 に更新の規則を 1 回、模擬の節はそれを指す | そのとおり。02、12 も更新 | 確認の記録 | build-and-test | Met |
| T8-ci-os | `team.md`(他の OS は CI で確認) | ubuntu、macos、windows の `cargo test` が通る | 未実行(push の前) | 既存の `ci.yml` | (既存の CI) | N/A(push のあとで確かめる。OS に依存するコードは足していない) |

`Pending` の行は残っていない。

## 準備の評価

- ビルドとテストの準備: できている。
- 配信の準備: この作業の範囲の外。

## 既知の限界

- CI(Windows と ubuntu)は未実行(T8)。push 後に確認する。
- `docs/spec/{go,python,typescript}/02-architecture.md` に、古い `state/record` の記述が残る(意図して範囲の外。確認の R-01)。
- 版は 0.9.4 のまま。`--dry-run` の出力の変化は、手で書き換えたステートの 1 形だけ(承認済み)。
