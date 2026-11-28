# テストの結果: v0.10.4

出典: `construction/code-generation/code-summary.md`、`unit-test-instructions.md`、`build-instructions.md`。

## ビルドと品質の検査(手元の macOS、コミット 480a30d の時点)

| 検査 | 結果 |
|---|---|
| `cargo build --locked` | 成功。`conductor --version` は `conductor 0.10.4` |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `Cargo.lock` の package | 46(v0.10.3 と同じ) |

## テスト

| 項目 | 結果 |
|---|---|
| `cargo test --workspace --locked` | 1,449 passed、0 failed、1 ignored(始める前と同じ) |
| `#[test]` の数 | 1,472(始める前と同じ) |
| 各ステップのテスト(`unit-test-instructions.md`) | すべて通った(コード生成の記録) |

- テストの名前と期待の値は変えていない。例外は次の 2 つ。
  - 版の期待(`binary_basic.rs` の `--version`)
  - 人の判断 A(334c574): `with_all_events` に 2 つの変種を足し、エラーの行のテストの例外の条件に `ResetAllKept` を足した

## 全体のカバレッジ

- 1 回目: `runtime` の `runner_log` の `br4_1_標準出力と標準エラーの両方がファイルに残り_各流れの中の順を保つ` が、77 行(`assert_eq!(log, LogOutcome::default())`)で落ち、止まった。
  - ジョブの終わった後に出力を写す処理を待つ上限は、`runner.rs` の `DRAIN_LIMIT`(2 秒)。計測の負荷でこの上限を超え、`output_cut_short` が付いたと見ている。
  - この intent では `crates/runtime/src/` を変えていない(版の `Cargo.toml` だけ)。
  - v0.10.3 の Build and Test でも、同じ `runner_log` が 1 度だけ落ちている。そのときは、落ちたテストの名前を残せなかった。
- 2 回目: 全体が通った。行カバレッジは **98.25%**(11,029 行のうち 193 行が通らなかった。下限は 80%)。

## 分け終わりの判定(コード生成の R-01)

| ファイル | 行(前 → 後) | 参考値 | 主な基準(移す項目が `mod.rs` に無い) |
|---|---|---|---|
| `walk/mod.rs` | 695 → 411 | 420 | 満たす |
| `args/mod.rs` | 679 → 341 | 340 | 満たす(1 行超えは `code-summary.md` に記録) |
| `report/mod.rs` | 746 → 195 | 250 | 満たす |

- `render` は 188 行から 64 行になった。各枝は、メソッドを 1 つ呼ぶだけになっている。
- `build_command` は 10 行になった。

## CI 46131106543(コミット cf9c6a3)

- 12 のジョブがすべて成功した。
  - `lint`、`docs-check`、`coverage`、`msrv-test`
  - `test`(ubuntu-24.04、macos-latest、windows-2022)
  - `build-check`(ubuntu-24.04、windows-2022)
  - `startup-bench`(ubuntu-24.04、macos-latest、windows-2022)
- 新しいファイルを含めても、ソースの検査(`nfr_source`)は 3 つの OS で通った。

## Target Verification Matrix の最終の判定

- すべて Met(`build-and-test-summary.md`)。
