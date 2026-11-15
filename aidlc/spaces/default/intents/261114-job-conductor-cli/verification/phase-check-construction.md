# Phase Check: Construction → Operation

**Verdict: PASS(条件つき)**。未解決の指摘はない。次の 2 点は、このワークフローの外で人が行う(下の「引き継ぎ」)。

## 確認した内容

- **すべての Unit が作られ、テストされた:** u1-core(115 件)、u2-runtime(152 件)、u3-cli(185 件)、u4-converter(112 件)。ワークスペース全体で 564 件成功、失敗 0(安定版 1.97 と、最小の Rust 1.85 の両方)。行カバレッジ 97.66%(下限 80%)。(`construction/build-and-test/test-results.md`)
- **Code Generation の未解決の指摘:** なし。各 Unit の adversarial review は、最後に READY で終わっている(u1-core 3 回、u2-runtime 1 回、u3-cli 1 回、u4-converter 2 回。u1-core は、Build and Test からの修復で、さらに 1 回 READY)。
- **Unit ごとの traceability:** Construction の各ステージの `traceability.json` に、欠けた ID も、無効な対象もない。`N/A` は、すべて理由つき。

| Unit | functional-design | nfr-requirements | nfr-design | infrastructure-design | code-generation |
|------|-------------------|------------------|------------|-----------------------|-----------------|
| u1-core | OK 36 / N/A 4 | OK 5 / N/A 1 | OK 7 | OK 10 / N/A 1 | OK 69 / N/A 5 |
| u2-runtime | OK 32 / N/A 6 | OK 5 / N/A 1 | OK 10 | OK 13 | OK 59 / N/A 7 |
| u3-cli | OK 29 / N/A 2 | OK 6 | OK 21 | OK 26 | OK 82 / N/A 2 |
| u4-converter | OK 3 | OK 5 / N/A 1 | OK 7 | OK 9 / N/A 2 | OK 22 |

- **Unit をまたぐ要件の確認(`cross-unit-traceability.md`):** すべての FR と、NFR1〜NFR4・NFR6 が対応済み。**NFR5(CI が行カバレッジ 80% 未満で失敗する)** は、Build and Test の時点で未対応だった。このステージで、`ci.yml` の `coverage` ジョブ(`--fail-under-lines 80`)が、これを担当する。ローカルの実測は 97.66%。**GitHub Actions 上での実行は、まだない**(初回の実行で確かめる)。
- **CI の関門が、Build and Test のコマンドを強制している:** `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace --locked`(3 OS)、Rust 1.85 のビルドとテスト、`cargo llvm-cov` の 80%(`quality-gates.md` の対応表)。

## 引き継ぎ(Operation 以降、またはこのワークフローの外へ)

1. **GitHub Actions の初回の実行:** ワークフローは、構文と、中のシェルの検査をローカルで確かめただけ。リポジトリに載せたあと、初回の実行で、ジョブが期待どおり動くことを確かめる(失敗したら、`ci.yml` を直す)。リポジトリの設定(必須のステータスチェック、`v*` タグの保護)は、手動で行う。
2. **Windows の Ctrl+C の手動確認:** 最初のリリースの公開の前に、`crates/u3-cli/tests/WINDOWS_CTRL_C_CHECKLIST.md` を、Windows の実機で実施する(Build and Test の BT-11。受け入れ済み)。
3. **既知の残り:** 上流の理由文が日本語のみ(英語のロケールで、種類は英語、理由は日本語。README に注記)、`escape_display` が 4 つの Unit に複製されている(共通のクレートへのまとめは将来の検討)、契約の文書(contract-summary)への差分の反映は、各 Unit の functional-spec の「契約への反映」の節にだけある、`startup-bench` の強制は 3 OS の実測のあと。
