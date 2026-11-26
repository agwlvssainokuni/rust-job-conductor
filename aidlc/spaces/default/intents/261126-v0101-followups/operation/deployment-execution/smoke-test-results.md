# smoke test の結果: v0.10.1

出典: `release.yml` の実行 45913389499 の `build` のジョブ、下書きのリリース `v0.10.1` の成果物、`deployment-execution-questions.md`(まとめの確かめ済み)、`operation/deployment-pipeline/deployment-strategy.md` の smoke test。

## `release.yml` の中の smoke test

| 対象 | `build` のジョブ | smoke test |
|---|---|---|
| `x86_64-unknown-linux-musl` | 成功 | 展開したバイナリの `--version`、静的リンクの確かめ(ジョブの中) |
| `aarch64-apple-darwin` | 成功 | 展開したバイナリの `--version`(ジョブの中) |
| `x86_64-pc-windows-msvc` | 成功 | 展開したバイナリの `--version`(ジョブの中) |
| `x86_64-apple-darwin` | 成功 | 行わない(クロスビルド。注記に明記) |

## 手元での確かめ

- 下書きのリリースの 5 つのファイルを取ってきて、`shasum -a 256 -c SHA256SUMS` で 4 つの成果物のチェックサムがすべて `OK`。
- `conductor-0.10.1-aarch64-apple-darwin.tar.gz` を展開し、`conductor --version` が `conductor 0.10.1` を出した(手元の macOS、Apple Silicon)。

## 判定

- 合格。Intel macOS の動作の確認が無いことは、v0.10.0 と同じく注記に書いてある(team.md の Deployment で見送りと決めた範囲)。
