# 意図の整理: 質問

## Sources

- [desc] Initial description: "v0.9.0 の機能拡張: (1) 環境変数の注入と .env ファイル(`[pipeline.env]`、`[job.env]`、`env_file`、組み込みの `CONDUCTOR_*`)、(2) `conductor graph`(`git log --graph` 風の text と Mermaid)、(3) `--dry-run`(ステートを読んで再開位置まで示す)、(4) `conductor init`(ひな形の作成、既にあるときは終了コード 69)。ユニットはこの順の 4 つ。決定事項の詳細は、メモリの project_v090_feature_plan.md を参照。"
- [scope] Workflow-selected scope: `feature`.

## Q1. [プロダクトオーナー] 解決したい課題

4 つの機能(環境変数の注入と `.env`、`graph`、`--dry-run`、`init`)を、v0.9.0 として足す一番の理由は何ですか。成功の基準と、優先順位の判断に使います。

- A. ジョブに値(環境変数)を渡す手段が定義の中になく、運用でスクリプトで包むなどの手間がかかっている
- B. 実行前に、再開位置やコマンドを確認する手段がなく、運用の事故や調査の手間が出ている
- C. 定義ファイルの作り始めと、フローの見える化の手段が弱く、使い始めにくい
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクトオーナー] 想定する利用者

この機能を使う人は、主に誰ですか。

- A. 自分たちのチーム(内部の開発者と運用担当)
- B. 公開された配布物を使う、外部の利用者
- C. 内部と外部の両方
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクトオーナー] 成功の基準

v0.9.0 の成功を、何で判断しますか。

- A. 4 つの機能が、決めた仕様どおりに動き、3 つの OS の CI で成功して、リリースできる
- B. 既存の定義とステート(v0.8.0)が、変更なしでそのまま動く(後方互換が保たれる)
- C. A と B の両方を満たす
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクトオーナー] きっかけ

この拡張を、今行う理由は何ですか。

- A. 運用での不便や、利用者からの要望がある
- B. v0.8.0(リトライ)のあとの、計画的な機能の拡充
- C. 外部からの依頼や、要請がある
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: B

## Q5. [プロダクトオーナー] 関係者

この作業で、意見や承認が必要な人は誰ですか。

- A. 開発者(作業を依頼している本人)だけ
- B. 開発者と、運用を担当する人
- C. 開発者、運用担当、配布物を使う利用者
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクトオーナー] 範囲と優先順位を決める人

4 つの機能の範囲や、優先順位は、誰が決めますか。

- A. 開発者本人が決める
- B. 開発者と運用担当が、相談して決める
- C. 開発者のほかに、承認する人がいる
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q7. [プロダクトオーナー] 周知と報告

完成した変更を、関係者や利用者に、どう伝えますか。

- A. 連絡の要件はない(リリースノートだけ)
- B. README の更新と、リリースノートで伝える
- C. README、リリースノートに加えて、別の連絡(説明の場や通知)を行う
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: B

## Q8. [プロダクトオーナー] 今回の最初の版に含めないもの

この最初の版(v0.9.0)で、やらないことは何ですか。

- A. 並列実行、ジョブのタイムアウト、排他制御、`error` への遷移は含めない
- B. `graph --state` と `--dry-run --assume`(第 2 段階の拡張)は含めない
- C. A と B の両方を含めない
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
