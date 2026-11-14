# Contract Summary(約束事のまとめ)

上流: unit-of-work(4つの Unit)、unit-of-work-dependency(依存の DAG と接点)、components(エンティティの形)、requirements(FR1〜FR7、NFR1〜NFR6)。`conductor` は単一バイナリで、Unit どうしの連携は、プログラムの呼び出し(同期)である。ネットワークの API はない。そのため、約束事は、Unit どうしの境界(型と呼び出し)と、バイナリの外から見える境界(コマンド・終了コード、定義ファイル、ステートファイル)の2種類である。 [Q1: A]

## Contracts

| # | Provider Unit | Consumer | Mechanism | Owner |
|---|---------------|----------|-----------|-------|
| C1 | U1 u1-core | U2、U3 | 共有する型(データの形)の定義 | U1 |
| C2 | U1 u1-core | U3 | 同期の関数呼び出し(定義の読み込みと検証、結果の分類と次の動きの判断) | U1 |
| C3 | U2 u2-runtime | U3 | 同期の関数呼び出し(ジョブの起動、ステートの読み書きと照合) | U2 |
| C4 | U4 u4-converter | U3 | 同期の関数呼び出し(定義ファイルの変換) | U4 |
| E1 | U3 u3-cli | External: 既存のジョブスケジューラ、利用者(シェル) | コマンドライン(コマンド、オプション、終了コード、標準出力・標準エラー) | U3 |
| E2 | U1 u1-core | External: 定義ファイル(`conductor.toml`)を書く利用者 | ファイル形式(TOML、書式の版つき) | U1 |
| E3 | U2 u2-runtime | External: 利用者、後続の `conductor` の実行(同じステートを読む) | ファイル形式(書式の版つき) | U2 |

- C1〜C4 は、バイナリの内部の境界であり、言語は Rust である。契約は、型の形と呼び出しの意味(入力・出力・失敗)で記述し、言語固有の細部(所有権、ライフタイムなど)は、Functional Design 以降で決める。
- E1〜E3 は、外部の境界であり、書式の互換性を守る(下の「契約の所有と変更」を参照)。

## C1 共有する型の定義(U1 が提供し、U2、U3 が使う)

```yaml
# shared-schema: 型の「形」のみ。型の厳密な定義と制約は Functional Design で定める
types:
  PipelineDefinition:
    fields: [name, description, jobs]            # jobs は JobDefinition の記述順の並び
  JobDefinition:
    fields: [id, command, warningExitCodes, successExitCodes, transitions]
    transitions: [{ on: success|warning, next: <jobId>|@end }]   # on: error は書けない
  JobExecution:
    fields: [jobId, command, exitCode, terminatedBySignal, startFailure, interrupted, started]
    terminatedBySignal: シグナル番号(シグナルで終了していなければ無し)。128 + シグナル番号の計算に使う
    started: ジョブを起動したか。合図が呼び出しの前にすでに立っていて、起動しなかったときは false(このとき interrupted は true、exitCode と terminatedBySignal は無し)
  Decision:
    values: [run, skip, reject]
    reject: 理由つき(順序外などの条件は、未解決事項を参照)
  JobOutcome:
    fields: [jobId, classification, reportedExitCode]
    classification: success|warning|error
  NextStep:
    fields: [jobId, nextJobId, skippedJobIds, skipReasonKind, endReached, completed, resumeJobId]
  JobState:
    fields: [jobId, status, exitCode, skipReason]
    status: success|warning|error|skipped|pending
  PipelineState:
    fields: [definitionKey, pipelineName, definitionPath, jobStates, jobOrder, resumeJobId, endReached, completed]
    note: definitionKey は仮(Functional Design で確定)
```

- 型は、データの形だけを持ち、入出力や OS に依存しない(unit-of-work-dependency の「共有する型の置き場」を参照)。
- 追加のフィールドは、既存の呼び出し側を壊さない(読む側は、知らないフィールドを無視する)。この規則は、バイナリの内部の型にだけ適用する(外部のファイル形式は、E2・E3 の規則による)。

