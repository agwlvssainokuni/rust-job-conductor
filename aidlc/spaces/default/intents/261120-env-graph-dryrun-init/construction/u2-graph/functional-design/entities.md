# エンティティ(Entities): U2 u2-graph(`conductor graph`)

出典: ユニットの定義(`inception/units-generation/unit-of-work.md` の U2)、要件(`requirements-analysis/requirements.md` の FR2.1〜FR2.7、FR1.12、FR5.1、FR5.2)、コンポーネントの一覧(`domain-design/components.md` の `CoreGraph`、`CoreFlow`、`CliReport`)、契約(`contract-design/contract-summary.md` の C2、C5、C6)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q5)。

この文書は、U2 が扱うデータの形(論理の型)を記す。図の構造(節と辺)と、レーンの配置の結果は、`conductor-core` に置き(文字列の整形はしない)、文字列にする部分は `conductor-cli` に置く。環境変数の値は、この Unit のどの型にも持たない(FR1.12)。

```yaml
entities:
  # --- 辺(持ち主: CoreFlow が取り出し、CoreGraph が使う) ---
  - name: EdgeKind
    owner: CoreFlow
    kind: enumeration
    allowed: [success, warning]
    note: 辺が、どの結果のときに通るか。エラーには辺がない(ジョブが止まるため)

  - name: EdgeTarget
    owner: CoreFlow
    kind: enumeration
    allowed: [job, end]
    note: "`job` はジョブ ID を持つ。`end` は終点(`@end`)。最後のジョブが完了したときの行き先も `end`"

  - name: FlowEdge
    owner: CoreFlow
    identifier: (from_job, to)
    description: 1 つのジョブから出る、1 つの辺(契約 C2 の edges)。成功と警告の行き先が同じなら、1 つにまとめ、kinds に両方を持つ
    attributes:
      - { name: from_job, type: string, required: true, note: 元のジョブの ID }
      - { name: to, type: EdgeTarget, required: true, note: 行き先 }
      - { name: kinds, type: list of EdgeKind, required: true, min: 1, note: この辺を通る結果。成功と警告の両方、または片方 }
      - { name: skipped_jobs, type: list of string, required: true, note: 元と行き先のあいだにあるジョブの ID(定義の記述順)。行き先が次のジョブなら空 }
      - { name: explicit, type: boolean, required: true, note: 定義に遷移として書かれた辺か(書かずに、記述順の次へ進む辺は false) }
    constraints:
      - 1 つのジョブから出る辺は、多くて 2 つ(成功と警告の行き先が違うとき)
      - 行き先は、元より後ろのジョブ、または終点(定義の検査が保証する)

  # --- 図の構造(持ち主: CoreGraph) ---
  - name: NodeState
    owner: CoreGraph
    kind: enumeration
    allowed: [ok, skipped, error, pending]
    note: "`--state` で重ねる状態。成功と警告は、`ok`(`[OK]`)にまとめる。スキップは `skipped`(`[--]`)、エラーは `error`(`[NG]`)、未実行は `pending`(`[  ]`)"

  - name: StateOverlay
    owner: CoreGraph
    kind: value object
    description: "`--state` で重ねる入力(任意)。ステートの内容から作る。環境変数も、コマンドも、持たない"
    attributes:
      - { name: states, type: list of (job_id, NodeState), required: true }
      - { name: resume_job_id, type: string, required: false, note: 再開位置。完了していないステートで、再開位置があるときだけ }

  - name: GraphNode
    owner: CoreGraph
    identifier: job_id
    description: 図の節。ジョブ 1 つ、または終点
    attributes:
      - { name: job_id, type: string, required: false, note: 終点では無し }
      - { name: is_end, type: boolean, required: true }
      - { name: state, type: NodeState, required: false, note: "`--state` のときだけ" }
      - { name: is_resume, type: boolean, required: true, note: 再開位置のジョブか }

  - name: FlowGraph
    owner: CoreGraph
    kind: aggregate
    description: 図の構造。節は、定義の記述順で、最後に終点
    attributes:
      - { name: nodes, type: list of GraphNode, required: true }
      - { name: edges, type: list of FlowEdge, required: true, note: 元のジョブの記述順。同じ元の中では、記述順の次への辺が先 }
    constraints:
      - 節の数 = ジョブの数 + 1(終点)
      - 辺の元は、すべてジョブの節

  - name: LaneGlyph
    owner: CoreGraph
    kind: enumeration
    allowed: [empty, node, end_node, line, fork, join, cross]
    note: "レーンの図の 1 マス。`node` は `*`、`end_node` は `o`、`line` は `|`、`fork` は `\\`、`join` は `/`、`cross` は交差(`|/|` の形のために、実装が使う補助。表示の文字は、CliReport が決める)"

  - name: LaneRow
    owner: CoreGraph
    kind: value object
    description: 図の 1 行。ノードの行、または、線をつなぐ行(分岐、合流、交差)
    attributes:
      - { name: glyphs, type: list of LaneGlyph, required: true, note: 左の列から }
      - { name: node_index, type: integer, required: false, note: ノードの行のときだけ。FlowGraph.nodes の位置 }
      - { name: note_target, type: string, required: false, note: 線をつなぐ行のうち、詳細(-v)の行の置き場所を示す補助 }

  - name: LaneLayout
    owner: CoreGraph
    kind: aggregate
    description: レーンの配置の結果。行の並び
    attributes:
      - { name: rows, type: list of LaneRow, required: true }
    constraints:
      - 全ノードが、ちょうど 1 回ずつ、記述順(最後が終点)に現れる
      - ファイルにも、プロセスにも、触れない

  # --- 表示の入力(持ち主: CliReport) ---
  - name: GraphFormat
    owner: CliReport
    kind: enumeration
    allowed: [text, mermaid]
    note: "`--format` の値。既定は `text`"

  - name: GraphOptions
    owner: CliArgs
    kind: value object
    attributes:
      - { name: format, type: GraphFormat, required: true, default: text }
      - { name: verbose, type: boolean, required: true, default: false, note: "`-v`" }
      - { name: with_state, type: boolean, required: true, default: false, note: "`--state`" }
      - { name: run_id, type: string, required: false, note: "`--run-id`。`--state` と一緒のときだけ許す(BR4.5)" }

relationships:
  - { from: FlowGraph, to: GraphNode, cardinality: "1 対 1..*", direction: 含む }
  - { from: FlowGraph, to: FlowEdge, cardinality: "1 対 0..*", direction: 含む }
  - { from: FlowEdge, to: JobDefinition, cardinality: "多対 1(from_job)", direction: 参照する, owner: CoreTypes }
  - { from: LaneLayout, to: LaneRow, cardinality: "1 対 1..*", direction: 含む }
  - { from: StateOverlay, to: PipelineState, cardinality: "導出(ステートの内容から作る)", direction: 参照する, owner: CoreTypes }
```

## 人が読む要約

- 辺(`FlowEdge`)は、実行のときに「次のジョブを決める」判断と、同じ関数(`CoreFlow` の遷移の解決)から取り出す。図と実行が食い違わない(FR2.4。BR1.5)。
- 図の構造(`FlowGraph`)と、レーンの配置(`LaneLayout`)は、`CoreGraph` が、定義と(任意の)`StateOverlay` だけから作る。文字への変換(`*`、`|`、`[OK]`、日本語と英語の注記、Mermaid の記法)は、`CliReport` が行う。
- 状態の重ね合わせは、入力(`StateOverlay`)を渡したときだけ。ステートの型(`PipelineState`)は、変えない。
- 環境変数は、この Unit のどの型にも出てこない(`-v` で、環境変数の値も名前も、出さない。FR1.12)。
