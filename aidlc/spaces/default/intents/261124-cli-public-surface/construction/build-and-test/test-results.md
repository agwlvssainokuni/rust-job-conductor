# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`3ea6048` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` | 警告なし(本番の形) |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,152 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.06%、関数 98.20%、領域 97.71%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ。cli は 0) |
| `conductor --version` | `conductor 0.9.8` |

## 件数の移り変わり(cli。単体 / 統合)

基準 `b4510e3` の 2 / 456 から、コミットごとに 9/449、12/446、13/445、16/442、76/382、106/352、113/345、120/338、156/302、164/294、164/294、164/294。どれも見込みどおりで、失敗は常に 0。全体は前後とも 1,152 通過・1 無視(core 382、runtime 197、converter 115 + 1 無視、cli 458)。

## テスト名の比較

- 名前の最後の部分で比べた差は 0 件。移した 162 件はパスだけが変わった。消したテストと足したテストは無い。

## 完了の判定の検索(`functional-spec.md` §7)

- `lib.rs` の公開は、設計の「残す 13」と一致(`pub mod`・`pub use args`・`pub use types` は 0 件)。末尾は `#[cfg(test)] mod tests;`。
- `tests/` から狭めた名前を参照する所は 0 件。`src/tests/` の中の `conductor_cli` は 0 件。`src/tests/` の外の `#[path` は 0 件。
- `src/tests/` の `unsafe`・ネットワークの語は 0 件。
- 仕様書 rust に、狭めた名前を公開として書く記述は残っていない。言語に依らない仕様書と凍結した仕様書、core・runtime・converter、`Cargo.toml`・`Cargo.lock` の差分は空。

## 未確認

- Linux、Windows、MSRV のジョブは、push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
