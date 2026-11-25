# ユニットの分割: 質問

前の機能追加(v0.9.0)では、機能ごとに 1 つのユニット(種類は `service`)にした。今回の要件(FR1〜FR8)と部品の設計(`components.md`、`decisions.md`)から、分け方と依存を決める。

## Q1. [設計] ユニットの分け方

7 つの機能を、どうユニットに分けますか。チームの実践は「機能を 1 つずつ順に作る」です。

A. 機能ごとに 1 つ(U1〜U7 が B1〜B7 に対応。それぞれ、その機能の README とデモの更新まで含める)
B. 関わる部分ごとに 3 つ(定義の書き方: B1・B2・B3、グラフ: B4、出力とステート: B5・B6・B7)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q2. [設計] 版を 0.10.0 にする作業

7 つを作り終えてから、`crates/cli/Cargo.toml` の版を 0.10.0 にします(FR8.2)。この作業をどこに置きますか。

A. 最後に作るユニットに含める
B. 版を上げるだけの別のユニットにする(種類は `packaging`。他のすべてのユニットに依存する)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q3. [設計] 時刻の整形の部品(CoreTime)を作る機能

部品の設計で、ログのファイル名(B5 の `log_dir`)に時刻を入れることにしました。時刻の整形(CoreTime)は B6 の部品ですが、作る順序は B5 → B6 です。どうしますか。

A. CoreTime を B5 で作り、B6 はそれを使う(作る順序はそのまま。B6 は B5 に依存する)
B. 作る順序を変え、B6 を B5 より先に作る(B5 は B6 に依存する)
C. ログのファイル名には時刻を入れず、試行の番号と実行ごとの通し番号などで分ける(B5 と B6 は独立)
D. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Q4. [設計] B2 と B3 の依存

無くてもよい `.env`(B2)と `unset_env`(B3)は、どちらも環境の統合(CoreEnvFile)と `.env` の読み取りを変えます。2 つを依存の関係にしますか。

A. B3 は B2 に依存する(B2 の変更の上に B3 を積む。同じコードを続けて変えるため)
B. 依存させない(それぞれが今のコードだけを前提にする)
C. Not yet defined(まだ決めていない)
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 分け方(Q1): 機能ごとに 1 つ、7 つのユニット(すべて種類は `service`)。u1-pipeline-retry、u2-optional-env-file、u3-unset-env、u4-graph-dot、u5-log-dir、u6-attempt-times、u7-watch-follow。各ユニットは、その機能の README とデモの更新まで含める
- 版(Q2): 0.10.0 への版の更新は、最後に作るユニット u7-watch-follow に含める。そのため u7 は他のすべてのユニットに依存させる
- 時刻の整形(Q3): CoreTime は u5-log-dir で作り、u6-attempt-times はそれを使う(u6 は u5 に依存)
- B2 と B3(Q4): u3-unset-env は u2-optional-env-file に依存する
- 依存のまとめ: u3 → u2、u6 → u5、u7 → u1〜u6。u1、u2、u4、u5 は互いに独立

- Looks correct
- Request changes

[Answer]: Looks correct
