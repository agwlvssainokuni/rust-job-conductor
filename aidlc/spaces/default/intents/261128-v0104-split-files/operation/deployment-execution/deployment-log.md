# 配る作業の記録: v0.10.4

出典:
- `deployment-execution-questions.md`(Q1 は B、Q2 は A、Q3 は A。まとめは確かめ済み)
- `operation/deployment-pipeline/` の `deployment-strategy.md`、`cd-config.md`、`rollback-runbook.md`

## 手順と結果(2026-11-28)

| 順 | 行ったこと | 誰が | 結果 |
|---|---|---|---|
| 1 | `main` を 9983b28 まで push した | 人 | `origin/main` と一致した |
| 2 | `main` の CI 46132075162 が通ったことを確かめた | AI | 成功 |
| 3 | 9983b28 に軽量のタグ `v0.10.4` を付けて push した | AI(人の頼み。Q1 B) | 成功 |
| 4 | `release.yml` の run 46132328751 を見届けた | AI | すべてのジョブが成功した(`verify-tag`、`checks` の 12、`build` の 4、`draft-release`) |
| 5 | 下書きの成果物を確かめた | AI | 4 つのアーカイブと `SHA256SUMS` がそろっていた。下書きのままで、prerelease ではない |
| 6 | 下書きの注記に変更点を書き足した(`gh release edit --notes-file … --draft=true`) | AI(Q2 A) | 下書きのまま |
| 7 | 公開する | 人(Q3 A) | 人が下書きを確かめてから行う |

## 戻し方

- `rollback-runbook.md` のとおり。
- 公開の前なら、下書きとタグを消して付け直す。
- 公開の後なら、v0.10.3 のリリースに戻せる。
