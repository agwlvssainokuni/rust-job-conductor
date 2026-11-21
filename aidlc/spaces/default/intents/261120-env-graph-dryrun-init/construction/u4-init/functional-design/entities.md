# エンティティ(Entities): U4 u4-init(`conductor init`)

出典: ユニットの定義(`inception/units-generation/unit-of-work.md` の U4)、要件(`requirements-analysis/requirements.md` の FR4.1〜FR4.5、FR5.1、FR5.2)、コンポーネントの一覧(`domain-design/components.md` の `RuntimeFileCreate`、`InitTemplate`、`CliArgs`、`CliFlows`)、契約(`contract-design/contract-summary.md` の C3、C4、C5)、U1 の設計(新しいキー)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q4)。

```yaml
entities:
  - name: InitTarget
    owner: CliArgs
    kind: value object
    description: "`conductor init` の出力先"
    attributes:
      - { name: kind, type: enum, allowed: [file, stdout], required: true, note: "パスが `-` なら stdout" }
      - { name: path, type: string, required: false, note: "`file` のとき。既定は `conductor.toml`。`--config` で指定" }

  - name: InitOptions
    owner: CliArgs
    kind: value object
    attributes:
      - { name: target, type: InitTarget, required: true }
      - { name: force, type: boolean, required: true, default: false, note: "`--force`。`stdout` のときは、無視する(BR1.5)" }
      - { name: pipeline_name, type: string, required: true, default: my-pipeline, note: 省略したときの名前は固定の `my-pipeline`(Q1) }

  - name: InitTemplate
    owner: CliReport
    identifier: locale
    description: 定義のひな形。1 種類で、コメントだけがロケール(日本語/英語)で変わる(FR4.5)
    attributes:
      - { name: locale, type: enum, allowed: [ja, en], required: true }
      - { name: text, type: string, required: true, note: "UTF-8、LF の改行、末尾に改行 1 つ。`[pipeline].name` に、パイプライン名を、TOML の基本文字列として、エスケープして書く(BR2.2)" }
    constraints:
      - 日本語と英語の両方が、必ず `validate`(定義の検査と `command` の分割)を通る(BR2.4)
      - "`[pipeline.env]` と `[job.retry]` は、有効な行として書く(コメントアウトしない)"

  - name: CreateOutcome
    owner: RuntimeFileCreate
    kind: enumeration
    allowed: [created, replaced]
    note: "新規に作った(`created`)、`--force` で置き換えた(`replaced`)"

  - name: CreateError
    owner: RuntimeFileCreate
    kind: enumeration
    allowed:
      - already_exists   # 既にある(`--force` なし)。終了コード 69
      - parent_missing   # 親のディレクトリが無い(作らない。Q4)
      - is_directory     # 作る先が、ディレクトリ
      - permission_denied # 書く権限がない
      - other_io         # そのほかの入出力の誤り
    note: "`already_exists` 以外は、終了コード 65(Q3)。表示は、種類の固定の文と、パス(エスケープと切り詰め)だけで、OS のエラーの文は含めない"

relationships:
  - { from: InitOptions, to: InitTarget, cardinality: "1 対 1", direction: 含む }
  - { from: InitTemplate, to: InitOptions, cardinality: "導出(ロケールと名前から作る)", direction: 参照する }
```

## 人が読む要約

- `init` は、定義も、`.env` も、ステートも読まない。入力は、出力先、`--force`、パイプライン名、ロケールだけ。
- ひな形は、文字列の 1 つの型(`InitTemplate`)で、名前だけが、実行のたびに変わる。環境変数の値は、ひな形の例の固定の文字列(`hello`)だけで、利用者の値は、どこにも出てこない。
- ファイルの作成の結果は、成功の 2 種類(新規、置き換え)と、誤りの 5 種類(`CreateError`)。
