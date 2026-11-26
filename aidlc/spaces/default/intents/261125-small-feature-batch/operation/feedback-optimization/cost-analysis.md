# 費用の分析: v0.10.0

出典: `feedback-optimization-questions.md`(まとめの確かめ済み)、`construction/ci-pipeline/ci-config.md`、`operation/deployment-pipeline/cd-config.md`。

- クラウドの資源は使っていない。費用は GitHub Actions の実行の分だけ。
- v0.10.0 の作業での実行: `main` への push の CI が数回(1 回あたり約 2〜3 分、13 ジョブ)と、`release.yml` が 1 回。
- 減らせる所: 作業の途中のコミットごとに push すると CI がそのたびに走る。今回は人が区切りで push したため回数は少ない。今の運用のままでよい。
- 新しい依存は足していない(依存の監査 `cargo audit` の対象も増えていない)。
