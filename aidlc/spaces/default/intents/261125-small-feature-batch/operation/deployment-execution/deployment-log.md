# 配布の記録: v0.10.0

出典: `deployment-execution-questions.md`(Q1〜Q3、まとめの確かめ済み)、`operation/deployment-pipeline/deployment-strategy.md`・`cd-config.md`・`rollback-runbook.md`、`construction/build-and-test/test-results.md`。日付は 2026-11-26(UTC)。

| 時刻(UTC) | 出来事 | 結果 |
|---|---|---|
| 00:15 | 人が `main`(5746167)を push。CI(45727407490) | 成功 |
| 00:17 | 人の指示で AI がタグ `v0.10.0` を 5746167 に付けて push | 成功 |
| 00:17〜 | `release.yml`(45727642798): `verify-tag` → `checks`(`ci.yml` の再利用)→ `build`(4 対象)→ `draft-release` | 全ジョブ成功 |
| — | 下書きのリリース `v0.10.0` ができた(draft、prerelease ではない) | 成功 |
| — | 人の指示で AI が下書きの注記に B1〜B7 と、v0.9.9 までのステートの注意を書き足した | 成功(下書きのまま) |
| 未 | 人が下書きを確かめて公開する | 人の作業 |

## 成果物

- `conductor-0.10.0-aarch64-apple-darwin.tar.gz`、`conductor-0.10.0-x86_64-apple-darwin.tar.gz`、`conductor-0.10.0-x86_64-pc-windows-msvc.zip`、`conductor-0.10.0-x86_64-unknown-linux-musl.tar.gz`、`SHA256SUMS`。

## 戻し方

- `rollback-runbook.md` のとおり。公開前なので、問題があれば下書きとタグを消して付け直せる。
