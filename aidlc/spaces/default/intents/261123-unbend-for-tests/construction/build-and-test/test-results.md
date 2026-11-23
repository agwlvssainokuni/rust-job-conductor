# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`8f07ec2` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,157 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.12%、関数 98.31%、領域 97.81%(下限 80%) |
| CI(`6df843d`、13 のジョブ) | すべて成功(ubuntu・macOS・Windows のテスト、lint、coverage、MSRV、build-check、docs-check、起動の性能) |

## 件数の移り変わり

| 時点 | core | runtime | converter | cli | 全体 |
|---|---|---|---|---|---|
| 基準(`ca9d1d5`) | 384 | 198 | 117 | 462 | 1,161 |
| 段 1 の後(`1091276`) | 383 | 198 | 116 | 459 | 1,156 |
| 段 2 の後(`81ea255`) | 384 | 198 | 116 | 459 | 1,157 |

どの段も見込みと一致した。無視は converter の 1 件のまま。テスト名の差は、消した 5 件(§4.6)、名前の変わった ★3 件(T1、T2、T30)、置き場所の変わった 2 件(パニックのフックの本体のテストを単体テストへ)、足した N1 だけ。

## 確かめたこと

- 完了の判定の検索: 消した名前(`language_resolved`、`color_resolved`、`removed`、`is_lane_drawable`、`LoadedEnvFiles::add`、`code()`、`ALL` 5 つ、`on_panic`、`install_panic_hook_with`、`AlreadyRegistered`、`REGISTERED` など)の参照は、`crates` の全体で 0 件。
- ステートファイル: 2b の前と後で、3 つの場面のバイト列が同じ。
- 性能: P4 は 117.2 → 121.3〜132.3 ms(上限 600 ms)。

## テストの変更

- 既存のテストの変更は T1〜T31 と、§4.6 の 5 件の削除と、ユーザーの決定で直した `crates/runtime/tests/flow_with_state.rs:140` のコメント 1 行だけ。期待の文字列は変えていない。
- 名前の変更は ★ T1・T2・T30 だけ。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
