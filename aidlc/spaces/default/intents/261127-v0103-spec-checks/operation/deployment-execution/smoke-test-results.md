# smoke test の結果: v0.10.3

出典: `deployment-log.md`、`operation/deployment-pipeline/deployment-strategy.md` の smoke test、`release.yml` の run 46093205491。

| 確かめ | どこで | 結果 |
|---|---|---|
| `build` のジョブで、展開したバイナリの `--version` を出す(Linux musl、Apple Silicon、Windows) | `release.yml` 46093205491 | 3 つとも成功。Linux は静的リンクも確かめた |
| Intel macOS | — | 動作の確認なし(クロスビルド。注記に書いてある) |
| 下書きの `conductor-0.10.3-aarch64-apple-darwin.tar.gz` を取ってきて、`SHA256SUMS` で照合した | 手元の macOS(Apple Silicon) | `OK` |
| 展開した `conductor --version` | 手元の macOS | `conductor 0.10.3` |
| アーカイブの中身 | 手元の macOS | `conductor` と `LICENSE` だけ。テスト専用の補助バイナリは入っていない(project.md の Forbidden) |

判定: 合格。
