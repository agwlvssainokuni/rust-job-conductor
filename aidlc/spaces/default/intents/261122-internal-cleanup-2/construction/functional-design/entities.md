# エンティティ(Entities): 動きを変えない整理の第 2 弾

永続のデータ(ステートファイル)の形と、利用者に見える出力は変えない。型を足し、分け、別名にし、置き場所を移す。詳細は `inventory.md`。

```yaml
entities:
  - name: SkipCause
    description: 起動前の判断でスキップする理由(core の types.rs。新規。DecisionReason のうちスキップの 3 つ)。Decision::Skip、apply_skip、DisplayEvent::JobSkipped、messages::skip_reason が使う。ステートに書く SkipReason(ByTransition、ByEnd)とは別の型。
    attributes:
      - name: SkipTarget
        logical_type: 遷移によるスキップの対象(BR3.5 の 1)
      - name: AfterEnd
        logical_type: '@end より後ろ(BR3.5 の 2)'
      - name: AlreadyExecuted
        logical_type: 成功・警告で実行済み(BR3.5 の 3)
  - name: RejectReason
    description: 起動前の判断で拒否する理由(core の types.rs。新規。DecisionReason のうち拒否の 1 つ)。Decision::Reject、PlanRejection.reason、DisplayEvent::Rejected、messages::reject_reason が使う。
    attributes:
      - name: OutOfOrder
        logical_type: 順序外の呼び出し(BR3.5 の 7、BR3.8。終了コード 66)
  - name: DecisionReason
    description: 消す(SkipCause と RejectReason に分かれる)。
  - name: RunIdArg
    description: --run-id の指定(cli の types.rs。新規。lib.rs で再公開)。CliCommand.run_id の型を Option<String> から Option<RunIdArg> にする(None は指定なしで、環境変数を見る)。検査は引数の段階の 1 回(RunIdArg::parse)。
    attributes:
      - name: Empty
        logical_type: --run-id に空を書いた(run ID なしに固定。環境変数を見ない)
      - name: Id
        logical_type: RunId(規則を満たす値)
  - name: TransitionTarget::END_LABEL / label
    description: 終点の語 @end の定数と、行き先の表示の語(core の types.rs。新規)。walk、cli の表示、graph の text と Mermaid が使う。
  - name: DisplayEvent::Transition
    description: 表示のイベント(既存。cli の公開の型)。target を &'a str から &'a TransitionTarget に変える(フィールド名は同じ)。表示は target.label() をエスケープしたもので、1 文字も変わらない。
  - name: DisplayEvent::Rejected / JobSkipped
    description: 表示のイベント(既存)。reason の型を DecisionReason から RejectReason / SkipCause に変える(フィールド名は同じ)。
  - name: PlanMode
    description: RunMode の別名にする(core の plan/mod.rs の pub use crate::types::RunMode as PlanMode。定義は消す)。ExecutionPlan.mode の型は RunMode。
  - name: EdgeKind / EdgeTarget
    description: TransitionOn / TransitionTarget の別名にする(core の graph/mod.rs の pub use … as。定義は消す)。FlowEdge の to と kinds の型はそれぞれ TransitionTarget と Vec<TransitionOn>。
  - name: From<Classification> for JobStatus
    description: 分類から状態への写し(core の types.rs。新規)。apply_outcome が使う。
  - name: TransitionOn::parse
    description: success・warning の語から戻す(core の types.rs。新規。as_str の逆。知らない語は None)。walk が使う。
  - name: Session
    description: 1 回の実行の材料(cli の ctx.rs。既存)。checked(Checked)と signal(Arc<InterruptSignal>)の 2 つのフィールドにする。Checked の 7 つのフィールドの写しをやめる。
  - name: escape_display
    description: 入力に由来する文字列の表示用の整形(上限なしも選べる)。本体を cli から core の error.rs に移す(core の sanitize はこれを呼ぶ)。cli の report::escape は再公開と包みだけ。converter の写しは残す。
  - name: resolve_run_id / aux_run_id
    description: run ID の解決(cli)。args.rs と ctx.rs から、新しい公開のモジュール run_id.rs に移す。resolve_run_id の引数は Option<&RunIdArg> になる。
  - name: report_execution / result_detail
    description: ジョブの実行のあとの表示(cli)。ctx.rs から step.rs に移す(非公開)。
```
