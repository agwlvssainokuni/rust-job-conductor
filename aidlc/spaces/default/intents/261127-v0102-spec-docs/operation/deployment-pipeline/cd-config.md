# 配る仕組みの設定: v0.10.2

出典: `deployment-pipeline-questions.md`(Q1〜Q5、まとめの確かめ済み)、`.github/workflows/release.yml`、`construction/build-and-test/build-and-test-summary.md`、team.md の Deployment、project.md の Mandated・Forbidden。

## 方針

- 配る仕組みは今の `release.yml` のまま。v0.10.1 から変わっていない(`git diff v0.10.1..HEAD -- .github` が空)。
- 起動はタグ `v*` の push だけ(Q2)。

## 流れ

| 順 | ジョブ | 中身 | 止まる条件 |
|---|---|---|---|
| 1 | `verify-tag` | タグが `main` の上にあり、`crates/cli/Cargo.toml` の版(0.10.2)と一致するか | 一致しない、`main` の上に無い |
| 2 | `checks` | `ci.yml` を再利用し、マージの関門と同じ検査をすべて通す | どれか 1 つでも落ちる |
| 3 | `build` | 4 対象(`x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`)。`conductor` だけを tar.gz(Windows は zip)に入れる。`--version` の smoke test(Intel macOS を除く)。Linux は静的リンクも確かめる | ビルドか smoke test の失敗 |
| 4 | `draft-release` | `SHA256SUMS` を作り、下書きのリリースを作る(`--draft --generate-notes`) | — |

## 成果物

- `conductor-0.10.2-<対象>.tar.gz`(Windows は `.zip`)と `SHA256SUMS`。テスト専用の補助バイナリは入れない(project.md の Forbidden)。

## v0.10.2 での確かめ

- 版は 0.10.2(10ce84e)。`target/debug/conductor --version` が `conductor 0.10.2`。
- `main` の CI 46079873817 の 12 のジョブがすべて成功。`checks` は同じ検査なので通る見込み。
