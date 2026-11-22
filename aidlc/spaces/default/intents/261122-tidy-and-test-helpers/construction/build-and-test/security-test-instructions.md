# セキュリティのテストの手順(Security Test Instructions)

- 誤りの表示に入力の値を含めない: 文言の定数をまとめた後も、誤りの文は固定の文と表示名だけ(`report_messages.rs`、`escape_table.rs`、core の `error::tests`)。
- 一時ファイル: 名前の形(`.<名前>.tmp.<pid>.<nanos>.<seq>`)と再試行の回数は同じで、連番を 1 つにした(D5)。`state_write.rs`(一時ファイルが残らない、古い一時ファイルの後始末)、`file_create.rs`、`flow_init.rs`、N4 で確かめた。
- `reset --all` の振り分け: 理由の文の一致ではなく原因の種類(`FailureCause`)で判定するようにした。振り分けの結果は同じ。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml`、`Cargo.lock` に差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
