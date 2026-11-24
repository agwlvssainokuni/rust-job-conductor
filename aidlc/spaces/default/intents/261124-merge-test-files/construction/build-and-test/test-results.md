# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`f4fafde` の時点。

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

## 件数

| 時点 | core | runtime | converter | cli | 全体 |
|---|---|---|---|---|---|
| 基準(`ba82c30`) | 112 / 269 | 69 / 128 | 96 / 19 + 1 無視 | 162 / 294 | 1,149 |
| 後(`f4fafde`) | 112 / 269 | 69 / 128 | 96 / 19 + 1 無視 | 172 / 284 | 1,149 |

(単体 / 統合)。どのコミットでも全体は 1,149 で、見込みと一致した。

## テスト名の比較

- 名前の最後の部分で比べた差は 0。パスが変わったのは 28 件(`raw::` 4、`tests::args_grammar::` 2、`tests::convert_roundtrip::` 2、`tests::docs::` 18、`tests::graph_report::` 2)。実行ファイルだけが変わったのは 6 件。

## 確かめたこと

- まとめた 19 ファイル(cli 13、converter 4、runtime 2)は無くなった。
- 移した 34 件の本体は、名前の衝突を避ける 5 行(D1・D2)と `mod raw` の字下げ(D3)のほかは元と同じ。
- ひな形のデータのファイルは元の原本とバイト単位で同じ。`git check-attr` は `text: set`、`eol: lf`。
- 仕様書に無くなったファイルの名前は残っていない(rust 12 の 142 行目もユーザーの指示で直した)。本番のコード、依存、CI、README、凍結した仕様書の差分は空。

## CI での確かめ(push の後)

- **D5 の最終確認: 確認済み**。`f4fafde` の CI(run 45503331915)の `test (windows-2022)` は失敗 0 で通った。ひな形を読むテスト(`flow_init`、`init_template`)も成功し、`.gitattributes` で LF のまま取り出されることを確かめた。
- 同じ CI の 12 のジョブ(ubuntu・macOS・Windows のテスト、MSRV、lint、coverage など)はすべて成功した。

上流: `construction/code-generation/unit-test-instructions.md`(指標の確かめ方)、`construction/code-generation/code-generation-plan.md`。
