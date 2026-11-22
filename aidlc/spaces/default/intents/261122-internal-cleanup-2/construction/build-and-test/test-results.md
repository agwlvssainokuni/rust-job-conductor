# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`cfca976` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,137 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.28%、関数 98.36%、領域 97.93%(下限 80%) |

## 件数の移り変わり

| 時点 | 通過 | 基準からの増分 |
|------|------|----------------|
| 基準(`65e5240`) | 1,128 | — |
| 段 1 のあと | 1,128 | +0 |
| 段 2 のあと | 1,134 | +6(性能 3、`label` の単体 1、graph の長い ID 2) |
| 段 3 のあと | 1,134 | +6 |
| 段 4 のあと | 1,137 | +9(`RunIdArg` 1、`TransitionOn::parse` 1、`From<Classification>` 1) |

- 見込みの +7 より 2 件多いのは `graph_long_id.rs` の 2 件。計画の Step 5 が「既存のテストに無ければ足す」としていたため。
- `flow_with_state.rs` は 20 件のまま(T9)。

## 性能(debug、5 回の中央値)

| 場面 | 上限 | 直す前 | 直した後 |
|------|------|--------|----------|
| A2(1,000 ジョブ、`@end` の列挙 ×100) | 50 ms | 約 291 ms | 約 5.3 ms |
| A3a(`.env` 1 ファイル 10,000 項目) | 50 ms | 約 362 ms | 約 5.1 ms |
| A3b(2 ファイル × 10,000 の同じ名前) | 100 ms | 約 1,704 ms | 約 8.9 ms |

- A3b の直す前は、BR4.3 の並びを確かめるため `env_file` を逆順にしたので、設計のときの 2,166 ms と違う。
- 計測つき(llvm-cov)でも上限の中で通った。

## テストの変更

- 既存のテストの変更は T1〜T9 の範囲だけで、期待する出力の文字列は変えていない(確認担当も確認)。
- 新しいファイル: `flow_perf.rs`、`env_merge_perf.rs`、`graph_long_id.rs`、`run_id_arg.rs`。

## 未確認

- Linux、Windows、MSRV のジョブは push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(SM1〜SM6 の確かめ方)、`construction/code-generation/code-generation-plan.md`。
