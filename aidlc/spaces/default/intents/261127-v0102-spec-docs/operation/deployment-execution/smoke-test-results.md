# smoke test の結果: v0.10.2

出典: `release.yml` の実行 46080776462 の `build` のジョブ、下書きのリリース `v0.10.2` の成果物、`deployment-execution-questions.md`(まとめの確かめ済み)、`operation/deployment-pipeline/deployment-strategy.md`。

## `release.yml` の中の smoke test

| 対象 | `build` のジョブ | smoke test |
|---|---|---|
| `x86_64-unknown-linux-musl` | 成功 | `--version`、静的リンクの確かめ |
| `aarch64-apple-darwin` | 成功 | `--version` |
| `x86_64-pc-windows-msvc` | 成功 | `--version` |
| `x86_64-apple-darwin` | 成功 | 行わない(クロスビルド。注記に明記) |

## 手元での確かめ

- 下書きの 5 つのファイルを取ってきて、`shasum -a 256 -c SHA256SUMS` で 4 つの成果物がすべて `OK`。
- `conductor-0.10.2-aarch64-apple-darwin.tar.gz` を展開し、`conductor --version` が `conductor 0.10.2`(手元の macOS、Apple Silicon)。

## 判定

- 合格。
