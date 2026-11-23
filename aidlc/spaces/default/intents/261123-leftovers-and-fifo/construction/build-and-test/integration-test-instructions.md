# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# cli の流れ(偽物の run_job の引数の変更、CONDUCTOR_JOB_ID による記録)
cargo test --locked -p conductor-cli

# runtime のステートと一時ファイル(形 B、replace_via_temp、bounded_read)
cargo test --locked -p conductor-runtime --test state_read --test state_write --test file_create

# converter の取り込み(形 B、N4)
cargo test --locked -p conductor-converter --test convert_ac
```

(ファイル名は `construction/code-generation/unit-test-instructions.md` の一覧に従う。)

結果: いずれも `cargo test --locked --workspace` の中で通った(1,161 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 3・4・9〜11。
