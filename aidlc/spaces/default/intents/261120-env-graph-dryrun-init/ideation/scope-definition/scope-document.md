# スコープ文書: v0.9.0 の機能拡張

出典: 意図の記述(`ideation/intent-capture/intent-statement.md`)、スコープ定義の質問(`scope-definition-questions.md`)。

## 1. 目的(意図の記述より)

- 課題: ジョブに値(環境変数)を渡す手段が定義の中になく、運用でスクリプトで包むなどの手間がかかっている(intent-statement.md の Problem Statement)。
- 利用者: 自分たちのチーム(内部の開発者と運用担当)(intent-statement.md の Target Customer)。
- きっかけ: v0.8.0(リトライ)のあとの、計画的な機能の拡充(intent-statement.md の Initiative Trigger)。
- 成功の基準: 4 つの機能が決めた仕様どおりに動き、3 つの OS の CI で成功して、リリースできる(intent-statement.md の Success Metrics)。

## 2. 範囲の内側(In scope)

| 項目 | 内容 | 出典 |
|------|------|------|
| 環境変数の注入と `.env` | `[pipeline.env]`、`[job.env]`、`env_file`、組み込みの `CONDUCTOR_*` | 意図の記述(Initial description)、Q1 |
| `conductor graph` | `git log --graph` 風の text と Mermaid | 意図の記述(Initial description)、Q1、Q8 |
| `--dry-run` | ステートを読んで、再開位置まで示す | 意図の記述(Initial description)、Q1 |
| `conductor init` | ひな形の作成。既にあるときは終了コード 69 | 意図の記述(Initial description)、Q1 |
| `graph --state` | 状態を重ねた図 | Q6 |
| `--dry-run --assume` | 終了コードを仮定した模擬 | Q6 |

## 3. 範囲の外側(Out of scope)

| 項目 | 出典 |
|------|------|
| 並列実行 | 意図の整理 Q8 |
| ジョブのタイムアウト | 意図の整理 Q8 |
| 排他制御 | 意図の整理 Q8 |
| `error` への遷移 | 意図の整理 Q8 |

## 4. 優先順位

| 区分 | 項目 | 出典 |
|------|------|------|
| 必須(最低限出すもの) | 環境変数の注入と `.env`、`graph` | Q1、Q2、Q8 |
| 後回しにしてよい | `--dry-run`、`init` | Q2、Q8 |
| 範囲に含めるが、優先順位は未確認 | `graph --state`、`--dry-run --assume` | Q6(優先順位は、Q2 に挙がっていない) |

## 5. 順序と依存

- 順序の方針は、依存の順: 環境変数の注入と `.env` → `graph` → `--dry-run` → `init`(Q4)。
- 依存: `--dry-run` は、環境変数の注入と `graph` の部品ができたあとに作る(Q3)。

## 6. 互換と期限

- v0.8.0 で書かれた既存の定義ファイルとステートファイルは、変更なしで動かなくてもよい(互換を壊してもよい)(Q7)。
- 守る期限はない(Q5)。

## 7. 価値の流れ(Value stream map)

定義を書く人 → 環境変数と `.env` で、ジョブに値を渡す(必須) → `graph` で、流れを確認する(必須) → `--dry-run` で、再開位置と実行内容を確認する(後回しにしてよい) → `init` で、定義の雛形を作る(後回しにしてよい)。

- `graph --state` と `--dry-run --assume` は、それぞれ `graph` と `--dry-run` の拡張である(Q6)。

## 8. 上流成果物との対応(Upstream traceability)

| 上流(intent-statement.md) | このスコープでの扱い |
|----------------------------|----------------------|
| Problem Statement | 範囲の内側の 6 項目に展開 |
| Target Customer | 利用者は内部のチーム(変更なし) |
| Success Metrics | 優先順位の「必須」を、最初に満たす |
| Initiative Trigger | 計画的な拡充として、順序を決める |
| Initial Scope Signal | 範囲の外側の 4 項目を、そのまま引き継ぐ |

## 9. Assumptions & Open Questions

- `graph --state` と `--dry-run --assume` の優先順位(必須か、後回しにしてよいか)は、確認していない。
- Q7 により互換を保つ義務はないが、実際にどのキーや書式を変えるかは、後の工程(要件分析)で決める。
