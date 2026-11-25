# セキュリティの設計: u4-graph-dot

出典: セキュリティの要件(`security-requirements.md` の NFR6.1、NFR6.2、NFR5.1、NFR4.1)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 3、5)、契約(`contract-summary.md` の C5)。

## 1. 注入を防ぐ escape(NFR6.1)

- DOT に書く文字列はラベルの値だけで、すべて二重引用符で囲む。節の ID(`n<i>`、`nend`)、属性の名前と値(`shape=box` など)、グラフの名前(`conductor`)は固定の文字列か数字で、入力を含まない。
- ラベルの escape は 1 つの関数 `escape_dot_label` にまとめ、ジョブ ID と種類の語の両方に通す。順序は `escape_full`(見える形にする)→ `\` を `\\` → `"` を `\"`。
- 確かめ方: 次の入力と出力の対の表を単体テストにする(`crates/cli/src/tests/graph_dot.rs`)。

| 入力 | 出力(二重引用符の中) |
|---|---|
| `a` | `a` |
| `a"b` | `a\"b` |
| `a\b` | `a\\b` |
| `a\nb`(本物の改行) | `a\\nb`(DOT では `\n` という文字として見える) |
| `a\lb`(バックスラッシュと l) | `a\\lb` |
| タブ | `\\t` |
| U+202E(書式の制御) | `\\u{202e}`(`escape_full` の見える形の `\` を二重に) |
| `日本語` | `日本語` |

- 加えて、`"`・`\`・改行を含むジョブ ID の定義を描き、出力の行の数が「節と辺の数 + 3」で、各行の二重引用符の数が偶数になることを確かめる(文書の外に出ていない)。

## 2. 入力の値を書かない(NFR6.2)

- 描画の関数は `FlowGraph` だけを受け取り、定義(コマンド、環境変数)とパイプライン名を受け取らない。`-v` の値も受け取らない(呼び出し側で無視する)。
- 確かめ方: 目印の文字列をコマンドとパイプライン名に入れた定義で、`--format dot` と `--format dot -v` の出力に目印が出ないことを、実バイナリのテスト(`binary_graph.rs`)で確かめる。

## 3. 依存とコードの規則(NFR5.1、NFR4.1)

- 変更は cli の `report/graph_dot.rs`(新規)、`report/mod.rs`、`args.rs`、`aux_commands.rs`、`types.rs`、`report/messages.rs` に閉じる。新しい依存、`unsafe`、`cfg` を書かない。`nfr_source.rs`(cli)と CI の `build-check` が確かめる。
