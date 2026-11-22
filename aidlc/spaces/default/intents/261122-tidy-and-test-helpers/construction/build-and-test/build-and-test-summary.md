# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,146 通過・0 失敗・1 無視、行カバレッジ 98.14%(下限 80%)。
- **動き**: 利用者から見える変更なし。文言の定数をまとめても、英語・日本語の文字列は 1 文字も変わらない(確認担当が確認)。版は上げない。
- **本体の整理**: 重複・到達しない分岐・文字列での比較・説明のずれを、設計の検索で 0 件か定義の 1 か所にした。`program_name` を消した。
- **テストの補助**: 写しを共通の場所に寄せ、`JobExecution` の手組みをなくした。cli の偽物は渡されたコマンドを記録し、cli の層で「分け直さない」ことを確かめられるようになった(N2、N3)。
- **テスト**: 既存のテストの変更は T1〜T46。テスト名の差は、T38 の削除、N1〜N5 の追加、置き場所の変わった 8 件だけ。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg` の追加なし。凍結した仕様書は変えていない。
- **残り**: Linux、Windows、MSRV 1.100 のジョブは push の後の CI で確かめる。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜10 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
