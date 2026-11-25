# ビジネスルール: u6-attempt-times(試行ごとの時刻と所要時間)

出典: 要件(`requirements.md` の FR6.1〜FR6.9、FR8.1、NFR1、NFR6)、ユニットの定義(`unit-of-work.md` の U6)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C4、C5)、設計の決定(ADR-004)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q3)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: 試行を始める前に、その試行の開始の時刻をステートに書いてからジョブを起動する
    category: policy
    applies_to: CliFlows(run_job_with_retry)
    trigger: 試行ごとの起動の直前
    logic: >
      U5 で取る試行ごとの時刻(now)を started_at にした Attempt をそのジョブの attempts の後ろに足し、
      ステートを Fast で書いてから起動する。リトライの何回目でも同じ。
    violation: 書き込みの失敗は今と同じく 65(起動しない)
    source: FR6.1

  - id: BR1.2
    statement: 試行が終わったら、終わりの時刻を書く
    category: policy
    applies_to: CliFlows
    trigger: 試行の終わり(起動の失敗、ログを作れずに起動しなかったとき、中断を受けて終わったときを含む)
    logic: >
      試行の後に now を取り、最後の Attempt の ended_at にする。リトライするときは、待つ前に Fast で書く。
      最後の試行は、今の結果の記録(apply_outcome、Durable)に一緒に入る
    violation: 書き込みの失敗は今と同じく 65
    source: FR6.1, FR6.9

  - id: BR1.3
    statement: 試行を始めなかったときは、試行を足さない
    category: constraint
    applies_to: CliFlows
    trigger: リトライの待ちの中断、起動前の判断のスキップや拒否
    logic: 起動していないので Attempt を足さない。それまでの試行はそのまま
    violation: 該当なし
    source: FR6.1

  - id: BR1.4
    statement: 再開しても前の試行を残し、後ろに足す
    category: policy
    applies_to: JobState.attempts
    trigger: 再開でジョブをもう一度動かすとき
    logic: >
      前の試行を消さない(Q2)。最後の試行に ended_at が無いまま(強制終了の後)新しい試行を始めるときは、
      その試行の ended_at を自身の started_at にして閉じる(時刻を作らず、FR6.9 を保つ)
    violation: 該当なし
    source: FR6.5, FR6.9

  - id: BR2.1
    statement: 試行の時刻は UTC の RFC 3339(秒まで、Z)で書き、自前で整形・読み取りする
    category: validation
    applies_to: ステートファイル
    trigger: 書き込みと読み込み
    logic: U5 の format_rfc3339 と parse_rfc3339 を使う。読めない書式は Corrupt(BAD_TIME)
    violation: 壊れたステート(65)
    source: FR6.2, FR6.6

  - id: BR2.2
    statement: ステートに足すのは開始と終了の時刻だけ
    category: constraint
    applies_to: ステートファイル
    trigger: 書き込み
    logic: "[[job.attempt]] のキーは started_at と ended_at だけ。知らないキーは UNKNOWN_KEY"
    violation: 壊れたステート(65)
    source: FR6.3

  - id: BR3.1
    statement: 動いたジョブに試行が 1 つも無いステートは壊れたステート
    category: validation
    applies_to: ステートの読み込み
    trigger: ステートを読むすべてのコマンド(run、pipeline、--dry-run、watch、graph --state)
    logic: >
      status が success・warning・error で attempts が空なら Corrupt(NO_ATTEMPT)。
      pending と skipped は空でよい。表示は今の壊れたステートと同じで、reset の案内を出す
    violation: 壊れたステート(65)
    source: FR6.4, FR6.7, FR6.8

  - id: BR3.2
    statement: ended_at が無くてよいのは各ジョブの最後の試行だけ
    category: validation
    applies_to: ステートの読み込み
    trigger: ステートの読み込み
    logic: 最後でない試行に ended_at が無ければ Corrupt(UNFINISHED_ATTEMPT)。終わりが始まりより前でも誤りにしない(時計の戻り)
    violation: 壊れたステート(65)
    source: FR6.9

  - id: BR4.1
    statement: "`watch` の表に所要時間の列を出す"
    category: calculation
    applies_to: CliReport(watch)
    trigger: "`watch`"
    logic: >
      試行が無ければ空。最後の試行に ended_at があれば、最初の started_at から最後の ended_at まで。
      最後の試行に ended_at が無ければ、動いている最中とみなし(Q1)、最初の started_at から今の時刻まで。
      負になれば 0
    violation: 該当なし
    source: FR6.5

  - id: BR4.2
    statement: 所要時間は単位つきの短い形で書く
    category: calculation
    applies_to: CliReport(watch)
    trigger: "`watch`"
    logic: 1 分未満は「17s」、1 時間未満は「1m05s」、それ以上は「2h03m04s」。上の単位が 0 なら省き、下の単位は 2 桁
    violation: 該当なし
    source: FR6.5

  - id: BR5.1
    statement: 試行の時刻を書かない定義の動きは変えない
    category: constraint
    applies_to: 全体
    trigger: すべての場面
    logic: 終了コード、表示(watch の列のほか)、ステートのほかのキーは今のまま
    violation: 該当なし
    source: NFR1

  - id: BR5.2
    statement: README とデモを更新する
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: README にステートの attempt、所要時間の列、強制終了の後の経過時間と reset、古いステートの 65 を書く。デモの watch の場面を合わせる
    violation: 該当なし
    source: FR8.1
```
