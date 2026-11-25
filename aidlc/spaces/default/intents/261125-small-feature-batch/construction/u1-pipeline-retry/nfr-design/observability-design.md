# 観測の設計: u1-pipeline-retry

出典: 観測の要件(`observability-requirements.md` の NFR1.5、NFR6.3)、技術の選択(`tech-stack-decisions.md`)、機能の仕様(`functional-spec.md` の 4、6)、契約(`contract-summary.md` の C3)。

メトリクス、トレース、アラート、ダッシュボードは当てはまらない(単体の CLI)。利用者が見る面は、リトライの通知、誤りの表示、`graph -v`、`--dry-run` である。

## 設計

- **同じ表示(NFR1.5)**: 表示する側(cli の `graph_text`、`plan_text`、リトライの通知)は、今と同じく各ジョブの `retry` を読む。既定は組み立てのときに各ジョブの `retry` に入るため、表示する側のコードは変えない。既定由来の印は付けない。
  - 確かめ方: 既定を当てはめたジョブと、同じ値の `[job.retry]` を書いたジョブを持つ 2 つの定義で、`graph -v` と `--dry-run` の出力が同じになることを cli のテスト(`src/tests/graph_report.rs`、`src/tests/plan_report.rs`)で確かめる。
- **誤りの位置(NFR6.3)**: 重なりの誤りの位置は、ジョブの表の始まり。読み取りのときに覚えるジョブの位置を使う。重なるジョブが 2 つなら誤りが 2 つ、それぞれの行で出る。
  - 確かめ方: `definition_retry.rs` で、重なるジョブが 2 つある定義の誤りの数と行を確かめる。
