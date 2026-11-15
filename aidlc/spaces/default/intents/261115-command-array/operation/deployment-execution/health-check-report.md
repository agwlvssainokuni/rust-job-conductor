# Health Check Report(v0.2.0)

## Sources

- Initial description: ジョブの command に、文字列に加えて、文字列の配列も書けるようにする。 [desc]
- Workflow-selected scope: express [scope]
- `deployment-log.md`、`smoke-test-results.md` [scope]

## 対象

`conductor` は、利用者の手元で動く単一の実行ファイルで、サービスとして動かさない(常駐しない、ヘルスチェックの窓口がない)。このため、健全性は、「配布した成果物が、正しく作られ、動くこと」で確かめる。

| 確認項目 | 基準 | 結果 |
|----------|------|------|
| 成果物の完全性 | 4 つのまとめたファイルと `SHA256SUMS` がそろう | そろっている(5 アセット) |
| 版の一致 | タグ・`Cargo.toml`・`conductor --version` が一致 | 0.2.0 で一致 |
| 品質の関門 | リリースの前に、`ci.yml` の全ジョブが成功 | 成功 |
| エラー率・性能 | 起動のオーバーヘッドの測定(警告のみのジョブ)が実行される | 実行された(基準内かは、ジョブの要約で確認) |
| 互換 | 文字列の `command` の既存の定義が、そのまま動く | 既存 564 件のテストが成功 |

## 公開後の確認(人が行う)

1. 下書きの内容(5 つのファイルとリリースの本文)を確認して公開する。
2. 公開後、別の環境で、README の手順(取得、SHA-256 の確認、展開、`--version`)が動くことを確かめる。
3. Windows の Ctrl+C の手動確認(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)は、v0.1.0 と同様に、この版でも、人の確認の項目。

## Assumptions & Open Questions

None.
