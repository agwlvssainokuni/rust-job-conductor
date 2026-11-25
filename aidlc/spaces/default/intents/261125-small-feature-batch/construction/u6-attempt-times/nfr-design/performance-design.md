# 性能の設計: u6-attempt-times

出典: 性能の要件(`performance-requirements.md` の NFR1.3、NFR1.9)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C4)。

## 設計

- **書く回数(NFR1.3)**: 1 回目の試行の開始の書き込み(Fast)が、今の空のステートの初期化の書き込み(Fast)を兼ねる。今と比べて増えるのは、ステートがすでにあるときの開始の 1 回と、リトライの前の 1 回だけ。最後の記録は今の Durable の 1 回。
- **読み込み(NFR1.9)**: 試行の表を 1 回ずつ見るだけで、手間は試行の数に比例する。
- 確かめ方: 流れのテストで、偽の書き込みの回数(1 回の試行で Fast 1、Durable 1。リトライ 1 回で Fast 3、Durable 1)。今の性能のテスト(`*_perf.rs`)が上限を変えずに通る。
