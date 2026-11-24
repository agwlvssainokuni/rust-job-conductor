# セキュリティのテストの手順(Security Test Instructions)

- 公開範囲: cli のライブラリが外に見せるのは、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目に要る型(`SignalError`)の 13 項目だけになった。
- ソースの検査(`nfr_source.rs`、D2): `src/tests/` の外の製品のコードは、今と同じく `unsafe`・`low_level`・ネットワーク・OS 分岐・`panic!(` の検査を受ける。`src/tests/` は OS 分岐と `panic!(` の検査だけを外し、`unsafe`・`low_level`・ネットワークの検査は受ける。`lib.rs` の末尾が `#[cfg(test)] mod tests;` であることと、`src/tests/` の外に `#[path` が無いことを確かめる。ユーザーは、規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」を製品のコードについての規則と読むことを認めた。
- 残る小さなリスク(確認担当の R-04): `#[path` の確かめは文字列の検索なので、`include!` などの別の書き方で `src/tests/` を読む道は止めない。末尾の宣言の確かめで、リスクは小さい。
- 誤りの表示に入力の値を含めない: 移した `report_messages` と、残る統合テストが期待を変えずに通る。
- cli は `#![forbid(unsafe_code)]` のまま。新しい依存なし(`Cargo.toml` と `Cargo.lock` の差分なし)。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
