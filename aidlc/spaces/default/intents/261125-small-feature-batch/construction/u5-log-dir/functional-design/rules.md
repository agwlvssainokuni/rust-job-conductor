# ビジネスルール: u5-log-dir(ジョブの出力をファイルにも残す)

出典: 要件(`requirements.md` の FR5.1〜FR5.5、FR6.2、FR6.6、FR8.1、NFR1、NFR5、NFR6)、ユニットの定義(`unit-of-work.md` の U5)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C2、C6)、設計の決定(ADR-002、ADR-005)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q5)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: "`[pipeline]` に `log_dir` を空でない文字列で書ける。相対パスは定義ファイルのディレクトリが基準"
    category: validation
    applies_to: PipelineDefinition.log_dir
    trigger: 定義の検証
    logic: >
      IF 文字列でない THEN TypeMismatch(「`log_dir` は文字列で書きます」)。IF 空 THEN EmptyField(「`log_dir` が空です」)。
      `[job]` に `log_dir` を書けば今と同じく知らないキー(UnknownKey)
    violation: 定義の誤り(64)
    source: FR5.1

  - id: BR1.2
    statement: "`log_dir` を書かない定義は、今と同じく出力を親から引き継ぐだけにする"
    category: constraint
    applies_to: RuntimeRunner
    trigger: ジョブの起動
    logic: IF log_dir が無い THEN ログを作らず、子の標準出力と標準エラーは今と同じく引き継ぐ(パイプにしない)
    violation: 該当なし
    source: FR5.5

  - id: BR2.1
    statement: 時刻は UTC で、ステート用は RFC 3339(`2026-11-25T09:12:48Z`)、ファイル名用は詰めた形(`20261125T091248Z`)にする
    category: calculation
    applies_to: Timestamp
    trigger: 時刻を書くとき
    logic: >
      UNIX 時刻の秒から、グレゴリオ暦の年月日と時分秒を自前で計算する(うるう年を含む。うるう秒は扱わない)。
      秒まで、末尾は Z、年は 4 桁
    violation: 該当なし
    source: FR6.2, FR6.6

  - id: BR2.2
    statement: RFC 3339 の読み取りは、BR2.1 の形だけを受け付ける
    category: validation
    applies_to: Timestamp
    trigger: 時刻を読むとき(U6 がステートで使う)
    logic: >
      `YYYY-MM-DDTHH:MM:SSZ` の 20 文字だけ。月・日・時・分・秒の範囲(日は月とうるう年に従う)を検査する。
      小数の秒、`+09:00` などの時差、小文字の t・z は受け付けない
    violation: 読み取りの誤り(使う側が扱う)
    source: FR6.2, FR6.6

  - id: BR2.3
    statement: 現在の時刻は runtime の時計から取り、試行の前に流れが 1 回だけ取る
    category: policy
    applies_to: RuntimeClock、CliFlows
    trigger: 各試行の前
    logic: >
      std の SystemTime から秒を取る。1970 年より前なら 0、9999 年を超えるなら上限にする(防御)。
      流れが取った値を、ログのファイル名と(U6 で)ステートの started_at の両方に使う
    violation: 該当なし
    source: FR5.3, FR6.6

  - id: BR3.1
    statement: "`log_dir` があるとき、試行ごとに 1 つの新しいファイルを `<安全な ID>.<詰めた時刻>.<試行の番号>.log` で作る"
    category: policy
    applies_to: RuntimeLog
    trigger: 各試行の起動の前
    logic: >
      ディレクトリが無ければ親を含めて作る。ファイルは「無いときだけ作る」で開き、今のファイルを上書きも追記もしない
    violation: 作れなければ BR5.1
    source: FR5.2, FR5.3, FR5.4

  - id: BR3.2
    statement: ジョブ ID のうち、ファイル名に使えない・危ない文字は `_` に置き換える
    category: calculation
    applies_to: LogFileName
    trigger: 名前を作るとき
    logic: >
      `/` `\` `:` `*` `?` `"` `<` `>` `|`、制御文字(U+0000〜U+001F、U+007F〜U+009F)を `_` にする。
      先頭の `.` も `_` にする(隠しファイルや `..` を作らない)。それ以外(日本語など)はそのまま
    violation: 該当なし
    source: FR5.3

  - id: BR3.3
    statement: 同じ名前があれば、末尾に `-2`、`-3` … を足して作る
    category: policy
    applies_to: LogFileName
    trigger: 「無いときだけ作る」が「すでにある」で失敗したとき
    logic: >
      `<安全な ID>.<時刻>.<試行の番号>-<n>.log`(n は 2 から)で作り直す。n は 1000 まで試し、
      それでも作れなければ BR5.1(作れない)
    violation: 該当なし
    source: FR5.3

  - id: BR3.4
    statement: ファイルの権限は、UNIX 系では本人だけが読み書きできる 0600、Windows は既定
    category: policy
    applies_to: RuntimeLog
    trigger: ファイルを作るとき
    logic: UNIX 系では作るときに 0600 を指定する(platform の中で)。ディレクトリの権限は既定(umask)のまま
    violation: 該当なし
    source: FR5.3, NFR6

  - id: BR4.1
    statement: "`log_dir` があるとき、子の標準出力と標準エラーをパイプで受け、端末とファイルの両方へ書く"
    category: policy
    applies_to: RuntimeRunner
    trigger: ジョブの実行中
    logic: >
      子の標準出力は conductor の標準出力へ、子の標準エラーは conductor の標準エラーへ、それぞれ読んだ順に書く。
      同じ内容を 1 つのファイルへ、読んだ順に書く。2 つの流れのそれぞれの順は保つ。2 つの流れの間の順は、
      読んだ順(子が書いた順にほぼ一致するが、厳密には保証しない)
    violation: 該当なし
    source: FR5.2

  - id: BR4.2
    statement: ファイルにはジョブの出力だけを書き、conductor は何も書き足さない
    category: constraint
    applies_to: ログのファイル
    trigger: すべての書き込み
    logic: 見出し、時刻、環境変数、コマンドなどを書かない
    violation: 該当なし
    source: FR5.2, NFR6

  - id: BR4.3
    statement: 子の標準入力は今と同じく引き継ぐ。子から見た標準出力と標準エラーは端末でなくなる
    category: policy
    applies_to: RuntimeRunner
    trigger: "`log_dir` があるとき"
    logic: 色やページャーの有無が変わりうることを README に書く(ADR-005 の結果)
    violation: 該当なし
    source: FR5.2

  - id: BR4.4
    statement: シグナルの転送、終了の待ち、終了コードの扱いは今と同じ
    category: constraint
    applies_to: RuntimeRunner
    trigger: ジョブの実行中
    logic: 今の待ちのくり返しをそのまま使い、出力を読むのは別のスレッドで行う
    violation: 該当なし
    source: FR5.2

  - id: BR4.5
    statement: ジョブが終わった後、出力の読み取りは最大 2 秒まで待つ
    category: policy
    applies_to: RuntimeRunner
    trigger: 子の終了の後
    logic: >
      子が終わってもパイプを持つ孫のプロセスが残ると、読み取りが終わらない。2 秒待っても終わらなければ
      それ以上待たず、以後の出力はファイルにも端末にも書かない。そのときは BR5.3 の警告を出す
    violation: 該当なし
    source: FR5.2

  - id: BR5.1
    statement: 起動の前にディレクトリかファイルを作れなければ、そのジョブを起動せず、起動できなかった(127)として扱う
    category: policy
    applies_to: CliFlows、RuntimeLog
    trigger: BR3.1 の失敗
    logic: >
      標準エラーに 1 行(「ログのファイルを作れません: <パス>」。パスは表示用に escape)。
      その試行は起動の失敗(127)と同じ扱いで、リトライの対象にもならない(今の起動の失敗と同じ)
    violation: 終了コード 127
    source: FR5.4

  - id: BR5.2
    statement: 実行中にファイルへの書き込みに失敗したら、警告を 1 回出してファイルへの書き込みだけをやめ、ジョブは最後まで動かす
    category: policy
    applies_to: RuntimeRunner
    trigger: ファイルへの書き込みの失敗
    logic: >
      端末への出力は続ける。ジョブの終了コードと結果は変えない。警告はジョブの終了の後に標準エラーに 1 行
      (「ログのファイルへの書き込みに失敗しました。途中からファイルに残していません: <パス>」)
    violation: 該当なし(警告のみ)
    source: FR5.4

  - id: BR5.3
    statement: 出力を待ち切れなかったときは、警告を 1 行出す
    category: policy
    applies_to: CliFlows
    trigger: BR4.5
    logic: 標準エラーに 1 行(「ジョブの終了の後も出力が続いたため、残りを残していません: <パス>」)。終了コードは変えない
    violation: 該当なし(警告のみ)
    source: FR5.2

  - id: BR5.4
    statement: "`--dry-run`・`validate`・`graph` はログのディレクトリもファイルも作らない"
    category: constraint
    applies_to: CliFlows
    trigger: それらのコマンド
    logic: ジョブを動かさないコマンドでは、log_dir の検証だけを行う
    violation: 該当なし
    source: FR5.4

  - id: BR6.1
    statement: 古いログは消さない(整理は範囲の外)
    category: policy
    applies_to: RuntimeLog
    trigger: すべての場面
    logic: conductor はログのファイルを消さず、`reset` もログに触れない。README に、利用者が消すことを書く
    violation: 該当なし
    source: FR5.3

  - id: BR6.2
    statement: README とデモを更新する
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: >
      README に `log_dir`、名前の形、権限、失敗の扱い(127 と警告)、端末でなくなること、整理しないことを書く。
      デモに `log_dir` を使う場面を足す(作ったファイルをデモの終わりで消す)
    violation: 該当なし
    source: FR8.1
```

## まとめ

| ID | 規則 | 種類 | 出典 |
|---|---|---|---|
| BR1.1 | `log_dir` の検証 | validation | FR5.1 |
| BR1.2 | 書かなければ今のまま | constraint | FR5.5 |
| BR2.1〜BR2.3 | 時刻の整形・読み取り・現在の時刻 | calculation/validation/policy | FR5.3、FR6.2、FR6.6 |
| BR3.1〜BR3.4 | 名前、置き換え、衝突、権限 | policy/calculation | FR5.2〜FR5.4、NFR6 |
| BR4.1〜BR4.5 | 出力の複製、書き足さない、端末でなくなる、シグナル、待ち切り | policy/constraint | FR5.2、NFR6 |
| BR5.1〜BR5.4 | 作れない(127)、書けない(警告)、待ち切れ(警告)、動かさないコマンド | policy/constraint | FR5.2、FR5.4 |
| BR6.1、BR6.2 | 整理しない、README とデモ | policy | FR5.3、FR8.1 |
