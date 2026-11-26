# 配る仕組みの設定: v0.10.1

出典: `deployment-pipeline-questions.md`(Q1〜Q5、まとめの確かめ済み)、`.github/workflows/release.yml`、`construction/build-and-test/build-and-test-summary.md`、team.md の Deployment、project.md の Mandated・Forbidden。

## 方針

- 配る仕組みは今の `release.yml` のまま。v0.10.0 から変わっていない(`git diff v0.10.0..HEAD -- .github` が空)。v0.10.1 のために変えるところも無い。
- 起動はタグ `v*` の push だけ。`main` への push では配らない(Q2)。

## 流れ

| 順 | ジョブ | 中身 | 止まる条件 |
|---|---|---|---|
| 1 | `verify-tag` | タグが `main` の上にあり、`crates/cli/Cargo.toml` の版(0.10.1)と一致するか。版に `-` を含めば prerelease | 一致しない、`main` の上に無い |
| 2 | `checks` | `ci.yml` を再利用し、マージの関門と同じ検査をすべて通す | どれか 1 つでも落ちる |
| 3 | `build` | 4 対象(`x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`)。`conductor` だけを tar.gz(Windows は zip)に入れる。展開して `--version` の smoke test(Intel macOS を除く)。Linux は静的リンクも確かめる | ビルドか smoke test の失敗 |
| 4 | `draft-release` | `SHA256SUMS` を作り、下書きのリリースを作る(`--draft --generate-notes`、Intel macOS の注記つき) | — |

## 成果物

- `conductor-<対象>.tar.gz`(Windows は `.zip`)と `SHA256SUMS`。テスト専用の補助バイナリは入れない(project.md の Forbidden)。

## v0.10.1 での確かめ

- `crates/cli/Cargo.toml` の版は 0.10.1(152c4a1)。
- `main` の最新(3a0b491。その後は記録のコミットだけ)で、CI の 12 ジョブがすべて成功(45912018052)。`checks` は同じ検査なので通る見込み。最初の CI(45908344996)で落ちた `coverage` は、やり直し 1 で直した(`construction/build-and-test/test-results.md`)。
- `checks` の中の `coverage` は、性能のテスト(`plan_simulate_perf.rs`)を含む。v0.10.0 までは時々上限を超えたが、v0.10.1 の F3 の直しで 1,000 ジョブの計画が手元で約 24ms から約 3.5ms になった。
