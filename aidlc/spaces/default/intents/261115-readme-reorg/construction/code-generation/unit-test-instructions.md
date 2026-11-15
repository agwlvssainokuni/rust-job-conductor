# Unit Test Instructions(readme-reorg)

## 実行の環境

- 文書(`README.md` と `demo/README.md`)だけの変更。テストコードは書かない(変更を 2 つの Markdown に限るため)。確認は、コマンドで行う。
- ツール: `cargo`(既存のテストと、ソースからのビルドの手順の確認)、`grep`、`git`、Python 3(目次のアンカーの確認)。

## この作業の確認の実行

このスコープは、Unit に分けない(zero-Unit)。次を実行して、結果を記録する。

```bash
# (a) CI の docs-check と同じ検索(.github/workflows/ci.yml の docs-check と同じ語句)
for pat in 'SHA-256' 'Gatekeeper' '終了コードの表' 'ステートファイルの場所' 'at-least-once' 'シグナル' 'convert'; do
  grep -q -- "$pat" README.md || echo "MISSING $pat"
done

# (b) 目次と、README 内のリンクの確認(GitHub のアンカーの規則で、見出しの slug を作って突き合わせる)
#     README.md の `](#...)` のリンクが、実在する見出しの slug に対応すること。`demo/README.md`、`LICENSE` へのリンクが実在すること

# (c) run ID の規則が 1 か所にだけあること(値の規則・優先順位・`--run-id ""` の固定の記述を、README の中で検索して数える)

# (d) 整理前の README と、整理後の README で、規則・終了コード・表示の語句が残っていること(git show HEAD:README.md との突き合わせ)

# (e) 変更の範囲
git diff --stat      # README.md と demo/README.md と、この intent の記録だけ

# (f) 既存の検査(コードは変えないので、変わらないことの確認)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked                       # 666 件
```

## 手順の確認(README に書いた手順を、実際に実行する)

- クイックスタート: `target/` の下の一時ディレクトリで、書いた定義を作り、`cargo build --release --locked --bin conductor` で作ったバイナリで、手順を順に実行し、終了コードと出力が、書いたとおりであること。
- ソースからのビルド: `cargo build --release --locked --bin conductor`、`cargo test --workspace --locked`、`cargo install --path crates/cli --locked --root <target/ の下の一時ディレクトリ>`(できた `conductor --version` を確認)が、成功すること。`git clone` の URL は、実在すること(`git ls-remote` で確認)。
- インストールの展開の手順: リリースのアーカイブ(`gh release download v0.6.0 -p '*aarch64-apple-darwin*'` を、`target/` の下の一時ディレクトリへ)を、README の手順どおりに展開して、`conductor` を取り出せること。

## カバレッジ

- 文書の変更で、`conductor` のコードは変わらないので、行カバレッジは変わらない。下限 80% は、Build and Test が、全体で確かめる。

## モックとスタブ

- 使わない。実際のバイナリ・実際のリリースを使う。

## テストデータ

- クイックスタートの定義は、README に書くものそのもの。一時ディレクトリは、`target/`(git の対象外)の下に作る。
