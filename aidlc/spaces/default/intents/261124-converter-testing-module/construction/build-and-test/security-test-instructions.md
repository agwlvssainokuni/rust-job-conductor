# セキュリティのテストの手順(Security Test Instructions)

- テストのための公開をなくした: converter の `#[doc(hidden)] pub mod testing` と `TargetFormat::opposite` を消した。`Value` の `PartialEq` は `#[cfg(test)]` の中だけにあり、本番の成果物に入らない(`cargo clippy -p conductor-converter --lib` で確かめた)。
- 誤りの表示に入力の値を含めない: 統合テスト `nfr_security.rs` と、移した単体テスト(`誤りの文は入力の値を含まない` など)が期待を変えずに通る。
- ランダムな入力でパニックしない、成功したランダムな構造は往復で等しい: `src/tests/nfr_security.rs` へ移した 2 件が通る。
- ソースの検査(`tests/nfr_security.rs`。src の直下のファイルの製品のコードに、プロセス・ネットワーク・環境変数・ファイル書き込みなどの語が無いこと)は、今も製品のコードの全体を見る。`src/tests/` と `PartialEq` の本体はテストのときだけのコードで検査の外になるが、禁止の語が無いことを検索で確かめた(BR3.2)。
- 新しい依存なし: `Cargo.toml` と `Cargo.lock` の差分なし。converter は core に依存しない。`unsafe` と src の `cfg(unix|windows)` の追加なし。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
