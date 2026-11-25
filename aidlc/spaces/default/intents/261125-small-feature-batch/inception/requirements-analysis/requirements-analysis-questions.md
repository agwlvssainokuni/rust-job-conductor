# 要件の分析: 質問

7 つの機能(B1〜B7)の境界を決めます。今の動きは、コードの知識(`api-documentation.md`)に基づいて書いています。

## Q1. [プロダクト] B1 リトライの既定: ジョブ側の設定との関係

`[pipeline.retry]` を全ジョブの既定にします。今はリトライをジョブの `[job.retry]` にしか書けません。ジョブ側にも `retry` があるとき、どう合わせますか。

A. ジョブに `retry` があれば、ジョブのものを丸ごと使う(項目ごとには混ぜない)
B. 項目ごとに上書きする(ジョブに書いた項目だけを置き換え、残りはパイプラインの既定を使う)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] B1 リトライの既定: 特定のジョブだけリトライしない

パイプラインに既定のリトライを書いたうえで、特定のジョブだけリトライさせない書き方は要りますか。

A. 要る(ジョブ側で既定を打ち消す書き方を用意する。名前は設計で決める)
B. 要らない(既定を使いたくないジョブがあるなら、既定を書かずに各ジョブに書く)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] B1 リトライの既定: 成功の終了コードと重なるとき

ジョブの `retry` では、リトライの終了コード(`exit_codes`)がそのジョブの成功・警告の終了コードと重なると、定義の誤り(64)になります。パイプラインの既定の `exit_codes` が、あるジョブの成功・警告の終了コードと重なったら、どうしますか。

A. 定義の誤り(64)にする(ジョブに書いたときと同じ)
B. そのジョブには既定を当てはめない(誤りにしない)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] B2 無くてもよい `.env`: 書き方

今の `env_file` は、文字列か文字列の配列で、ファイルが無いと常に定義の誤り(64)です。「無ければ読まずに進む」ファイルを、どう書きますか。

A. `env_file` の項目に表の形も許す(例: `{ path = ".env.local", required = false }`)。今の文字列の書き方はそのまま
B. 別のキーを足す(例: `optional_env_file`)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] B2 無くてもよい `.env`: 無かったときの知らせ

無くてもよい `.env` が無かったとき、利用者に知らせますか。なお、ファイルはあるが読めない・書式が誤っているときは、今までどおり定義の誤り(64)にする予定です。

A. 何も出さずに進む
B. 標準エラーに 1 行の注意を出して進む
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] B3 `unset_env`: 書く場所と形

親から引き継いだ環境変数のうち、ジョブに渡さないものを `unset_env` に書きます。どこに、どんな形で書けますか。

A. `[pipeline]` と `[job]` の両方に、変数の名前の配列で書く(名前の完全一致だけ)
B. A に加えて、接頭辞のパターン(例: `AWS_*`)も書ける
C. ジョブにだけ書ける(名前の配列)
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q7. [プロダクト] B3 `unset_env`: `env` にも同じ名前があるとき

`unset_env` に書いた名前が、`env` や `env_file` にもあるとき、どうしますか(`unset_env` は、親から引き継いだ分だけを消す予定です)。

A. 定義の誤り(64)にする(書き間違いの可能性が高いため)
B. 誤りにしない。親の値を消したうえで、`env` や `env_file` の値を渡す
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q8. [プロダクト] B4 `graph --format dot`: ほかの指定との組み合わせ

今の `graph` は、`text` と `mermaid` のどちらでも、`-v`(詳しい表示)と `--state`(ステートの状態を重ねる)が使えます。DOT ではどうしますか。

A. text・mermaid と同じく、`-v` と `--state` の両方に対応する
B. 流れの構造だけを出す(`-v` と `--state` は、DOT では使えない誤りにする)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q9. [プロダクト] B5 `log_dir`: ファイルの分け方

今、ジョブの標準出力と標準エラーは親から引き継ぐだけで、端末に出ます。`log_dir` を書いたときは、端末にも出したうえで、ファイルにも残します。ファイルはどう分けますか。

A. ジョブごとに 1 つのファイルに、標準出力と標準エラーをまとめて残す
B. ジョブごとに、標準出力と標準エラーを別々のファイルに残す
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q10. [プロダクト] B5 `log_dir`: 前の実行のファイル

同じジョブを再び動かしたとき(再実行やリトライを含む)、前のファイルをどう扱いますか。

A. 実行ごと・試行ごとに別のファイルにする(名前に時刻や試行の番号を入れる)
B. 同じファイルに追記する
C. 上書きする(最後の実行だけを残す)
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q11. [プロダクト] B5 `log_dir`: ディレクトリが無いとき

`log_dir` に書いたディレクトリが無いとき、どうしますか。

A. 作ってから書く
B. 誤りにして、ジョブを動かさない
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q12. [プロダクト] B6 時刻: ステートに書く形

ステートに書く開始と終了の時刻の形を決めます。

