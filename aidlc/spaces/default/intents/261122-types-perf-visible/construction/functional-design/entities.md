# エンティティ(Entities): 型と構造の整理・計算量・利用者から見える変更

永続のデータ(ステートファイル)の形は変えない。利用者に見える出力は、段 3 の 2 つ(converter の文法の誤りの理由の文、読む権限のない 1 MiB 超の `.env` の誤りの種類)と版の表示だけが変わる。公開の型の変更は段 1 のもの(契約 C1 と計画・図の型)。詳細は `inventory.md`。

```yaml
entities:
  # ---- core: 計画(plan) ----
  - name: ResumeInfo
    description: 計画の再開位置(core の plan/mod.rs。公開。構造体から列挙型に)。ResumeKind を消す。
    attributes:
      - name: FromStart
        logical_type: ステートなし、または完了したステート
      - name: AfterStop
        logical_type: (String)。前回の停止の再開位置のジョブ ID
      - name: Individual
        logical_type: (String)。run の対象のジョブ ID
  - name: PlanDisposition
    description: 計画の中のジョブの扱い(公開。Copy・Eq のまま)。WillRun が模擬の結果を持つ。読み方の補助 result(self) -> Option<SimulatedResult>。
    attributes:
      - name: WillRun
        logical_type: (SimulatedResult)
      - name: AlreadyExecuted / SkippedByTransition / SkippedAfterEnd / NotReached
        logical_type: 今のまま(結果を持たない)
  - name: PlanEntry
    description: 計画の 1 つのジョブ(公開)。result を消す(結果は disposition が持つ)。job_id、disposition、is_resume、command_words、env_names、retry は今のまま。
  - name: NextStepNote
    description: run の計画の、仮定の結果による次の動き(公開。構造体から列挙型に)。
    attributes:
      - name: NextJob
        logical_type: '{ job_id: String, skipped_job_ids: Vec<String> }'
      - name: EndReached
        logical_type: '@end に到達(後ろのジョブは、呼ばれたときにスキップ)。完了していない'
      - name: Completed
        logical_type: 完了する
      - name: Stopped
        logical_type: エラーで停止する

  # ---- core: 図(graph) ----
  - name: NodeKind
    description: 図の節の種類(core の graph/mod.rs。新しい名前。公開)。遷移の行き先の型の別名 pub use crate::types::TransitionTarget as NodeKind(EdgeTarget と同じ方法。D1。決定(ユーザー、2026-11-23))。
    attributes:
      - name: Job
        logical_type: (String)。ジョブ ID
      - name: End
        logical_type: 終点(@end)
  - name: GraphNode
    description: 図の節(公開)。job_id と is_end を kind にまとめる。読み方の補助 job_id(&self) -> Option<&str>、is_end(&self) -> bool。
    attributes:
      - name: kind
        logical_type: NodeKind
      - name: state
        logical_type: Option<NodeState>(今のまま)
      - name: is_resume
        logical_type: bool(今のまま)

  # ---- core: 共有の型(契約 C1) ----
  - name: Backoff
    description: リトライの待ち時間を伸ばす設定(core の types.rs。新規。公開。Copy)。倍率と上限は必ず組で持つ。
    attributes:
      - name: multiplier
        logical_type: u32(2〜10)
      - name: max_interval_secs
        logical_type: u64(interval_secs 以上)
  - name: RetryPolicy
    description: ジョブのリトライの設定(公開)。multiplier と max_interval_secs を backoff にまとめる。
    attributes:
      - name: exit_codes / max / interval_secs
        logical_type: 今のまま
      - name: backoff
        logical_type: Option<Backoff>(書かなければ None で、待ち時間は毎回 interval_secs)
  - name: RetryDecision
    description: リトライするかの判断(core の flow/retry.rs。公開)。Retry が知らせに要る値を持つ。
    attributes:
      - name: Stop
        logical_type: 今のまま
      - name: Retry
        logical_type: '{ wait_secs: u64, last_exit_code: i32, max: u32 }'
  - name: JobExecution
    description: 1 回のジョブの起動の結果(公開)。job_id と command を消し、result だけを持つ(D2。決定(ユーザー、2026-11-23))。組み立ての関数は job を受け取らない new(result)、exited(exit_code)、not_started(signal)、start_failed()。
    attributes:
      - name: result
        logical_type: ExecutionResult(今のまま)
  - name: JobOutcome
    description: 分類と報告の終了コード(公開)。job_id を消す。classification、reported_exit_code、conductor_error は今のまま。
  - name: NextStep
    description: 次の動き(公開)。job_id を消す。next_job_id、skipped_job_ids、skip_reason_kind、end_reached、completed、resume_job_id は今のまま。

  # ---- core: 誤りの型 ----
  - name: EnvFileError
    description: .env の読み取りの誤り(core の envfile/mod.rs。公開)。reason のフィールドを消し、メソッド reason(&self) -> &'static str(種類の固定の文)にする。file、line、kind は今のまま。
  - name: render_error
    description: 誤りの表示の書式(core の error.rs。新規。pub(crate))。fn(file: &str, line: Option<usize>, kind: &str, reason: &str) -> String。DefinitionError と EnvFileError の Display が使う。ファイル名は sanitize で整える。
  - name: DefinitionErrorKind::label / EnvFileErrorKind::label
    description: 種類の日本語の文(core。新規。公開。&'static str)。Display はこれを書く(文は今の #[error] の文と同じ。D10。決定(ユーザー、2026-11-23))。
  - name: ConversionErrorKind::label
    description: 変換の誤りの種類の日本語の文(converter の error.rs。新規。公開)。Display はこれを書く。
  - name: StateErrorKind::label
    description: ステートの誤りの種類の日本語の文(runtime の error.rs。既存。非公開から公開に)。

  # ---- core: crate の中だけの関数(段 2) ----
  - name: apply_outcome_in / apply_skip_in / apply_interrupt_in
    description: ステートをその場で書き換える記録(core の flow/record.rs。新規。pub(crate))。公開の apply_outcome・apply_skip・apply_interrupt は、複製してからこれを呼ぶ。apply_outcome_in・apply_skip_in は、探索の手がかりの位置 hint を受け取る(公開の関数は 0)。
  - name: locate
    description: job_states の中のジョブの位置(core の flow/record.rs。新規。非公開)。fn(states: &[JobState], from: usize, job_id: &str) -> Option<usize>。states[from..] を先に、なければ states[..from] を探す。
  - name: resolve_transition
    description: 遷移の解決(core の flow/next_step.rs。公開から pub(crate) に)。遷移先の位置を引く関数を引数で受け取る。classify_and_decide は job_index を、flow_edges は位置の表を渡す。
  - name: merge_env_names
    description: 環境変数の名前と出どころだけの要約(core の envfile/merge.rs。新規。pub(crate))。fn(def, job, files, ctx) -> Vec<(&str, EnvOrigin)>。値を読まない。並べ方と最後に残す規則は merge_env と共有する。

  # ---- cli ----
  - name: Checked
    description: 実行の前の検査を終えた材料(cli の ctx.rs。非公開)。定義の部分を base: CheckedDefinition にまとめる。
    attributes:
      - name: base
        logical_type: CheckedDefinition { path, definition, commands }
      - name: run_id / state / remove_on_complete / env_files
        logical_type: 今のまま
  - name: Ctx::prepare_session
    description: 本番(run・pipeline)の準備の入口(cli の ctx.rs。新規。pub(crate))。シグナルの監視を始めてから、渡された検査(check_job か check_pipeline)を呼び、(Session, T) を返す(D3。決定(ユーザー、2026-11-23))。prepare_run・prepare_pipeline を消す。
  - name: SkipRecord
    description: スキップの記録の結果(cli の step.rs。新規。pub(crate))。
    attributes:
      - name: state
        logical_type: PipelineState(記録後)
      - name: written
        logical_type: bool(土台から変わり、書いた)
  - name: messages の関数の形
    description: cli の report/messages.rs(pub(crate) に)。固定の文の定数 Text、種類ごとの fn(kind) -> Text、値を埋める fn(lang, …) -> String の 3 つ。definition_kind・env_file_kind・env_file_reason・conversion_kind・state_kind・plan_disposition・class_word は fn(…) -> Text に。

  # ---- runtime ----
  - name: run_job
    description: 1 つのジョブの起動(runtime の runner.rs。公開)。job を受け取らない fn(line: &CommandLine, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution(v0.9.6 までは job を受け取った)。

  # ---- converter ----
  - name: reason の文(converter の error.rs の reason)
    description: 文法の誤りの理由。SYNTAX を消し、TOML_SYNTAX(TOML として読めません)、YAML_SYNTAX(YAML として読めません)、BROKEN_STRUCTURE(入力の構造を解釈できません。通常は使われない)を置く(D4、D5。決定(ユーザー、2026-11-23))。

  # ---- テストの補助 ----
  - name: テストの補助の関数(exec、exited、outcome、step)
    description: 組み立ての関数から job が外れて使われなくなった引数は、名前を _ で始めて残し、呼ぶ側を変えない(functional-spec.md §4)。
  - name: crates/core/tests/plan_simulate_perf.rs、graph_lookup_perf.rs、env_parse_perf.rs
    description: 新規の性能のテスト(N4〜N8。N9(P6)は D6 の決定で足さない)。common::median で 5 回の中央値を、直した後の見積もりの約 5 倍の余裕のある上限(P1・P2 150 ms、P3 350 ms、P4 600 ms、P5 100 ms)と比べる。plan_simulate_perf.rs は N4・N5 の 2 件。
```
