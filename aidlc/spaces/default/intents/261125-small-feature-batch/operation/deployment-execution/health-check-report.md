# 確かめの報告: v0.10.0

出典: `deployment-log.md`、`smoke-test-results.md`、`construction/ci-pipeline/ci-config.md`、`construction/build-and-test/test-results.md`。

## 判定: 公開できる状態(公開は人の作業)

- 単一バイナリの CLI で、動き続けるサービスが無いため、稼働の監視(ヘルスチェックのエンドポイント、エラー率)は当てはまらない。代わりに、配る物そのものの確かめで判定する。
- 配る物: 4 対象のビルドと smoke test が通り、ハッシュと中身も一致した(`smoke-test-results.md`)。
- 中身の品質: 同じコミットで CI の 13 ジョブ(3 つの OS、MSRV、カバレッジ 80% 以上)が通っている(45727407490、`release.yml` の `checks`)。
- 注記: B1〜B7 と、v0.9.9 までのステートを `reset` する注意を書いた。

## 残っていること

- 人が下書きを確かめて公開する。
- 公開の後に問題があれば、`rollback-runbook.md` の「公開の後」の手順(前の版に戻す、v0.10.1 を出す)。
