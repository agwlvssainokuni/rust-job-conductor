# Construction → Operation の確かめ: v0.10.0

出典: `construction/build-and-test/cross-unit-traceability.md`、`build-and-test-summary.md`、`test-results.md`、各ユニットの `code-generation/traceability.json`、`construction/ci-pipeline/ci-config.md`・`quality-gates.md`。

## 判定: 合格

| 確かめること | 結果 |
|---|---|
| すべてのユニットを作り、テストした | u1〜u7 のコード生成、レビュー(READY)、確認のコマンド(行カバレッジ 97.75〜98.18%)と承認が済んでいる |
| コード生成の対応表に未解決が無い | 各ユニットの `traceability.json` は OK か Deferred(NFR2.3 の全体のカバレッジ、NFR3.1 の CI)だけ。Deferred は Build and Test(T-COV)と CI(45726504409)で満たした |
| ユニットをまたぐ FR・NFR の関門 | FR1.1〜FR8.2 はすべて作られている。NFR2・NFR7 は Build and Test で、NFR3 は CI で満たした(`cross-unit-traceability.md` の未対応だった 3 つ) |
| CI の関門が Build and Test のコマンドを効かせる | `quality-gates.md` の対応表のとおり。最新の CI は 13 ジョブすべて成功 |

## 残っていること

- タグ `v0.10.0` とリリースの下書きの公開は、人が行う(project.md の Mandated)。
- レビューで残った Minor(README の「止まった」の注意の言い方など)は、ふるまいを変えない改善として残る。
