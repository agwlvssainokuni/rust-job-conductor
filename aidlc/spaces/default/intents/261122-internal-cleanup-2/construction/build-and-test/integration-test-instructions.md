# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# 段 3 の流れ(誤りの順序と表示がそのまま)
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_dry_run --test flow_run_id --test flow_env

# 段 4(型の変更の後も期待の文字列が同じ)
cargo test --locked -p conductor-cli --test args_grammar --test report_messages --test run_id_arg --test graph_long_id
cargo test --locked -p conductor-runtime --test flow_with_state
```

(ファイル名は `construction/code-generation/unit-test-instructions.md` の一覧に従う。)

結果: いずれも `cargo test --locked --workspace` の中で通った(1,137 通過)。`flow_with_state.rs` は今の cli の流れ(`remove_on_complete` の有無、完了の後の `read_state`)を確かめる形になった。上流: `construction/code-generation/code-generation-plan.md` の Step 10。
