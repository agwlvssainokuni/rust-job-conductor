# Feedback & Optimization の質問: v0.10.0

出典: `operation/deployment-execution/deployment-log.md`・`health-check-report.md`、`construction/build-and-test/build-and-test-summary.md`・`test-results.md`、`construction/ci-pipeline/ci-config.md`、各ユニットのレビュー(`construction/*/*/reviews/`)、`code-summary.md` の「気になる点」。

SLO、費用、構成のずれについては、動き続けるサービスもクラウドの資源も無いため、記録から答えが決まる(SLO は無い、費用は GitHub Actions の分だけ、`ci.yml`・`release.yml` は変えていない)。人に聞くのは、残った小さな課題の扱いだけ。

## Q1. [利用者] 残った小さな課題をどう扱うか

v0.10.0 の作業で、ふるまいを変えない小さな課題が残った。

- (a) README の `watch --follow` の「止まったと判定して終わることがある」の言い方(u7 のレビュー)
- (b) `log_dir` の Windows の予約名の検査が名前の前後の空白を除かない、待ち切れの「止める」印を読み取りの後でしか見ない(u5 のレビュー)
- (c) CI の `coverage` のジョブで、性能のテスト `plan_simulate_perf.rs` の 150ms の上限が 2 回揺れた
- (d) u6 の作業中に、全体のテストで 1 度だけ出た原因不明の失敗(その後は起きていない)
- (e) デモを `DEMO_PAUSE=1` で見せると、場面 4 の `watch --follow` が画面を消す

A. まとめて次の整理の intent(v0.10.1)の候補として残す
B. 今は残すだけにして、困ったときに個別に扱う
X. Other (please specify)

[Answer]: A

## Q2. [利用者] 作業のやり方で、次から変えたいことはあるか

今回、ユニットのコードを書く担当が、決まり(ファイルの道具を使う)に反してシェルや `python3` でファイルを直したことが 3 回あった(結果には影響なし)。また、テストの手順の書き方の誤り(`--` の抜け)が Build and Test まで見つからなかった。

A. 次から、コードを書く担当への指示で「ファイルの道具だけを使う」を強め、テストの手順はユニットの確認のときに一度走らせる
B. 今のままでよい
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

まとめ: SLO は無く、費用は GitHub Actions の分だけ、CI と配布の設定のずれは無い。残った小さな課題 (a)〜(e) は、次の整理の intent(v0.10.1)の候補として残す。次からは、コードを書く担当への指示で「ファイルの道具だけを使う」を強め、テストの手順はユニットの確認のときに一度走らせる。

- Looks correct
- Request changes

[Answer]: Looks correct
