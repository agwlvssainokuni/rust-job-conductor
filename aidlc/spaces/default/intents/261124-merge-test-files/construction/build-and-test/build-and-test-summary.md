# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS、本番の形、Windows の対象)、1,149 通過・0 失敗・1 無視、行カバレッジ 98.06%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。
- **統合**: テストのファイル 19 を減らした。cli の README のテスト 10 ファイルを `src/tests/docs.rs` の 1 つに、converter・cli・runtime の小さなファイル 9 つをまとめ先へ移した。
- **写しの解消**: cli のテストの補助を `tests/common/fakes.rs` の 1 つに、init のひな形の原本を `tests/fixtures/init_{ja,en}.toml` の 1 組にした。改行を LF に固定する `.gitattributes` を足した。
- **テスト**: 名前・確かめの行・期待の値は変えていない(名前の差 0)。消したテスト・足したテストは無い。
- **仕様書**: rust と言語に依らない 11 章のテストのファイルの表と根拠の欄を直した。rust 12 の 142 行目もユーザーの指示で直した。
- **制約**: 本番のコード、依存、CI、README は変えていない。凍結した仕様書も変えていない。版は 0.9.8 のまま。
- **残り**: Windows の CI でひな形のテストが通ること(`.gitattributes` が効くこと)を、push の後に確かめる。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜8 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
