# 意図の記述: v0.9.0 の機能拡張

## Problem Statement

- ジョブに値(環境変数)を渡す手段が定義の中になく、運用でスクリプトで包むなどの手間がかかっている。 [Q1]
- この課題に対し、`conductor` v0.9.0 で、環境変数の注入と `.env` ファイル、`conductor graph`、`--dry-run`、`conductor init` の 4 つの機能を足す。 [desc]
- 環境変数の注入と `.env` ファイルは、`[pipeline.env]`、`[job.env]`、`env_file`、組み込みの `CONDUCTOR_*` で行う。 [desc]
- `conductor graph` は、`git log --graph` 風の text と Mermaid でフローを出す。 [desc]
- `--dry-run` は、ステートを読んで、再開位置まで示す。 [desc]
- `conductor init` は、ひな形を作る。ファイルが既にあるときは、終了コード 69 になる。 [desc]
- 4 つのユニットは、この順(環境変数の注入と `.env`、`graph`、`--dry-run`、`init`)で進める。 [desc]

## Target Customer

- 主な利用者は、自分たちのチーム(内部の開発者と運用担当)である。 [Q2]
- 抱えている不便は、ジョブに値を渡すために、運用でスクリプトで包むなどの手間がかかっていることである。 [Q1]

## Success Metrics

- 4 つの機能が、決めた仕様どおりに動く。 [Q3]
- 3 つの OS の CI で成功する。 [Q3]
- リリースできる。 [Q3]

## Initiative Trigger

- v0.8.0(リトライ)のあとの、計画的な機能の拡充である。 [Q4]

## Initial Scope Signal

- ワークフローが選んだスコープ(workflow-selected): `feature`。 [scope]
- 利用者が確認した、この最初の版に含めないもの: 並列実行、ジョブのタイムアウト、排他制御、`error` への遷移。 [Q8]

## Assumptions & Open Questions

None.
