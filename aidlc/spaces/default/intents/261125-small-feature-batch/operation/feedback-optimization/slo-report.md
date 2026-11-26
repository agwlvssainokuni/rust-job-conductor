# SLO の報告: v0.10.0

出典: `feedback-optimization-questions.md`(まとめの確かめ済み)、`operation/deployment-execution/health-check-report.md`、team.md の Deployment。

- **SLO は定めていない**。conductor は利用者の手元で動く単一バイナリの CLI で、動き続けるサービス、可用性、応答の時間の約束が無い。
- 代わりの品質の目安は、配る前の関門として守っている。
  - 行カバレッジ 80% 以上(v0.10.0 は 98.18%)
  - 3 つの OS と MSRV 1.100 でテストが通る(CI 45727407490)
  - 性能のテストの上限(`*_perf.rs`、`nfr_performance.rs`)。起動の性能(`startup-bench`)は警告のみ
- 利用者からの報告の受け口は GitHub の Issues。v0.10.0 はまだ公開前なので、利用の報告は無い。
