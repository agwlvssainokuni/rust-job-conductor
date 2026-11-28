# 健全性の確かめ: v0.10.4

出典: `deployment-log.md`、`smoke-test-results.md`、`construction/build-and-test/build-and-test-summary.md`。

## 対象

これは単一バイナリの CLI で、常に動いているサービスはありません。そのため、リリースの成果物と CI の結果で確かめました。

## 結果

| 項目 | 状態 | 根拠 |
|---|---|---|
| `main` の CI | 成功 | 46132075162(9983b28) |
| リリースの流れ | 成功 | 46132328751(すべてのジョブ) |
| 成果物 | そろっている(4 つと `SHA256SUMS`) | `gh release view v0.10.4` |
| 版 | 0.10.4 | 手元の `conductor --version` |
| 動きの互換 | v0.10.3 と同じ | 要件の NFR1。コードで変えたのは版だけ |
| リリースの状態 | 下書き | 公開は人が行う |

## 見張り

- 利用者からの連絡は、GitHub の Issues で受けます。
- 不具合が見つかったときは、`rollback-runbook.md` の「公開の後」に従います。
