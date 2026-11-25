# ビジネスルール: u4-graph-dot(`graph --format dot`)

出典: 要件(`requirements.md` の FR4.1〜FR4.4、FR8.1、NFR6)、ユニットの定義(`unit-of-work.md` の U4)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md` の CliArgs、CliReport)、契約(`contract-summary.md` の C5)、設計の決定(ADR-006)、機能の設計の質問(`functional-design-questions.md` の Q1、Q2)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: "`graph --format` に `dot` を足す。`text`・`mermaid`・`dot` 以外は今と同じ使い方の誤り"
    category: validation
    applies_to: GraphFormat
    trigger: 引数の解析
    logic: IF 値が dot THEN GraphFormat::Dot。ELSE IF text・mermaid THEN 今のまま。ELSE 今と同じ使い方の誤り(文に dot を足す)
    violation: 使い方の誤り(今と同じ終了コード)
    source: FR4.1, FR4.4

  - id: BR1.2
    statement: 使い方の表(`args.rs`、`report/messages.rs`、README)の `--format` の値に `dot` を足し、3 か所を揃える
    category: policy
    applies_to: 文書
    trigger: このユニットの変更
    logic: 3 か所の `<text|mermaid>` を `<text|mermaid|dot>` にする
    violation: 文書のテスト(`src/tests/docs.rs`)の失敗
    source: FR4.1, FR8.1

  - id: BR2.1
    statement: DOT は、text・mermaid と同じ `FlowGraph` から、mermaid と同じ節・辺を、同じ順で描く
    category: calculation
    applies_to: DotDocument
    trigger: "`graph --format dot`"
    logic: >
      `digraph conductor {`、`rankdir=TB`、節(記述順、終点は最後)、辺(`FlowGraph` の辺の順)、`}` の順に書く。
      ジョブの節の数と辺の数は mermaid と同じ(行き先の分からない辺は mermaid と同じく出さない)
    violation: 該当なし
    source: FR4.1

  - id: BR2.2
    statement: 節の ID は連番で、ジョブ ID はラベルにだけ書く
    category: policy
    applies_to: DotNode
    trigger: BR2.1
    logic: ジョブは `n<記述順の番号>`、終点は `nend`。ジョブは `shape=box`、終点は `shape=oval`
    violation: 該当なし
    source: FR4.1, FR4.3

  - id: BR2.3
    statement: 主線の辺はラベルなし、それ以外は種類のラベル
    category: policy
    applies_to: DotEdge
    trigger: BR2.1
    logic: IF 主線の辺(mermaid と同じ判定)THEN ラベルなし ELSE 成功・警告・両方の語(言語に従う。mermaid と同じ語)
    violation: 該当なし
    source: FR4.1

  - id: BR2.4
    statement: "`--state` のとき、ジョブの状態をラベルの先頭の記号で示し、再開位置は太い枠にする。色は使わない"
    category: policy
    applies_to: DotNode
    trigger: "`--state`(`--run-id` を含む)"
    logic: >
      ラベルの先頭に text・mermaid と同じ記号(`[OK] `、`[NG] `、`[--] `、`[  ] `)を付ける。
      再開位置のジョブには `penwidth=3` を付ける。`--state` でなければ記号も太い枠も無い
    violation: 該当なし
    source: FR4.2

  - id: BR2.5
    statement: "`-v` は DOT に影響しない(mermaid と同じ)。`-v` を付けても誤りにしない"
    category: policy
    applies_to: DotDocument
    trigger: "`graph --format dot -v`"
    logic: IF -v THEN 出力は -v なしと同じ
    violation: 該当なし
    source: FR4.2

  - id: BR3.1
    statement: ラベルの文字列は DOT の二重引用符の文字列として escape する
    category: validation
    applies_to: DotNode.label, DotEdge.label
    trigger: ラベルを書くとき
    logic: >
      まず text と同じ表示の escape(`escape_full`。制御文字や書式の制御文字を見える形にする)をかけ、
      そのあと `\` を `\\`、`"` を `\"` にする。これで改行や DOT の `\n`・`\l` などの特別な列が
      ラベルの中で働かず、どのジョブ ID でも文書が壊れない
    violation: 該当なし
    source: FR4.3

  - id: BR3.2
    statement: DOT に、ジョブ ID と終点の名前と種類の語のほかの入力の値(コマンド、環境変数、パイプライン名)を書かない
    category: constraint
    applies_to: DotDocument
    trigger: すべての出力
    logic: グラフの名前は固定の `conductor`。コマンドとリトライの要約は出さない(BR2.5)
    violation: 該当なし
    source: FR4.3

  - id: BR4.1
    statement: "`--format dot` を使わないときの `graph` の出力は変えない"
    category: constraint
    applies_to: graph
    trigger: すべての場面
    logic: text と mermaid の描画は変えない(共有する補助関数の振る舞いも変えない)
    violation: 該当なし
    source: FR4.4

  - id: BR4.2
    statement: README とデモに `--format dot` を足す
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: README の `graph` の節に `--format dot` と例(`| dot -Tsvg`)を足す。デモの `graph` の場面に `--format dot` を足す
    violation: 該当なし
    source: FR8.1
```

## まとめ

| ID | 規則 | 種類 | 出典 |
|---|---|---|---|
| BR1.1 | `--format dot` を足す | validation | FR4.1、FR4.4 |
| BR1.2 | 使い方の表の 3 か所を揃える | policy | FR4.1、FR8.1 |
| BR2.1 | mermaid と同じ構造・順で描く | calculation | FR4.1 |
| BR2.2 | 節の ID は連番、形は box と oval | policy | FR4.1、FR4.3 |
| BR2.3 | 主線はラベルなし、他は種類の語 | policy | FR4.1 |
| BR2.4 | `--state` は記号と太い枠 | policy | FR4.2 |
| BR2.5 | `-v` は影響しない | policy | FR4.2 |
| BR3.1 | ラベルの escape | validation | FR4.3 |
| BR3.2 | ジョブ ID 以外の入力を書かない | constraint | FR4.3 |
| BR4.1 | text・mermaid は変えない | constraint | FR4.4 |
| BR4.2 | README とデモ | policy | FR8.1 |
