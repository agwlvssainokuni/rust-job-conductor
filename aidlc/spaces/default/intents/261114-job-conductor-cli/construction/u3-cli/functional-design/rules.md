# Rules(u3-cli)

上流: requirements(FR1〜FR7、NFR1〜NFR6)、contract-summary(E1、C2、C3、C4)、entities(この Unit の entities.md)、u1-core・u2-runtime・u4-converter の rules(この Unit は、それらの結果を、順に呼んで組み立てる)、Refined Mockups(コマンドの入出力のイメージ)、この Unit の質問への回答(Q1〜Q3)。各規則は、`BR{group}.{seq}` の ID を持ち、出典(FR / NFR / Q)を示す。

```yaml
rules:
  # グループ1: コマンドとオプション(Cli)
  - { id: BR1.1, statement: コマンドは、run・pipeline・watch・reset・convert の5つだけ, category: constraint, applies_to: CliCommand, trigger: 起動時の引数の解釈, logic: "`conductor run <job-id>`、`conductor pipeline <pipeline-name>`、`conductor watch`、`conductor reset`、`conductor convert <file> --to <yaml|toml>`。知らないコマンド、必須の引数がない、余分な引数、`--to` の値が yaml・toml 以外、は UsageError", violation: UsageError。ジョブを実行せず、使い方を標準エラーに表示して、終了コード 68 で終わる, source: E1、Q3 }
  - { id: BR1.2, statement: "`--config <パス>` は、run・pipeline・watch・reset で指定できる", category: constraint, applies_to: CliCommand, trigger: 引数の解釈, logic: "`--config` は、run、pipeline、watch、reset だけが受け付ける。値がなければ UsageError(missingValue)。convert に指定したら UsageError(optionNotAllowed)", violation: UsageError, source: FR1.6、AC1.3.2 }
  - { id: BR1.3, statement: "`-v` / `--verbose` は、run と pipeline だけが受け付ける", category: constraint, applies_to: CliCommand, trigger: 引数の解釈, logic: "watch、reset、convert に付けたら UsageError(optionNotAllowed)", violation: UsageError, source: E1、Refined Mockups の Q1 }
  - { id: BR1.4, statement: 定義ファイルの場所を決める, category: policy, applies_to: CliCommand, trigger: run・pipeline・watch・reset の実行時, logic: "`--config` があればそのパス。なければ、実行ディレクトリの `conductor.toml`。ステートの場所は、その定義ファイルのパスから、u2-runtime が決める(定義ファイルと同じディレクトリの `<定義ファイル名>.state`)", violation: なし, source: FR1.6、AC1.3.1、AC1.3.2 }
  - { id: BR1.5, statement: 配布の形は、単一の実行ファイル `conductor` である, category: constraint, applies_to: CliCommand, trigger: ビルド・配布, logic: "外部のランタイムに依存しない、OS ネイティブの実行ファイルとして、Linux・macOS・Windows の3つで動く。テスト専用の補助の実行ファイルは含めない", violation: なし, source: NFR2、NFR3、AC2.5.4 }

  - { id: BR1.6, statement: "`--help`・`-h` は使い方を、`--version`・`-V` は版を、標準出力に出して、終了コード 0 で終わる。引数のない `conductor` は、使い方を標準エラーに出して、終了コード 68", category: policy, applies_to: CliCommand, trigger: 引数の解釈, logic: "`--help`、`-h`、`--version`、`-V` は、コマンドの解釈より先に処理する。引数がない場合は、使い方の誤り(UsageError。missingArgument)", violation: なし, source: E1 }

  # グループ2: 個別実行(`conductor run <job-id>`)
  - { id: BR2.1, statement: 実行の前に、定義と command を検査する, category: validation, applies_to: CliCommand, trigger: run の開始時, logic: "(1) 定義ファイルを読んで検証する(u1-core の `load_definition`。パイプライン名は指定しない)。誤りがあれば、すべての誤り(ファイル・行・種類)を表示して、終了コード 64 で終わる(ジョブは実行しない。ファイルが存在しない、読めない場合も同じ。AC1.3.3)。(2) 定義のすべてのジョブの command を検査する(u2-runtime の `check_command`)。誤り(閉じていない引用符、空)があれば、定義の誤りとして、ジョブID と原因を表示して、終了コード 64 で終わる。(3) 指定された job-id が、定義にない場合は、UsageError(unknownJob)として、終了コード 68 で終わる", violation: 終了コード 64 または 68。ステートは変更しない, source: FR1.4、FR1.6、AC1.3.3、NFR4 }
  - { id: BR2.2, statement: 実行の前に、ステートを読み、定義との食い違いを検査する, category: validation, applies_to: PipelineState, trigger: run の、BR2.1 のあと, logic: "u2-runtime の `read_state` で読む。StateError(読めない、壊れている、版の不一致など)なら、path と原因を表示して、終了コード 65 で終わる(ジョブは実行しない)。空(`None`)なら、空のステートとして続ける。読めたら、`check_consistency` で定義と照合し、`Mismatch` なら、原因(パイプライン名、ジョブID、並び)と `conductor reset` を促す案内を表示して、終了コード 65 で終わる", violation: 終了コード 65。ジョブを実行せず、ステートを変更しない, source: FR4.4、FR4.5、FR4.6、AC4.1.3、AC4.4.1、AC4.4.3 }
  - { id: BR2.3, statement: ジョブを起動する前の判断に従う, category: policy, applies_to: Decision, trigger: run の、BR2.2 のあと, logic: "u1-core の `decide_before_run`(ジョブID、定義、各ジョブの状態、ステートの `resume_job_id` と `end_reached`)を呼ぶ。Reject なら、理由を表示して、終了コード 66 で終わる(ジョブを実行せず、ステートを変更しない)。Skip なら、BR2.4。Run なら、BR2.5", violation: なし, source: FR2.4、FR4.3、AC2.3.3、AC4.3.1 }
  - { id: BR2.4, statement: スキップのとき、起動せずに、ステートに記録し、0 で終わる, category: policy, applies_to: PipelineState, trigger: Decision が Skip のとき, logic: "u2-runtime の `apply_skip`(理由、定義つき)でステートに記録する。ステートが空(`None`)で、ここで初めて記録するなら、記録のあとに書く。`alreadyExecuted` は、記録を変えない(書き込みも省いてよい)。書き込み(`write_state`。`Durable`)に失敗したら、終了コード 65。記録したステートが `completed` なら、`reset_state` で初期化する(失敗は、終了コード 65)。-v のとき、スキップの表示を出す。終了コード 0", violation: 書き込み・初期化の失敗は 65, source: FR2.4、FR4.2、AC2.3.3、AC2.4.3、AC4.2.2、AC4.2.3 }
  - { id: BR2.5, statement: 実行のとき、初期ステートを書いてから、ジョブを起動し、結果を記録する, category: policy, applies_to: JobExecution, trigger: Decision が Run のとき, logic: "(1) ステートが空(`None`、または completed のまま残っていた)なら、`initial_state` を、`Fast` で書く(書けなければ、終了コード 65。ジョブは起動しない)。(2) 中断の合図を用意し(BR4.1)、u2-runtime の `run_job` を呼ぶ。(3) u1-core の `classify_and_decide`(ジョブ、JobExecution、各ジョブの状態、定義、実行の方式 Individual)で、JobOutcome と NextStep を得る。(4) `apply_outcome` で記録し、`write_state`(`Durable`)で書く(失敗は、終了コード 65。ジョブはすでに実行されている)。(5) NextStep が completed なら、`reset_state` で初期化する(失敗は 65)。(6) 終了コードは、BR5.1 で決める", violation: 書き込み・初期化の失敗は 65, source: FR2.1〜FR2.5、FR4.1、FR4.2、AC2.1.6、AC4.1.1、AC4.2.1 }
  - { id: BR2.6, statement: 成功コードに含まれない 0 で終わったジョブは、エラーとして表示する, category: policy, applies_to: JobOutcome, trigger: JobOutcome の conductorError が true のとき, logic: "原因(「終了コード 0 は、success_exit_codes に含まれない」)を、常に表示する。終了コードは 1", violation: なし, source: FR2.5、AC2.1.6 }

  # グループ3: 一括実行(`conductor pipeline <pipeline-name>`)
  - { id: BR3.1, statement: 実行の前の検査は、個別実行と同じ。パイプライン名も照合する, category: validation, applies_to: CliCommand, trigger: pipeline の開始時, logic: "BR2.1 の (1)(2) と BR2.2 を行う。ただし、`load_definition` に、`expectedPipelineName` として、引数のパイプライン名を渡す(不一致は、u1-core が定義の誤りとして返す。終了コード 64)", violation: 終了コード 64 または 65, source: FR1.5、FR3.1、AC1.2.5 }
  - { id: BR3.2, statement: ジョブを、記述順に、1つずつ、ステートに従って実行する, category: policy, applies_to: JobExecution, trigger: pipeline の実行中, logic: "ステートが空なら、最初のジョブの `decide_before_run` が Run を返したあと、初期ステートを書く(Fast。BR2.5 と同じ順序)。ジョブを、**定義の記述順に**、1つずつ取り出して、`decide_before_run` で決める(次のジョブを、NextStep の nextJobId で決めない。遷移先とスキップは、ステートが運ぶ)。Skip なら、`apply_skip` で記録して書き、次のジョブへ(`apply_outcome` が、すでに skippedJobIds を skipped と記録していても、同じ記録の再確認で、害はない。alreadyExecuted は何も変えない)。Run なら、BR2.5 の (2)〜(4)(実行の方式は Pipeline)。**ジョブの結果、またはスキップを記録するたびに、ステートの completed を確かめ、true なら、`reset_state` で初期化して、終了コード 0 で終わる**(最後のジョブの完了、`@end`、スキップで完了した場合のすべて)。記述順のジョブがなくなったのに completed でない場合(エラーの停止を除く)は、起こらない想定だが、起きたら、ステートを初期化せず、`conductor` 自身のエラー(ステートの誤り 65、「完了の判定ができない」)として、終了する。エラーで停止したジョブより前の、成功・警告で記録済みのジョブは、`alreadyExecuted` でスキップされ、エラーのジョブから再開する(AC4.3.2)", violation: Reject は、一連の実行では、通常は起きない。起きたら、終了コード 66, source: FR3.1、FR4.2、FR4.3、AC3.1.1、AC3.1.2、AC4.3.2 }

  - { id: BR3.3, statement: "`@end` と最後のジョブで完了し、ステートを初期化する", category: policy, applies_to: PipelineState, trigger: NextStep が completed のとき, logic: "一連の実行では、`@end` に到達した場合(実行の方式 Pipeline)も、最後のジョブが終わった場合も、その場で完了とし、`reset_state` で初期化し、終了コード 0 で終わる", violation: 初期化の失敗は 65, source: FR3.1、FR4.2、AC3.1.3、AC4.2.4 }
  - { id: BR3.4, statement: エラーで停止し、再試行しない, category: policy, applies_to: JobOutcome, trigger: 分類が error のとき, logic: "以降のジョブを実行せず、ステートにエラーの位置(`resume_job_id`)を残して、そのジョブの終了コードで終わる。自動の再試行はしない", violation: なし, source: FR3.2、AC3.2.1〜AC3.2.3 }

  # グループ4: シグナル(Cli)
  - { id: BR4.1, statement: ジョブを起動するコマンドは、シグナルを受け取る準備をして、中断の合図を用意する, category: policy, applies_to: InterruptSignal, trigger: run と pipeline の、ジョブの実行の前, logic: "UNIX 系: SIGHUP、SIGINT、SIGQUIT、SIGTERM を受け取る。Windows: Ctrl+C(番号 2)を受け取る。受け取ったら、中断の合図(u2-runtime の InterruptSignal)を、最初の番号で立てる(2回目以降は、番号を変えない)。シグナルの受け取りの準備は、定義の検査の前に行う(検査中に届いたシグナルが、失われないように)。watch、reset、convert では、準備しない(既定の動作で終わる)", violation: なし, source: FR2.9、AC2.5.6 }
  - { id: BR4.2, statement: 合図は、検査のあいだと、ジョブのあいだに確かめる, category: policy, applies_to: InterruptSignal, trigger: 定義とステートの検査のあと、ジョブの起動の前、一連の実行のジョブのあいだ, logic: "ジョブの起動の前に、合図が立っていれば、ジョブを起動しない(run_job が、started が false の JobExecution を返す)。一連の実行で、ジョブが終わったあと、次のジョブへ進む前に、合図が立っていれば、u1-core の `resume_point_on_interrupt` と `apply_interrupt` で、次に実行するジョブを再開位置として記録して(`write_state`)、終了する(エラーとしては記録しない)", violation: なし, source: FR2.9、AC2.5.6 }
  - { id: BR4.2a, statement: 検査・判断の途中で届いたシグナルの扱い, category: policy, applies_to: InterruptSignal, trigger: 定義とステートの検査のあいだ、判断のあいだ, logic: "合図が立っても、検査と判断は、最後まで進める。判断が Reject・Skip・定義の誤り・ステートの誤り・使い方の誤りの場合は、その終了コードが優先する(中断の終了コードにしない)。Run の場合は、`run_job` が、起動の前に合図を見て、起動せず、started が false の JobExecution を返す(BR4.2)。その場合、ステートが空なら、初期ステートを書いてから(BR2.5 の (1))、再開位置を記録して終わる(ステートファイルが、中断のあとに残ることがある。これは、意図した動作で、次の実行が、先頭から始まる)", violation: なし, source: FR2.9 }
  - { id: BR4.3, statement: 中断で終わったときの終了コードは、128 + 受け取ったシグナルの番号, category: policy, applies_to: ExitCodeMap, trigger: 中断で終わったとき, logic: "ジョブの実行中、起動前、ジョブのあいだの、いずれも、終了コードは 128 + 最初に受け取ったシグナルの番号(u1-core の JobOutcome の `reportedExitCode`。ジョブのあいだは、この Unit が計算する)。ジョブが起動していたときは、ジョブの終了を待ってから、記録して終わる", violation: なし, source: FR2.9、E1 の interrupted_by_signal }

  # グループ5: 終了コード(Cli)
  - { id: BR5.1, statement: 終了コードを決める, category: calculation, applies_to: ExitCodeMap, trigger: 実行が終わるとき, logic: "成功・警告・スキップは 0。エラーは、JobOutcome の `reportedExitCode`(ジョブの終了コードそのまま。起動失敗は 127、シグナル終了・中断は 128 + 番号。成功コードに含まれない 0 は 1)。conductor 自身のエラーは、定義の誤り 64、ステートの誤り・食い違い 65、拒否 66、変換の誤り 67、使い方の誤り 68", violation: なし, source: FR2.5、E1、u1-core の BR2.2〜BR2.4・BR5.1、Q3 }
  - { id: BR5.2, statement: 終了コードは、3つの OS で同じ範囲(0〜255)に合わせる, category: calculation, applies_to: ExitCodeMap, trigger: プロセスを終了するとき, logic: "`reportedExitCode` が、0〜255 の範囲外(255 を超える、または負)のとき、**どの OS でも**、下位 8 ビット(0〜255)にする。ただし、エラーのジョブで、下位 8 ビットが 0 になる場合は、1 にする(エラーを、成功に見せない)。Windows の大きなジョブの終了コード(例外による終了など)も、同じ規則で合わせる(元の値は、-v のときの表示で、そのまま示す)。これにより、同じ定義と同じ結果に、3つの OS で、同じ終了コードを返す。契約 E1 の、「ジョブの終了コードをそのまま」は、0〜255 の範囲の値に限る、と確定する", violation: なし, source: FR2.5、AC2.3.2、AC2.5.5、AC3.1.4 }

  - { id: BR5.3, statement: 3つの OS で、同じ定義に、同じ結果にする, category: constraint, applies_to: ExitCodeMap, trigger: 全体, logic: "この Unit の判断(終了コード(BR5.2 で範囲を揃える)、スキップ、遷移、完了、表示の内容)に、OS による違いを持ち込まない。OS による違い(シグナル、プロセス)は、u2-runtime の `platform` に閉じ込められている。この Unit の OS 固有のコードは、シグナルを受け取る準備、標準出力が端末かどうかの判定、Windows で端末の制御文字を有効にする設定だけ。シグナルの種類(UNIX 系の4つと、Windows の Ctrl+C)は、OS が持つシグナルの違いであり、同じ結果の対象外(Functional Design の Q4 のとおり)", violation: なし, source: NFR3、AC2.5.5、AC3.1.4 }


  # グループ6: 表示(Reporter)
  - { id: BR6.1, statement: 表示の言語は、環境のロケールで決める, category: policy, applies_to: Language, trigger: 表示が必要になったとき(それまで判定しない), logic: "環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` を、この順に見て、最初に空でない値が `ja` で始まれば日本語、それ以外、または見つからなければ英語。判定は、最初の表示が必要になるまで行わない(起動を速くするため)。ジョブ本体の出力は、言語に関わらず、そのまま", violation: なし, source: FR2.6、NFR1 }
  - { id: BR6.2, statement: 既定では、ジョブ本体の出力だけを流し、conductor 自身のエラーは常に表示する, category: policy, applies_to: DisplayEvent, trigger: 表示のとき, logic: "`level = error` は、常に標準エラーに表示する。`level = verbose` は、`-v` のときだけ、標準エラーに表示する。`level = output` は、標準出力に出す。ジョブが成功し、スキップされたとき、既定では、何も表示しない(AC2.4.1、AC2.4.3)", violation: なし, source: FR2.6、AC2.4.1〜AC2.4.3 }
  - { id: BR6.3, statement: "`-v` で、ジョブの結果、スキップ、遷移を表示する", category: policy, applies_to: DisplayEvent, trigger: -v のとき, logic: "1段階だけ。ジョブの結果(ジョブID、結果 success/warning/error、終了コード)、スキップ(ジョブID、理由)、遷移(結果、遷移先、スキップしたジョブID)を、`[conductor] ...` の形で標準エラーに出す。中断を受け取ったときも出す", violation: なし, source: FR2.6、AC2.4.4、Q1(Refined Mockups) }
  - { id: BR6.4, statement: エラーの表示は、固定の文と場所で組み立てる, category: constraint, applies_to: DisplayEvent, trigger: エラーの表示のとき, logic: "`conductor: error: <場所>: <種類の文>: <原因>` の形。定義の誤りは、ファイル名・行・種類・原因(複数なら、1行ずつ)。ステートの誤りは、ステートファイルのパスと原因。入力に由来する文字列(ジョブID、キー名、パス)は、制御文字などをエスケープして表示する。ライブラリのエラー文、ソースの抜粋は含めない。ジョブを起動できないときは、`cannot start command '<コマンド名>'`(コマンド名は、定義の command の最初の語)", violation: なし, source: FR1.4、FR2.5、AC2.4.2、NFR4 }
  - { id: BR6.5, statement: 色は、端末に出すときだけ。状態は、文字でも表す, category: policy, applies_to: ColorMode, trigger: watch の表示のとき, logic: "状態(success、warning、skipped、error、pending)は、色に関わらず、常に文字で書く。色(制御文字)は、標準出力が端末で、かつ環境変数 `NO_COLOR` が、設定されていない、または空のときだけ使う(設定されていて空でなければ、使わない)。ファイルやパイプへの出力には、制御文字を含めない", violation: なし, source: NFR6、AC5.1.3 }


  - { id: BR6.6, statement: 起動の速さ(NFR1)を守る, category: constraint, applies_to: Cli, trigger: run・pipeline の起動から、ジョブの起動(またはスキップの判定)まで, logic: "ジョブ10個程度の定義で、20 回測定した p95 が 100ms 以内(Linux・macOS・Windows のそれぞれ)。起動の経路に、不要な初期化(ロケールの判定、表示の準備、convert に関わるライブラリの初期化)を含めない。この Unit の起動の経路は、定義の読み込み、command の検査、ステートの読み込み、照合、初期ステートの書き込み(Fast)で、各 Unit の配分(u1-core 20ms 以内(仮)、u2-runtime 20ms 以内(仮))の合計を含む。測定は、Code Generation のテストと CI で行う", violation: 基準を超えたら、警告として記録する(失敗にするかは、実測のあとに決める), source: NFR1、AC2.5.3 }

  # グループ7: 補助のコマンド
  - { id: BR7.1, statement: watch は、現在の状態を一覧で1回表示して終わる, category: policy, applies_to: DisplayEvent, trigger: watch の実行時, logic: "定義ファイルの場所(BR1.4)から、`read_state` でステートを読む(定義ファイルは、読まず、検証もしない。このため、定義との食い違いは、検査しない。ステートのパイプライン名を、表の前の1行で示し、古いステートかを、利用者が見分けられるようにする)。StateError なら、終了コード 65。空(`None`)なら、空であることを表示して、終了コード 0。あれば、パイプライン名の1行と、ジョブID、状態、補足(終了コード、スキップの理由)の3列の表を、標準出力に1回出して、終了コード 0。`completed` のまま残ったステートは、空として扱われる(u2-runtime)。表の見出しと、補足の文言は、言語に従う。標準出力への書き込みの失敗(閉じたパイプなど)は、終了コード 65 で終わる(書き込みの失敗を、パニックにしない)", violation: 終了コード 65, source: FR5.1、AC5.1.1〜AC5.1.3、NFR6 }

  - { id: BR7.2, statement: reset は、ステートを初期化する, category: policy, applies_to: PipelineState, trigger: reset の実行時, logic: "定義ファイルの場所(BR1.4)から、u2-runtime の `reset_state` を呼ぶ(読み込み・検査を迂回する。定義ファイルも読まない)。ステートがなくても、エラーにしない。終了コード 0。削除の失敗は、終了コード 65。表示は、エラーのときだけ", violation: 終了コード 65, source: FR6.1、AC6.1.1、AC6.1.2、AC4.3.3、AC4.4.2 }
  - { id: BR7.3, statement: convert は、変換結果を標準出力に書く, category: policy, applies_to: CliCommand, trigger: convert の実行時, logic: "u4-converter の `convert`(入力のパス、`--to` の形式)を呼ぶ。成功したら、変換結果の文字列を、標準出力に、そのまま書いて、終了コード 0。ConversionError なら、種類と場所を、標準エラーに表示して、終了コード 67(標準出力には何も書かない)。標準出力への書き込みの失敗(閉じたパイプなど)は、終了コード 67。出力先を指定するオプションは、足さない(ファイルへは、シェルのリダイレクトで書く。Q2: A)。変換した結果が、conductor の定義として正しいかは、検査しない(読み込み時に行われる)", violation: 終了コード 67, source: FR7.1、FR7.2、AC7.1.1〜AC7.1.3、Q2 }
