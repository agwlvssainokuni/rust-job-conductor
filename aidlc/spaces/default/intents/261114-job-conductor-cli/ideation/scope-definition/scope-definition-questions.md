# Scope Definition & Prioritization — 質問

## Q1. [プロダクト] 初版で必ず必要な機能(Must Have)

初版(mvp)で、これがなければ使えないと言える機能を選んでください(複数選択可 / select all that apply)。

- A. 個別実行(ジョブIDを指定して1ステップずつ呼び出す)と、警告・特定条件でのスキップ・ジャンプ
- B. 一連の実行(パイプライン全体を `conductor` 自体が駆動する)
- C. 定義ファイル(`conductor.toml`)による、ジョブ・終了コードの対応・遷移ルールの一元定義
- D. 実行状態をローカルのステートファイルで保持する機能
- X. Other (please specify)

[Answer]: A, B, C, D

## Q2. [プロダクト] 状態の確認とリセット機能の優先度

実行状態を確認する機能(`watch`)と、状態を初期化する機能(`reset`)の位置づけを確認します。

- A. 両方とも初版で必須(Must Have)
- B. 初期化(`reset`)は必須、確認(`watch`)は初版に含めるが後回しでもよい(Should Have)
- C. 両方とも初版では後回しでもよい(Could Have)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q3. [プロダクト] YAML ⇔ TOML 変換機能の優先度

定義ファイルの YAML と TOML の相互変換機能(`convert`)は、初版でどう扱いますか。

- A. 初版に含める(Must Have)
- B. 初版に含めるが、他の機能が優先(Should Have / Could Have)
- C. 初版の対象外とし、後続で扱う
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q4. [プロダクト] 機能間の依存関係

機能の作り方の順序について、どの考え方で進めたいですか(実現性評価では複数OS対応と Rust への不慣れがリスクとして挙がっています)。

- A. 依存関係の順(定義ファイルとステート → 個別実行 → 一連の実行 → 補助機能)
- B. リスクの高いものを先に(プロセス実行・複数OS対応を早期に確かめる)
- C. 価値の高いものを先に(個別実行での分岐・スキップを先に動かす)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクト] 機能ごとの期限

特定の機能に結びついた期限はありますか。

- A. ない(期限の制約はなし)
- B. ある(内容は Other に記入)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
