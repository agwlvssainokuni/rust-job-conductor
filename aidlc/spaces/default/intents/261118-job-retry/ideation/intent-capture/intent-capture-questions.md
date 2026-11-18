# Intent Capture — 確認事項

## Sources

- [desc] Initial description: "ジョブが所定の終了コードで終わったとき、最大 N 回、設定した間隔を空けて自動でリトライする機能を追加する"
- [scope] Workflow-selected scope: `feature`.

## Q1. [プロダクト] この機能で解決したい問題は何ですか?

「リトライ」で乗り越えたい失敗の種類を知るための質問です。最も近いものを選んでください。

- A. 一時的な失敗(接続エラー、ロックの競合など)でジョブが止まるたびに、手で再実行している手間をなくしたい
- B. 失敗の原因が時間とともに直る(相手のサービスが復旧する)まで、ジョブを繰り返したい
- C. 利用者から、自動で再実行してほしいという要望があった
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] いまは、止まったジョブをどのように再実行していますか?

現状のやり方が分かると、この機能が置き換えるものが分かります。

- A. 止まったあとに、人が同じ `conductor` のコマンドを手で再実行している
- B. 呼び出す側のスクリプトやスケジューラの設定で、再実行している
- C. 再実行はしていない(次の定期実行を待っている)
- D. Not yet defined
- X. Other (please specify)

[Answer]: B

## Q3. [プロダクト] 成功したと言えるのは、どんな状態ですか?(複数選択可)

- A. 一時的な失敗で止まったあとに、手で再実行する回数が減る
- B. 夜間などの定期実行が、人の手を借りずに最後まで終わる割合が上がる
- C. リトライを設定しない定義ファイルは、これまでと完全に同じ動作のままである
- D. Not yet defined
- X. Other (please specify)

[Answer]: A, B, C

## Q4. [プロダクト] いま、この機能を作ろうと思ったきっかけは何ですか?

- A. 実際の運用で、一時的な失敗によってジョブが止まる事例があった
- B. 作者や利用者の、機能の要望
- C. 同じ種類のツールとの機能の差
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 機能の範囲や優先順位は、だれが決めますか?

- A. 作者がひとりで決める
- B. 作者が決めるが、利用者(ジョブスケジューラの運用者)の意見を聞く
- C. Not identified
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] 最初の版でやらないことは何ですか?(複数選択可)

「やらない」と決めたことは、あとで要件に入れません。

- A. リトライの間隔を、回ごとに伸ばしていく方式(指数バックオフ)
- B. 終了コード以外(コマンドの出力の内容など)でリトライを決めること
- C. リトライの回数や結果を、外部に通知すること
- D. None(上のどれも、最初の版に含めてよい)
- X. Other (please specify)

[Answer]: B, C

## Q7. [プロダクト] 利用者へは、この変更をどのように知らせますか?

- A. README の更新だけで知らせる
- B. README の更新と、GitHub Releases のリリースノートで知らせる
- C. Not applicable(特に知らせない)
- D. Not yet defined
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

- Looks correct
- Request changes

[Answer]: Looks correct
