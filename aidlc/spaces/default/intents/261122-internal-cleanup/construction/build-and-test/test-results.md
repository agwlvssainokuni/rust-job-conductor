# テストの結果(Test Results): 動きを変えない整理のリファクタリング

実行日 2026-11-22、macOS(ローカル)、HEAD d619fca。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,117 通過、0 失敗、1 無視 |
| `cargo llvm-cov --fail-under-lines 80` | 行 98.19%(9,486 行中 172 行が未実行)、関数 98.12%、領域 97.84% |

## 件数の移り変わり(開発担当の記録と、確認担当の再実行)

| 時点 | 通過 / 無視 |
|------|-------------|
| 基準(513115b) | 1,123 / 1 |
| 段 1 のあと | 1,123 / 1 |
| 段 2 のあと | 1,123 / 1 |
| 段 3 のあと | 1,123 / 1 |
| 段 4 のあと | 1,117 / 1 |

減った 6 件は、段 4a で入力と期待の全文が同じ重複としてまとめた graph のテスト(64 → 58)。一覧は `construction/code-generation/code-summary.md`。converter のテストは改名だけで 117 件(116 通過、1 無視)のまま。

## テストのファイルの変更(段 1〜3)

`git diff 513115b..e99b38c` で、テストのファイルの変更は `crates/cli/tests/report_messages.rs` だけ。変えたのは `DisplayEvent` の組み立ての値と import で、期待の文字列は変わっていない(ユーザーが設計で承認した範囲)。src の中の `mod tests` も変わっていない。

## CI で確かめること

- Windows: 日本語のテスト名を `--exact` で子プロセスに渡すテスト(converter の「環境を調べる子プロセス」)。
- MSRV 1.100 でのビルドとテスト。
