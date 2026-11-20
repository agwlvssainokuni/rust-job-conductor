# スコープの定義: 質問

## Q1. [プロダクトオーナー] 最小限の範囲

v0.9.0 として、最低限出すものは何ですか(意図の整理で、4 つの機能が挙がっています)。

- A. 4 つの機能すべて(環境変数の注入と `.env`、`graph`、`--dry-run`、`init`)
- B. 環境変数の注入と `.env` だけ
- C. 環境変数の注入と `.env`、`graph` の 2 つ
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: C

## Q2. [プロダクトオーナー] 必須と任意

4 つの機能のうち、必須のものと、間に合わなければ後回しにしてよいものは、どれですか。

- A. 4 つとも必須
- B. 環境変数の注入と `.env` は必須。残りの 3 つは、後回しにしてよい
- C. 環境変数の注入と `.env`、`--dry-run` は必須。`graph` と `init` は、後回しにしてよい
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: B

## Q3. [開発者] 機能どうしの依存

`--dry-run` は、環境変数の注入(どの変数がジョブに渡るかを示す)と、`graph`(流れを決める部品)を使う予定です。機能どうしの依存を、どう扱いますか。

- A. 依存の順に作る(`--dry-run` は、環境変数と `graph` の部品ができたあとに作る)
- B. 依存を持たせず、4 つの機能を、それぞれ独立に作る(共通の部品は、重複してもよい)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクトオーナー] 進める順序の方針

4 つのユニットを、どの方針で並べますか。

- A. 依存の順(環境変数と `.env` → `graph` → `--dry-run` → `init`)
- B. 価値の高い順
- C. リスクの高い順(難しいものを先に)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q5. [プロダクトオーナー] 期限

特定の機能、または v0.9.0 全体に、守る期限はありますか。

- A. 期限はない
- B. 特定の機能に期限がある(「その他」に、機能と期限を書く)
- C. v0.9.0 全体に期限がある(「その他」に、期限を書く)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクトオーナー] 第 2 段階の拡張の扱い

`graph --state`(状態を重ねた図)と、`--dry-run --assume`(終了コードを仮定した模擬)は、どう扱いますか。

- A. v0.9.0 の範囲には含めず、4 つのユニットが済んでから、別に判断する
- B. v0.9.0 の範囲に含める
- C. 含めない(予定もしない)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: B

## Q7. [開発者] 既存の定義とステートとの互換

v0.8.0 で書かれた既存の定義ファイルと、ステートファイルは、v0.9.0 でも、変更なしで動く必要がありますか。

- A. 動く必要がある(互換を保つ。新しいキーを足すだけで、既存の意味は変えない)
- B. 動かなくてもよい(互換を壊してもよい)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: B

## Q8. [プロダクトオーナー] 最小限の範囲と、後回しの範囲の食い違い

Q1 では「環境変数の注入と `.env`、`graph` の 2 つ」を最低限出すものとし、Q2 では「環境変数の注入と `.env` だけが必須で、残りの 3 つ(`graph`、`--dry-run`、`init`)は後回しにしてよい」としています。`graph` は、最低限に含まれるのに、後回しにもできる、という食い違いがあります。どちらが意図に合いますか。

- A. `graph` は最低限に含める(Q1 を優先する。必須は、環境変数の注入と `.env`、`graph` の 2 つ。`--dry-run` と `init` は、後回しにしてよい)
- B. `graph` は後回しにしてよい(Q2 を優先する。最低限出すのは、環境変数の注入と `.env` だけ)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
