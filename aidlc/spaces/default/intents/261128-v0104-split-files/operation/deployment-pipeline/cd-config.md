# 配る仕組みの設定: v0.10.4

出典:
- `deployment-pipeline-questions.md`(Q1〜Q5。まとめは確かめ済み)
- `.github/workflows/release.yml`
- `construction/build-and-test/build-and-test-summary.md`
- team.md の Deployment
- project.md の Mandated と Forbidden

## 方針

- 配る仕組みは、今の `release.yml` をそのまま使う。v0.10.3 から変わっていない(最後に変えたのは 8cd1ba0)。
- 配る手順が始まるのは、タグ `v*` を push したときだけ(Q2)。

## 流れ

| 順 | ジョブ | 中身 | 止まる条件 |
|---|---|---|---|
| 1 | `verify-tag` | タグが `main` の上にあること、`crates/cli/Cargo.toml` の版(0.10.4)と一致することを確かめる | 一致しない。または `main` の上に無い |
| 2 | `checks` | `ci.yml` を再利用し、マージの前の関門と同じ検査をすべて通す | どれか 1 つでも落ちる |
| 3 | `build` | 4 対象をビルドし、`conductor` だけを tar.gz(Windows は zip)に入れる。`--version` で smoke test をする(Intel macOS を除く)。Linux では静的リンクも確かめる | ビルドか smoke test が失敗する |
| 4 | `draft-release` | `SHA256SUMS` を作り、下書きのリリースを作る | — |

## 成果物

- `conductor-0.10.4-<対象>.tar.gz`(Windows は `.zip`)と `SHA256SUMS`。
- テスト専用の補助バイナリは入れない(project.md の Forbidden)。

## v0.10.4 での確かめ

- 版は 0.10.4(3852f7a)。
- `main` の CI 46131106543 は、12 のジョブがすべて成功した。
