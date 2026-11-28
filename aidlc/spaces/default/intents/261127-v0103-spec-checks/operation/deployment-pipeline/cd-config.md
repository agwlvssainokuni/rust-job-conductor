# 配る仕組みの設定: v0.10.3

出典:
- `deployment-pipeline-questions.md`(Q1〜Q5、まとめの確かめ済み)
- `.github/workflows/release.yml`
- `construction/build-and-test/build-and-test-summary.md`
- team.md の Deployment
- project.md の Mandated・Forbidden

## 方針

- 配る仕組みは、今の `release.yml` をそのまま使う。v0.10.2 から変わっていない(最後に変えたのは 8cd1ba0)。
- 起動するのは、タグ `v*` の push だけ(Q2)。

## 流れ

| 順 | ジョブ | 中身 | 止まる条件 |
|---|---|---|---|
| 1 | `verify-tag` | タグが `main` の上にあり、`crates/cli/Cargo.toml` の版(0.10.3)と一致するかを確かめる | 一致しない、または `main` の上に無い |
| 2 | `checks` | `ci.yml` を再利用し、マージの関門と同じ検査をすべて通す。v0.10.3 では `spec_numbers` も含む | どれか 1 つでも落ちる |
| 3 | `build` | 4 対象(`x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`)をビルドし、`conductor` だけを tar.gz(Windows は zip)に入れる。`--version` の smoke test を行う(Intel macOS を除く)。Linux は静的リンクも確かめる | ビルドか smoke test が失敗する |
| 4 | `draft-release` | `SHA256SUMS` を作り、下書きのリリースを作る(`--draft --generate-notes`) | — |

## 成果物

- `conductor-0.10.3-<対象>.tar.gz`(Windows は `.zip`)と `SHA256SUMS`。
- テスト専用の補助バイナリは入れない(project.md の Forbidden)。

## v0.10.3 での確かめ

- 版は 0.10.3(ad775bc)。`target/debug/conductor --version` は `conductor 0.10.3` を出す。
- `main` の CI 46092396041 では、12 のジョブがすべて成功した。`checks` も同じ検査なので、通る見込み。
