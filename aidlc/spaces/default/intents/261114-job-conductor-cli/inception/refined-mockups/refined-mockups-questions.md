# Refined Mockups — 質問

`conductor` は画面を持たない CLI のため、ここでは「コマンドの名前・オプション・定義ファイルの書き方・表示の文言」を、利用者から見た体験として具体化します。Rough Mockups と Requirements で「設計で決める」とした項目を確認します。

## Q1. [プロダクト] `conductor` 自身の表示を有効にするオプション

既定では `conductor` 自身の表示は出さず、オプションで有効にします(Rough Mockups の Q2、Q3)。オプションの名前と段階を確認します。

- A. `-v` / `--verbose` の1段階。ジョブの結果(成功・警告・エラー)、スキップ、遷移を表示する
- B. `-v` を重ねて段階を分ける(`-v` でジョブの結果、`-vv` でスキップと遷移の詳細も表示する)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [開発] 定義ファイルのパスを指定するオプション

既定は実行ディレクトリの `conductor.toml` で、オプションで別のパスを指定できます(US1.3)。

- A. `-f <パス>` / `--file <パス>`
- B. `--config <パス>`
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q3. [開発] 「パイプラインをここで終える」の書き方

遷移先に、ジョブIDではなく「ここで終える」を指定できるようにします(要件 FR1.3)。書き方を確認します。

- A. `next = "@end"`(`@` を付けた予約された値。ジョブIDと衝突しない)
- B. `next = "end"`(予約語。`end` というジョブIDは使えない)
- C. 別のキーで書く(例: `end = true`)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [開発] 「成功とみなす終了コード」のキー名

定義の例では、警告とみなす終了コードを `warning_exit_codes` で指定します。成功とみなす0以外の終了コードのキー名を確認します。

- A. `success_exit_codes`(`warning_exit_codes` と対になる名前)
- B. `ok_exit_codes`
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A. 書かなければ 0 のみ、書けばそれのみが有効(0 を書かなければ 0 でも成功にならない)

## Q5. [プロダクト] `conductor` 自身の表示の言語

`watch` の一覧、エラーの原因、`-v` の表示など、`conductor` 自身が出す文言の言語を確認します。

- A. 英語に固定する(スケジューラのログや OS を問わず、扱いやすい)
- B. 日本語に固定する
- C. 環境のロケールに従う(英語と日本語を切り替える)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: C

## Q6. [プロダクト] `watch` の一覧の見せ方

`watch` は、現在の状態を一覧で1回表示します(Rough Mockups の Q4)。表の項目を確認します。

- A. ジョブIDと状態(成功・警告・スキップ・エラー・未実行)の2列
- B. ジョブID、状態、補足(スキップの理由、エラーの終了コードなど)の3列
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q7. [運用] コマンドを起動できない場合・シグナルで終了した場合の終了コード

ジョブ自身の終了コードがない場合に、`conductor` が返す非0の値を確認します(要件 FR2.2)。

- A. コマンドを起動できないときは 127、シグナルで終了したときは 128 + シグナル番号(シェルの慣習に合わせる)
- B. どちらも固定の 1 を返す
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A. conductor がシグナルを受け取ったら、起動したコマンドにもシグナルを送るか、という確認あり(Q8 参照)

## Q8. [運用] `conductor` がシグナルを受け取ったときの扱い(Q7 への確認から追補)

スケジューラがジョブを取り消すとき、`conductor` にシグナルが届きます。起動したジョブの扱いを確認します。

- A. 受け取ったシグナルを、起動したジョブにも転送し、ジョブの終了を待つ。その後、ジョブがシグナルで終わったものとしてエラー扱い(Q7 の 128 + シグナル番号)にし、ステートにエラーの位置を残して終了する。Windows では、終了を要求する同等の動きに読み替える
- B. 転送しない。`conductor` だけが終了する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
