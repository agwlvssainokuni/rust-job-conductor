# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS、本番の形の `--lib`、Windows の対象)、1,152 通過・0 失敗・1 無視、行カバレッジ 98.09%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。
- **削除**: converter の `#[doc(hidden)] pub mod testing` と `TargetFormat::opposite` を本番から消した。`Value` の `PartialEq` は `#[cfg(test)]` の中だけにした。
- **テストの移動**: 途中の表現を比べるテストなど 45 件を `crates/converter/src/tests/` の単体テストへ移した(名前、確かめの行、期待の値は変えない)。消したのは `opposite` の単体テスト 1 件だけ。
- **仕様書**: rust の 02・09・11 章を直した。言語に依らない仕様書は変えていない(D4)。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg(unix|windows)` の追加なし。core、runtime、cli は無変更。凍結した仕様書は変えていない。版は 0.9.8 のまま。
- **残り**: Linux、Windows、MSRV のジョブは push の後の CI で確かめる。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜8 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
