# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,157 通過・0 失敗・1 無視、行カバレッジ 98.10%(下限 80%)。`conductor --version` は `conductor 0.9.7`。
- **段 1(型と構造)**: 計画・図・リトライの型を列挙型にし、読まれない項目と二重の項目をなくし、cli の準備の入口と表示の関数の形をまとめ、外から使われない `pub` を狭めた。利用者から見える動きは変わらない。
- **段 2(計算量)**: 計画のステートの複製をやめ、図と辺は表で引き、`.env` の重複キーは位置の表で扱った。性能のテスト 5 件を足し、直す前と後の時間を記録した(例: 計画 1,000 ジョブ 95 → 22 ms)。
- **段 3(利用者から見える変更)**: 変換の文法の誤りの理由が形式ごとの文になった。`.env` の読み込みは定義ファイルと同じ順になり、読む権限のない大きな `.env` は「読めない」と報告される。仕様書に「v0.9.6 までは…」の注記。版は 0.9.7。
- **テスト**: 既存のテストの変更は T1〜T26。期待の文字列の変更は ★ の 3 件だけ。足したテストは 11 件。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg`、runtime のパニックしうる書き方の追加なし。凍結した仕様書は変えていない。
- **残り**: Linux、Windows、MSRV 1.100 のジョブと、CI の機械での性能のテスト(特に余裕の小さい P3)は push の後の CI で確かめる。タグとリリースは頼まれたら行う。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜13 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
