# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`22e550d` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,146 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.14%、関数 98.20%、領域 97.76%(下限 80%) |

## 件数の移り変わり

| 時点 | 通過 | 理由 |
|------|------|------|
| 基準(`af018bc`) | 1,142 | — |
| 段 1 のあと | 1,141 | `program_name` の単体テストを 1 件消した(T38) |
| 段 2 のあと | 1,141 | |
| 段 3 のあと | 1,142 | N4 |
| 段 4 のあと | 1,142 | |
| 段 5 のあと | 1,143 | N1 |
| 段 6 のあと | 1,143 | |
| 段 7 のあと | 1,146 | N2、N3、N5 |

クレートごと: core 115 + 261、runtime 68 + 125、converter 54 + 63(ほかに 1 無視)、cli 460。どれも見込みと一致した。

## テスト名の比較

基準と最後の `cargo test --locked --workspace -- --list` の差は、次のものだけだった。
- 消えた名前: T38 の 1 件
- 置き場所だけ変わった名前: 一時ファイルの単体テスト 4 件(`state::writer::tests` → `temp_file::tests`、T22)、整合の単体テスト 4 件(runtime → core、T28・T29)
- 足した名前: N1〜N5 の 5 件

## テストの変更

- 既存のテストの変更は T1〜T46 の範囲だけ。期待する出力の文字列は変えていない。
- 失敗したときだけ出る `expect` の文が 2 つ変わった(T6。承認済みの見出しの文と、`docs_retry.rs` の「TOML の例がある」)。
- T9 の `status_in` は、ジョブがステートに無いとき失敗する(厳しくなる方向。承認済みの ★)。

## 未確認

- Linux、Windows、MSRV のジョブは push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
