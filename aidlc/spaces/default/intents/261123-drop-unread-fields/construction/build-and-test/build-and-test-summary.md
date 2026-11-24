# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS と Windows の対象)、1,153 通過・0 失敗・1 無視、行カバレッジ 98.08%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。
- **削除**: core の型から、読まれない項目(`FlowEdge.explicit`、`EnvOrigin` と `origin`、`EnvFileContent.path`、`EnvEntry.line`)を消した。計画の `EnvNameEntry` は名前だけの構造体として残した(D1)。
- **仕様書**: rust と言語に依らない仕様書の 21 か所を参照実装に合わせた(Q2: A。`LoadedEnvFiles::add` の古い記述も消した D3)。
- **残したもの**: 定義の `format_version` と `description`(Q3: B)。cli・runtime・converter の本番のコードは無変更。
- **テスト**: 変更は T1〜T20、消した確かめの行は 13 行、名前の変更は ★ 2 件、消したテストと足したテストは無し。期待の文字列は変えていない。
- **確かめられなくなった点**: グラフの辺が「書いた遷移」か「記述順の次」かの区別(本番は読まない)。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg` の追加なし。凍結した仕様書は変えていない。版は 0.9.8 のまま。
- **残り**: Linux、Windows、MSRV のジョブは push の後の CI で確かめる。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜7 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