```

## 規則の要約

| グループ | 範囲 | 主な規則 |
|----------|------|----------|
| BR1 | コマンドとオプション | BR1.1〜BR1.5(5つのコマンド、`--config`、`-v` の範囲、定義ファイルの場所、単一の実行ファイル) |
| BR2 | 個別実行 | BR2.1〜BR2.6(検査、ステート、判断、スキップ、実行と記録、成功コードに含まれない 0) |
| BR3 | 一括実行 | BR3.1〜BR3.4(検査とパイプライン名、記述順の実行、完了と初期化、エラーで停止) |
| BR4 | シグナル | BR4.1〜BR4.3(受け取りの準備、合図の確認、中断の終了コード) |
| BR5 | 終了コード | BR5.1〜BR5.3(終了コードの決め方、OS の範囲、3つの OS で同じ結果) |
| BR6 | 表示 | BR6.1〜BR6.5(言語、既定と常に表示、`-v`、エラーの組み立て、色) |
| BR7 | 補助のコマンド | BR7.1〜BR7.3(watch、reset、convert) |

## Assumptions & Open Questions

- 一括実行の途中で、`resume_job_id` が指すジョブより前に、pending のジョブが残る(たとえば、個別実行で飛ばした後に、一括実行をした)場合は、u1-core の `decide_before_run`(BR3.5)の判断に従う。 [assumption]
- `-v` の表示の文言(`[conductor] job=extract result=warning (exit code 1)` など)は、Refined Mockups のイメージを出発点に、Code Generation で確定する。確定した文言を、テストで固定する。 [assumption]
- 日本語の表示の文言は、Code Generation で、英語の文言と対にして作る。環境のロケールの判定は、BR6.1 のとおり、最初の表示が必要になるまで行わない。 [assumption]
- 一括実行(`pipeline`)の中断(BR4.2)の、ジョブのあいだの確認は、各ジョブの終了後と、次のジョブの起動前の、2か所で十分とする(起動前の確認は run_job が行う)。 [assumption]
- u1-core の定義ファイルの受け付けるキー(`jobs` → `job`。Q1: A)の変更は、この Unit の Code Generation の計画に、u1-core の修正の手順として含める。 [assumption]
