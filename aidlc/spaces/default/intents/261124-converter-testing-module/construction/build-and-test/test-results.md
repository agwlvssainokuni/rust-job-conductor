# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`6ab33c3` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo clippy -p conductor-converter --lib --locked -- -D warnings` | 警告なし(本番の形。`PartialEq` なし) |
| `cargo clippy ... --target x86_64-pc-windows-msvc -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,152 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.09%、関数 98.20%、領域 97.77%(下限 80%) |
| `cargo doc --workspace --no-deps --locked` | 警告 5 件(基準と同じ。この intent と無関係) |
| `conductor --version` | `conductor 0.9.8` |

## 件数の移り変わり(converter。単体 / 統合。統合のほかに 1 無視)

| 時点 | converter | 全体 |
|---|---|---|
| 基準(`8ded1e9`) | 52 / 64 | 1,153 |
| コミット 1〜4(移す) | 80/36、93/23、95/21、97/19 | 1,153 |
| コミット 5(`opposite` のテストを消す) | 96 / 19 | 1,152 |
| コミット 6 | 96 / 19 | 1,152 |

core、runtime、cli の件数は変わらない。見込み(1,153 − 1 + 0)と一致した。

## テスト名の比較

- 名前の最後の部分で比べた差は、消した 1 件(`反対の形式は_yaml_と_toml_を入れ替え_2_回で元に戻る`)だけ。名前の変わったテストは無い。
- 移した 45 件はパスだけが変わった(`tests::read_resolve` 28、`tests::convert_roundtrip` 13、`tests::convert_ac` 2、`tests::nfr_security` 2)。本体を比べると、確かめの行、入力、期待の値は変わっていない。

## 完了の判定の検索(`functional-spec.md` §7)

- `testing::`、`read_tree`、`mod testing`、`doc(hidden)`、`opposite` は、コードと仕様書(rust、language-agnostic)で 0 件。
- `impl PartialEq for Value` の直前は `#[cfg(test)]`。`lib.rs` の `#[cfg(test)]` はファイルの最後の `mod tests;` の 1 件だけ。
- `src/tests/` の禁止の語の検索は 0 件。
- core、runtime、cli、言語に依らない仕様書、凍結した仕様書、`Cargo.toml`・`Cargo.lock` の差分は空。

## 未確認

- Linux、Windows、MSRV のジョブは、push の後の CI で確かめる。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