## C2 U1 が提供する呼び出し(U3 が使う)

```yaml
# shared-schema: 関数の「意味」。関数名と引数名は仮
operations:
  load_definition:
    input: [path, expectedPipelineName]   # path は既定で実行ディレクトリの conductor.toml、--config で指定。expectedPipelineName は pipeline コマンドで渡された名前(run では省略)
    output: PipelineDefinition
    failure: DefinitionError              # ファイル名・行・原因(複数を返してよい)。パイプライン名の不一致もここで返す(要件 FR1.5)
  decide_before_run:                      # ジョブを起動する「前」の判断
    input: [jobId, pipelineDefinition, jobStates]
    output: Decision
    Decision:
      run:     ジョブを実行する
      skip:    実行せず、終了コード 0 で終わる(スキップ対象、または @end より後。要件 FR2.4、FR4.2)
      reject:  実行しない(順序外などの理由つき。具体の条件は未解決事項を参照)
    failure: なし                         # 純粋な判断。入出力なし
  classify_and_decide:                    # ジョブを実行した「後」の判断
    input: [jobDefinition, jobExecution, jobStates, pipelineDefinition]
    output: [JobOutcome, NextStep]        # 終了コードの規則(警告は 0、成功コードに含まれない 0 は 1、起動失敗は 127、シグナルは 128 + シグナル番号)を含む。ジョブを起動して中断された実行(started が true で interrupted)は、エラー(128 + terminatedBySignal の番号)に分類し、そのジョブを再開位置にする。ジョブを起動しなかった実行(started が false)は、エラーとして扱わず、そのジョブを再開位置にし(ジョブとジョブのあいだの中断と同じ扱い)、スケジューラへ返す終了コードは、受け取ったシグナルの番号による 128 + シグナル番号とする
    failure: なし
  resume_point_on_interrupt:              # ジョブとジョブのあいだで中断されたときの判断
    input: [pipelineDefinition, jobStates]
    output: [resumeJobId, endReached, completed]   # resumeJobId は、次に実行するジョブ。最後のジョブの直後なら、resumeJobId は無しで、completed が true。@end に到達ずみなら endReached が true
    failure: なし
```

- 失敗の種類: `DefinitionError`(定義の誤り・パイプライン名の不一致。要件 FR1.4、FR1.5)。判断の3つの操作は、失敗せず、必ず結果を返す。
- 呼び出しの順序: Cli は、ジョブごとに、`decide_before_run` → (run のとき)`run_job` → `classify_and_decide` → ステートの書き込みの順に呼ぶ。skip のときは、`run_job` を呼ばず、スキップをステートに記録(必要なら)して、0 で終了する。
- タイムアウト・再試行: なし。

## C3 U2 が提供する呼び出し(U3 が使う)

```yaml
operations:
  run_job:
    input: [jobDefinition, interruptSignal]   # interruptSignal は Cli が用意する読み取り専用の中断の合図
    output: JobExecution                       # 終了コード、起動失敗、シグナル終了、中断(interrupted)を区別
    failure: なし(起動失敗は JobExecution の startFailure で返す)
    notes: >
      ジョブの標準出力・標準エラーは、そのまま通す。時間制限はない。
      中断の合図が立ったら、受け取ったシグナルをジョブに転送して終了を待ち、interrupted を立てて返す。
      呼び出しの前に、すでに合図が立っているときは、ジョブを起動せず、started が false で、interrupted の JobExecution を返す。
      2回目以降のシグナルは、転送済みとして扱い、追加の動作はしない(強制終了はしない)。
      合図は、シグナルの番号を運ぶ(terminatedBySignal の算出に使う)
  read_state:
    input: [stateLocation]
    output: PipelineState | empty
    failure: StateError                        # 読めない・壊れている・版が合わない(パスと原因)
  check_consistency:
    input: [pipelineState, jobIdsAndOrder]     # 定義のジョブID と記述順だけ
    output: ok | mismatch
  write_state:
    input: [stateLocation, pipelineState]
    failure: StateError
    notes: 原子的な更新(中断されても壊れた内容が残らない)
  reset_state:
    input: [stateLocation]
    failure: StateError                        # 書き込み・削除の失敗のみ
    notes: 破損・食い違い・版の不一致の検査を迂回して、初期化できる
```

