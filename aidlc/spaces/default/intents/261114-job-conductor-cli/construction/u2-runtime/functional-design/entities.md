# Entities(u2-runtime)

上流: unit-of-work(U2 u2-runtime の責務)、components(JobRunner、StateStore)、contract-summary(C1、C3、E3)、requirements、この Unit の質問への回答(Q1〜Q4)。この Unit は、ジョブの直接起動とシグナルの転送(JobRunner)、ステートの保持・照合・初期化(StateStore)を受け持つ。**共有する型(JobExecution、JobState、PipelineState)の形は、u1-core の entities.md が定める**(契約 C1。この Unit は、その値を生成し、保持する)。ここには、この Unit が足す型と、ステートファイルの形(契約 E3)を置く。技術に依存しない記述であり、言語やデータ形式は(ステートファイルの形を除いて)定めない。

```yaml
entities:
  - name: CommandLine
    owner: JobRunner
    description: ジョブの command の文字列を、コマンド名と引数に分けたもの(シェルを介さずに起動する)
    attributes:
      - { name: program, type: text, required: true, constraints: 空でない }
      - { name: args, type: list of text, required: true, default: 空, constraints: 引数は、1つずつ、そのままの文字列で渡される(メタ文字を解釈しない) }

  - name: CommandError
    owner: JobRunner
    description: command の文字列を分けられないときの報告(定義の誤りとして、ジョブを起動する前に検知できる)
    attributes:
      - { name: kind, type: enum, required: true, allowed: [unclosedQuote, empty] }
      - { name: reason, type: text, required: true, constraints: 固定の文。入力の値は含めない }

  - name: InterruptSignal
    owner: JobRunner
    description: Cli が用意する読み取り専用の中断の合図(契約 C3 の interruptSignal)。立っているあいだ、受け取ったシグナルの番号を運ぶ
    attributes:
      - { name: raised, type: boolean, required: true, default: false }
      - { name: signalNumber, type: integer, required: false, constraints: raised が true のとき、最初に受け取ったシグナルの番号。Windows では 2(Ctrl+C のみ。Q4: A) }

  - name: StateLocation
    owner: StateStore
    description: ステートファイルの場所(Q1: A)
    attributes:
      - { name: path, type: text, required: true, constraints: 定義ファイルと同じディレクトリの `<定義ファイル名>.state`(例: `conductor.toml.state`) }

  - name: StateError
    owner: StateStore
    description: ステートの読み書きの失敗の報告(契約 C3 の failure)
    attributes:
      - { name: path, type: text, required: true }
      - { name: kind, type: enum, required: true, allowed: [unreadable, tooLarge, corrupt, versionMismatch, writeFailed, removeFailed] }
      - { name: reason, type: text, required: true, constraints: 固定の文。入力の値と、ライブラリのエラー文は含めない }

  - name: ConsistencyResult
    owner: StateStore
    description: ステートと定義の照合の結果(契約 C3 の check_consistency)
    attributes:
      - { name: value, type: enum, required: true, allowed: [ok, mismatch] }
      - { name: mismatchKind, type: enum, required: false, allowed: [pipelineName, jobIds, jobOrder], constraints: mismatch のときのみ。pipelineName はパイプライン名の違い、jobIds はジョブIDの追加・削除・改名、jobOrder は記述順だけの違い }

  - name: StateFile
    owner: StateStore
    description: ステートファイルの内容(契約 E3。TOML。書式の版つき)。PipelineState を、そのまま書き出したもの
    attributes:
      - { name: format_version, type: integer, required: true, constraints: 初版は 1。異なる版(古い版も、新しすぎる版も)は、変換せずエラー }
      - { name: definition_key, type: text, required: true, constraints: 定義ファイルのパス(記録した文字列)。表示と診断のためで、照合には使わない(規則 BR2.5) }
      - { name: definition_path, type: text, required: true }
      - { name: pipeline_name, type: text, required: true }
      - { name: job_order, type: list of text, required: true, constraints: 記録時のジョブIDと記述順 }
      - { name: jobs, type: list of table, required: true, constraints: "各行は { id, status, exit_code(省略可), skip_reason(省略可) }。status は pending / success / warning / error / skipped。skip_reason は byTransition / byEnd(status が skipped のときのみ)" }
      - { name: resume_job_id, type: text, required: false }
      - { name: end_reached, type: boolean, required: true, default: false }
      - { name: completed, type: boolean, required: true, default: false }
```

## 要約

- JobExecution の「生成」は JobRunner が、JobState と PipelineState の「生成と保持」は StateStore が行う。共有する型の形は、u1-core が定める(entities.md、契約 C1)。
- `definitionKey`(仮だったもの)は、**定義ファイルのパス(記録した文字列)**と確定し、**照合には使わない**(ステートファイルの場所が、定義ファイルのパスで決まるため。Q1: A)。照合は、ジョブIDと記述順とパイプライン名で行う(BR2.5)。
- ステートファイルは、TOML で、契約 E3 のキー名を確定する(`format_version`、`definition_key`、`definition_path`、`pipeline_name`、`job_order`、`jobs`、`resume_job_id`、`end_reached`、`completed`)。

## Assumptions & Open Questions

- `definition_path` を、E3 の項目に足した(PipelineState の `definitionPath` に対応する)。契約 E3 の更新は、u3-cli の設計で行う。 [assumption]
- 論理的な型(text、integer など)は、技術に依存しない記述であり、実装の型への対応は Code Generation で決める。 [assumption]
