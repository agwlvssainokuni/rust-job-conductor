# セキュリティのテストの手順(Security Test Instructions)

- 環境変数の値を表示やステートに出さない: 計画(`--dry-run`)は名前だけを持つ(`EnvNameEntry { name }`)。出どころを消しても、値を持たないことは変わらない。★1 のテスト(`計画は環境変数の値を持たない_名前だけ`)と ★2 のテストで確かめる。
- 誤りの表示に入力の値を含めない: `.env` の誤りと警告の表示は変えていない(警告の `file`・`line`・`name` は残る)。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に足していない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
