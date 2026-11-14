# Entities(u1-core)

上流: unit-of-work(U1 u1-core の責務)、components(エンティティの所有)、contract-summary(C1 共有する型、C2 呼び出し)、requirements。この Unit は、定義(DefinitionLoader)と判断(Flow)を受け持ち、共有する型の定義(データの形だけ)をすべて置く。エンティティの所有(振る舞い)は components.md のままである。技術に依存しない記述であり、言語やデータ形式は定めない。

```yaml
entities:
  - name: PipelineDefinition
    owner: DefinitionLoader
    description: 定義ファイル1つに書かれたパイプラインの定義。1ファイルに1つだけ置く
    attributes:
      - { name: name, type: text, required: true, unique: true, constraints: 空でない。pipeline コマンドに渡す名前と一致させる }
      - { name: description, type: text, required: false, default: なし }
      - { name: formatVersion, type: integer, required: true, constraints: 対応する版(初版は 1)のみ }
      - { name: jobs, type: ordered list of JobDefinition, required: true, constraints: 1つ以上。定義ファイルに書いた順を保つ。ジョブIDは一意 }
    relationships:
      - { to: JobDefinition, cardinality: one-to-many, direction: PipelineDefinition -> JobDefinition }

  - name: JobDefinition
    owner: DefinitionLoader
    description: 1つのジョブの定義(何を実行し、結果をどう分類し、どこへ進むか)
    attributes:
      - { name: id, type: text, required: true, unique: true, constraints: 空でない。定義の中で一意。"@end" で始めない }
      - { name: command, type: text, required: true, constraints: 空でない。コマンド名と引数に分けて直接起動する書式 }
      - { name: warningExitCodes, type: list of integer, required: false, default: 空, constraints: successExitCodes と重ならない。0 を含まない }
      - { name: successExitCodes, type: list of integer, required: false, default: "[0]", constraints: 書いた場合は、その値だけが成功。warningExitCodes と重ならない }
      - { name: transitions, type: list of Transition, required: false, default: 空, constraints: on の値ごとに高々1つ }
    relationships:
      - { to: Transition, cardinality: one-to-many, direction: JobDefinition -> Transition }

  - name: Transition
    owner: DefinitionLoader
    description: 結果ごとの遷移ルール(JobDefinition の一部で、独立した識別子を持たない値)
    attributes:
      - { name: on, type: enum, required: true, allowed: [success, warning], constraints: error は書けない }
      - { name: next, type: text, required: true, constraints: 記述順で後ろのジョブID、または "@end" のみ }

  - name: DefinitionError
    owner: DefinitionLoader
    description: 定義の誤りの報告(1回の検証で複数を返してよい)
    attributes:
      - { name: file, type: text, required: true }
      - { name: line, type: integer, required: false }
      - { name: kind, type: enum, required: true, allowed: [fileMissing, unreadable, typeMismatch, unknownKey, versionMissing, versionUnsupported, unknownTarget, errorTransition, backwardOrSelfTarget, overlappingExitCodes, zeroAsWarning, pipelineNameMismatch, duplicateJobId, invalidJobId, noJobs, emptyField, duplicateTransition] }
      - { name: reason, type: text, required: true }

  - name: JobExecution
    owner: JobRunner
    description: 1回のジョブの起動の結果(型の定義だけを U1 に置く。起動と値の生成は U2 の責務)
    attributes:
      - { name: jobId, type: text, required: true, references: JobDefinition.id }
      - { name: command, type: text, required: true }
      - { name: started, type: boolean, required: true, constraints: 合図が呼び出しの前に立っていて起動しなかったときだけ false }
      - { name: exitCode, type: integer, required: false, constraints: started が true で、終了コードがあるときのみ }
      - { name: terminatedBySignal, type: integer, required: false, constraints: シグナルで終了したときの、シグナル番号 }
      - { name: startFailure, type: boolean, required: true, default: false }
      - { name: interrupted, type: boolean, required: true, default: false, constraints: conductor がシグナルを受け取り、中断の合図が立ったとき }
      - { name: signalNumber, type: integer, required: false, constraints: interrupted が true のとき、受け取ったシグナルの番号(判断に渡す) }
    relationships:
      - { to: JobDefinition, cardinality: many-to-one, direction: JobExecution -> JobDefinition }

  - name: JobState
    owner: StateStore
    description: パイプラインの中の1つのジョブの、これまでの状態(型の定義だけを U1 に置く)
    attributes:
      - { name: jobId, type: text, required: true, references: JobDefinition.id }
      - { name: status, type: enum, required: true, allowed: [pending, success, warning, error, skipped], default: pending }
      - { name: exitCode, type: integer, required: false }
      - { name: skipReason, type: enum, required: false, allowed: [byTransition, byEnd], constraints: status が skipped のときのみ。byTransition は遷移によるスキップ、byEnd は @end によるスキップ }
    relationships:
      - { to: JobDefinition, cardinality: many-to-one, direction: JobState -> JobDefinition }

  - name: PipelineState
    owner: StateStore
    description: 定義ファイル1つに対する、実行状態の全体(型の定義だけを U1 に置く)
    attributes:
      - { name: definitionKey, type: text, required: true, unique: true, constraints: 仮。確定は u2-runtime の Functional Design }
      - { name: pipelineName, type: text, required: true }
      - { name: definitionPath, type: text, required: true }
      - { name: jobOrder, type: ordered list of text, required: true, constraints: 記録時のジョブID と記述順 }
      - { name: jobStates, type: list of JobState, required: true }
      - { name: resumeJobId, type: text, required: false, constraints: 次に実行する、または再開するジョブのID }
      - { name: endReached, type: boolean, required: true, default: false }
      - { name: completed, type: boolean, required: true, default: false }
    relationships:
      - { to: JobState, cardinality: one-to-many, direction: PipelineState -> JobState }

  - name: Decision
    owner: Flow
    description: ジョブを起動する前の判断の結果(契約 C1 の値)
    attributes:
      - { name: value, type: enum, required: true, allowed: [run, skip, reject] }
      - { name: reason, type: enum, required: false, allowed: [skipTarget, afterEnd, alreadyExecuted, outOfOrder], constraints: skip と reject のときに理由を持つ。outOfOrder は reject のときのみ }

  - name: JobOutcome
    owner: Flow
    description: 実行したジョブの結果の分類と、スケジューラへ返す終了コード
    attributes:
      - { name: jobId, type: text, required: true, references: JobDefinition.id }
      - { name: classification, type: enum, required: true, allowed: [success, warning, error] }
      - { name: reportedExitCode, type: integer, required: true, constraints: 規則 BR2.2・BR2.3・BR2.4 による。ジョブの終了コードがそのまま返る場合は、64〜67 などの conductor 自身の終了コードと重なりうる(Assumptions を参照) }
      - { name: conductorError, type: boolean, required: true, default: false, constraints: 成功コードに含まれない 0 のときなど、conductor 自身が原因を表示するエラーのとき true }
    relationships:
      - { to: JobExecution, cardinality: one-to-one, direction: JobOutcome -> JobExecution }

  - name: NextStep
    owner: Flow
    description: 結果の分類を受けた、次の動き
    attributes:
      - { name: jobId, type: text, required: true, references: JobDefinition.id }
      - { name: nextJobId, type: text, required: false, constraints: 次に実行するジョブ。完了または停止のときは無し }
      - { name: skippedJobIds, type: list of text, required: true, default: 空 }
      - { name: skipReasonKind, type: enum, required: false, allowed: [byTransition, byEnd] }
      - { name: endReached, type: boolean, required: true, default: false }
      - { name: completed, type: boolean, required: true, default: false }
      - { name: resumeJobId, type: text, required: false, constraints: エラーで停止したときは、そのジョブ }
    relationships:
      - { to: JobDefinition, cardinality: many-to-one, direction: NextStep -> JobDefinition }
      - { to: JobState, cardinality: many-to-many, direction: NextStep -> JobState }
```

## 要約

- U1 が値の「生成」を受け持つのは、PipelineDefinition、JobDefinition、Transition、DefinitionError(DefinitionLoader)と、Decision、JobOutcome、NextStep(Flow)。JobExecution、JobState、PipelineState は、型の定義だけを U1 に置き、値の生成と保持は U2 が行う。 [contract-summary の C1、unit-of-work-dependency]
- 識別子の意味: ジョブIDは、定義の中で一意である。PipelineState の `definitionKey` は仮で、u2-runtime の Functional Design で確定する。

## Assumptions & Open Questions

- `definitionKey` の確定(別パスの定義ファイルとステートの紐づけ)は、u2-runtime の Functional Design で行う。 [assumption]
- ジョブの終了コードをそのまま返す場合(エラー)は、conductor 自身のエラーの終了コード(64〜67)と、同じ値になりうる。スケジューラは、0 / 非0 しか見ないため、実害は小さい。完全に区別するには、ジョブの終了コードの範囲を制限する必要があるが、要件にはないため、行わない。 [assumption]
- 論理的な型(text、integer など)は、技術に依存しない記述であり、実装の型への対応は Code Generation で決める。 [assumption]
