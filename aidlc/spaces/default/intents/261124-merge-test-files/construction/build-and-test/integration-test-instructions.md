# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテストと実バイナリのテストを使う。

```bash
# converter: 受け入れ基準(9 件)と表示の検査(10 件 + 1 無視)
cargo test --locked -p conductor-converter

# runtime: 実行(runner_basic 17 件)とステートの読み取り(state_read 19 件。raw:: の 4 件を含む)
cargo test --locked -p conductor-runtime

# cli: 単体(README の docs.rs 18 件を含む 172 件)と統合(284 件)。補助の fakes.rs を両方が読む
cargo test --locked -p conductor-cli
```

- cli の統合テストと単体テストは、同じ `crates/cli/tests/common/fakes.rs`(偽の Env・Sink とステートの誤り)を読む。
- init のひな形の原本は `crates/cli/tests/fixtures/init_{ja,en}.toml` にあり、`tests/flow_init.rs` と `src/tests/init_template.rs` の両方が読む。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,149 通過)。Windows で LF のまま取り出されるかは、push の後の CI の `test`(windows)で確かめる。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜6、`construction/code-generation/unit-test-instructions.md`。
