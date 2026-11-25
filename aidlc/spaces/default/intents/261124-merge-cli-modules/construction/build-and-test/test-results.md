# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`ac16c17` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` | 警告なし(本番の形) |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,149 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.06%、関数 98.19%、領域 97.71%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ) |
| `conductor --version` | `conductor 0.9.8` |

## 件数とテスト名

- 前(`9fe86f5`)も後も 1,149 通過・1 無視。cli は 456 件(単体 172、統合 284)。
- テスト名の一覧(パスつき)は前後とも 1,150 行で、差は 0。テストの変更は `src/tests/escape_table.rs` の `use` の 1 行(T1)だけ。

## 確かめたこと

- 消した 4 ファイル(`retry.rs`、`report/escape.rs`、`graph.rs`、`init.rs`)は無い。cli の `src/` の `.rs` は 40 から 36。
- 消えたモジュールのパスの参照は、コードと仕様書で 0 件(仕様書の検索は除外の前の出力も見た。当たったのは core の別のファイルだけ)。
- 移した関数の本体は、可視性の語のほかは元と同じ。
- `platform` のファイルは `mod.rs`・`unix.rs`・`windows.rs` のまま。再公開は 2 行。
- core・runtime・converter、依存、言語に依らない仕様書、凍結した仕様書の差分は空。
- レビューの R-01 を直した(`step.rs` の説明の重なった 1 文を消した)。

## CI での確かめ(push の後)

- `ac16c17` の CI(run 45540522737)の 12 のジョブはすべて成功した。`test (windows-2022)` と `build-check (windows-2022)` が通り、シグナルの監視の入口(`platform`)が Windows でもこれまでどおり動くことを確かめた。Linux と macOS のテスト、MSRV も通った。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