A. UTC の RFC 3339(例: `2026-11-25T09:12:48Z`)
B. その場所の時刻と時差(例: `2026-11-25T18:12:48+09:00`)
C. UNIX 時刻の秒(例: `1795597968`)
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: B

## Q13. [プロダクト] B6 時刻: リトライしたときの記録

リトライしたジョブの時刻を、どう残しますか。

A. 最初の試行の開始と、最後の試行の終了だけを残す(所要時間は待ち時間を含む)
B. 試行ごとに開始と終了を残す
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: B

## Q14. [開発者] B6 時刻: 古いステートの扱い

時刻を持たない v0.9.9 までのステートは、読めなくてよいと決めました(スコープ Q5)。読んだときの扱いを決めます。

A. ステートの `format_version` を 2 に上げる。1 のステートは「版が違う」誤り(65)とし、ステートを消して作り直すよう案内する
B. `format_version` は 1 のままにする。時刻が無いステートは壊れたステートの誤り(65)にする
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: B

## Q15. [プロダクト] B7 `watch` の継続表示: 始め方と終わり方

今の `watch` は 1 回表示して終わります。継続表示は、どう始めて、どう終えますか(1 回だけ表示する今の動きは、既定として残します)。

A. オプションで始める(名前は設計で決める)。一定の間隔(既定 2 秒、オプションで変えられる)で表示し直し、Ctrl-C で終わる
B. A と同じ。加えて、パイプラインが終わったら(完了・中断)自動で終わる
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: B

## Q16. [プロダクト] B7 `watch` の継続表示: 表示し直しの方法

表示し直すとき、前の表示をどうしますか。

A. 端末なら画面を消して描き直す。端末でない(パイプやファイルへの出力)なら、追記していく
B. 常に追記していく(画面は消さない)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q17. [開発者] B6 時刻: その場所の時差の求め方(Q12 の追加の質問)

Q12 で「その場所の時刻と時差」を選びました。時差を求めるには、OS に尋ねる必要があります。UNIX 系では `platform/unix.rs` の中で求められます(`unsafe` は今もそこだけに許されています)。一方、Windows では OS の関数の呼び出しに `unsafe` が要ります。これは「NEVER `unsafe` を runtime の `platform/unix.rs` の外に書かない」の規則に反します。新しい依存も足せません。どうしますか。

A. 時刻は UTC の RFC 3339(例: `2026-11-25T09:12:48Z`)にする(Q12 を A に変える)
B. UNIX 系はその場所の時刻と時差、Windows は UTC(末尾 `Z`)にする
C. 規則を変え、Windows の `platform` にも `unsafe` を 1 か所だけ許す
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q18. [開発者] B7 `watch` の継続表示: Windows での描き直し(Q16 の追加の質問)

画面を消して描き直すには、端末への制御の文字(ANSI)を使います。今の `watch` の色は、UNIX 系の端末のときだけ使っています。Windows の端末で制御の文字を確実に効かせるには、OS の関数の呼び出し(`unsafe`)が要ります。Windows ではどうしますか。

A. 色と同じく、描き直しは UNIX 系の端末だけにする。Windows では追記していく
B. Windows でも描き直す(Q17 の C と同じく、規則を変える必要がある)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- B1 リトライの既定: ジョブに `retry` があればジョブのものを丸ごと使う(Q1)。特定のジョブで既定を打ち消す書き方を用意する(Q2)。既定の `exit_codes` があるジョブの成功・警告と重なれば、定義の誤り(64)(Q3)
- B2 無くてもよい `.env`: `env_file` の項目に表の形(例: `{ path = ".env.local", required = false }`)も許す(Q4)。無いときは何も出さずに進む(Q5)。あるが読めない・書式の誤りは今までどおり 64
- B3 `unset_env`: `[pipeline]` と `[job]` の両方に、名前の配列で書く(完全一致)(Q6)。同じ名前が `env`・`env_file` にもあれば定義の誤り(64)(Q7)
- B4 `graph --format dot`: text・mermaid と同じく `-v` と `--state` に対応する(Q8)
- B5 `log_dir`: 端末にも出しつつ、ジョブごとに標準出力と標準エラーを 1 つのファイルにまとめる(Q9)。実行・試行ごとに別のファイル(Q10)。ディレクトリが無ければ作る(Q11)
- B6 時刻: ステートに UTC の RFC 3339 で書く(Q12 の後 Q17 で変更)。試行ごとに開始と終了を残す(Q13)。`format_version` は 1 のまま、時刻の無いステートは壊れたステート(65)(Q14)
- B7 `watch` の継続表示: オプションで始め、一定の間隔(既定 2 秒、変更可)で表示し直す。Ctrl-C か、パイプラインの終わり(完了・中断)で終わる(Q15)。UNIX 系の端末なら描き直し、それ以外(Windows、パイプ、ファイル)は追記(Q16、Q18)

- Looks correct
- Request changes

[Answer]: Looks correct
