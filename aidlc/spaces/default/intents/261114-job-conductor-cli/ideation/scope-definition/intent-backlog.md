# Intent Backlog

上流: intent-statement、feasibility-assessment、constraint-register。優先度は MoSCoW、順序は「依存関係の順」[Q4]。以下は Units Generation の入力となる暫定的なまとまり(proto-Unit)であり、確定した分割ではない。

| 順 | Proto-Unit | 含む機能 | 優先度 | Source |
|----|-----------|----------|--------|--------|
| 1 | 定義ファイルとステート管理 | `conductor.toml` による一元定義、ローカルのステートファイルでの状態保持 | Must Have | [Q1, Q4] |
| 2 | 個別実行 | ジョブIDを指定した1ステップ実行、警告・特定条件でのスキップ・ジャンプ | Must Have | [Q1, Q4] |
| 3 | 一連の実行 | パイプライン全体の駆動 | Must Have | [Q1, Q4] |
| 4 | 補助機能: 状態の初期化 | `reset` | Must Have | [Q2, Q4] |
| 5 | 補助機能: 状態の確認 | `watch` | Should Have | [Q2, Q4] |
| 6 | 補助機能: 定義ファイルの変換 | `convert`(YAML ⇔ TOML) | Should Have / Could Have | [Q3, Q4] |

## Dependencies

- 順 2 と 3 は、順 1(定義ファイルとステート)に依存する。 [Q4]
- 順 4〜6 は補助機能として、実行系の後に扱う。 [Q4]

## Risks Carried Forward

- 複数OS対応と Rust への不慣れは、実現性評価のリスク(R1, R2)として引き継ぐ。 raid-log の R1, R2

## Assumptions & Open Questions

None.
