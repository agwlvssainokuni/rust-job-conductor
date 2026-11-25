# エンティティ: u2-optional-env-file(無くてもよい `.env`)

出典: ユニットの定義(`unit-of-work.md` の U2)、要件とユニットの対応(`unit-of-work-story-map.md`。FR2.1〜FR2.5、FR8.1 がこのユニット)、要件(`requirements.md` の FR2.1〜FR2.5)、コンポーネントの一覧(`components.md` の CoreTypes の EnvFileRef、CoreDefinition、CoreEnvFile、RuntimeEnvFile)、契約(`contract-summary.md` の C1、C3)、機能の設計の質問(`functional-design-questions.md` の Q1、Q2)。

定義の `env_file` の項目を、文字列から EnvFileRef(パスと required)に広げる。文字列は `required = true` の EnvFileRef として読む。`.env` の中身の型(EnvFileContent など)は変えない。

```yaml
entities:
  - name: EnvFileRef
    kind: 組み立て後の定義(CoreTypes。新規。契約 C1)
    description: 定義に書いた `env_file` の 1 項目。
    attributes:
      - name: path
        type: 文字列
        required: true
        constraints: 空でない。相対なら定義ファイルのディレクトリが基準(今のまま)。`~` と変数は展開しない
      - name: required
        type: 真偽
        required: true
        default: true
        constraints: 文字列で書いたとき、表で省いたときは true
    relationships:
      - to: PipelineDefinition
        cardinality: "多 対 1"
        direction: "`[pipeline].env_file` の並び(書いた順)"
      - to: JobDefinition
        cardinality: "多 対 1"
        direction: "`[job].env_file` の並び(書いた順)"

  - name: PipelineDefinition
    kind: 組み立て後の定義(既存。CoreTypes)
    description: パイプラインの定義。
    attributes:
      - name: env_file
        type: EnvFileRef の並び
        required: true
        constraints: 今の文字列の並びから型を変える。書かなければ空

  - name: JobDefinition
    kind: 組み立て後の定義(既存。CoreTypes)
    description: ジョブの定義。
    attributes:
      - name: env_file
        type: EnvFileRef の並び
        required: true
        constraints: 今の文字列の並びから型を変える。書かなければ空

  - name: EnvFileTarget
    kind: 読む対象(CoreEnvFile の env_file_refs の結果。契約 C1 の load_env_files の入力)
    description: 1 回のコマンドで読む `.env` の 1 項目。書いたパスの文字列ごとに 1 つ。
    attributes:
      - name: path
        type: 文字列
        required: true
        unique: true
        constraints: 書いたパスの文字列で重複を除く(今のまま)
      - name: required
        type: 真偽
        required: true
        constraints: 同じ文字列を複数の場所に書いたときは、どれか 1 つでも true なら true(Q1 の A)

  - name: LoadedEnvFiles
    kind: 読み取りの結果(既存。CoreEnvFile)
    description: 読めたファイルの中身と、書いたパスの文字列からの引き当て。
    attributes:
      - name: by_ref
        type: 書いたパスの文字列 → 中身の位置
        required: true
        constraints: 無くてもよいファイルが無かったときは、その文字列を引き当てに載せない(統合で何も足さない。今の merge の扱い)
```

## まとめ

| エンティティ | 種類 | このユニットでの変化 |
|---|---|---|
| EnvFileRef | 組み立て後の定義 | 新規。パスと required |
| PipelineDefinition / JobDefinition | 組み立て後の定義 | `env_file` を文字列の並びから EnvFileRef の並びに |
| EnvFileTarget | 読む対象 | `env_file_refs` の結果に required を添える(同じ文字列は required の論理和) |
| LoadedEnvFiles | 読み取りの結果 | 形は変えない。無くてもよいファイルが無ければ載せない |

`.env` の中身の型、誤りの種類(EnvFileErrorKind)、統合の結果(EffectiveEnv)は変えない。変換のツール(`converter`)は定義を型に依らない木として扱うため、表の `env_file` はそのまま通る。
