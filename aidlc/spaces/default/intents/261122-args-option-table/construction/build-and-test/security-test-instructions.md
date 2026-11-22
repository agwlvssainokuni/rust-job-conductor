# セキュリティのテストの手順(Security Test Instructions)

- エラーの表示に入力の値を含めない: `convert --to` の誤りが値を含まなくなった(規則に揃った)。`to_の値の誤りの対象は_to_で値を含まない` と `utf8_でない_to_の値は_invalid_value_で対象は_to`(UNIX)で確かめる。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml`、`Cargo.lock` に差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。
