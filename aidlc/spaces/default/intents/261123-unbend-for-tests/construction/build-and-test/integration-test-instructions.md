# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# cli の流れ(偽の Env の色の判定の記録、シグナルの 1 回の登録、パニックのフックの単体テスト)
cargo test --locked -p conductor-cli

# runtime のステートの読み書き(definition_key と job_order を型から消した後の形)
cargo test --locked -p conductor-runtime

# core の判断(decide_from_state、N1)と、誤りの種類の写し(tests/common/kinds.rs)
cargo test --locked -p conductor-core

# converter の誤りの種類の写しと、表示に入力の値を含まない統合テスト
cargo test --locked -p conductor-converter
```

- ステートファイルの形: 2b の前と後のバイナリで、同じ定義から `pipeline p`、`run a`、`run --run-id r1` を実行し、できたステートファイルのバイト列が同じだった(`construction/code-generation/code-summary.md`)。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,157 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜12、`construction/code-generation/unit-test-instructions.md`。