- 失敗の種類: `StateError`(ステートの読み書きの失敗、破損、版の不一致)。起動の失敗は、失敗ではなく、`JobExecution` の値として返す。
- 報告の担当: `StateError` と、`check_consistency` の `mismatch` は、Cli が受け取り、Reporter に渡して表示し、ジョブを実行せずに非0で終了する(`reset` は、検査を迂回するため、対象外)。
- タイムアウト・再試行: なし。ジョブの実行時間の上限は設けない。

## C4 U4 が提供する呼び出し(U3 が使う)

```yaml
operations:
  convert:
    input: [inputPath, targetFormat]           # yaml | toml
    output: convertedText
    failure: ConversionError                   # 入力が読めない、文法が誤っている(原因)
    notes: >
      キーと値の意味が同じ。コメントと記述順は保持しない。変換した結果は、文字列として返す。
      その書き出し先(標準出力か、ファイルか)と、書き出しの失敗の報告は、Cli の責務とする
      (具体は設計で定める)。ConversionError は、Cli が Reporter に渡して表示し、非0で終了する
```

## E1 コマンドラインの約束事(U3 が提供し、スケジューラ・利用者が使う)

```yaml
commands:
  - conductor run <job-id>
  - conductor pipeline <pipeline-name>
  - conductor watch
  - conductor reset
  - conductor convert <file> --to <format>
options:
  --config <path>:  定義ファイルのパス(run, pipeline, watch, reset)
  -v, --verbose:    conductor 自身の表示を有効にする(run, pipeline)
exit_codes:
  success_or_warning: 0
  skipped: 0
  job_error: ジョブの終了コードをそのまま(非0)
  success_code_not_listed_zero: 1          # success_exit_codes に含まれない 0 で終了した場合
  command_not_started: 127
  killed_by_signal: 128 + シグナル番号
  conductor_own_error: 非0(定義・ステートの誤り、拒否(reject)。具体の値は設計で定める)
  interrupted_by_signal: 128 + 受け取ったシグナルの番号(ジョブを起動する前、起動中、ジョブとジョブのあいだのいずれも)
reject:
  動き: 拒否(reject)が返ったときは、ジョブを実行せず、理由を conductor 自身のエラーとして常に表示し、非0で終了する。ステートは変更しない
output:
  default: ジョブ本体の標準出力・標準エラーのみ。conductor 自身のエラーは常に表示
  language: 環境のロケール(英語・日本語)
```

- スケジューラが見るのは、終了コードだけ(0 / 非0)である(constraint-register の制約 C2。上の契約 C2 とは別のもの)。
- 破壊的変更(コマンド名・オプション名・終了コードの意味の変更)は、メジャーの変更として扱い、利用者に周知する。オプションの追加は、既存の利用を壊さない。

## E2 定義ファイル(`conductor.toml`)の約束事(U1 が提供し、利用者が書く)

```yaml
# shared-schema: 書式の「形」。具体の例は mockups.md を参照
format_version: 1                    # 書式の版。キーの名称は仮
pipeline:   { name, description }
jobs[]:
  id: string
  command: string                    # コマンド名と引数に分けて直接起動(シェルを介さない)
  warning_exit_codes: [int]          # 省略可
  success_exit_codes: [int]          # 省略可。省略時は 0 のみが成功。書けば、その値だけが成功
  transitions[]:                     # 省略可
    - { on: success|warning, next: <job-id>|"@end" }   # on: error は書けない
rules:
  - 遷移先は、記述順で後ろのジョブID、または "@end" のみ
  - 版のキー(format_version)は必須。無い定義は、エラー(版が必要、と表示)           # Q2: A
  - 版が、この `conductor` の対応する版より新しい定義は、エラーにする              # Q2: A
  - 初版では、対応する版は 1 のみ。将来、古い版の定義を読めるようにするかは、その版を出すときに決める
  - 検査の順序: まず版を確認し(版のエラーを先に出す)、次に知らないキー、その他の誤りを確認する
  - 知らないキーは、エラーにする(ファイル名・行・キーを表示)         # Q3: A
  - success_exit_codes と warning_exit_codes に同じ値を書くこと、0 を警告に指定することは、エラー
```

