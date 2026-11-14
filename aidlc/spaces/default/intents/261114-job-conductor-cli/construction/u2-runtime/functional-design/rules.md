# Rules(u2-runtime)

上流: requirements(FR2、FR4、FR6、NFR1〜NFR3)、contract-summary(C3、E3)、entities(この Unit の entities.md)、u1-core の rules(分類・次の動き。この Unit が、その結果を保持する)、この Unit の質問への回答(Q1〜Q4)。各規則は、`BR{group}.{seq}` の ID を持ち、出典(FR / NFR / Q)を示す。

```yaml
rules:
  # グループ1: ジョブの起動(JobRunner)
  - { id: BR1.1, statement: command の文字列は、3つの OS で同じ規則で、コマンド名と引数に分ける, category: policy, applies_to: CommandLine, trigger: ジョブの起動前(および、定義の検査時), logic: "空白(スペース・タブ)で区切る。`'...'` の中は、そのまま1つの引数(空白を含められる)。`\"...\"` の中は、`\\\"` と `\\\\` だけを解釈し、ほかは、そのまま。引用符で囲んでいない部分と、引用符で囲んだ部分は、つなげて1つの引数にする(例: `--msg='a b'` は `--msg=a b`)。先頭の語が、コマンド名。変数の展開、`*` などの展開、`|` `;` `&&` `>` などの解釈はしない(それらは、普通の文字として、引数に入る)。閉じていない引用符は、CommandError(unclosedQuote)。空の command(空白だけを含む)は CommandError(empty)", violation: "起動前に検知でき、ジョブは起動しない。呼び出し側(Cli)は、全ジョブの command を、実行の前に検査し、誤りを定義の誤りとして報告する", source: FR2.1、AC2.1.1、Q3 }
  - { id: BR1.2, statement: ジョブは、シェルを介さずに、直接起動する, category: policy, applies_to: JobExecution, trigger: ジョブの起動時, logic: "BR1.1 で分けたコマンド名を実行ファイルとして、引数を、1つずつ、そのまま渡して起動する。引数に含まれるメタ文字(`;` `&&` `|` など)は、解釈されない", violation: なし, source: FR2.1、AC2.1.1 }
  - { id: BR1.3, statement: 作業ディレクトリ、環境変数、標準入出力は、conductor から引き継ぐ, category: policy, applies_to: JobExecution, trigger: ジョブの起動時, logic: "ジョブは、conductor と同じ作業ディレクトリ、同じ環境変数で起動する。標準入力・標準出力・標準エラーは、そのまま引き継ぐ(ジョブの出力を、加工しない)", violation: なし, source: AC2.1.2、FR2.1(要件の Assumptions) }
  - { id: BR1.4, statement: ジョブの終了を、終了コード・起動失敗・シグナル終了に区別して、JobExecution で返す, category: calculation, applies_to: JobExecution, trigger: ジョブの終了時, logic: "起動できた(started が true)。終了コードがあれば exitCode に入れる。実行ファイルが見つからない、実行できない(権限など)ときは startFailure を true にし、exitCode と terminatedBySignal は無し。シグナルで終了したとき(UNIX 系)は、terminatedBySignal にシグナル番号を入れ、exitCode は無し。Windows では、terminatedBySignal を持たない(終了は、すべて終了コードとして扱う。Q4: A)", violation: なし(分類とエラーの判断は u1-core の Flow), source: FR2.1、FR2.2、AC2.1.5、Q4 }
  - { id: BR1.5, statement: ジョブの実行に、時間の上限を設けず、終了を待つ。自動の再試行はしない, category: policy, applies_to: JobExecution, trigger: ジョブの実行中, logic: "ジョブの終了まで待つ。時間切れにしない。失敗しても、この Unit は、自動で起動し直さない", violation: なし, source: FR2.7、AC2.5.1、AC4.3.4 }
  - { id: BR1.6, statement: 中断の合図が、起動の前に立っていれば、ジョブを起動しない, category: policy, applies_to: JobExecution, trigger: run_job の呼び出し時, logic: "IF InterruptSignal の raised が、起動の前に true THEN ジョブを起動せず、started を false、interrupted を true、signalNumber に合図の番号を入れた JobExecution を返す(exitCode と terminatedBySignal は無し)", violation: なし, source: FR2.9、contract-summary の C3 }
  - { id: BR1.7, statement: ジョブの実行中に中断の合図が立ったら、シグナルをジョブに転送し、終了を待つ, category: policy, applies_to: JobExecution, trigger: ジョブの実行中, logic: "UNIX 系: 受け取ったシグナル(合図の番号)を、ジョブのプロセスに、1回だけ送る。2回目以降の合図は、転送済みとして扱い、追加の動作はしない(強制終了しない)。そのあと、ジョブの終了を待つ。Windows(Q4: A): 対象は Ctrl+C(番号 2)だけ。同じコンソールのジョブには、OS が Ctrl+C を配るため、この Unit は追加の送信をせず、ジョブの終了を待つ。ジョブの終了を観測した直後に、合図が立っていれば、interrupted を true、signalNumber に合図の番号を入れる(ジョブ自身の終了コードや terminatedBySignal は、そのまま持つ)。観測の直後より後に立った合図は、次の呼び出しの前に立ったものとして扱う", violation: なし, source: FR2.9、AC2.5.6、Q4 }

  # グループ2: ステート(StateStore)
  - { id: BR2.1, statement: ステートファイルの場所は、定義ファイルのパスで決まる, category: policy, applies_to: StateLocation, trigger: ステートの読み書き時, logic: "定義ファイルと同じディレクトリの `<定義ファイル名>.state`(例: `conductor.toml.state`)。`--config` で別の定義ファイルを指定すれば、別のステートになる", violation: なし, source: FR4.1、AC4.1.1、Q1 }
  - { id: BR2.2, statement: ステートは、TOML の書式の版つきで保持する, category: policy, applies_to: StateFile, trigger: ステートの書き込み・読み込み時, logic: "entities.md の StateFile のキーで、TOML として書く。format_version は 1。キーは、知らないものがあればエラー(corrupt)", violation: なし, source: contract-summary の E3、Q2 }
  - { id: BR2.3, statement: ステートが読めない・壊れている・版が合わないときは、エラーを返し、ジョブを実行させない, category: validation, applies_to: StateError, trigger: read_state の呼び出し時, logic: "IF ファイルが存在しない THEN 空のステート(エラーではない)を返す. ELSE IF 読めない(権限など)THEN StateError(unreadable). ELSE IF 大きさが上限(1 MiB)を超える THEN StateError(tooLarge). ELSE IF TOML として読めない、キー・型・値が誤っている(未知のキー、status の値が不正、skip_reason が skipped でないジョブにある、job_order と jobs が食い違うなど)THEN StateError(corrupt). ELSE IF format_version が、この conductor の対応する版(1)と異なる THEN StateError(versionMismatch)(変換しない。reset を促す)。いずれも、path と固定の reason を持つ", violation: 呼び出し側(Cli)は、ジョブを実行せずに、path と原因を表示して、非0(65)で終了する, source: FR4.4、FR4.5、AC4.1.3、AC5.1.2 }
  - { id: BR2.4, statement: ステートの書き込みは、原子的に行い、中断されても壊れた内容を残さない, category: constraint, applies_to: StateFile, trigger: write_state の呼び出し時, logic: "同じディレクトリの一時ファイル(名前に、プロセス番号と連番を含める)に、全内容を書き、ディスクへの書き込みを確かめてから、ステートファイルへ、置き換えで移す(rename)。書き込みの途中で中断されても、ステートファイルは、直前の完全な内容のまま。残った一時ファイルは、読まない。書き込みに失敗したら StateError(writeFailed) を返し、一時ファイルを消す(消せなければ、そのまま)", violation: StateError(writeFailed)。呼び出し側は、非0(65)で終了する, source: FR4.1、AC4.1.2 }
  - { id: BR2.5, statement: ステートと定義の食い違いを検知する, category: validation, applies_to: ConsistencyResult, trigger: check_consistency の呼び出し時, logic: "IF ステートの pipeline_name が、定義の name と異なる THEN mismatch(pipelineName). ELSE IF job_order と、定義のジョブIDの集合が異なる(追加・削除・改名)THEN mismatch(jobIds). ELSE IF 集合は同じで、並びが異なる THEN mismatch(jobOrder)(AC4.4.3). ELSE ok。definition_key と definition_path は、照合に使わない(定義ファイルの置き場所の移動で、食い違いにしないため)", violation: 呼び出し側(Cli)は、ジョブを実行せずに、食い違いの原因と reset を促す表示をして、非0(65)で終了する。reset は、この検査を迂回する, source: FR4.6、AC4.4.1〜AC4.4.3 }
  - { id: BR2.6, statement: ステートの初期化(reset)は、検査を迂回して、ステートファイルを消す, category: policy, applies_to: StateFile, trigger: reset_state の呼び出し時, logic: "ステートファイルを読まず(破損、食い違い、版の不一致の検査をしない)、削除する。ファイルが存在しなければ、何もせず成功(エラーではない。AC6.1.2)。削除に失敗したら StateError(removeFailed)", violation: StateError(removeFailed), source: FR6.1、AC4.3.3、AC4.4.2、AC6.1.1、AC6.1.2 }
  - { id: BR2.7, statement: パイプラインが完了したら、ステートを初期化する(呼び出し側が、BR2.6 を呼ぶ), category: policy, applies_to: StateFile, trigger: NextStep の completed が true のとき, logic: "Cli が、結果を記録したあと、reset_state を呼ぶ。この Unit は、判断(completed)をせず、初期化の手段(BR2.6)だけを提供する。初期化に失敗したら、StateError(removeFailed)", violation: StateError(removeFailed), source: FR4.2、AC4.2.1〜AC4.2.4 }
  - { id: BR2.8, statement: 空のステートは、定義から作る。ジョブの結果と、スキップは、ステートに記録する, category: calculation, applies_to: PipelineState, trigger: ジョブの結果の記録時(apply_outcome)、スキップの記録時(apply_skip)、中断の記録時(apply_interrupt), logic: "初期のステート(ステートが無いとき): すべてのジョブが pending、resume_job_id は無し、end_reached と completed は false、job_order は定義のとおり。ジョブの結果の記録: JobOutcome と NextStep(u1-core が決めたもの)に従い、そのジョブの status(success / warning / error)と exit_code を書く。NextStep の skippedJobIds の各ジョブを skipped にし、skip_reason に skipReasonKind を書く。resume_job_id、end_reached、completed は、NextStep のとおりに書く(エラーなら resume_job_id はそのジョブ。成功・警告で、再実行のエラーが解けたら、無しに戻す)。スキップの記録(decide_before_run が skip のとき、呼び出し側が、そのジョブを記録する): そのジョブを skipped にし、理由(skipTarget は byTransition、afterEnd は byEnd。alreadyExecuted は、すでに記録済みのため、書き換えない)を書く。中断の記録(ジョブを起動しない、またはジョブのあいだの中断): resume_job_id などを、u1-core の resume_point_on_interrupt の結果のとおりに書く(エラーとしては記録しない)", violation: なし, source: FR4.1、FR4.3、AC2.1.6、AC2.2.1〜AC2.2.4、AC3.2.1〜AC3.2.3、AC4.1.1、AC4.3.1、AC4.3.2 }
  - { id: BR2.9, statement: ステートは、ジョブの結果ごと、スキップごとに、書き込む, category: policy, applies_to: StateFile, trigger: 各ジョブの結果・スキップ・中断のあと, logic: "呼び出し側は、記録するたびに write_state を呼ぶ(まとめて最後に1回、にしない)。途中でプロセスが落ちても、直前の記録までが残る", violation: なし, source: FR4.1、AC4.1.1 }
  - { id: BR2.10, statement: 排他制御はしない, category: policy, applies_to: StateFile, trigger: ステートの読み書き時, logic: "ファイルのロックをしない。同じ定義に対する同時の実行は、それぞれ実行される(ステートの整合は保証しない。原子的な置き換えにより、壊れた内容は残らないが、後に書いた方が勝つ)", violation: なし, source: FR2.8、AC2.5.2 }
