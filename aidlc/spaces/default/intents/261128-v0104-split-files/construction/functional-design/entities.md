# 実体: v0.10.4 の大きなファイルの分割

出典: 要件(`requirements.md`、承認済み)、設計の質問(`functional-design-questions.md` の Q1〜Q4)、引き渡し(`developer-scan.md` の 1〜3)。

この作業は refactor で、Units もドメインの設計もありません。本番のデータの形(型、ステート、定義の書式)は変えません。ここでの実体は、分けた後のモジュールの形を表すものです。

```yaml
entities:
  - name: ParentModule
    description: 分ける前のファイル。分けた後は、入口と振り分けと pub use を持つ mod.rs になる
    attributes:
      - name: path
        type: string
        required: true
        unique: true
        constraints: crates/core/src/definition/walk/mod.rs、crates/cli/src/args/mod.rs、crates/cli/src/report/mod.rs
      - name: line_limit
        type: integer
        required: true
        constraints: walk 420、args 300、report 220(functional-spec の 5 章)
      - name: kept_items
        type: list
        required: true
        constraints: functional-spec の 2.1・3.1・4.1 の「mod.rs に残す項目」
    relationships:
      - target: ChildModule
        cardinality: one-to-many
  - name: ChildModule
    description: 親から項目を移した子のファイル
    attributes:
      - name: path
        type: string
        required: true
        unique: true
        constraints: walk の values・transition・cross_check、args の options・tokens・values、report の event・lines・escape・render
      - name: items
        type: list
        required: true
        constraints: functional-spec の 2.1・3.1・4.1 の「移す項目」
      - name: visibility
        type: enum
        allowed: [private, pub(super), pub(crate), pub]
        constraints: 今の見える範囲を超えない。親や兄弟が使うものだけ pub(super)。今 pub・pub(crate) の型と関数は、そのまま残し、親で pub use する
  - name: RenderSection
    description: render.rs の中の表示の形の節(9 つ)
    attributes:
      - name: form
        type: enum
        allowed: [A, B, C, D, E, F, G, H, I]
      - name: methods
        type: list
        constraints: 変種ごとのメソッド。取り出した中身を引数で受け取り、String を返す
      - name: detects_language
        type: boolean
        constraints: A と pass_through は false。ほかは true
  - name: BuildStep
    description: build_command から分けた 6 つの関数
    attributes:
      - name: order
        type: integer
        constraints: 1〜6(functional-spec の 3.2)。順を変えない
      - name: name
        type: string
        constraints: collect_tokens、command_kind、check_options、fill_options、check_operands、fill_operand
```

## まとめ

- `ParentModule` は 3 つです。分けた後は、入口・振り分け・`pub use`・残す項目だけを持ち、行数の上限を守ります。
- `ChildModule` は 10 個です。見える範囲は、今より広げません。
- `RenderSection` は 9 つの節です。網羅の `match` の各枝は、そのどれかのメソッドを呼びます。
- `BuildStep` は 6 つの関数です。今と同じ順に誤りを確かめます。
