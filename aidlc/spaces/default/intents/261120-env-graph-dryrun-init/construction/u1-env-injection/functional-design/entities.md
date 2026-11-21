# エンティティ(Entities): U1 u1-env-injection(環境変数の注入と `.env`)

出典: ユニットの定義(`inception/units-generation/unit-of-work.md` の U1)、要件(`requirements-analysis/requirements.md` の FR1.1〜FR1.13)、コンポーネントの一覧(`domain-design/components.md` の `CoreTypes`、`CoreEnvFile`、`RuntimeEnvLoader`)、契約(`contract-design/contract-summary.md` の C1、C3、C4)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q8)。

この文書は、U1 が扱うデータの形(論理の型)を、技術に依存しない形で記す。型の持ち主(コンポーネント)を、エンティティごとに 1 つ決める。環境変数の「値」は秘密として扱い、各エンティティで、値を持つ属性に `secret: true` を付ける(表示、エラー、ステートに出さない。FR1.12)。

```yaml
entities:
  # --- 既存のエンティティの変更(持ち主: CoreTypes) ---
  - name: PipelineDefinition
    owner: CoreTypes
    change: 属性の追加(`env`、`env_file`)。ほかの属性は変えない
    description: 定義ファイル 1 つに書かれたパイプラインの定義
    attributes_added:
      - name: env
        type: list of EnvVar
        required: false
        default: 空の並び
        note: "[pipeline.env] の内容。定義ファイルに書いた順を保つ"
      - name: env_file
        type: list of string
        required: false
        default: 空の並び
        note: "[pipeline].env_file の内容。文字列 1 つで書いたときは、要素 1 つの並びにする。書いた順を保つ"
    constraints:
      - format_version は 1 のまま(FR1.13)

  - name: JobDefinition
    owner: CoreTypes
    change: 属性の追加(`env`、`env_file`)。ほかの属性は変えない
    description: 1 つのジョブの定義
    attributes_added:
      - name: env
        type: list of EnvVar
        required: false
        default: 空の並び
        note: "[job.env] の内容"
      - name: env_file
        type: list of string
        required: false
        default: 空の並び
        note: "[job].env_file の内容"

  # --- 新しいエンティティ(定義の値。持ち主: CoreTypes) ---
  - name: EnvVar
    owner: CoreTypes
    kind: value object
    description: 定義の `env` の表の 1 項目(名前と値)
    attributes:
      - { name: name, type: string, required: true, constraints: [名前の規則(BR1.2)、予約名でない(BR1.3)、同じ表の中で大文字小文字だけが違う名前がない(BR1.4)] }
      - { name: value, type: string, required: true, secret: true, constraints: [NUL を含まない(BR1.5)] }

  # --- 新しいエンティティ(.env と統合。持ち主: CoreEnvFile) ---
  - name: EnvFileContent
    owner: CoreEnvFile
    identifier: path
    description: 1 つの `.env` ファイルを読み取った結果(ファイルの内容そのものではなく、項目の並び)
    attributes:
      - { name: path, type: string, required: true, note: 解決済みのパス。表示のときは、エスケープと切り詰めをする }
      - { name: entries, type: list of EnvEntry, required: true, note: 書いた順。同じ名前の重複は、後ろの項目が先頭より後に並ぶ }
      - { name: warnings, type: list of EnvWarning, required: true, note: 重複キーの警告(BR2.7) }
    constraints:
      - 項目の数は 10,000 以下(BR3.2)

  - name: EnvEntry
    owner: CoreEnvFile
    kind: value object
    description: `.env` の 1 項目
    attributes:
      - { name: name, type: string, required: true }
      - { name: value, type: string, required: true, secret: true }
      - { name: line, type: integer, required: true, min: 1, note: 項目を書いた行 }

  - name: EnvWarning
    owner: CoreEnvFile
    kind: value object
    description: 誤りではないが、利用者に知らせる事実
    attributes:
      - { name: file, type: string, required: true }
      - { name: line, type: integer, required: true, note: 後ろの(上書きする側の)項目の行 }
      - { name: name, type: string, required: true, note: 重複した名前(名前は表示してよい) }
      - { name: kind, type: enum, allowed: [duplicateKey], required: true }

  - name: EnvFileError
    owner: CoreEnvFile
    kind: value object
    description: `.env` の読み取りの誤り(契約 C1 の errors)。値を含まない
    attributes:
      - { name: file, type: string, required: true }
      - { name: line, type: integer, required: false, note: 位置が決まらない誤り(ファイルが無い、大きすぎる)は無し }
      - { name: kind, type: EnvFileErrorKind, required: true }
      - { name: reason, type: string, required: true, note: 固定の文から組み立てる。入力の値を含まない }

  - name: EnvFileErrorKind
    owner: CoreEnvFile
    kind: enumeration
    allowed:
      - fileMissing        # ファイルが無い(FR1.8)
      - unreadable         # 読めない(権限、ディレクトリなど。FR1.8)
      - tooLarge           # 1 MiB を超える(FR1.9)
      - notUtf8            # UTF-8 でない
      - tooManyEntries     # 項目が 10,000 を超える(FR1.9)
      - invalidName        # 名前が規則に合わない(FR1.6、FR1.2)
      - reservedName       # CONDUCTOR_ で始まる名前(FR1.11)
      - caseConflict       # 同じファイルの中で、大文字小文字だけが違う名前(NFR3)
      - missingEquals      # KEY=VALUE の形でない
      - unterminatedQuote  # 閉じていない引用符(FR1.6)
      - invalidEscape      # 二重引用符の中の、許されないエスケープ(FR1.6)
      - trailingGarbage    # 閉じた引用符のあとに、空白とコメント以外がある
      - nulInValue         # NUL を含む値(FR1.9)

  - name: LoadedEnvFiles
    owner: CoreEnvFile
    kind: aggregate
    description: 1 回の呼び出しで読み取った、すべての `.env` の結果(解決済みのパスをキーにする)
    attributes:
      - { name: files, type: map of path to EnvFileContent, required: true }
    constraints:
      - 同じ解決済みのパスは、1 回だけ読む(BR6.4)

  - name: EnvContext
    owner: CoreEnvFile
    kind: value object
    description: 自動の変数(CONDUCTOR_*)を決める入力
    attributes:
      - { name: pipeline_name, type: string, required: true }
      - { name: job_id, type: string, required: true }
      - { name: run_id, type: string, required: false, note: 無いときは CONDUCTOR_RUN_ID を作らない(BR5.3) }
      - { name: attempt, type: integer, required: true, min: 1, note: 1 から始まり、リトライのたびに 1 増える(BR5.2) }

  - name: EnvOrigin
    owner: CoreEnvFile
    kind: enumeration
    allowed: [pipelineEnvFile, pipelineEnv, jobEnvFile, jobEnv, builtin]
    note: 名前の出どころ(契約 C1 の origin)。引き継いだ環境は、conductor が足すものではないので、含めない

  - name: EffectiveEnv
    owner: CoreEnvFile
    identifier: job_id
    description: 1 つのジョブの 1 回の起動に、引き継いだ環境へ「足す」環境変数。層を統合した結果
    attributes:
      - { name: job_id, type: string, required: true }
      - { name: entries, type: list of EffectiveEntry, required: true, note: 層の順に並ぶ。完全に同じ名前は、1 つ(後ろの層が勝つ) }
    derived:
      - names: 名前と出どころの並び(値を含まない。契約 C1 の names と origin)
    constraints:
      - 値は、表示と Debug の出力に含めない(BR8.1)

  - name: EffectiveEntry
    owner: CoreEnvFile
    kind: value object
    attributes:
      - { name: name, type: string, required: true }
      - { name: value, type: string, required: true, secret: true }
      - { name: origin, type: EnvOrigin, required: true }

  # --- 新しい種類の誤り(定義の検査。持ち主: CoreTypes の DefinitionErrorKind) ---
  - name: DefinitionErrorKind
    owner: CoreTypes
    change: 種類の追加
    allowed_added:
      - reservedEnvName    # CONDUCTOR_ で始まる名前(`env` の表。FR1.11)
      - envNameCaseConflict # 同じ表の中で、大文字小文字だけが違う名前(NFR3)
      - invalidEnvValue    # NUL を含む値(`env` の表。FR1.9)
    reused:
      - invalidEnvName     # 名前が規則に合わない(既存。`run_id_env` と同じ規則。FR1.2)
      - typeMismatch       # 値が文字列でない、env_file の要素が文字列でない(FR1.1、FR1.4)
      - emptyField         # env_file の要素が空の文字列

relationships:
  - { from: PipelineDefinition, to: EnvVar, cardinality: "1 対 0..*", direction: 含む }
  - { from: JobDefinition, to: EnvVar, cardinality: "1 対 0..*", direction: 含む }
  - { from: EnvFileContent, to: EnvEntry, cardinality: "1 対 0..10000", direction: 含む }
  - { from: EnvFileContent, to: EnvWarning, cardinality: "1 対 0..*", direction: 含む }
  - { from: LoadedEnvFiles, to: EnvFileContent, cardinality: "1 対 0..*", direction: 含む }
  - { from: EffectiveEnv, to: EffectiveEntry, cardinality: "1 対 0..*", direction: 含む }
  - { from: EffectiveEnv, to: JobDefinition, cardinality: "1 対 1(1 回の起動につき)", direction: 参照する }
```

## 人が読む要約

- 定義の型に足すのは、`env`(名前と値の並び)と `env_file`(パスの並び)の 2 つの属性だけで、`PipelineDefinition` と `JobDefinition` の両方に足す。どちらも省略でき、省略すると空の並びになる(新しいキーを使わない既存の定義は、変更なしで読める。FR1.13)。
- `.env` の読み取りの結果(`EnvFileContent`)は、項目、警告、(誤りは別に `EnvFileError` として)を持つ。値を持つのは、`EnvVar`、`EnvEntry`、`EffectiveEntry` の `value` だけで、この 3 つの `value` が、秘密として扱う唯一の場所である。
- 統合の結果(`EffectiveEnv`)は、「引き継いだ環境に足すもの」だけを持つ。引き継いだ環境そのものは、持たない(子のプロセスが、OS の規則で、そのまま引き継ぐ。Q8 の A)。名前の要約(`names`)は、この足すものの名前と出どころの並びで、値を含まない。
- ステートの型(`PipelineState`)は変えない。環境変数を、ステートに書かない(BR8.3)。
- エンティティの持ち主は、すべて 1 つ。定義の値は `CoreTypes`、`.env` と統合に関わる型は `CoreEnvFile`。
