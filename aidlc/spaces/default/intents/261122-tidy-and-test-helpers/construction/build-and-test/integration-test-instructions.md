# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# cli の流れ(偽物の中身を直した。N2・N3 を足した)
cargo test --locked -p conductor-cli --test flow_run --test flow_retry --test flow_pipeline --test nfr_robust

# 表示の文言(定数をまとめた後も期待の文字列が同じ)
cargo test --locked -p conductor-cli --test report_messages --test escape_table

# runtime のステートと一時ファイル
cargo test --locked -p conductor-runtime
```

(ファイル名は `construction/code-generation/unit-test-instructions.md` の一覧に従う。)

結果: いずれも `cargo test --locked --workspace` の中で通った(1,146 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 3・7。
