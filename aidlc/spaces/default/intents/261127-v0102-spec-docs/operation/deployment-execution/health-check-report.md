# 健全性の報告: v0.10.2

出典: `deployment-log.md`、`smoke-test-results.md`、`construction/build-and-test/build-and-test-summary.md`、CI 46080572120、`release.yml` の実行 46080776462、`deployment-execution-questions.md`(まとめの確かめ済み)、team.md の Deployment。

## 対象の性質

- conductor は利用者の手元で動く単一バイナリの CLI。稼働の監視の指標は無い。健全性は、配る前の関門と成果物の確かめで見る。v0.10.2 は動きを変えていない。

## 健全性の目安

| 目安 | 期待 | 実際 | 判定 |
|---|---|---|---|
| 版の一致 | タグ `v0.10.2` が `main` の上にあり、版 0.10.2 と一致 | `verify-tag` 成功、対象 98137b9 | 良 |
| マージの関門と同じ検査 | `checks` がすべて成功 | 成功(`main` の CI 46080572120 も 12 のジョブがすべて成功) | 良 |
| 成果物の完全さ | 4 つの成果物と `SHA256SUMS`、チェックサムの一致 | すべて一致 | 良 |
| 起動 | `--version` が `conductor 0.10.2` | 3 対象(ジョブ)と手元の Apple Silicon で確かめた | 良 |
| 公開の状態 | 下書きのまま | `isDraft: true` | 良 |

## 公開の後に見ること

- 利用者からの報告は GitHub の Issues。仕様書の誤りの報告が来たら、次の整理の intent の材料にする。

## 残っている人の作業

- 下書きのリリース `v0.10.2` を確かめて公開する。
