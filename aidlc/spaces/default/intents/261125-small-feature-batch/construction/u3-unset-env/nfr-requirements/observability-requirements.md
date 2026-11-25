# 観測の要件: u3-unset-env

出典: 要件(`requirements.md` の FR3.2、FR3.3、NFR6)、機能の仕様(`functional-spec.md` の 2、4)、ビジネスルール(`rules.md` の BR2.2、BR4.1)、契約(`contract-summary.md` の C1)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.5 | `--dry-run` は、消す名前が 1 つ以上あるジョブだけ、足す名前の行の後に消す名前の行を英日で出す。無いジョブの表示は今のまま | 表示のテスト(`src/tests/plan_report.rs`) | BR4.1 |
| NFR6.4 | `.env` の中身との重なりの誤りは、今の `.env` の誤りと同じ形(ファイル名、行、種類、理由)で出す | 表示のテスト | BR2.2 |
