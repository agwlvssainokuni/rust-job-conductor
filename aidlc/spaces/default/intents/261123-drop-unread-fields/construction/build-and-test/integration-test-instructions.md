# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテストと実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# core: graph の辺、環境変数の重ね合わせ、.env の読み取り、計画
cargo test --locked -p conductor-core

# cli: 計画の表示(--dry-run の環境変数の名前の並び)、graph の表示、.env の警告
cargo test --locked -p conductor-cli
```

- cli の本番のコードは変えていない。cli の計画の表示のテスト(`plan_report.rs`)は、名前だけの `EnvNameEntry { name }` を組み立てる書き方に直し、期待の文字列は変えずに通る。
- runtime と converter は変えていない。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,153 通過)。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜5、`construction/code-generation/unit-test-instructions.md`。
