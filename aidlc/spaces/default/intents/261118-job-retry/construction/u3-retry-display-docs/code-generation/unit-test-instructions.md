# Unit Test Instructions — u3-retry-display-docs

## テストの方針

- 方法: test-after(各層を実装してから、その層のテストを書いて実行する)。テスト戦略は Standard、スコープは feature(行カバレッジ 80% の下限と、CI 実行を加える)
- 新しいテストは、`crates/cli/tests/docs_retry.rs`(約 4 件)。文言を直した場合は、u2 が足した `report_messages.rs` のテストを、直した文言に合わせて更新する(既存のアサーションは変えない)。既存のテストは、アサーションを変えずに通す
- この Unit の中心は文書(README)のため、テストは、README の内容と、README の例が実際に読めることを確かめる

## 実行環境

- Rust 1.100.0 以降(`rustc --version`)

## この Unit に絞った実行コマンド

| 目的 | コマンド |
|---|---|
| 変更前の基準(全体) | `cargo test --workspace --locked` |
| README のテスト(新規) | `cargo test -p conductor-cli --test docs_retry --locked` |
| cli の文言(文言を直した場合) | `cargo test -p conductor-cli --test report_messages --locked` |
| MSRV の記載の一致 | `cargo test -p conductor-cli --test msrv_consistency --locked` |
| README の必須の語(CI の `docs-check` と同じ) | `grep -c -e 'SHA-256' README.md` ほか(`SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert` の 7 語を、それぞれ 1 回以上含むこと) |
| 書式 | `cargo fmt --all -- --check` |
| 静的解析(CI の `lint` と同じ) | `cargo clippy --workspace --all-targets --locked -- -D warnings` |

## カバレッジ

- 下限は行カバレッジ 80%(チームの方針と CI)。この下限は変えない。この Unit はソースをほとんど変えないため、全体の値はほぼ変わらない(Build and Test と、Unit の完了の確認が測る)

## モックとテストデータ

- モックは使わない。README のファイルを、そのまま読む。README の例の定義は、テストの中で取り出して、一時ファイル(`tempfile`、既存の dev-dependency)に書き、`conductor-core` の読み取りに通す

## OS ごとの扱い

- ローカル(macOS)で実行する。Linux と Windows は、CI で実行する(マージ前に CI を通す)。改行コードの違い(CRLF)で README の検査が壊れないようにする
