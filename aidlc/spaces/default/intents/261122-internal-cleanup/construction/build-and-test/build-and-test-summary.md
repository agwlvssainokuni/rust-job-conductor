# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,117 通過・0 失敗・1 無視、行カバレッジ 98.19%(下限 80%)。
- **動き**: 利用者から見える動きは変わっていない。段 1〜3 は既存のテストを変えずに件数 1,123 のまま通り、変えたテストは承認済みの `report_messages.rs` の組み立ての値だけ。
- **件数の減少**: 6 件。段 4a で重複をまとめた分(`code-summary.md` に一覧)。
- **制約**: 新しい依存、`unsafe`、`platform` の外の `cfg` の追加なし。
- **残り**: Windows と MSRV 1.100 は push の後の CI で確かめる。凍結した 5 つの仕様書の 11 章は、converter の旧いテスト名を参照したまま(凍結の方針)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。
