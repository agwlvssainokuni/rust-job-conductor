# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS と Windows の対象)、1,153 通過・0 失敗・1 無視、行カバレッジ 98.08%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。CI(`6908726`)は 13 のジョブがすべて成功(3 つの OS のテスト、MSRV を含む)。
- **削除**: テストだけが使う本番のコード(`Command` の `From`、`RunIdArg::as_str`、`InterruptSignal` の `is_raised`・`PartialEq`・`Eq`、`Default` 4 つ、`sanitize`、`EnvFileError::reason()`)を消した。`InterruptSignal` の `Default` は clippy の慣例のため残した(D1)。
- **公開範囲**: core の `decide_before_run`・`classify`・定数と関数 4 つ、cli の再公開と `pub mod` 4 つを、使う範囲に狭めた。
- **回答で決めたもの**: `RunIdError` を 1 つに、runtime の core の再公開をやめた、誤りの種類の `Display`・`Error` と使われない `Hash` を消した。誤りの型の `Display` は残した。
- **テスト**: 変更は T1〜T32、消したのは 4 件、足したものは無し。期待の文字列とテスト名は変えていない。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg`、runtime のパニックしうる書き方の追加なし。凍結した仕様書と language-agnostic は変えていない。版は 0.9.8 のまま。
- **残り**: 仕様書 rust 02 の 42 行目で、前からある版の注記と今回の変更が同じ行に混ざる(確認担当の R-08、任意の整理)。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜11 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
