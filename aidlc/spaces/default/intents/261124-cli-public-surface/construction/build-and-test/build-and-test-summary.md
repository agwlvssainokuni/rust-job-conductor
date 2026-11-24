# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS、本番の形の `--lib --bins`、Windows の対象)、1,152 通過・0 失敗・1 無視、行カバレッジ 98.06%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。
- **公開範囲**: cli のライブラリの公開を、`main` が使う 4 つ、依存の注入の継ぎ目、継ぎ目に要る型(`SignalError`)の 13 項目だけにした。`parse_args`、`types` の 11 個、`exit`・`report`・`run_id` を crate の中に狭めた。本番の処理は変えていない。
- **テスト**: 狭めた項目を直接呼ぶ 162 件を `crates/cli/src/tests/` の単体テストへ移した(名前・確かめの行・期待の値は不変)。`flow_init.rs` は期待のひな形を原本の写しで作る形に直した。消したテストと足したテストは無い。
- **ソースの検査(D2)**: `src/tests/` を OS 分岐と `panic!(` の検査の外にした。`unsafe`・ネットワークの検査は残し、`lib.rs` の末尾の宣言と `#[path` の不在を確かめる。ユーザーは、規則「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」を製品のコードについての規則と読むことを認めた(規則の文そのものは変えていない)。
- **仕様書**: rust の 02・07・08・11 章を直した。言語に依らない仕様書は変えていない。
- **制約**: 新しい依存、`unsafe`、製品のコードへの `cfg(unix|windows)` の追加なし。core・runtime・converter は無変更。版は 0.9.8 のまま。
- **残り**: Linux、Windows、MSRV のジョブは push の後の CI で確かめる。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜10 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
