# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`6908726` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,153 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.08%、関数 98.20%、領域 97.77%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ。この intent と無関係) |
| `conductor --version` | `conductor 0.9.8` |
| CI(`6908726`、13 のジョブ) | すべて成功(ubuntu・macOS・Windows のテスト、lint、coverage、MSRV、build-check、docs-check、起動の性能) |

## 件数の移り変わり

| 時点 | core | runtime | converter | cli | 全体 |
|---|---|---|---|---|---|
| 基準(`23b8975`) | 384 | 198 | 116 | 459 | 1,157 |
| 最後(`e4a2bf1`) | 382 | 197 | 116 | 458 | 1,153 |

見込み(1,157 − 4 + 0 = 1,153)と一致した。無視は converter の 1 件のまま。テスト名の差は、消した 4 件(§4.6)だけ。名前の変わったテストは無い。

## 確かめたこと

- 完了の判定の検索(`functional-spec.md` §7): 消す・狭める名前の参照はすべて 0 件。`#[default]` は cli の `GraphFormat` の 1 件、cli の `pub mod` は `exit`・`report`・`run_id` の 3 件で、期待どおり。
- 2 つのコミットにまたがる import(T20・T23・T25)は、途中のコミットでもコンパイルが通った。

## テストの変更

- 既存のテストの変更は T1〜T32 と、§4.6 の 4 件の削除だけ。期待の文字列とテスト名は変えていない。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
