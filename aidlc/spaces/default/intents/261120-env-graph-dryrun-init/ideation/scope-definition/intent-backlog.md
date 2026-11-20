# 意図のバックログ: v0.9.0 の機能拡張

出典: スコープ文書(`scope-document.md`)、スコープ定義の質問(`scope-definition-questions.md`)。優先順位は MoSCoW で示す。

## 1. バックログ(ユニットの候補 = proto-Unit)

| 順 | ID | 項目 | 優先順位 | 依存 | 出典 |
|----|----|------|----------|------|------|
| 1 | B1 | 環境変数の注入と `.env`(`[pipeline.env]`、`[job.env]`、`env_file`、組み込みの `CONDUCTOR_*`) | Must | なし | Q1、Q2、Q4 |
| 2 | B2 | `conductor graph`(`git log --graph` 風の text と Mermaid) | Must | なし | Q1、Q4、Q8 |
| 3 | B3 | `--dry-run`(ステートを読んで、再開位置まで示す) | Could(後回しにしてよい) | B1、B2 | Q2、Q3、Q4、Q8 |
| 4 | B4 | `conductor init`(ひな形の作成。既にあるときは終了コード 69) | Could(後回しにしてよい) | なし(ひな形に、B1 の `env` の例を入れる予定) | Q2、Q4、Q8 |
| 5 | B5 | `graph --state`(状態を重ねた図) | 未確認 | B2 | Q6 |
| 6 | B6 | `--dry-run --assume`(終了コードを仮定した模擬) | 未確認 | B3 | Q6 |

- 順序の方針は、依存の順(Q4)。
- B5 と B6 は、第 2 段階の拡張として、範囲に含める(Q6)。優先順位は、確認していない。

## 2. 依存の図

B1 → B3、B2 → B3、B2 → B5、B3 → B6。B4 は、B1 の `env` の例を使う予定だが、必須の依存ではない。

## 3. 範囲外(バックログに入れないもの)

並列実行、ジョブのタイムアウト、排他制御、`error` への遷移(意図の整理 Q8)。

## 4. Assumptions & Open Questions

- B5 と B6 の優先順位(Must/Should/Could)は、確認していない。
- B4 が B1 に依存するかどうかは、設計の段階で決める。
