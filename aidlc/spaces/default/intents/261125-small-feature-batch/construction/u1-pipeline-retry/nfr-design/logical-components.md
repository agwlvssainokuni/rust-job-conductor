# 論理の部品: u1-pipeline-retry

出典: 技術の選択(`tech-stack-decisions.md`)、セキュリティの要件(`security-requirements.md`)、性能の要件(`performance-requirements.md`)、規模の要件(`scalability-requirements.md`)、信頼性の要件(`reliability-requirements.md`)、観測の要件(`observability-requirements.md`)、機能の仕様(`functional-spec.md` の 1)、契約(`contract-summary.md` の C3)、コンポーネントの一覧(`components.md` の CoreDefinition、CoreTypes)。

配布するのは単一のバイナリ `conductor` で、サービスの境界やインフラの部品は無い。このユニットが触る論理の部品と、失敗の影響の範囲を示す。

| 部品 | 場所 | このユニットでの変化 | 当てはまる非機能の設計 |
|---|---|---|---|
| CoreDefinition(定義の検証) | `crates/core/src/definition/walk/mod.rs`、`walk/retry.rs` | `[pipeline]` の `retry` の読み取り、ジョブの `retry` の 3 つの形、既定との重なりの検査、実効のリトライの組み立て | セキュリティ(入力の検証、誤りの文)、性能(ジョブごとに 1 回)、信頼性(後方互換) |
| CoreTypes(定義の型) | `crates/core/src/types.rs` | 形は変えない。`JobDefinition.retry` の文書を「実効のリトライ」に直す | 信頼性(後方互換) |
| CoreDefinition の誤りの種類 | `crates/core/src/error.rs` | 種類も種類の文も変えない | セキュリティ、信頼性 |
| リトライの判断と流れ、表示 | core の `flow`、`plan`、cli の `report`、`pipeline` | 変えない(実効のリトライを読むだけ) | 観測(同じ表示) |
| 文書とデモ | `README.md`、`demo/retry.toml`、`demo/windows/retry.toml`、`demo/README.md` | 既定と打ち消しの説明と例を足す | 信頼性(デモのテスト) |

## 失敗の影響の範囲

- 誤りは定義の検証で止まり(64)、ジョブを 1 つも動かさない。影響は、その定義ファイルを使う 1 回の実行だけ。
- 既定の設定を誤ってもパイプラインの途中では効かない。検証を通った既定だけが組み立てに入る。

## 共有の資源

- 無い。ステートとログのファイルは、このユニットでは変えない。
