# smoke test の結果: v0.10.4

出典: `deployment-log.md`、`operation/deployment-pipeline/deployment-strategy.md` の smoke test、`release.yml` の run 46132328751。

| 確かめ | どこで | 結果 |
|---|---|---|
| `build` のジョブで、展開したバイナリの `--version`(Linux musl、Apple Silicon、Windows) | `release.yml` 46132328751 | 3 つとも成功。Linux は静的リンクも確かめた |
| Intel macOS | — | 動作は確かめていない(クロスビルドのため。注記に書いた) |
| 下書きから `conductor-0.10.4-aarch64-apple-darwin.tar.gz` を取ってきて、`SHA256SUMS` で照合 | 手元の macOS(Apple Silicon) | `OK` |
| 展開した `conductor --version` | 手元の macOS | `conductor 0.10.4` |
| アーカイブの中身 | 手元の macOS | `conductor` と `LICENSE` だけ。テスト専用の補助バイナリは入っていない(project.md の Forbidden) |

判定: 合格。
