# エンティティ(Entities): U3 u3-dry-run(`--dry-run` と `--assume`)

出典: ユニットの定義(`inception/units-generation/unit-of-work.md` の U3)、要件(`requirements-analysis/requirements.md` の FR3.1〜FR3.4、FR1.12、FR5.1、FR5.2)、コンポーネントの一覧(`domain-design/components.md` の `CorePlan`、`CliArgs`、`CliReport`、`CliFlows`)、契約(`contract-design/contract-summary.md` の C1、C2、C5、C7)、U1 の設計(`construction/u1-env-injection/functional-design/`。名前の要約)、U2 の設計(`construction/u2-graph/`。リトライの要約の整形、コマンドの表示)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q4)。

計画の生成(`CorePlan`)は、`conductor-core` の純粋な関数で、ファイルにもプロセスにも環境変数にも触れない。環境変数の値は、この Unit のどの型にも持たない(名前だけ。FR1.12)。

```yaml
entities:
  - name: Assumption
    owner: CorePlan
    kind: value object
    description: "`--assume <ジョブID>=<終了コード>` の 1 つ分"
    attributes:
      - { name: job_id, type: string, required: true, note: 定義にあるジョブの ID }
      - { name: exit_code, type: integer, required: true, min: 0, max: 255 }
    constraints:
      - 同じ job_id は、1 回だけ(BR2.1)

  - name: PlanDisposition
    owner: CorePlan
    kind: enumeration
    allowed:
      - will_run              # 実行する
      - already_executed      # 実行済み(成功・警告で記録済み)
      - skipped_by_transition # 遷移でスキップ
      - skipped_after_end     # @end のあとでスキップ
      - not_reached           # 模擬で、エラーで停止したあとのため、実行されない
    note: "起動前の判断(`Run`、`Skip(SkipTarget)`、`Skip(AfterEnd)`、`Skip(AlreadyExecuted)`)と、模擬の停止から決まる。`Reject` は、計画の項目にせず、誤りにする(BR4.3)"

  - name: SimulatedResult
    owner: CorePlan
    kind: value object
    description: 模擬での、ジョブの結果(`will_run` のジョブだけ)
    attributes:
      - { name: exit_code, type: integer, required: true }
      - { name: classification, type: Classification, required: true, note: 成功、警告、エラー(既存の分類。`classify_and_decide` の結果) }
      - { name: assumed, type: boolean, required: true, note: "`--assume` で指定したか(false は、既定の成功)" }
      - { name: retryable, type: boolean, required: true, note: 仮定した終了コードが、リトライの対象に含まれるか(注記のため。模擬ではリトライを繰り返さない) }

  - name: PlanEntry
    owner: CorePlan
    identifier: job_id
    description: 計画の 1 つのジョブ(契約 C7 の job_entry)
    attributes:
      - { name: job_id, type: string, required: true }
      - { name: disposition, type: PlanDisposition, required: true }
      - { name: is_resume, type: boolean, required: true, note: 再開位置のジョブか }
      - { name: command_words, type: list of string, required: true, note: "分割後のコマンド(先頭がプログラム。`CliFlows` が `runtime` の分割で作って渡す。Q1 の前提)" }
      - { name: env_names, type: list of EnvNameEntry, required: true, note: 渡される環境変数の名前と出どころ(U1 の名前の要約。値を含まない) }
      - { name: retry, type: RetryPolicy, required: false, note: リトライの設定(定義のとおり) }
      - { name: result, type: SimulatedResult, required: false, note: "`will_run` のときだけ" }
    constraints:
      - 環境変数の値を持たない(BR5.1)
      - "`will_run` でないジョブにも、command_words と env_names と retry を持つ(計画の全体を、ジョブごとに読めるように)。ただし、表示では、`will_run` のジョブだけに、これらの詳細を出す(BR3.3)"

  - name: EnvNameEntry
    owner: CoreEnvFile
    kind: value object
    note: U1 の名前の要約(`EffectiveEnv::names()` の 1 項目: 名前と出どころ `EnvOrigin`)。この Unit は、そのまま使う

  - name: ResumeInfo
    owner: CorePlan
    kind: value object
    attributes:
      - { name: kind, type: enum, allowed: [from_start, after_stop, individual], required: true, note: "`from_start`: ステートが無い(完了したステートを含む)。`after_stop`: 前回の停止(エラー、または中断)で、再開位置がある。`individual`: `run` の対象のジョブ" }
      - { name: job_id, type: string, required: false, note: "`after_stop` と `individual` のとき" }

  - name: PlanEnd
    owner: CorePlan
    kind: enumeration
    allowed: [completed, end_reached, stopped_at_error]
    note: 模擬の最後。`completed`(最後のジョブまで終わる)、`end_reached`(`@end` に到達して終わる)、`stopped_at_error`(エラーで止まる。停止したジョブの ID を持つ)

  - name: ExecutionPlan
    owner: CorePlan
    identifier: pipeline_name
    description: 実行計画の全体(契約 C7 の出力)
    attributes:
      - { name: pipeline_name, type: string, required: true }
      - { name: run_id, type: string, required: false }
      - { name: mode, type: enum, allowed: [pipeline, individual], required: true }
      - { name: resume, type: ResumeInfo, required: true }
      - { name: assumptions, type: list of Assumption, required: true, note: 指定された仮定(指定順) }
      - { name: unused_assumptions, type: list of string, required: true, note: 模擬で使われなかった仮定のジョブ ID(Q1 の A) }
      - { name: entries, type: list of PlanEntry, required: true, note: "`pipeline` は、定義の記述順のすべてのジョブ。`run` は、対象の 1 ジョブ" }
      - { name: end, type: PlanEnd, required: true }
      - { name: next_step_note, type: NextStepNote, required: false, note: "`run` のときだけ(BR1.6)" }
    constraints:
      - 値を含まない(環境変数の値、`.env` の内容)

  - name: NextStepNote
    owner: CorePlan
    kind: value object
    description: "`run <job> --dry-run` の、仮定の結果(既定は成功)による、次の動き"
    attributes:
      - { name: next_job_id, type: string, required: false }
      - { name: skipped_job_ids, type: list of string, required: true }
      - { name: completed, type: boolean, required: true }
      - { name: end_reached, type: boolean, required: true, note: "`@end` に到達したとき(後ろに未処理のジョブがあれば、それらは、呼ばれたときにスキップされる。`classify_and_decide` の個別実行の結果)" }
      - { name: stopped, type: boolean, required: true, note: エラーで停止するとき }

  - name: DryRunOptions
    owner: CliArgs
    kind: value object
    attributes:
      - { name: dry_run, type: boolean, required: true, default: false, note: "`--dry-run`。`pipeline` と `run` だけで使える" }
      - { name: assumptions, type: list of Assumption, required: true, note: "`--assume` の指定(複数回)。`--dry-run` と一緒のときだけ" }

relationships:
  - { from: ExecutionPlan, to: PlanEntry, cardinality: "1 対 1..*", direction: 含む }
  - { from: ExecutionPlan, to: Assumption, cardinality: "1 対 0..*", direction: 含む }
  - { from: ExecutionPlan, to: ResumeInfo, cardinality: "1 対 1", direction: 含む }
  - { from: PlanEntry, to: SimulatedResult, cardinality: "1 対 0..1", direction: 含む }
  - { from: PlanEntry, to: JobDefinition, cardinality: "多対 1(job_id)", direction: 参照する, owner: CoreTypes }
  - { from: ExecutionPlan, to: PipelineState, cardinality: "導出(ステートの内容から作る)", direction: 参照する, owner: CoreTypes }
```

## 人が読む要約

- 計画(`ExecutionPlan`)は、定義、(読んだ)ステートの内容、分けたコマンド、U1 の名前の要約、仮定から作る。何も書かない(ステートに、何も。FR3.1)。
- 模擬の途中の状態(ジョブの状態の一時的なコピー)は、計画を作る関数の中だけで持ち、返さず、保存しない。
- 環境変数は、名前と出どころだけ(U1 の `EnvNameEntry`)。値を持つ型は、この Unit にない(BR5.1)。
