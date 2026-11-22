# セキュリティのテストの手順(Security Test Instructions)

- `unsafe` を使わない: core と converter に `#![forbid(unsafe_code)]` を足した(コンパイラが強制)。cli は従来どおり。`unsafe` は runtime の `platform/unix.rs` だけ。
- `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- エラーの表示に入力の値を含めない: `RunIdArg::parse` の誤りは対象が `--run-id` で値を含まない(`run_id_arg.rs`、`args_grammar.rs`)。escape の本体を core の `sanitize` に寄せた後も、`escape_table.rs` と core の `error::tests` が通った。
- 新しい依存なし: `Cargo.toml`、`Cargo.lock` に差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`(SM6)。
