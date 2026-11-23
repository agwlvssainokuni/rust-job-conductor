# ビルドとテストのまとめ(Build and Test Summary)

- **結果**: すべて合格。fmt 差分なし、clippy 警告なし、1,157 通過・0 失敗・1 無視、行カバレッジ 98.12%(下限 80%)。CI(`6df843d`)は 13 のジョブがすべて成功(3 つの OS のテスト、MSRV を含む)。
- **段 1(テストの都合で曲げられた所)**: テストのためだけの関数・項目・引数・公開範囲・差し替え口を本番から消した(言語と色の確認用のメソッド、`reset --all` の数、`is_lane_drawable`、`LoadedEnvFiles::add`、`remove_stale_temps` の `now`、`code()`、`ALL` 5 つ、パニックのフックの差し替え口、シグナルの「登録済み」と印、`node_state_of`・`flow_edges` の公開)。消した名前の参照は 0 件。
- **段 2(ふつうの整理)**: 古い説明、ステートの 2 項目(バイト列は前後で同じ)、`CommandError`、`ExitCodeClass`、小さい整理、判断の重複を直した。
- **テスト**: 変更は T1〜T31、消したのは 5 件、足したのは N1。名前の変更は ★3 件だけ。期待の文字列は変えていない。パニックのフックを設定すると 70 で終わることは、自動のテストでは確かめられなくなった(Q3)。
- **ユーザーの決定で足したこと**: 言語に依らない仕様書の「すでに登録済み」を「登録はプロセスで 1 回だけ」に直した。runtime のテストのコメント 1 行を直した。
- **制約**: 新しい依存、`unsafe`、`src/` の `cfg`、runtime のパニックしうる書き方の追加なし。凍結した仕様書は変えていない。版は 0.9.8 のまま。
- **残り**: 申し送りのコミット `8f07ec2` 以降は未 push。タグとリリースはこの intent では行わない(版を上げないため)。

詳しくは `test-results.md`、`cross-unit-traceability.md`。上流: `construction/code-generation/code-generation-plan.md`(Step 1〜14 はすべて完了)、`construction/code-generation/unit-test-instructions.md`(実行のコマンドと指標の確かめ方に従った)、`construction/code-generation/code-summary.md`。
