# ビジネスルール(Rules): U3 u3-dry-run(`--dry-run` と `--assume`)

出典: 要件(`requirements-analysis/requirements.md` の FR3.1〜FR3.4、FR1.12、FR5.1、FR5.2、NFR1、NFR5、NFR6)、契約(`contract-design/contract-summary.md` の C1、C2、C5、C7)、U1 と U2 の設計、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q4)、`entities.md`、`functional-spec.md`(期待する出力の例 P1〜P5)。

ルールは `BR{グループ}.{連番}`。グループ: BR1 計画の生成と模擬、BR2 `--assume`、BR3 出力、BR4 誤りと終了コード、BR5 秘匿、BR6 メッセージと文書。

```yaml
rules:
  # --- BR1: 計画の生成と模擬 ---
  - id: BR1.1
    statement: "`--dry-run` は、ジョブを実行せず、ステートを作成も更新も削除もしない。シグナルの準備もしない"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`pipeline --dry-run`、`run --dry-run`"
    logic: "`pipeline`、`run` の準備のうち、シグナルの準備と、ステートの書き込みを除く。ステートは、`read_state`(本番と同じ。完了したステートは「無い」として返る)で読むだけ"
    violation: なし(テストで、書き込みと削除が呼ばれないことを確かめる)
    source: FR3.1

  - id: BR1.2
    statement: "計画は、`pipeline` の流れと同じ判断(`decide_before_run`、`classify_and_decide`)を、実行の代わりに、仮定の終了コードで進めて作る。判断を二重に書かない"
    category: policy
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: "模擬の状態(ジョブの状態のコピー)から出発し、定義の記述順に、各ジョブを `decide_before_run` で決める。`Run` なら、仮定(なければ 0)の終了コードの実行結果(`JobExecution`)を作り、`classify_and_decide`(`RunMode::Pipeline`)で、分類と次の動きを決める。次の動きのスキップを、模擬の状態に反映する"
    violation: なし
    source: FR3.2、契約 C2

  - id: BR1.3
    statement: "ジョブごとの扱いは、起動前の判断から決める: `Run` は実行する(will_run)、`Skip(AlreadyExecuted)` は実行済み、`Skip(SkipTarget)` は遷移でスキップ、`Skip(AfterEnd)` は `@end` のあとでスキップ。模擬で、エラーで停止したあとのジョブは、実行されない(not_reached)"
    category: calculation
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: "エラー(分類がエラー、または conductor 自身のエラー)の結果で、模擬を止める。それ以降のジョブは、not_reached。`@end` への到達と、最後までの完了は、停止ではない(PlanEnd に反映する)"
    violation: なし
    source: FR3.2、FR3.4

  - id: BR1.4
    statement: "再開位置: 読んだステートに再開位置があれば、そのジョブ(after_stop。理由は「前回の停止」)。ステートが無い(完了したステートを含む)ときは、最初から(from_start)"
    category: calculation
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: "`pipeline` のとき: 再開位置のジョブの is_resume を true にする。再開位置が無ければ、どのジョブも false。`run` のとき: ステートの再開位置は示さず、ヘッダーの行を「対象のジョブ: <ID>」にし、is_resume は false(「<- 再開位置」を付けない)。ステートの読み方は、本番の `pipeline` と同じ(`read_state`。完了したステートは「無い」として返るので、最初から)。`--dry-run` は、「いま実行したら何が起こるか」を示すので、本番と同じ読み方にする。U2 の `graph --state` は、「過去の実行の状態」を示すので、完了したステートも読む。この違いは、意図したものである"
    violation: なし
    source: FR3.2

  - id: BR1.5
    statement: "計画の各ジョブに、分割後のコマンド、渡される環境変数の名前と出どころ、リトライの設定を持たせる。コマンドの分割は `CliFlows` が行い(`runtime` の `check_command`)、結果を渡す。環境変数の名前は、U1 の名前の要約(試行 1、run ID があれば run ID)から作る"
    category: policy
    applies_to: [CorePlan, CliFlows]
    trigger: 計画の生成
    logic: "`core` は `runtime` を呼ばない。分割の誤りは、`check_definition`(本番と同じ検査)で、計画の前に見つかる(BR4.1)。名前の要約に、引き継いだ環境の名前は含まない"
    violation: なし
    source: FR3.2、契約 C1、質問への前提

  - id: BR1.6
    statement: "`run <ジョブID> --dry-run` の計画は、対象の 1 ジョブの項目と、仮定(既定は成功)の結果による「次の動き」(次のジョブ、スキップされるジョブ、完了、停止)を 1 行で示す。対象でないジョブへの仮定は、使われなかった仮定として注記する"
    category: policy
    applies_to: [CorePlan]
    trigger: "`run --dry-run`"
    logic: "`decide_before_run`(個別実行)で対象を決める。`Run` なら、`classify_and_decide`(`RunMode::Individual`)で次の動きを決める。「次の動き」の表示は、次の表で固定する(ロケールに従う): エラーで停止(stopped)=「停止」。`@end` に到達(end_reached)で、後ろに未処理のジョブがある=「`@end` に到達(残りのジョブは、呼ばれたときにスキップ)」。完了(completed)=「完了」。それ以外で、次のジョブがある=「次のジョブ <ID>」(スキップされるジョブがあれば、続けて「(スキップ: <ID>、…)」)。仮定した結果で、エラーまたは警告になるときは、先頭に「(仮定: 終了コード <n> → <分類>)」(既定の 0 がエラーになるときは、「(既定: 終了コード 0 → エラー)」)を添える。個別実行の `run` のとき、ステートの再開位置は示さず、対象のジョブだけを「対象のジョブ」として示す(BR1.4)"
    violation: なし
    source: FR3.2

  - id: BR1.7
    statement: リトライは、模擬で繰り返さない。仮定した終了コードを、「リトライを終えた最後の結果」として扱う。リトライの対象の終了コードを仮定したときは、その旨を計画に注記する
    category: policy
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: "SimulatedResult.retryable = (仮定した終了コード が、そのジョブのリトライの終了コードに含まれる)。表示は「(リトライの対象)」を添える"
    violation: なし
    source: FR3.2、FR3.4

  - id: BR1.8
    statement: "計画が示さないこと: 中断(シグナル)、起動の失敗(プログラムが無い、実行権がない。分割は検査するが、PATH の探索はしない)、リトライの回数ごとの結果、ジョブごとに変わりうる実際の終了コード。計画は、仮定した終了コード 1 つを、そのジョブの最終の結果として、進める。この限界は、README にも書く"
    category: policy
    applies_to: [CorePlan, CliReport]
    trigger: 計画の生成、文書
    logic: "実行との一致のテストは、「仮定どおりの終了コードを返す偽の `Runtime`、リトライなし、中断なし」の条件で行う。模擬の状態の更新は、`core` の小さな関数(ジョブの状態の記録、再開位置、`end_reached`、スキップの反映)として書く。`Skip` の判断では、模擬の状態を更新しなくてよい(後続の `decide_before_run` の判断は、`pending` と `end_reached`、再開位置だけで決まる)。本番の `apply_outcome` と `apply_skip` は `conductor-runtime` にあり、`core` は呼べないので、同じ結果になることを、cli の統合テストで確かめる"
    violation: なし
    source: FR3.2、FR3.4

  # --- BR2: --assume ---
  - id: BR2.1
    statement: "`--assume <ジョブID>=<終了コード>` は、複数回指定できる。不正な入力は、使い方の誤り(終了コード 68): `=` がない、終了コードが整数でない、定義にないジョブ ID、同じジョブ ID の重複、終了コードが 0〜255 の範囲外。誤りの表示に、入力の値(ジョブ ID も終了コードも)を含めない。誤りの種類だけを示す"
    category: validation
    applies_to: [CliArgs, CliFlows]
    trigger: 引数の解釈と、定義の読み込みのあと
    logic: "書式と範囲と重複は、引数の解釈のとき(定義を読む前)に検査する。値は、最初の `=` で、ジョブ ID と終了コードに分ける(ジョブ ID が `=` を含む定義は、`--assume` で指定できない。ジョブ ID の規則は定義の検査に従う)。終了コードの範囲は 0〜255(プロセスの終了コードの範囲。定義の成功・警告の終了コードは、32 ビットの整数で書けるが、実行で得られるのは 0〜255 なので、仮定も、この範囲にする)。127 以上の値も、通常の終了コードとして分類する(起動の失敗やシグナルの結果は、模擬しない。BR1.8)。誤りの表示は、`UnknownJob` のように、入力の値(ジョブ ID)を表示せず、誤りの種類だけにする(新しい誤りの種類は、`with_subject` を使わない)。ジョブ ID が定義にあるかは、定義を読んだあとに検査する"
    violation: 使い方の誤り(終了コード 68)
    source: FR3.4、質問 Q1

  - id: BR2.2
    statement: "`--assume` は、`--dry-run` と一緒のときだけ使える。`--dry-run` なしで付けたときは、使い方の誤り(終了コード 68)"
    category: validation
    applies_to: [CliArgs]
    trigger: 引数の解釈
    logic: "`--dry-run` は、`pipeline` と `run` だけで使える。それ以外のコマンドで付けたときは、許されないオプション(68)"
    violation: 終了コード 68
    source: FR3.4、契約 C5

  - id: BR2.3
    statement: "仮定したジョブが、模擬で実行されない(遷移でスキップされる、実行済み、停止のあと、`run` の対象でない)ときは、誤りにせず、計画に「使われなかった仮定」として、ジョブ ID を注記する"
    category: policy
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: 模擬で will_run にならなかったジョブの仮定を、unused_assumptions に、指定順で並べる
    violation: なし
    source: FR3.4、質問 Q1

  - id: BR2.4
    statement: "指定しなかったジョブは、模擬で、成功(終了コード 0)で終わるものとする。何も指定しなければ、「すべて成功したときの流れ」が出る"
    category: policy
    applies_to: [CorePlan]
    trigger: 計画の生成
    logic: "仮定が無いジョブの実行結果は、終了コード 0。その結果は、そのジョブの成功コードの定義に従って、分類する(`success_exit_codes` に 0 が含まれなければ、エラー)"
    violation: なし
    source: FR3.4、質問 Q2

  # --- BR3: 出力 ---
  - id: BR3.1
    statement: "出力の項目と順序(契約 C7): ヘッダー(パイプライン名、run ID)、再開位置、仮定(`--assume` があるとき)、使われなかった仮定(あるとき)、ジョブごとの項目(見出しの行と、字下げした詳細の行)、模擬の結果(完了、`@end` に到達、停止)"
    category: policy
    applies_to: [CliReport]
    trigger: 計画の表示
    logic: "`functional-spec.md` の例 P1〜P5 の形に、1 文字も違わず一致させる。標準出力に、1 回で書く。各行の末尾の空白は取り除く"
    violation: なし
    source: FR3.2、契約 C7、質問 Q3

  - id: BR3.2
    statement: "ジョブの見出しの行: `<番号>. [<扱い>] <ジョブ ID>`。扱いの表示は、実行する、実行済み、スキップ(遷移)、スキップ(@end のあと)、実行されない(停止のあと)。仮定があるジョブは、扱いの中に「仮定: 終了コード <n> → 成功|警告|エラー」を添え、リトライの対象なら「(リトライの対象)」を添える。再開位置のジョブは、行末に「  <- 再開位置」"
    category: policy
    applies_to: [CliReport]
    trigger: 計画の表示
    logic: "ロケール(日本語/英語)に従う。番号は 1 から、定義の記述順(`run` は 1 つだけ)。仮定(指定)がある、または、既定の仮定(終了コード 0)の結果が、警告またはエラーになるジョブは、扱いの括弧の中に、「仮定: 終了コード <n> → 成功|警告|エラー」(既定の 0 のときは「既定: 終了コード 0 → 警告|エラー」)を添える。リトライの対象のときは、同じ括弧の中に、「、リトライの対象」を続ける(`success_exit_codes` に 0 を含まないジョブは、既定でエラーになるので、「既定: 終了コード 0 → エラー」と示され、その次から「実行されない(停止のあと)」になる理由が分かる)。ジョブ ID と、使われなかった仮定のジョブ ID は、`escape_path` と同じに、制御文字をエスケープして表示する。英語の語句は、`functional-spec.md` の英語の例 P2e、P4e で固定する"
    violation: なし
    source: FR3.2、質問 Q3

  - id: BR3.3
    statement: "詳細の行は、扱いが「実行する」のジョブにだけ出す。内容は、この順: 「コマンド: <分割後の単語の配列>」、「環境変数: <名前をカンマで区切る>」、「リトライ: <要約>」(リトライの設定があるときだけ)。字下げは 5 つの空白。コマンドの表示と、リトライの要約の整形は、U2 の `-v`(BR3.3、BR3.1)と同じ"
    category: policy
    applies_to: [CliReport]
    trigger: 計画の表示
    logic: "コマンドは、各単語を二重引用符で囲み、制御文字をエスケープして、`[\"echo\", \"a\"]` の形にする。環境変数は、名前だけ(U1 の名前の要約の順)"
    violation: なし
    source: FR3.2、FR1.12

  - id: BR3.4
    statement: 計画は、標準出力に出す。標準出力に書けないときは、終了コード 65(`graph`、`watch` と同じ)
    category: policy
    applies_to: [CliFlows]
    trigger: 計画の表示
    logic: 計画の全体を作ってから、1 回で書く。誤りのときは、標準出力に何も書かない
    violation: 終了コード 65
    source: FR3.1

  # --- BR4: 誤りと終了コード ---
  - id: BR4.1
    statement: "定義の誤り(`command` の分割の誤りを含む)、`.env` の誤り(U1)は、本番と同じ誤りと終了コード(64)で止まり、計画を出さない"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`--dry-run`"
    logic: "`pipeline`、`run` の準備と同じ検査(`check_definition`、`.env` の読み取り)を、同じ位置で行う"
    violation: 終了コード 64
    source: FR3.3、U1 の BR6.1

  - id: BR4.2
    statement: "ステートの誤り(読めない、壊れている、定義と食い違う)は、本番と同じ誤りと終了コード(65)で止まり、計画を出さない"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`--dry-run`"
    logic: "`read_state`(本番と同じ)と、`check_consistency` を使い、同じ表示にする"
    violation: 終了コード 65
    source: FR3.3

  - id: BR4.3
    statement: "本番なら順序外として拒否される(`Reject`)ときは、本番と同じに、拒否の理由を表示し、終了コード 66 で止まる。計画は出さない(`pipeline` の模擬で拒否が起きたとき、`run` で対象が拒否されるとき)"
    category: constraint
    applies_to: [CorePlan, CliFlows]
    trigger: 計画の生成
    logic: "模擬の中で `decide_before_run` が `Reject` を返したら、計画の生成が、拒否(ジョブ ID と理由)を返し、`CliFlows` が、本番と同じ表示(`DisplayEvent::Rejected`)と 66 にする"
    violation: 終了コード 66
    source: FR3.3、質問 Q4

  - id: BR4.4
    statement: "計画を出せたときの終了コードは 0。模擬の結果がエラーで停止でも、計画を出せたので 0(本番の終了コードは、示さない)"
    category: policy
    applies_to: [CliFlows]
    trigger: 計画の表示
    logic: 計画を標準出力に書けたら 0
    violation: なし
    source: FR3.1

  - id: BR4.5
    statement: "`--run-id` は、`--dry-run` と併用できる(読むステートの場所を決める。本番と同じ)。`--remove-on-complete` も併用できるが、何も削除しないので、効果がない(受け付けて、無視する)"
    category: policy
    applies_to: [CliArgs, CliFlows]
    trigger: 引数の解釈
    logic: 既存の引数の許可の一覧に、`--dry-run` と `--assume` を足す。他のオプションの扱いは、変えない
    violation: なし
    source: FR3.1

  # --- BR5: 秘匿 ---
  - id: BR5.1
    statement: 環境変数の値を、計画の出力、エラーの表示、ステートに出さない。出してよいのは、名前だけ。`.env` の内容も出さない。`--assume` の誤りの表示に、入力の値を含めない
    category: constraint
    applies_to: [CorePlan, CliReport]
    trigger: 計画の表示、誤りの表示
    logic: "計画の型に、値を持たせない。表示の部品には、名前だけを渡す"
    violation: なし(テストで、秘密の値が出力のどこにも無いことを確かめる)
    source: FR1.12、NFR2

  # --- BR6: メッセージと文書 ---
  - id: BR6.1
    statement: 新しいメッセージ(計画の見出し、扱いの表示、使い方、`--assume` の誤り)は、日本語と英語の両方で用意する
    category: policy
    applies_to: [CliReport]
    trigger: メッセージの表示
    logic: 既存のメッセージの網羅のテストに、新しい種類を足す
    violation: なし
    source: FR5.1

  - id: BR6.2
    statement: README に、`--dry-run` と `--assume` の使い方と出力例を、版(v0.9.0 以降)つきで書く。デモに、`--dry-run` の場面を足す
    category: policy
    applies_to: [CliReport]
    trigger: ユニットの完了
    logic: README の必須語句の検査(`docs-check`)に合わせて書き、デモのテストを足す
    violation: なし
    source: FR5.2

  - id: BR6.3
    statement: ジョブが 1,000 個ある定義で、`--dry-run` は 1 秒以内に終わる
    category: constraint
    applies_to: [CorePlan, CliReport]
    trigger: テスト
    logic: "模擬は、ジョブの数に比例する時間で行う(ジョブごとに、状態のコピーを作り直さない)。環境変数の名前の要約と、整形を含めて、ジョブ 1,000 個(すべてが実行する)で 1 秒以内"
    violation: テストの失敗
    source: NFR1
```

## ルールの要約(`rules` の YAML から作った表)

| 規則 | 内容 | 出典 |
|------|------|------|
| BR1.1 | 実行せず、書かず、シグナルの準備もしない | FR3.1 |
| BR1.2 | 実行と同じ判断で、仮定の終了コードで進める | FR3.2 |
| BR1.3 | ジョブごとの扱い、停止のあとは実行されない | FR3.2、FR3.4 |
| BR1.4 | 再開位置 | FR3.2 |
| BR1.5 | コマンド、環境変数の名前、リトライの設定 | FR3.2、C1 |
| BR1.6 | `run` の計画と次の動き | FR3.2 |
| BR1.7 | リトライは繰り返さない | FR3.2、FR3.4 |
| BR2.1 | `--assume` の不正な入力は 68 | FR3.4、Q1 |
| BR2.2 | `--assume` は `--dry-run` と一緒だけ | FR3.4、C5 |
| BR2.3 | 使われなかった仮定は注記 | FR3.4、Q1 |
| BR2.4 | 指定のないジョブは成功(0) | FR3.4、Q2 |
| BR3.1 | 出力の項目と順序 | FR3.2、C7、Q3 |
| BR3.2 | ジョブの見出しの行 | FR3.2、Q3 |
| BR3.3 | 詳細の行 | FR3.2、FR1.12 |
| BR3.4 | 標準出力、出力失敗は 65 | FR3.1 |
| BR4.1 | 定義と `.env` の誤りは 64 | FR3.3 |
| BR4.2 | ステートの誤りは 65 | FR3.3 |
| BR4.3 | 拒否は 66 | FR3.3、Q4 |
| BR4.4 | 計画を出せたら 0 | FR3.1 |
| BR4.5 | `--run-id`、`--remove-on-complete` との併用 | FR3.1 |
| BR5.1 | 値を出さない | FR1.12、NFR2 |
| BR6.1 | メッセージは日本語と英語 | FR5.1 |
| BR6.2 | README とデモ | FR5.2 |
| BR6.3 | 1,000 ジョブで 1 秒以内 | NFR1 |
