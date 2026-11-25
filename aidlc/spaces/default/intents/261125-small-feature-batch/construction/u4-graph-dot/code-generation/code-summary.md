# コードのまとめ: u4-graph-dot

## 変えたファイル

| ファイル | 変更 |
|---|---|
| `crates/cli/src/report/graph_dot.rs`(新規) | `render_dot`(mermaid と同じ作り。`HashMap` で節の ID を引き、節と辺を 1 回ずつ走査)と `escape_dot_label`(`escape_full` → `\` を `\\` → `"` を `\"`) |
| `crates/cli/src/types.rs`、`args.rs`、`aux_commands.rs`、`report/mod.rs` | `GraphFormat::Dot`、`dot` の受け付け、描画の分岐、モジュールの公開 |
| `crates/cli/src/report/messages.rs` | 英日の使い方の文 4 か所と `--format` の説明の行に `dot` |
| `crates/cli/src/tests/graph_dot.rs`(新規)、`tests/mod.rs` | 単体テスト 9 件(escape の対の表、行の形と数、記号と太い枠、主線と種類の語、mermaid との数の一致、性能) |
| `crates/cli/src/tests/args_grammar.rs`、`tests/flow_graph.rs` | `--format dot` を誤りの例にしていたテストを `svg` に替え、`dot` を受け付ける確認を足した。`flow_graph.rs` に `--format dot` の流れと誤りの表示が text と同じことを足した |
| `crates/cli/tests/binary_graph.rs` | 実バイナリの `--format dot`、`--state`、`-v`、`svg` の誤り、目印が出ないこと |
| `crates/cli/src/tests/docs.rs`、`README.md` | 使い方の 2 か所と表、`graph` の節の説明と `dot -Tsvg` の例、`-v` の扱い |
| `demo/run-demo.sh`、`demo/windows/run-demo.cmd`、`demo/README.md`、`crates/cli/tests/unix_demo.rs`、`windows_demo.rs` | 場面 12 に `graph --format dot`、`digraph conductor {` の行の確認 |

core、`docs/spec/rust/`、graph_layout_oracle の参照出力は変えていない。

## 主な判断

- DOT の文法は、各行が決まった形に一致するか(ラベルの中の `\` の後ろは `\` か `"` だけ、制御文字なし)と、行の数「節(終点を含む)+ 辺 + 3」で確かめる。
- 性能のテストは絶対の上限だけ(1 万のジョブの直線で 1 秒。実測はデバッグ版で 9 件合わせて 0.06 秒)。
- `--format` の値の誤りの文には、もともと値の一覧が無い。`dot` は使い方の文と説明の行に足した。
- 本番にテスト専用の項目、新しい依存、`unsafe`、`cfg` は無い。

## 計画からのずれ

- 計画の「直すファイル」に無かった `args_grammar.rs` と `flow_graph.rs` の今のテストを直した。`--format dot` を使い方の誤りの例にしていたため、このままでは落ちる。誤りの例を `svg` に替え、`args_grammar.rs` のテストは名前を変えて `dot` を受け付ける確認を足した。

## コミット(`main`、push はしていない)

32fc1ac(引数と描画)、a012100(描画の単体テスト)、629435e(流れと実バイナリのテスト)、fade8dc(デモ)、dfd427d(README)。

## テスト

| コマンド | 変更前 | 変更後 |
|---|---|---|
| `cargo test -p conductor-cli --lib --locked -- tests::graph_dot tests::graph_report tests::docs` | 49 | 58、失敗 0 |
| `cargo test -p conductor-cli --test flow_graph --test binary_graph --test unix_demo --locked` | 8 / 10 / 3 | 10 / 14 / 3、失敗 0 |
| `cargo test --workspace --locked` | | 失敗 0 |
| `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | | 通過 |

## 書いたが、ここでは走らせていないテスト

- `crates/cli/tests/windows_demo.rs` に足した `digraph` の確認: Windows でだけビルドされる。CI の windows で確かめる。
- 行カバレッジ 80% の下限: ユニットの確認のコマンドで全体を測る。
