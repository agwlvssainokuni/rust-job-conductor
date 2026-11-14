# Rough Mockups & Concept Visualization — 質問

`conductor` は画面を持たないコマンドラインツール(Web UI / GUI は初版の対象外)なので、ここでは「コマンドの入出力のイメージ」と「システム間のやり取り」を決めます。

## Q1. [プロダクト] 最初に磨き込むべき主な利用場面

初版の中心となる利用場面はどれですか(コマンドの入出力イメージを描く優先順位になります)。

- A. 既存スケジューラから `conductor run <job-id>` が1ステップずつ呼ばれる場面(個別実行)
- B. ローカルで `conductor pipeline <pipeline-name>` により一括実行する場面(一連の実行)
- C. A と B を同じ重みで扱う
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] `run` 実行時に画面(標準出力)へ何を見せるか

`conductor run` が呼ばれたとき、`conductor` 自身が出す表示について確認します。

- A. ジョブ本体の標準出力・標準エラーをそのまま通し、`conductor` 自身の表示は最小限にする
- B. A に加えて、各ジョブの結果(成功 / 警告 / スキップ / 失敗)を1行で表示する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A. conductor自身の表示は、オプションで有効化しない限り、なし。

## Q3. [プロダクト] ジョブをスキップしたときの見え方

スキップ時は何もせず終了コード 0 で終わります。スケジューラ側では「正常終了」と区別できないため、表示の有無を確認します。

- A. 「スキップした」ことと理由を標準出力に1行表示する
- B. 何も表示しない(終了コード 0 のみ)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B. conductor自身の表示は、オプションで有効化しない限り、無し。

## Q4. [プロダクト] `watch` の見せ方

現在の実行状態を確認するコマンド(`watch`)の表示について確認します。

- A. 現在の状態を一覧で1回表示して終了する
- B. 状態が変わるたびに更新して表示し続ける
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 定義ファイルに誤りがあるときの見え方

`conductor.toml` の記述ミスや存在しないジョブIDの指定があった場合の扱いを確認します。

- A. ジョブを一切実行せず、原因(ファイル名・行・内容)を表示して異常終了する
- B. 誤りのある箇所までは実行し、その箇所で停止して表示する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] 出力の色・アクセシビリティ

コマンドの表示で色を使うかを確認します(色だけに頼らない表示の方針にも関わります)。

- A. 色は使わず、文字だけで状態が分かるようにする
- B. 色を使うが、文字でも状態が分かるようにし、端末以外への出力では自動で無効にする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q7. [プロダクト] `conductor` 自身のエラー表示の扱い(追補)

Q2・Q3 では、`conductor` 自身の表示はオプションで有効にしない限り出さないとしました。一方 Q5 では、定義ファイルの誤りの原因を表示するとしています。両者の関係を確認します。

- A. 定義ファイルの誤りなど `conductor` 自身のエラーは、オプションの有無にかかわらず常に表示する
- B. エラーも、オプションで有効にしたときだけ表示する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
