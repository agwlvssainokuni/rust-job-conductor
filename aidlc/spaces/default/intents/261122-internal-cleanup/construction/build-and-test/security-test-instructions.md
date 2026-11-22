# セキュリティのテストの手順(Security Test Instructions)

新しい入力の経路も依存も無い。既存の性質を保っていることを、既存のテストで確かめる。

- エラーの表示に入力の値を含めない: `error_messages_never_contain_input_values` を改名した「誤りの文は入力の値を含まない」(converter)、各 `error::tests`、`nfr_security.rs`。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml`、`Cargo.lock` に差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。
