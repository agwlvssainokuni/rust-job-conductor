# Deployment Log(v0.2.0)

## Sources

- Initial description: ジョブの command に、文字列に加えて、文字列の配列も書けるようにする。 [desc]
- Workflow-selected scope: express [scope]
- Build and Test の結果(`construction/build-and-test/test-results.md`)
- 質問: 「v0.2.0 をここでリリースしますか」→ いまリリースする [Q1]

## 実行の記録(2026-11-15)

| 順 | 操作 | 結果 |
|----|------|------|
| 1 | `main` を push(人が実施。HEAD は `cae1460` までの記録を含む) | 成功。CI の最新の実行は成功 |
| 2 | `git tag -a v0.2.0` を `main` の HEAD に付け、`git push origin v0.2.0` | タグが作られた |
| 3 | `release.yml` が起動(実行 43670011969) | 成功 |
| 3a | `verify-tag`: タグが `main` の履歴にあり、`crates/cli/Cargo.toml` の版(0.2.0)と一致 | 成功 |
| 3b | `checks`(`ci.yml` の全ジョブ: lint、build-check(Linux・Windows、Rust 1.85)、test(3 OS)、msrv-test、coverage、docs-check、startup-bench) | すべて成功 |
| 3c | `build`(4 ターゲット)とスモークテスト(展開して `--version`、Linux は静的リンクの確認) | すべて成功 |
| 3d | `draft-release`: `SHA256SUMS` を作り、**下書き(draft)**のリリースを作成 | 成功 |

- 下書きのアセット(5 つ): `conductor-0.2.0-aarch64-apple-darwin.tar.gz`、`conductor-0.2.0-x86_64-apple-darwin.tar.gz`、`conductor-0.2.0-x86_64-pc-windows-msvc.zip`、`conductor-0.2.0-x86_64-unknown-linux-musl.tar.gz`、`SHA256SUMS`。
- **公開は、まだしていない**(人が、下書きの内容を確認して公開する)。

## 新しい確認(この版で、初めて CI で実行されたもの)

- Windows のテスト `cmd_への配列の危険な引数は注入にならない`(`test (windows-2022)`)が成功した。Build and Test で `N/A`(CI に委ねる)としていた確認が、リリースの実行の中の CI で確かめられた。

## ロールバック

- 公開前(いま): 下書きのリリースを削除し、タグを削除する(`git push origin :refs/tags/v0.2.0` と、手元の `git tag -d v0.2.0`)。利用者には、見えていない。
- 公開後に問題が見つかった場合: GitHub のリリースを下書きに戻す、または削除し、`v0.2.1` で直したタグを付けて再リリースする(同じタグの付け直しはしない)。利用者は、直前の版(v0.1.0)のリリースから、実行ファイルを取り直す。配布物は状態を持たない(ステートファイルの形式は、この版で変わっていない)ので、データの巻き戻しは要らない。
- v0.1.x に戻す利用者への注意: 配列の `command` を使った定義は、v0.1.x では定義の誤りになる(README に書いてある)。

## Assumptions & Open Questions

None.
