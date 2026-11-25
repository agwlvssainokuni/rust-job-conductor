# ビジネスルール: u7-watch-follow(`watch` の継続表示)

出典: 要件(`requirements.md` の FR7.1〜FR7.5、FR8.1、FR8.2、NFR1、NFR6)、ユニットの定義(`unit-of-work.md` の U7)、要件とユニットの対応(`unit-of-work-story-map.md`)、コンポーネントの一覧(`components.md`)、契約(`contract-summary.md` の C5)、機能の設計の質問(`functional-design-questions.md` の Q1〜Q4)。規則の番号はこのユニットの中のもの。

```yaml
rules:
  - id: BR1.1
    statement: "`watch` は `--follow` があるときだけ表示し直す。無ければ今と同じ 1 回"
    category: policy
    applies_to: CliArgs、CliFlows
    trigger: "`conductor watch`"
    logic: "`--follow` が無ければ今の run_watch のまま(表示、終了コード)"
    violation: 該当なし
    source: FR7.1

  - id: BR1.2
    statement: "`--interval` は 1〜3600 の整数の秒で、`--follow` と一緒のときだけ使える。既定は 2"
    category: validation
    applies_to: CliArgs
    trigger: 引数の解析
    logic: >
      整数でない・範囲外・空は使い方の誤り。`--follow` の無い `--interval`、`watch` 以外のコマンドの `--follow` も使い方の誤り。
      どれも今の使い方の誤りの表示と終了コード(68)。値は表示しない
    violation: 使い方の誤り(68)
    source: FR7.2

  - id: BR2.1
    statement: 描くたびにステートを読み直し、今の時刻を取り、表を描く
    category: policy
    applies_to: CliFlows
    trigger: 継続表示の 1 回ごと
    logic: 読む → now() → 表(U6 の所要時間の列を含む)→ 終わりの判定 → 間隔だけ待つ(中断の合図で待ちを打ち切る)
    violation: 該当なし
    source: FR7.1, FR7.2

  - id: BR2.2
    statement: UNIX 系の端末では画面を消して描き直し、ほかは追記する
    category: policy
    applies_to: CliReport
    trigger: 継続表示の 2 回目からの描画
    logic: >
      標準出力が端末で、OS が UNIX 系なら、1 回目から毎回、描く前に画面を消してカーソルを左上に戻す。
      Windows、パイプ、ファイルでは、前の表の後に空の 1 行を置いて次の表を足す
    violation: 該当なし
    source: FR7.4

  - id: BR3.1
    statement: パイプラインが終わったら、最後の表示をしてから自動で終わる
    category: calculation
    applies_to: CliFlows、CoreFlow(PipelineActivity)
    trigger: 継続表示の 1 回ごと
    logic: >
      完了(Completed)、一度見えたステートが消えた(Removed)、止まった(Stopped: 動いている試行が無く、
      error のジョブがあるか、再開の位置のジョブが試行を持っている)なら、その回の表示を最後に終わる。
      Between(始まる前、ジョブとジョブの間)と Running は続ける
    violation: 該当なし
    source: FR7.3

  - id: BR3.2
    statement: 中断の合図(Ctrl-C など)で終わる
    category: policy
    applies_to: CliFlows
    trigger: 待ちの途中、または描く前に合図が立っている
    logic: すぐに終わる(描きかけの表は無い)。終了コードは 0(Q3)
    violation: 該当なし
    source: FR7.3

  - id: BR3.3
    statement: 自動で終わったときと中断で終わったときの終了コードは 0
    category: policy
    applies_to: CliFlows
    trigger: 継続表示の終わり
    logic: Completed、Removed、Stopped、Interrupted は 0。読めない・書けないは 65。合図の準備の失敗は今の run・pipeline と同じ 70
    violation: 該当なし
    source: FR7.3

  - id: BR3.4
    statement: リトライの待ちの間は、最後の試行を開いたままにして、止まったと見えないようにする
    category: constraint
    applies_to: CliFlows(run_job_with_retry。U6 の流れの直し)
    trigger: リトライの待ち
    logic: >
      試行の終わりの時刻はメモリに持ち、待つ前には書かない。次の試行を始めるときに end_attempt(持っていた時刻)と
      start_attempt(now) をして 1 回書く。待ちの途中の中断では end_attempt(持っていた時刻)をしてから抜ける
    violation: 該当なし
    source: FR7.3

  - id: BR4.1
    statement: ステートがまだ無ければ待つ
    category: policy
    applies_to: CliFlows
    trigger: ステートのファイルが無い(または長さ 0)
    logic: 今の「ステートが無い」表示を出して待ち、できたら表を描く。一度見えた後に無くなったら Removed で終わる
    violation: 該当なし
    source: FR7.5

  - id: BR4.2
    statement: ステートが読めなければ、今の watch と同じ誤りの表示で 65
    category: validation
    applies_to: CliFlows
    trigger: 読み込みの誤り(壊れている、古い形、大きすぎる)
    logic: 今の state_error の表示(reset の案内を含む)をして 65 で終わる。値は表示しない
    violation: 65
    source: FR7.5

  - id: BR5.1
    statement: 使い方の表を 3 か所で揃え、README とデモを更新する
    category: policy
    applies_to: 文書
    trigger: このユニットの最後
    logic: "`args.rs`、`report/messages.rs`、README の使い方に `--follow` と `--interval` を足す。README に継続表示の動き、終わる条件、終了コードを書く"
    violation: 該当なし
    source: FR8.1

  - id: BR5.2
    statement: 7 つの機能を作り終えたら版を 0.10.0 にする
    category: policy
    applies_to: "`crates/cli/Cargo.toml`"
    trigger: このユニットの最後のコミット
    logic: 「版を 0.10.0 にする」の 1 コミット。版に合わせる文書やテストがあれば同じコミットで直す
    violation: 該当なし
    source: FR8.2
```
