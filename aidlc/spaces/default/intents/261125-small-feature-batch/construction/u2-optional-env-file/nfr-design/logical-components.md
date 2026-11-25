# 論理の部品: u2-optional-env-file

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C1)。

| 部品 | 場所 | 変化 | 当てはまる非機能の設計 |
|---|---|---|---|
| CoreTypes | `crates/core/src/types.rs` | EnvFileRef を足し、`env_file` の型を変える | 信頼性(後方互換) |
| CoreDefinition | `crates/core/src/definition/walk/env.rs` | 表の項目の読み取りと検査 | セキュリティ(入力の検証、誤りの文) |
| CoreEnvFile | `crates/core/src/envfile/mod.rs`、`merge.rs` | `env_file_refs` が EnvFileRef の並び(required の論理和)を返す。統合は `path` で引く | 性能(比例する手間) |
| RuntimeEnvFile | `crates/runtime/src/envfile.rs` | 解決したパスでまとめ、無くてもよいものが無ければ飛ばす | セキュリティ、観測 |
| CliDeps / CliFlows | `crates/cli/src/deps.rs`、`ctx.rs` | `load_env_files` の引数の型だけ | 信頼性 |
| 文書とデモ | `README.md`、`demo/env.toml`、`demo/windows/env.toml`、`demo/README.md` | 表の形と無くてもよいファイルの例 | 信頼性(デモのテスト) |

## 失敗の影響の範囲

- 読み取りの誤りは今と同じく 64 で、ジョブを 1 つも動かさない。無くてもよいファイルが無いことは失敗ではないため、影響は「その変数が渡らない」ことだけ。

## 共有の資源

- `C1` の環境の統合は U3 と共有する(U3 が消す名前を足す)。このユニットは統合の出力の形を変えない。
