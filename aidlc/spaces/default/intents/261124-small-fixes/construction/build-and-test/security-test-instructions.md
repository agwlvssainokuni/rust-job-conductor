# セキュリティのテストの手順(Security Test Instructions)

- ソースの検査の強化(`crates/cli/tests/nfr_source.rs`、D4): `src/tests/` の外の製品のコードで、`#[path` と、`include!`・`include_str!`・`include_bytes!` を、行の空白を除いて探し、見つけたら失敗する。今の製品のコードでは通る。製品のファイルに `include_str!` を一時的に足すとこのテストだけが失敗することを、実装の中で確かめた(足した行はコミットしていない)。
- 既知の限界(テストのコメントに書いた): 空白を除いた部分一致なので、`my_include!` や文字列・`/* */` のコメントの中の語にも当たる(誤検出の側)。`#[cfg_attr(…, path = "…")]` の形は見逃す。行の `//` 以降を切り落とすので、文字列の中の `//`(URL など)より後ろに書いたものも見逃す(レビューの R-01。ユーザーの指示で書き足した)。
- `unsafe`・`low_level`・ネットワーク・製品のコードへの `cfg(unix)`/`cfg(windows)` を足していない。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
