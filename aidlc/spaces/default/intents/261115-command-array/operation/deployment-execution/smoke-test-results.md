# Smoke Test Results(v0.2.0)

## Sources

- Initial description: ジョブの command に、文字列に加えて、文字列の配列も書けるようにする。 [desc]
- Workflow-selected scope: express [scope]
- `deployment-log.md`(下書きのリリースの内容) [scope]

## ワークフローの中のスモークテスト(`release.yml` の `build`)

| ターゲット | 確認 | 結果 |
|-----------|------|------|
| x86_64-unknown-linux-musl | まとめたファイルを展開して `conductor --version`。静的リンク | 成功 |
| aarch64-apple-darwin | 展開して `conductor --version` | 成功 |
| x86_64-pc-windows-msvc | 展開して `conductor --version` | 成功 |
| x86_64-apple-darwin | クロスビルド。実行できないため、確認を省く(リリースの本文に記載) | 省略 |

## 手元の確認(下書きのリリースから、実際に取得して実行。macOS・Apple シリコン)

| 確認 | 結果 |
|------|------|
| `SHA256SUMS` と、取得した `conductor-0.2.0-aarch64-apple-darwin.tar.gz` の SHA-256 が一致 | `OK` |
| 展開すると、`conductor` と `LICENSE` がある | 確認 |
| `conductor --version` | `conductor 0.2.0` |
| 配列の `command` のジョブ(`["sh","-c","echo \"[$1]\"; exit 0","x","a b"]`)を `pipeline -v` で実行 | 終了コード 0。`[a b]` と表示(`a b` が 1 つの引数として渡った) |
| `convert --to yaml` で配列が YAML の列になる | 確認 |

## 判定

合格。Linux と Windows の実行ファイルの実行は、ワークフローのスモークテストと CI のテストで確かめた(手元では、macOS だけ)。

## Assumptions & Open Questions

None.
