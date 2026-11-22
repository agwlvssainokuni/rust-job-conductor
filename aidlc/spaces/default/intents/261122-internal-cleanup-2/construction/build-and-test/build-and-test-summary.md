# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,137 通過・0 失敗・1 無視、行カバレッジ 98.28%(下限 80%)。
- **動き**: 利用者から見える変更なし。誤りの順序と表示、graph の行き先(`escape_path`)、`.env` の並び、終了コードは元と同じ(確認担当が確認)。
- **計算量**: `@end` の列挙と `.env` の合成が線形になった。性能のテスト 3 件が上限の約 1/10(A2 5.3 ms、A3a 5.1 ms、A3b 8.9 ms)。
- **テスト**: 既存のテストの変更は承認済みの T1〜T9 だけで、期待する出力の文字列は不変。足したテストは 9 件。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg` の追加なし。core と converter に `forbid(unsafe_code)`。
- **見送り**: `.env` の解析の重複の除去(任意。上限の入力でも 40.6 ms)。
- **残り**: Linux、Windows、MSRV 1.100 のジョブは push の後の CI で確かめる。版の更新はしない(動きを変えないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜11 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標 SM1〜SM6 の確かめ方に従った)、`construction/code-generation/code-summary.md`。
