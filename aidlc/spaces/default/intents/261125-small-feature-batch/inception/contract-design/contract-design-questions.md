# 契約の設計: 質問

ユニットどうしの境界(u2 → u3 の環境の統合、u5 → u6 の時刻の整形と現在の時刻)は、どちらも同じプログラムの中の関数の呼び出しで、形は機能の設計で詰める。ここでは、利用者から見える面(定義ファイル、ステートファイル、コマンドライン、ログのファイル)のうち、形がまだ決まっていないものを決める。

## Q1. [設計] B6 ステートの試行の書き方

ステートは TOML で、各ジョブは `[[job]]`(`id`、`status`、`exit_code`、`skip_reason`)です。試行の時刻を、どう書きますか。

A. ジョブの下に表の配列 `[[job.attempt]]` を足し、各試行に `started_at` と、無くてもよい `ended_at` を書く
B. ジョブに `attempts = [{ started_at = "...", ended_at = "..." }, ...]` の 1 行の配列で書く
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] B7 `watch` の継続表示のオプションの名前

継続表示を始めるオプションと、間隔のオプションの名前を決めます(間隔の既定は 2 秒)。

A. `--follow` と `--interval <秒>`(`tail -f` や `kubectl logs --follow` に合わせる)
B. `--repeat` と `--every <秒>`
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q3. [設計] B5 ログのファイルの置き方と名前

`log_dir` の下に、試行ごとのファイルを、上書きせずに作ります。どう置きますか。

A. `log_dir` の直下に `<ジョブの ID>.<開始の時刻>.<試行の番号>.log`(時刻はファイル名に使えるよう `20261125T091248Z` の形)
B. 実行ごとのディレクトリを作る: `log_dir/<実行の開始の時刻、または run ID>/<ジョブの ID>.<試行の番号>.log`
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- ステートの試行(Q1): 各 `[[job]]` の下に `[[job.attempt]]` を足し、`started_at` と、無くてもよい `ended_at` を UTC の RFC 3339 で書く
- `watch` のオプション(Q2): `--follow` で継続表示を始め、`--interval <秒>` で間隔を変える(既定 2 秒)
- ログのファイル(Q3): `log_dir` の直下に `<ジョブの ID>.<開始の時刻>.<試行の番号>.log`(時刻は `20261125T091248Z` の形)
- ユニットどうしの境界: u2 → u3(環境の統合)と u5 → u6(時刻の整形と現在の時刻)は、同じプログラムの中の関数の呼び出し。形は機能の設計で詰める

- Looks correct
- Request changes

[Answer]: Looks correct
