# 観測の設計: u3-unset-env

出典: 観測の要件(`observability-requirements.md` の NFR1.5、NFR6.4)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 2)、契約(`contract-summary.md` の C1)。

## 設計

- **`--dry-run`(NFR1.5)**: 計画の各ジョブに、名前だけの統合で得た `removals` を持たせる。表示は、今の環境変数の名前の行の後に、消す名前が 1 つ以上あるときだけ「消す環境変数」の行を英日で出す(`plan_text.rs` と `messages.rs`)。
- **`.env` の重なりの誤り(NFR6.4)**: `EnvFileError` に新しい種類 `UnsetConflict` を足し、今の表示(ファイル名、行、種類の文、理由の文)をそのまま使う。
- 確かめ方: `src/tests/plan_report.rs` に、消す名前のあるジョブと無いジョブの表示(英日)。`.env` の誤りの表示のテスト。
