# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,142 通過・0 失敗・1 無視、行カバレッジ 98.23%(下限 80%)。`conductor --version` は `conductor 0.9.6`。
- **段 1・2**: 利用者から見える動きは変わらない。起動の結果は `ExecutionResult` の 3 つの場合になり、`run_job` は分けたコマンドを受け取ってリトライのたびに分け直さない。
- **段 3(利用者から見える変更)**: 定義ファイルは「種別を見る → 開く → 開いたものから大きさ」の順(形 B)。読む権限のない大きなファイルは「読めない」になる(N3 は直す前に落ち、直した後に通った)。ほかの場合の種類は変わらず、名前付きパイプでも止まらない。仕様書に「v0.9.5 までは…」の注記。
- **テスト**: 既存のテストの変更は T1〜T20(T19・T20 は実装中に見つかった一覧の漏れで、ユーザーが追加を認めた)。期待する出力の文字列の変更は版の 1 行だけ。足したテストは N1〜N5 の 5 件。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg` の追加なし。converter は版の行だけ。
- **残り**: Linux、Windows、MSRV 1.100 のジョブは push の後の CI で確かめる。タグとリリースは、頼まれたら行う。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜12 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標 SM1〜SM5 の確かめ方に従った)、`construction/code-generation/code-summary.md`。
