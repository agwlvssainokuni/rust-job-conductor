# ビジネスルール(Rules): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

詳細の場所は `inventory.md`、承認が要るテストの変更は `functional-spec.md` の §4、決めてほしい点は §9。

```yaml
rules:
  - id: BR1.1
    statement: JobExecution を job_id、command、result(ExecutionResult)の 3 つの項目にする。ExecutionResult は NotStarted { signal }、StartFailed、Exited { exit_code, terminated_by_signal, interrupted_by } の 3 つの場合だけを持つ。started、start_failure、interrupted、signal_number の項目を消す。組み立ての関数は new、exited、not_started、start_failed。今の項目と同じ意味の読み出しの関数は置かない。
    category: constraint
    applies_to: crates/core/src/types.rs
    trigger: 段 1
    logic: functional-spec.md §2.1.1
    violation: あり得ない組み合わせ(起動せず中断でもない、起動の失敗と中断が同時、番号の無い中断)を作れる型が残る
    source: intent-statement.md(段 1、Q2 の A)
  - id: BR1.2
    statement: 今のコードが作る値(inventory.md 1b の P1〜P13)は、すべて新しい型のどれか 1 つの場合に対応させる。runtime の run_job、core の JobExecution::exited、cli の retry.rs、テストの偽物(ScriptedRuntime、MemRuntime)が、その対応のとおりに作る。偽物は本物と同じく、StartFailed に中断を足さない。
    category: policy
    applies_to: crates/runtime/src/runner.rs、crates/core/src/types.rs、crates/cli/src/retry.rs、crates/cli/tests/common/mod.rs
    trigger: 段 1
    logic: inventory.md 1b、functional-spec.md §2.1.2
    violation: 対応の無い値がある、または 2 つの場合に当たる値がある
    source: intent-statement.md(段 1)
  - id: BR1.3
    statement: 読む側(classify、retry_decision の is_retry_target、apply_outcome の exit_code、cli の result_detail・中断の表示・run_step の after・RetryScheduled の last_exit_code)の結果が、すべての対応する値について今と同じになる。classify の判定の順は、NotStarted、Exited の中断、StartFailed、Exited のシグナル、Exited の終了コードなし、Exited の終了コード。起動せず中断でもない場合の分岐は消す。
    category: validation
    applies_to: crates/core/src/flow/{classify,retry,record}.rs、crates/cli/src/{step,retry}.rs
    trigger: 段 1
    logic: functional-spec.md §2.1.3 の同値の表、§5 の N1
    violation: 分類、返す終了コード、conductor_error、表示、リトライの判断、記録する終了コードのどれかが変わる
    source: intent-statement.md(段 1)
  - id: BR1.4
    statement: ステートファイルの書式と値を変えない。JobExecution は直列化しない。ステートに届くのは apply_outcome が写す exit_code だけで、NotStarted と StartFailed は None、Exited は観測した値。
    category: constraint
    applies_to: crates/core/src/flow/record.rs、ステートファイル
    trigger: 段 1
    logic: inventory.md 1e、functional-spec.md §2.1.4
    violation: ステートのキーや値が変わる、または既存のステートのテストが落ちる
    source: intent-statement.md(段 1)
  - id: BR1.5
    statement: 仕様書 rust の 02、05、07、11、12 の JobExecution の記述を 3 つの場合の言葉に直し、05 §2.3 の順 1' と 07 §4.4 の順 2(起こらない組み合わせ)を消す。language-agnostic は段 1 では変えない(D2 の決定)。
    category: policy
    applies_to: docs/spec/rust/{02,05,07,11,12}-*.md
    trigger: 段 1
    logic: inventory.md 1f
    violation: 消えた項目の名前(started、start_failure、interrupted、signal_number)が、JobExecution の説明として rust の仕様書に残る
    source: intent-statement.md(段 1)
  - id: BR2.1
    statement: 'runtime の run_job の署名を (job, line: &CommandLine, env, signal) にし、中で command を分けない(command_line、split、check_command を呼ばない)。引数の名前は line にし、Command::new(&line.program) の書き方を保つ。'
    category: constraint
    applies_to: crates/runtime/src/runner.rs、crates/runtime/src/split.rs
    trigger: 段 2
    logic: inventory.md 2b、functional-spec.md §2.2.1
    violation: run_job の中に分ける呼び出しが残る、または nfr_source.rs が落ちる
    source: intent-statement.md(段 2)
  - id: BR2.2
    statement: 'cli の Runtime トレイトの run_job も line: &CommandLine を受け取る。run_job_with_retry は run_step が持つ分割済みのコマンド(session.checked.commands の同じ位置)を受け取り、リトライのたびに同じものを渡す。分割は Ctx::check_definition の 1 回だけ。'
    category: policy
    applies_to: crates/cli/src/{deps,retry,step}.rs
    trigger: 段 2
    logic: functional-spec.md §2.2.1
    violation: 分割が 2 回以上ある、またはリトライで分け直す
    source: intent-statement.md(段 2)
  - id: BR2.3
    statement: 分けられない command は、run_job に届かない(型で不要)。今どおり実行の前の検査が定義の誤りとして 64 で報告する。run_job に残す防御は、line.program が空または空白だけなら StartFailed を返す 1 行だけ(D4 の決定)。この確認は起動前の合図の確認より前に置く(今の順「分割の失敗 → 合図」に合わせる。後ろに置くと、合図が立っていたときに NotStarted に変わる)。
    category: validation
    applies_to: crates/runtime/src/runner.rs
    trigger: 段 2
    logic: functional-spec.md §2.2.2、§5 の N2
    violation: 空のプログラム名で OS の挙動に任せる、パニックする、または分けられない command で 64 以外になる
    source: intent-statement.md(段 2、確かめる手段)
  - id: BR2.4
    statement: 仕様書 rust の 4 つの章(02、05、07、12)と 11 章のテストの説明を、新しい署名と防御の行き先に合わせる。署名には「v0.9.5 までは run_job(job, env, signal) で、中で分け直した」の注記を付ける。language-agnostic は D2 の決定により最小限だけ直す(05 §1.2 の起動の手順 1 と §7.5 の run_job の引数、07 §3.6 の防御、§4.1 の入力と手順 2、§4.3 の start_failure の中身)。真偽値の観測モデルは変えない。
    category: policy
    applies_to: docs/spec/rust/{02,05,07,11,12}-*.md、docs/spec/language-agnostic/{05,07}-*.md
    trigger: 段 2
    logic: inventory.md 2d、functional-spec.md §2.2.3
    violation: 「run_job の内側でも分割し直す」の記述が rust の仕様書に残る
    source: intent-statement.md(段 2)
  - id: BR3.1
    statement: core の read_definition は形 B の順にする(D1 の決定)。種別(通常のファイルか)だけ先にパスで見て、開き、開いたハンドルのメタデータで種別と大きさを見て、上限 + 1 バイトまで読み、UTF-8 を確かめる。パスのメタデータで大きさは見ない。読む権限のない 1 MiB 超のファイルは Unreadable(権限)になる。Q3 の「先に開く」を字のとおりには守らない理由(FIFO で止まらないため)は functional-spec.md §2.3.1 に記録する。
    category: policy
    applies_to: crates/core/src/definition/intake.rs
    trigger: 段 3
    logic: functional-spec.md §2.3.1、§2.3.2、§5 の N3
    violation: パスのメタデータの大きさで TooLarge を先に返す
    source: intent-statement.md(段 3、Q3 の A)
  - id: BR3.2
    statement: ファイルが無い(FileMissing)、通常のファイルでない(Unreadable、通常でない。Windows のディレクトリも含む)、読めるが上限超過(TooLarge)、1 MiB 超で UTF-8 でない(TooLarge)、UTF-8 でない(Unreadable)の種類と理由を変えない。書き手のいない名前付きパイプも、開かずに Unreadable(今と同じ。種別を見たあと開く前に FIFO に差し替わる競合を除いて、止まらない)。
    category: validation
    applies_to: crates/core/src/definition/intake.rs
    trigger: 段 3
    logic: inventory.md 3d の既存のテスト、functional-spec.md §5 の N4、N5
    violation: 既存の取り込みのテストが落ちる、または Windows でディレクトリが「権限」の理由になる
    source: intent-statement.md(段 3、Q3 の A)
  - id: BR3.3
    statement: converter の読み込み(crates/converter/src/intake.rs)を変えない。converter は core に依存しない。
    category: constraint
    applies_to: crates/converter
    trigger: 段 3
    logic: git diff で converter の差分が版の行だけ
    violation: converter のコードが変わる、または core に依存する
    source: intent-statement.md(段 3、Q3 の A)
  - id: BR3.4
    statement: 誤りの表示に入力の値(パスの中身、ファイルの内容)を含めない。理由の文は今の固定の文のまま使い、新しい文を足さない。
    category: constraint
    applies_to: crates/core/src/definition/intake.rs
    trigger: 段 3
    logic: nfr_security.rs が変えずに通る
    violation: 表示に内容が出る、または理由の文が変わる
    source: intent-statement.md(段 3)
  - id: BR3.5
    statement: 仕様書 rust の 03 §2.2 と 02 の intake.rs の行、language-agnostic の 03 §3.2 を新しい順の表にし、「v0.9.5 までは、パスのメタデータで大きさを先に見たため、1 MiB を超える読む権限のないファイルは tooLarge だった」の注記を付ける(rust 04-cli.md の --to と同じ書き方)。09 章(converter)は変えない。
    category: policy
    applies_to: docs/spec/rust/{02,03}-*.md、docs/spec/language-agnostic/03-definition-format.md
    trigger: 段 3
    logic: inventory.md 3e、functional-spec.md §2.3.3
    violation: 仕様書の順序の表が実装と違う、または注記が無い
    source: intent-statement.md(段 3)
  - id: BR3.6
    statement: この intent の最後のコミットで版を 0.9.6 にする。Cargo.toml × 4、Cargo.lock、crates/cli/tests/binary_basic.rs の期待(T18)、仕様書の 0.9.5 の表記。昔の版を指す「v0.9.5 までは…」の注記は上げない。タグとリリースはしない。
    category: policy
    applies_to: crates/*/Cargo.toml、Cargo.lock、crates/cli/tests/binary_basic.rs、docs/spec/rust、docs/spec/language-agnostic
    trigger: 段 3 の最後
    logic: inventory.md 3f(手本は ea09b46)
    violation: 版の表記が食い違う(release.yml の verify-tag、--version のテスト)、または注記の版まで上がる
    source: intent-statement.md(版、Q4 の B)
  - id: BR4.1
    statement: 段 1・2 は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、--dry-run の出力)を変えない。段 3 で変わるのは、読む権限のない 1 MiB 超の定義ファイルの誤りの種類だけ。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテスト
    violation: 回帰、または告げていない変化
    source: intent-statement.md
  - id: BR4.2
    statement: 既存のテストは変えずに通す。例外は functional-spec.md §4 の T1〜T18 だけで、組み立て、import、項目の読み方(§4.1 の規則)、偽物の run_job の引数に限る。★ の中身への影響と T18 の期待の文字列の変更は、ユーザーの承認が要る。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、§7
    violation: 一覧に無いテストの変更、または T18 以外の期待する出力の文字列の変更
    source: intent-statement.md(Q2 の A)
  - id: BR4.3
    statement: 段の前と後でテストの件数を記録する。合格は失敗 0 で、件数が「基準 1,137 + 足したテスト」に一致すること(macOS で 段 1 の後 1,138、段 2 の後 1,139、段 3 の後 1,142)。無視は 1 のまま。増減は理由と一緒に記録する。
    category: validation
    applies_to: テスト
    trigger: 各段の終わり
    logic: functional-spec.md §5、§7
    violation: 件数の説明できない増減
    source: intent-statement.md(Q1 の A)
  - id: BR4.4
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存を足さない。unsafe は runtime の platform/unix.rs だけ、cfg(unix)/cfg(windows) は platform の外に置かない(製品のコード)。テスト名とドキュメントコメントは日本語。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(合格の条件)
```
