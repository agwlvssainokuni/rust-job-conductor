# エンティティ(Entities): 動きを変えない整理のリファクタリング

永続のデータとステートファイルの形は変えない。型の置き場所と、内部の型をいくつか足す。

```yaml
entities:
  - name: CheckedDefinition
    description: 検査済みの定義と、分割したコマンド(cli の内部。新規)。check_definition と check_definition_and_env が返す。
    attributes:
      - name: path
        logical_type: 定義ファイルのパス
      - name: definition
        logical_type: PipelineDefinition
      - name: commands
        logical_type: list of CommandLine(ジョブと同じ並び)
  - name: StateRead
    description: ステートを読む方法(cli の内部。新規)。Live は read_state(完了は空)、Raw は read_state_raw(完了も読む。graph --state)。
  - name: ConsistencyResult / MismatchKind
    description: ステートと定義の照合の結果(既存。runtime から core へ移る。runtime は再エクスポート)。
  - name: DisplayEvent
    description: 表示のイベント(既存。cli の公開の型)。JobResult の result を Option<Classification> に、Transition の on を TransitionOn に変える(フィールド名は同じ)。
  - name: JobExecution::exited
    description: 終了コードで終わった実行結果を作る関数(core。新規)。
```
