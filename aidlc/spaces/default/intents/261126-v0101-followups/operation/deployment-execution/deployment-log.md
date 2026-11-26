# 配る作業の記録: v0.10.1

出典: `deployment-execution-questions.md`(Q1 B、Q2 A、Q3 A、まとめの確かめ済み)、`operation/deployment-pipeline/cd-config.md`・`deployment-strategy.md`、GitHub Actions の実行(45913141134、45913389499)、`gh release view v0.10.1`。

| 順 | 時刻(UTC、2026-11-26 から 27) | 行ったこと | 結果 |
|---|---|---|---|
| 1 | — | 人が `main` を push(f5f08ce) | `main` と `origin/main` が f5f08ce で一致 |
| 2 | — | `main` の CI 45913141134 を確かめた | 12 のジョブがすべて成功(`lint`、`build-check` ×2、`test` ×3、`msrv-test`、`coverage`、`docs-check`、`startup-bench` ×3) |
| 3 | — | 人の頼み(Q1 B)で、AI が f5f08ce に軽量のタグ `v0.10.1` を付けて push(v0.10.0 と同じ形) | `* [new tag] v0.10.1 -> v0.10.1` |
| 4 | — | `release.yml` の実行 45913389499 を確かめた | `verify-tag`、`checks`(`ci.yml` の再利用)、`build` ×4、`draft-release` がすべて成功。対象のコミットは f5f08ce |
| 5 | — | 下書きのリリースの成果物を確かめた(`smoke-test-results.md`) | 4 つの成果物と `SHA256SUMS`。チェックサムがすべて一致 |
| 6 | — | 下書きの注記に、v0.10.1 の変更点を書き足した(Q2 A)。`gh release edit v0.10.1 --notes-file … --draft=true` | 下書きのまま(`isDraft: true`)。F2 のログの名前の変化の注意を含む |
| 7 | — | 公開 | **していない**。人が下書きを確かめてから公開する(Q3 A、project.md の Mandated) |

## 成果物(下書きのリリース `v0.10.1`)

| ファイル | 大きさ(バイト) |
|---|---|
| `conductor-0.10.1-aarch64-apple-darwin.tar.gz` | 616,978 |
| `conductor-0.10.1-x86_64-apple-darwin.tar.gz` | 662,306 |
| `conductor-0.10.1-x86_64-pc-windows-msvc.zip` | 599,929 |
| `conductor-0.10.1-x86_64-unknown-linux-musl.tar.gz` | 733,451 |
| `SHA256SUMS` | 447 |

- prerelease ではない(版に `-` を含まない)。

## 戻し

- 戻しは要らなかった。必要になったときは `operation/deployment-pipeline/rollback-runbook.md` に従う。
