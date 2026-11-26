# 品質の関門: v0.10.0

出典: `ci-config.md`、Build and Test のまとめ(`construction/build-and-test/build-and-test-summary.md` の Target Verification Matrix)、team.md の Testing Posture・Code Style、project.md の Mandated・Forbidden。

## Build and Test の検査と CI の関門の対応

| Build and Test の目標 | 手元のコマンド | CI の関門 | CI の結果(45726504409) |
|---|---|---|---|
| T-TESTS(すべてのテスト) | `cargo test --workspace --locked` | `test`(ubuntu・macos・windows) | 成功 |
| T-COV(行 80% 以上) | `cargo llvm-cov ... --fail-under-lines 80` | `coverage` | 成功 |
| T-FMT、T-CLIPPY | `cargo fmt --check`、`cargo clippy -D warnings` | `lint` | 成功 |
| T-DEPS(依存を足さない) | `Cargo.lock` の package の数 | `build-check`(`-sys` と C の依存の検査) | 成功 |
| T-SRC、T-SECRET、T-PERF、T-COMPAT、T-DEMO | 全体のテストの中 | `test` | 成功 |
| T-DOCS | `tests::docs` | `test`、`docs-check` | 成功 |
| T-CI(3 つの OS と MSRV、Windows だけのテスト) | —(Build and Test では N/A、この段階に任せた) | `test`(windows を含む)、`msrv-test`、`build-check`(1.100.0) | 成功 |
| T-VERSION | `conductor --version` | `release.yml` の `verify-tag` と smoke test(タグのとき) | タグはまだ付けない |

## 関門ではないもの

- `startup-bench`(起動の性能)と `audit`(`cargo audit`)は警告のみ。team.md の決定どおり v0.10.0 でも強制しない。
- 署名・公証、Intel macOS の動作の確認は見送り(team.md)。

## 関門を下げていないこと

- 行カバレッジの下限 80%、`-D warnings`、`--locked`、MSRV 1.100 はどれも変えていない。Windows の失敗はテストの補助を直して通し、テストを外したり無視にしたりしていない。
