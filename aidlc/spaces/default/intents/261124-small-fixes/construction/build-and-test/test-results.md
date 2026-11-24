# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`ce95937` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` | 警告なし(本番の形) |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,149 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.06%、関数 98.20%、領域 97.71%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ) |
| `conductor --version` | `conductor 0.9.8` |

## 件数の移り変わり

| 時点 | 全体 | 変わった所 |
|---|---|---|
| 基準(`c3a2a5b`) | 1,152 | |
| コミット 1〜6(説明と仕様書) | 1,152 | なし |
| コミット 7(T1) | 1,151 | core 統合 270 → 269 |
| コミット 8(T2) | 1,151 | `plan_perf` が無くなり、`plan_simulate_perf` が 2 → 3 |
| コミット 9(T3) | 1,149 | cli 単体 164 → 162 |
| コミット 10(T4)と R-01 の直し | 1,149 | `nfr_source` は 6 のまま |

見込み(1,152 − 3 + 0)と一致した。テスト名の差は、消した 3 件だけ。

## 完了の判定の検索(`functional-spec.md` §7)

- `decide_before_run`(`flow/decide.rs` の外)は 0 件。
- 仕様書 rust 02: `resolve_run_id` は `Option<&RunIdArg>`。`consistency` は再公開の一覧と公開モジュールの一覧にある。3 段落の「今の形」に `v0.` の注記は 0 件で、版の履歴は古い順。
- `plan_perf` の参照は 0 件。
- 製品のコードの `#[path` と include のマクロは 0 件。
- 依存、CI、README、runtime、converter、凍結した仕様書、`signals_order.rs`・`signals_real.rs` の差分は空。本番の src の差はコメントの行だけ。

## 未確認

- Linux、Windows、MSRV のジョブは、push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