## E3 ステートファイルの約束事(U2 が提供し、`conductor` の後続の実行が読む)

```yaml
format_version: 1                    # 書式の版。キーの名称は仮
definition_key: <仮>                 # 定義ファイルとの紐づけ。Functional Design で確定
pipeline_name: string
job_order: [job-id]                  # 記録時のジョブIDと記述順(食い違いの検知に使う)
jobs[]:   { id, status: success|warning|error|skipped|pending, exit_code, skip_reason }
resume_job_id: job-id | null
end_reached: bool
completed: bool
rules:
  - 版が、この conductor の対応する版と異なるステート(古い版も、新しすぎる版も)は、変換せず、エラーにして reset を促す   # Q2: A
  - 書き込みは原子的に行う(中断されても壊れた内容が残らない)
  - 読めない・壊れている場合は、ジョブを実行せず、パスと原因を表示して非0で終了する
  - 定義のジョブ構成(IDと記述順)と食い違う場合は、ジョブを実行せず、reset を促す
```

## 契約の所有と変更

- **所有:** C1・C2・E2 は U1、C3・E3 は U2、C4 は U4、E1 は U3 が所有する。各契約の変更は、所有する Unit が提案し、その契約を使う Unit に影響を確認する。
- **内部の境界(C1〜C4):** 同じバイナリの中にあり、同時にビルドされるため、互換性を保つ必要はない。変更は、使う側の同時の修正と、テストで守る。追加は、既存の呼び出し側を壊さない形にする。
- **外部の境界(E1〜E3):**
  - 破壊的変更(キーの削除・意味の変更、コマンド・終了コードの意味の変更)は、書式の版を上げるか、メジャーの変更とする。
  - 定義ファイルは、新しいキーを足すと、古い `conductor` はエラーにする(知らないキーはエラー。Q3: A)。そのため、キーの追加も、書式の版を上げて扱う。
  - ステートファイルは、版が合わなければ、変換せずエラーにして、`reset` を促す。
- **エラーとタイムアウト:** どの境界にも、タイムアウトと自動の再試行はない。エラーは、失敗の種類(`DefinitionError`、`StateError`、`ConversionError`)として返し、表示は Cli(Reporter)が受け持つ。

## Open Questions

| Contract | Question | Blocks |
|----------|----------|--------|
| E2、E3 | 書式の版のキーの名称と、初版の値(`format_version = 1` は仮) | U1、U2 |
| E3 | ステートファイルの具体的な直列化の形式(テキストの構造化形式など)と、ファイルの名前・保存場所、`definitionKey` の確定 | U2 |
| E1 | `conductor` 自身のエラー(定義・ステートの誤り)の具体的な終了コードの値 | U3 |
| C2 | 個別実行で、順序外のジョブが呼ばれた場合と、実行済みのジョブが再び呼ばれた場合に、`decide_before_run` が `run`・`skip`・`reject` のどれを返すか(契約は、この3つの結果を持つことまでを確定する) | U1 |
| C1 | 共有する型を、U1 のどのまとまり(モジュール)に置き、どこまで公開するか | U1、U2、U3 |

## Assumptions & Open Questions

- 関数名・引数名・キーの名称(`format_version` など)は、約束事の「意味」を示すための仮のものであり、確定は Functional Design で行う。 [assumption]
- C1〜C4 の失敗の種類(`DefinitionError`、`StateError`、`ConversionError`)の、型の詳細は、Functional Design で定める。 [assumption]
