# smoke test の結果: v0.10.0

出典: `deployment-log.md`、`operation/deployment-pipeline/deployment-strategy.md` の smoke test、`release.yml` の実行 45727642798、下書きの成果物を手元に取ってきた確かめ(2026-11-26)。

| 確かめ | 場所 | 結果 |
|---|---|---|
| 展開したバイナリの `--version`(Linux musl、Apple Silicon、Windows) | `release.yml` の `build` | 成功 |
| Linux のバイナリの静的リンク | `release.yml` の `build` | 成功 |
| Intel macOS | — | クロスビルドのため動作の確認なし(注記に記載。team.md の決定) |
| 4 つのファイルが `SHA256SUMS` と一致 | 手元(`shasum -a 256 -c`) | 4 件 OK |
| アーカイブの中身が `conductor`(Windows は `conductor.exe`)と `LICENSE` だけ(テスト専用の補助バイナリを含まない) | 手元(`tar tzf`、`unzip -l`) | OK |
| Apple Silicon 版を展開して `--version` | 手元(macOS) | `conductor 0.10.0` |
