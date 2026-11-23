# 結合のテストの手順(Integration Test Instructions)

新しい結合のテストの環境は無い。既存の流れのテスト(偽の `Runtime`/`Sink`/`Env`/`Signals` を注入)と実バイナリのテスト(`conductor-cli-test-helper`)を使う。

```bash
# core: 定義の読み取り、判断(decide_from_state)、graph、env の重ね合わせ
cargo test --locked -p conductor-core

# runtime: コマンドの実行(テストの補助は文字列から Command を作る形)、ステート、再公開をやめた後の use
cargo test --locked -p conductor-runtime

# converter: 誤りの種類(Error と Hash を消した後)
cargo test --locked -p conductor-converter

# cli: 引数(RunIdArg)、シグナル、エスケープの表、流れ
cargo test --locked -p conductor-cli
```

- runtime の core の再公開をやめた後も、cli と runtime のテストは `conductor_core` から同じ実体を使う。

結果: いずれも `cargo test --locked --workspace` の中で通った(1,153 通過)。CI(`6908726`)では ubuntu・macOS・Windows で通った。上流: `construction/code-generation/code-generation-plan.md` の Step 1〜9、`construction/code-generation/unit-test-instructions.md`。
