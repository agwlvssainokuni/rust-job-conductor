# セキュリティのテストの手順(Security Test Instructions)

- 誤りの表示に入力の値を含めない: 理由はすべて固定の文。`CommandErrorKind::reason()` の 2 つの文は前と同じ。converter の性質は、同じ名前の統合テスト(表示はどの種類でも入力の値を含まない)で確かめる。src の単体テストを消しても、統合テストのほうが秘密の値まで確かめる。
- パニックのフックは本番では固定の英語の文を出して 70 で終わる(変更なし)。フックを設定すると 70 で終わることは、自動のテストでは確かめられなくなった(ユーザーが Q3 で認めた)。本体(文と終了コード)は単体テストで確かめる。
- シグナルの監視の登録はプロセスで 1 回だけ。本番の登録は `main` の 1 か所で、動きは変わらない。
- runtime の製品のコードに、パニックしうる書き方(`unwrap`、`expect`、`panic!` など)を足していない: `nfr_source.rs`。cli の src の単体テストも `panic!(` を使わない。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に足していない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
