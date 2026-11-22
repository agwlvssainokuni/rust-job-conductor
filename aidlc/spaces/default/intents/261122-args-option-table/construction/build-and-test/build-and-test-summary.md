# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,128 通過・0 失敗・1 無視、行カバレッジ 98.18%(下限 80%)。`args.rs` は 98.11%。
- **動き**: 利用者から見える変更は `convert --to` の誤りの対象だけ(値 → `--to`)。誤りの報告の順は、表化の前に足したテストで固定し、表化の後も通った。
- **テスト**: 既存のテストは無変更。足したのは報告の順のテスト 9 件と `--to` のテスト 2 件。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg` の追加なし。公開の型は不変。
- **残り**: Windows と MSRV 1.100 は push の後の CI で確かめる。`--to` の変更は版の更新の候補。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜10 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標 SM1〜SM5 の確かめ方に従った)、`construction/code-generation/code-summary.md`。