```

## 規則の要約

| グループ | 範囲 | 主な規則 |
|----------|------|----------|
| BR1 | ジョブの起動(JobRunner) | BR1.1〜BR1.7(分け方、直接起動、引き継ぎ、結果の区別、時間の上限なし、起動前の中断、シグナルの転送) |
| BR2 | ステート(StateStore) | BR2.1〜BR2.10(場所、形式、読み取りの失敗、原子的な書き込み、食い違い、初期化、記録、排他なし) |

## Assumptions & Open Questions

- 大きさの上限(1 MiB。BR2.3)は、u1-core の定義ファイルの上限に合わせた、この Unit の仮の値である。NFR Requirements で確定する。 [assumption]
- BR1.7 の「観測の直後に合図が立っていれば interrupted」は、ジョブの終了とシグナルの到着が競合したときの、決め打ちの規則である(競合の幅は、極めて小さい)。ジョブが成功で終わっても、競合で interrupted になれば、そのジョブは、エラーとして扱われ、次の実行で、再実行される。 [assumption]
- BR1.1 の分け方の規則は、`shlex` 系のライブラリに頼らず、この Unit の中の小さな関数で実装する想定である(OS による違いを出さない。Q3: A)。具体は、NFR Requirements の技術選択で決める。 [assumption]
- BR2.8 の「記録する」関数(`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`)は、この Unit の公開の呼び出しに足す(契約 C3 には、まだ無い)。契約の更新は、u3-cli の設計で行う。 [assumption]
- command の検査(BR1.1 の CommandError)を、定義の読み込みのとき(全ジョブ)に行う呼び出しは、契約 C3 に足す(`check_command`)。u3-cli の設計で、定義の誤りとして扱う(終了コード 64)。 [assumption]
