# エンティティ(Entities): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

永続のデータ(ステートファイル)の形は変えない。型を足し、項目を場合分けに移し、関数の引数を変える。詳細は `inventory.md` と `functional-spec.md` §2。

```yaml
entities:
  - name: ExecutionResult
    description: 1 回の起動の結果の場合分け(core の types.rs。新規。conductor_core のルートから公開する。lib.rs の pub use types::* に含まれ、use conductor_core::ExecutionResult で使える)。derive は Debug、Clone、Copy、PartialEq、Eq。classify、retry_decision、apply_outcome、cli の step.rs と retry.rs が match で読む。ステートには書かない。名前は D5 で決定(ユーザー、2026-11-23)。
    attributes:
      - name: NotStarted
        logical_type: '{ signal: i32 }。起動しなかった(起動前の合図、またはリトライの待機中の合図)。signal は最初に受け取ったシグナルの番号'
      - name: StartFailed
        logical_type: 起動に失敗した(spawn の失敗、終了の状態を取得できない、空のプログラム名)。終了コード・シグナル・中断を持たない
      - name: Exited
        logical_type: '{ exit_code: Option<i32>, terminated_by_signal: Option<i32>, interrupted_by: Option<i32> }。起動して終了を観測した。interrupted_by は観測の直後に合図が立っていたときの番号'
  - name: JobExecution
    description: 1 回の起動の結果(core の types.rs。既存。契約 C1)。項目を job_id、command、result の 3 つにする。started、start_failure、interrupted、signal_number を消し、exit_code と terminated_by_signal は ExecutionResult::Exited の中に移す。直列化しない(今と同じ)。
    attributes:
      - name: job_id
        logical_type: String(変わらない)
      - name: command
        logical_type: Command(変わらない。定義の command そのもの)
      - name: result
        logical_type: ExecutionResult(新規)
  - name: JobExecution::new / exited / not_started / start_failed
    description: 組み立ての関数(core の types.rs)。exited は既存(--dry-run の模擬が使う)。new、not_started、start_failed は新規で、runtime の run_job、cli の retry.rs、テストの偽物が使う。今の項目と同じ意味の読み出しの関数は置かない。
  - name: run_job
    description: 'ジョブの起動と待ち(runtime の runner.rs。既存。公開)。署名を (job, line: &CommandLine, env, signal) -> JobExecution にする。中で command を分けない。line.program が空・空白だけなら StartFailed(この確認は起動前の合図の確認より前)。'
  - name: Runtime::run_job
    description: 'cli の差し替え口のトレイト(cli の deps.rs。既存。公開)。同じく line: &CommandLine を受け取る。SystemRuntime はそのまま runtime の run_job に渡す。テストの偽物(ScriptedRuntime、MemRuntime)は受け取って使わない。'
  - name: run_job_with_retry
    description: 'リトライを含む実行(cli の retry.rs。既存。非公開)。line: &CommandLine を受け取り、試行ごとに同じものを Runtime::run_job に渡す。待機中の合図の結果は JobExecution::not_started で作る。'
  - name: CommandLine
    description: 分割したコマンド(runtime の types.rs。既存。変えない)。program と args は公開のまま(D4 の決定。空のプログラム名は run_job の 1 行の防御で StartFailed)。正規の作り方は check_command だけ。
  - name: command_line(split.rs)
    description: 分割の内部関数(runtime。既存。非公開)。呼ぶのは check_command だけになる(run_job から外れる)。関数は残すか、check_command に畳む。
  - name: read_definition
    description: 定義ファイルの読み取り(core の definition/intake.rs。既存。公開)。順序を「種別だけパスで見て(D1 で形 B に決定)→ 開く → 開いたハンドルで種別と大きさ → 上限 + 1 バイトまで読む → UTF-8」にする。戻り値の型と DefinitionErrorKind の種類、理由の文は変えない。load_definition と cli の run_id_env が使う。
  - name: DefinitionErrorKind(fileMissing / unreadable / tooLarge)
    description: 取り込みの誤りの種類(core。既存。変えない)。読む権限のない 1 MiB 超のファイルは tooLarge ではなく unreadable になる(利用者から見える変更)。
```
