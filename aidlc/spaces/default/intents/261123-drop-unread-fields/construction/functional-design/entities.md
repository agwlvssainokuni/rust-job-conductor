# エンティティ(Entities): 読まれない項目の削除

永続のデータ(ステートファイル)の形は変えない。利用者に見える出力も変えない。変わるのは core の公開の型の項目(書かれるが本番で読まれない項目を消す)だけ。計画の名前の型 `EnvNameEntry` は、名前だけの構造体として残す(D1。決定: 代案。ユーザー、2026-11-24)。cli の src は変えない。詳細は `inventory.md`。

```yaml
entities:
  # ---- core の graph ----
  - name: graph::FlowEdge
    description: 1 つのジョブから出る、1 つの辺(core。公開)。explicit を消す(inventory.md 1)。
    attributes:
      - name: from_job / to / kinds / skipped_jobs
        logical_type: 今のまま
      - name: explicit
        logical_type: (消す)

  # ---- core の envfile ----
  - name: EnvOrigin
    description: 環境変数の名前の出どころ(core。公開)。型ごと消す(inventory.md 2b)。
  - name: EffectiveEntry
    description: ジョブに足す環境の 1 項目(core。公開)。origin を消す。手書きの Debug は値を伏せ字にし、name と value を書く(inventory.md 2b)。
    attributes:
      - name: name / value
        logical_type: 今のまま
      - name: origin
        logical_type: (消す)
  - name: EnvFileContent
    description: 1 つの .env を読み取った結果(core。公開)。path を消す(inventory.md 3a)。
    attributes:
      - name: entries / warnings
        logical_type: 今のまま
      - name: path
        logical_type: (消す)
  - name: EnvEntry
    description: .env の 1 項目(core。公開)。line を消す。手書きの Debug は値を伏せ字にし、name と value を書く(inventory.md 3b)。
    attributes:
      - name: name / value
        logical_type: 今のまま
      - name: line
        logical_type: (消す)
  - name: EnvWarning
    description: 重複キーの警告(core。公開)。変えない。line は後ろの項目の行(今の line_no)のまま。
    attributes:
      - name: file / line / name / kind
        logical_type: 今のまま

  # ---- core の plan ----
  - name: EnvNameEntry
    description: 計画の環境変数の名前(core。公開)。origin を消し、名前だけの構造体として残す(D1。決定: 代案。ユーザー、2026-11-24)(inventory.md 2a)。再公開は今のまま。
    attributes:
      - name: name
        logical_type: String(今のまま)
      - name: origin
        logical_type: (消す)
  - name: PlanEntry
    description: 計画の 1 つのジョブ(core。公開)。変えない(env_names は並びの順の EnvNameEntry のまま。中身が名前だけになる)。
    attributes:
      - name: env_names
        logical_type: 'Vec<EnvNameEntry>(今のまま)'
      - name: job_id / disposition / is_resume / command_words / retry
        logical_type: 今のまま

  # ---- 範囲の外(変えない) ----
  - name: PipelineDefinition
    description: 定義のデータ(core。公開)。format_version と description は残す(Q3: B)。
```

## 利用者に見える変わること

なし。標準出力と標準エラーの文、終了コード、ステートファイル(バイト列)、`--dry-run` と `graph` の出力、`.env` の誤りと警告の表示、`conductor --version`(`conductor 0.9.8`)は変わらない。ライブラリとしての公開の型の項目(上の型)だけが変わる。
