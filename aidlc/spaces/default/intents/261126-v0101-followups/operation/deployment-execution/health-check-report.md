# 健全性の報告: v0.10.1

出典: `deployment-log.md`、`smoke-test-results.md`、`construction/build-and-test/build-and-test-summary.md`、CI 45913141134、`release.yml` の実行 45913389499、`deployment-execution-questions.md`(まとめの確かめ済み)、team.md の Deployment。

## 対象の性質

- conductor は利用者の手元で動く単一バイナリの CLI。動き続けるサービスではないため、配った後の稼働の監視、誤りの率、応答の時間の指標は無い(v0.10.0 と同じ)。健全性は、配る前の関門と成果物の確かめで見る。

## 健全性の目安

| 目安 | 期待 | 実際 | 判定 |
|---|---|---|---|
| 版の一致 | タグ `v0.10.1` が `main` の上にあり、`crates/cli/Cargo.toml` の 0.10.1 と一致 | `verify-tag` 成功、対象 f5f08ce | 良 |
| マージの関門と同じ検査 | `checks` がすべて成功 | 成功(`main` の CI 45913141134 も 12 のジョブがすべて成功) | 良 |
| 性能のテスト | `coverage` の中の `plan_simulate_perf.rs` が上限 150ms の内 | 成功(F3 の直しの後、CI 45912018052、45913141134 で連続して通った) | 良 |
| 成果物の完全さ | 4 つの成果物と `SHA256SUMS`、チェックサムの一致 | すべて一致 | 良 |
| 起動 | `--version` が `conductor 0.10.1` | 3 対象(ジョブ)と手元の Apple Silicon で確かめた | 良 |
| 公開の状態 | 下書きのまま(人が公開する) | `isDraft: true` | 良 |

## 公開の後に見ること

- 利用者からの報告の受け口は GitHub の Issues。v0.10.1 はまだ公開前。
- F2 のログの名前の変化(空白を除くと予約名になるジョブ ID だけ)についての問い合わせが来たら、注記と README の「名前の置き換え」を示す。
- 性能のテストの上限超えが再び起きたら、`construction/build-and-test/build-and-test-summary.md` の「原因不明の失敗の記録の手順」で記録する。

## 残っている人の作業

- 下書きのリリース `v0.10.1` を確かめて公開する。
