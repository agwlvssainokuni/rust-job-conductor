# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし(macOS、本番の形、Windows の対象)、1,149 通過・0 失敗・1 無視、行カバレッジ 98.06%(下限 80%)、`cargo doc` の警告は基準の 5 件のまま。
- **古い説明**: core の `plan/mod.rs`・`types.rs` の `decide_before_run` を `decide_from_state` に、`lib.rs` の「runtime が再公開する」を実物に合わせた。cli の `CliCommand::new` の説明の不要な一文を消した。
- **仕様書**: rust 02 の `resolve_run_id` の型、`consistency` の一覧、非公開のモジュールの一覧を直し、3 段落の版の注記を「今の形」と「版の履歴」に分けた。rust 05・06・11 と、言語に依らない 11 章の根拠の欄 1 か所も直した。
- **テスト**: 重なる 3 件を消し(core 1、cli 2)、1 件を `plan_simulate_perf.rs` へ移して `plan_perf.rs` をなくした。`signals_order.rs` は残した。
- **ソースの検査**: cli の製品のコードで include の 3 つのマクロも止める。既知の限界(R-01 を含む)をコメントに書いた。
- **制約**: 新しい依存、`unsafe`、製品のコードへの `cfg(unix|windows)` の追加なし。凍結した仕様書は変えていない。版は 0.9.8 のまま。
- **残り**: Linux、Windows、MSRV のジョブは push の後の CI で確かめる。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜9 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
