# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。converter の統合テスト(`tests/`)は、公開の入口(`convert`、`convert_text`)だけで確かめる 19 件(と無視 1 件)が残る。途中の表現を比べる 45 件は `src/tests/` の単体テストへ移った。

```bash
# converter: 単体(src/tests/ を含む 96 件)と統合(19 件 + 1 無視)
cargo test --locked -p conductor-converter

# cli: convert の実バイナリのテスト(変換の出力、誤りの表示、終了コード)
cargo test --locked -p conductor-cli
```

- cli の `convert` のテスト(`binary_basic.rs`、`flow_aux.rs`、`report_messages.rs` など)は、期待を変えずに通る。利用者から見える動きは変わらない。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,152 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜6、`construction/code-generation/unit-test-instructions.md`。
