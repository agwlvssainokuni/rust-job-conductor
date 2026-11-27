# 配る作業の記録: v0.10.2

出典: `deployment-execution-questions.md`(Q1 B、Q2 A、Q3 A、まとめの確かめ済み)、`operation/deployment-pipeline/cd-config.md`・`deployment-strategy.md`、GitHub Actions の実行(46080572120、46080776462)、`gh release view v0.10.2`。

| 順 | 行ったこと | 結果 |
|---|---|---|
| 1 | 人が `main` を push(98137b9) | `main` と `origin/main` が 98137b9 で一致 |
| 2 | `main` の CI 46080572120 を確かめた | 12 のジョブがすべて成功 |
| 3 | 人の頼み(Q1 B)で、AI が 98137b9 に軽量のタグ `v0.10.2` を付けて push(v0.10.1 と同じ形) | `* [new tag] v0.10.2 -> v0.10.2` |
| 4 | `release.yml` の実行 46080776462 を確かめた | `verify-tag`、`checks`、`build` ×4、`draft-release` がすべて成功。対象のコミットは 98137b9 |
| 5 | 下書きのリリースの成果物を確かめた(`smoke-test-results.md`) | 4 つの成果物と `SHA256SUMS`。チェックサムがすべて一致 |
| 6 | 下書きの注記に、v0.10.2 の変更点を書き足した(Q2 A)。`gh release edit v0.10.2 --notes-file … --draft=true` | 下書きのまま(`isDraft: true`)。「動きは v0.10.1 と同じ」を含む |
| 7 | 公開 | **していない**。人が下書きを確かめてから公開する(Q3 A) |

## 成果物(下書きのリリース `v0.10.2`)

| ファイル | 大きさ(バイト) |
|---|---|
| `conductor-0.10.2-aarch64-apple-darwin.tar.gz` | 616,904 |
| `conductor-0.10.2-x86_64-apple-darwin.tar.gz` | 662,138 |
| `conductor-0.10.2-x86_64-pc-windows-msvc.zip` | 600,127 |
| `conductor-0.10.2-x86_64-unknown-linux-musl.tar.gz` | 732,947 |
| `SHA256SUMS` | 447 |

- prerelease ではない。戻しは要らなかった(必要なときは `operation/deployment-pipeline/rollback-runbook.md`)。
