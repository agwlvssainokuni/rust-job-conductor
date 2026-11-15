# CI Pipeline — 質問

すでに決まっていること(チームの実践と、u1〜u4 の Infrastructure Design): CI は GitHub Actions、統合ブランチは `main`(squash しない)、フォーマットは `cargo fmt`、リンタは `cargo clippy`(失敗はマージをブロック)、行カバレッジ 80% の下限、Linux・macOS・Windows でテストを CI で実行、タグを付けると 4 つのターゲット(Linux x86_64 musl、macOS aarch64、macOS x86_64、Windows x86_64 MSVC)のリリースを作り、GitHub Releases に `.tar.gz`(Windows は `.zip`)と SHA-256 を添付する。ここでは、まだ決まっていない 3 点を確認します。

## Q1. [品質の関門] マージ前に必ず通す CI のジョブ

- A. 次の 5 つをすべて、マージのブロックにする: (1) `cargo fmt --check`、(2) `cargo clippy -D warnings`、(3) Linux・macOS・Windows でのテスト(`cargo test --workspace --locked`)、(4) 最小の Rust 1.85 でのビルドとテスト、(5) Linux で `cargo llvm-cov` の行カバレッジが 80% 未満なら失敗(要件 NFR5)(推奨)
- B. A の (1)〜(3) と (5) だけ。最小の Rust 1.85 のビルドは、ブロックにしない(警告のみ)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [依存の検査] 依存の脆弱性や、禁止するライセンスの検査

- A. 検査しない。要件にない(いまは、依存が少ない)。必要になったら足す
- B. `cargo audit`(既知の脆弱性)を、プルリクエストと、週に 1 回の定期実行で行い、結果は警告として出す(マージはブロックしない)(推奨)
- C. `cargo deny`(脆弱性・ライセンス・重複)を、マージのブロックにする
- X. Other (please specify)

[Answer]: B

## Q3. [リリース] タグの付け方と、リリースの前の確認

- A. `v` で始まるタグ(例 `v0.1.0`)を付けると、リリースのワークフローが動く。タグの版が、`Cargo.toml` の版と一致しなければ失敗。リリースのビルドの前に、同じ品質の関門(Q1 の A)を通す。GitHub Releases の作成は、下書き(draft)として作り、人が確認して公開する(推奨)
- B. A と同じだが、下書きにせず、自動で公開する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
