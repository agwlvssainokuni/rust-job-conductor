# テストの結果(Test Results): args.rs のオプションの表化

実行日 2026-11-22、macOS(ローカル)、HEAD 5483477。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,128 通過、0 失敗、1 無視 |
| `cargo llvm-cov --fail-under-lines 80` | 全体: 行 98.18%、関数 98.15%、領域 97.84%。`crates/cli/src/args.rs`: 行 98.11%(265 行中 5 行が未実行) |

## 件数の移り変わり(全体の通過 / 無視、`args_grammar.rs`)

| 時点 | 全体 | `args_grammar.rs` |
|------|------|-------------------|
| 基準(ba1cdfb) | 1,117 / 1 | 49 |
| 報告の順のテストを足したあと(a175253) | 1,126 / 1 | 58 |
| 表化のあと(0300b32、bee8f58) | 1,126 / 1 | 58 |
| `--to` の変更のあと(61690c3) | 1,128 / 1 | 60 |

- 既存のテストの削除行・変更行は 0(`git diff ba1cdfb..HEAD -- crates/cli/tests`)。
- 報告の順のテスト 9 件は、表化の前のコードで通ることを確かめてからコミットした(今の動きの固定)。

## CI で確かめること

- Windows、ubuntu でのテスト。UTF-8 でない `--to` の値のテストは `#[cfg(unix)]` なので、Windows では動かない(確認の指摘 R-01。受け入れ済み)。
- MSRV 1.100 でのビルドとテスト(ローカルの rustc も 1.100.0)。
