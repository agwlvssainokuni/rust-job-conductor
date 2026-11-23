# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# cli の流れと表示(型の変更の後も期待の文字列が同じ。N3)
cargo test --locked -p conductor-cli --test flow_run --test flow_pipeline --test flow_retry --test flow_dry_run --test report_messages

# runtime の起動と .env(run_job の引数の変更、N11・N12)
cargo test --locked -p conductor-runtime

# 変換(N10)と版(T25)
cargo test --locked -p conductor-converter --test convert_ac
cargo test --locked -p conductor-cli --test binary_basic
```

(ファイル名は `construction/code-generation/unit-test-instructions.md` の一覧に従う。)

結果: いずれも `cargo test --locked --workspace` の中で通った(1,157 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 3・5・6・9〜11。
