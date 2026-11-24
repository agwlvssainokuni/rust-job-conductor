# セキュリティのテストの手順(Security Test Instructions)

- 製品のコードの検査は、数と中身を変えずに通った: cli の `nfr_source` 6 件、runtime の `nfr_source` 5 件、converter の `nfr_security` 10 件 + 1 無視。converter の src の直下のファイルは 11 のまま(10 以上の条件)。
- cli の補助を単体テストから読む `#[path]` は `src/tests/` の中にあり、cli の検査(`src/tests/` の外の `#[path`・include のマクロを止める)に当たらない。ひな形を読む `include_str!` も、テストの側にだけある。
- 誤りの表示に入力の値を含めない: converter の `nfr_security.rs`(表示の検査)と cli の表示のテストは、期待を変えずに通る。
- 本番のコード、`unsafe`、製品のコードへの `cfg(unix|windows)`、依存の追加はない。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
