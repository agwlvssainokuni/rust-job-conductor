# セキュリティのテストの手順(Security Test Instructions)

- 誤りの表示に入力の値を含めない: ライブラリの誤りの型の `Display` は残した(Q5)。「表示に値を含めない」ことを確かめる `nfr_security.rs` などのテストは、期待を変えずに通った。
- `sanitize` を `escape_display` に置き換えた所は、切り詰めの長さ(256 と 64)が同じ。制御文字のエスケープと切り詰めは、エスケープの表のテストと core の単体テストで確かめる。
- run ID の検査: `RunIdError` を 1 つにしても、拒否する条件(空、64 バイト超、許可外の文字、先頭が `.`)は同じ。単体テストで 4 つの場合の入力が通らないことを確かめる。表示は `invalid value`(68)のまま。
- runtime の製品のコードに、パニックしうる書き方(`unwrap`、`expect`、`panic!` など)を足していない: `nfr_source.rs`。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に足していない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
