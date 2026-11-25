# 機能の設計の質問: u6-attempt-times(試行ごとの時刻と所要時間)

出典: ユニットの定義(`unit-of-work.md` の U6)、要件(`requirements.md` の FR6.1〜FR6.9、FR8.1)、契約(`contract-summary.md` の C4、C5)、設計の決定(ADR-004)、今のコード(`crates/core/src/types.rs` の `JobState`、`crates/cli/src/step.rs` の `run_job_with_retry`、`crates/cli/src/report/watch.rs`、`crates/runtime/src/state/`)。

すでに決まっていること(質問しない): ステートの各ジョブに `[[job.attempt]]` を足し、試行ごとに `started_at` と `ended_at` を UTC の RFC 3339 で書く(FR6.1、FR6.2、C4)。時刻のほかは書かない(FR6.3)。`format_version` は 1 のまま。動いたジョブ(`success`・`warning`・`error`)に試行が 1 つも無いステートは 65 で、`conductor reset` の案内を出す(FR6.4、FR6.7、FR6.8)。`ended_at` が無くてよいのは各ジョブの最後の試行だけ(FR6.9)。`watch` の表に所要時間の列を足す(FR6.5、C5)。時刻は U5 の `now()` で取り、ログのファイル名と同じ値を使う。状態の値(5 つ)は増やさない。

## Q1. [利用者] 「動いている最中」と「中断で止まった」の見分け方

最後の試行に `ended_at` が無いとき、`watch` は「動いている最中なら経過時間、中断で止まっていれば空」と決まっています(FR6.5)。今の conductor は、Ctrl-C などの中断を受けたら、ジョブの終わりを待って結果を書きます。そのため `ended_at` が残らないのは、動いている最中か、conductor が強制終了(`kill -9`、電源断など)されたときだけです。ステートには時刻のほかは書けません(FR6.3)。

A. `ended_at` が無ければ「動いている最中」とみなし、開始からの経過時間を出す。中断を受けたときは、必ず `ended_at` を書いてから終わる。強制終了の後は経過時間が増え続けるが、README に書き、`conductor reset` か再実行で直る
B. 実行中を示す印のファイル(ロックファイルなど)を別に作り、それがあれば「動いている最中」とする。強制終了の後に残った印は、PID などで見分ける
X. Other (please specify)

[Answer]: A

## Q2. [利用者] 再開したときの試行の記録

エラーで止まったジョブを、もう一度 `run` や `pipeline` で動かした(再開した)ときの扱いです。

A. 前の試行を残し、その後ろに足す。所要時間は、最初の試行の開始から最後の試行の終わりまで(エラーの後に待った時間も入る)
B. そのジョブを動かし直すときに、前の試行を消して新しく記録する。所要時間は、今回の試行だけ
X. Other (please specify)

[Answer]: A

## Q3. [利用者] 所要時間の書き方

`watch` の表の所要時間の列の書き方です。

A. 単位つきの短い形。`17s`、`1m05s`、`2h03m04s`(0 の上の単位は省く)
B. 時計の形。`00:00:17`、`00:01:05`、`02:03:04`
C. 秒の数だけ。`17`、`65`、`7384`
X. Other (please specify)

[Answer]: A
