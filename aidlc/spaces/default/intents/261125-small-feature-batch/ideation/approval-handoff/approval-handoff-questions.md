# 承認と引き継ぎ: 質問

## Q1. [プロダクトオーナー] 意図と範囲への同意

意図の表明(`intent-statement.md`)とスコープ文書(`scope-document.md`)の内容に、関係者は同意していますか。関係者は開発者ご自身 1 人です(意図の表明 Q6)。

- A. 開発者ご自身が、意図と範囲に同意している
- B. 直したい点がある(書いてください)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクトオーナー] リスクの認識

次の 3 点をリスクとして理解したうえで進めますか。(1) 時刻を持たない v0.9.9 までのステートファイルは、v0.10.0 で読めなくなる(スコープ Q5)。(2) 7 つとも必須なので、1 つが手間取ると v0.10.0 の公開全体が遅れる(スコープ Q1)。(3) 各機能の合否の基準と境界(リトライの既定とジョブ側の設定の優先関係、`watch` の継続表示の終わり方など)は、まだ決まっていない(意図の表明のレビュー R-01・R-04)。

- A. 3 点ともリスクとして理解したうえで進める
- B. 一部について、先に手を打ちたい(どれを、どうするかを書いてください)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクトオーナー] 予算と資源

この作業のために、追加の予算や人員は要りますか。

- A. 要らない(開発者ご自身が AI の支援で進める)
- B. 要る(内容を書いてください)
- C. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクトオーナー] 進めるかどうかの判断(Go/No-go)

意図の把握とスコープの定義が済んだ今、次の工程(Inception: 要件などを決める工程)に進みますか。

- A. 進める(Go)
- B. 条件つきで進める(条件を書いてください)
- C. 進めない(No-go)
- D. Not yet defined(まだ決めていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- 意図と範囲(Q1): 開発者ご自身が同意している
- リスク(Q2): 古いステートが読めなくなること、7 つとも必須で 1 つの遅れが全体に響くこと、合否の基準と境界が未定であることの 3 点を、リスクとして理解したうえで進める
- 予算と資源(Q3): 追加は要らない。開発者ご自身が AI の支援で進める
- 判断(Q4): Go。次の工程(Inception)に進む

- Looks correct
- Request changes

[Answer]: Looks correct
