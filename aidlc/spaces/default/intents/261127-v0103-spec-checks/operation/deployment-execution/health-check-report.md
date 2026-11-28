# 健全性の確かめ: v0.10.3

出典: `deployment-log.md`、`smoke-test-results.md`、`construction/build-and-test/build-and-test-summary.md`。

## 対象

単一バイナリの CLI で、常に動いているサービスは無い。そのため、健全性は次のもので確かめる。

- リリースの成果物
- CI の結果

## 結果

| 項目 | 状態 | 根拠 |
|---|---|---|
| `main` の CI | 成功 | 46092972210(9c54b0c) |
| リリースの流れ | 成功 | 46093205491(すべてのジョブ) |
| 成果物 | そろっている(4 つと `SHA256SUMS`) | `gh release view v0.10.3` |
| 版 | 0.10.3 | 手元の `conductor --version` |
| 動きの互換 | v0.10.2 と同じ | 要件の NFR1。コードの変更は版だけ |
| リリースの状態 | 下書き | 公開は人が行う |

## 見張り

- 利用者からの連絡は、GitHub の Issues で受ける。
- 不具合が見つかったら、`rollback-runbook.md` の「公開の後」の手順に従う。
