# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# 段 1・2(cli の流れ。偽物だけを直し、期待は無変更)
cargo test --locked -p conductor-cli

# runtime の起動(本物のプロセス)
cargo test --locked -p conductor-runtime --test runner_basic --test runner_signal --test runner_env --test nfr_robust --test flow_with_state

# 段 3 と版
cargo test --locked -p conductor-cli --test binary_basic
```

(ファイル名は `construction/code-generation/unit-test-instructions.md` の一覧に従う。)

結果: いずれも `cargo test --locked --workspace` の中で通った(1,142 通過)。cli の流れのテストは、偽物(T1、T13)のほかは 1 行も変えずに通った。上流: `construction/code-generation/code-generation-plan.md` の Step 2、5。
