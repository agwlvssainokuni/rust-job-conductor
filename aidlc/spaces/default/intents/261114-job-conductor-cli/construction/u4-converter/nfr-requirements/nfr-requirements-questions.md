# NFR Requirements(u4-converter)— 質問

u4-converter は、`conductor convert` の変換だけを受け持つ library です。サーバーも常駐の仕組みもないため、性能・可用性・スケーラビリティ・可観測性の要件は、この Unit の種類には当てはまりません。ここで決めるのは、入力の安全(NFR4)と、技術の選択(YAML・TOML のライブラリ)です。判断が必要な3点を確認します。

## Q1. [技術] YAML のライブラリの選び方

Functional Design で、YAML のライブラリに次を求めました: YAML 1.2 のコアスキーマで読む(`yes`・`on` は文字列)、アンカー・エイリアス・マージキーを扱える、重複するキーと再帰するアンカーを検知できる、文法の誤りの位置(行)を取れる、Rust だけで書かれている、保守が続いている。

- A. ライブラリの名前は、いま固定せず、この条件で Code Generation の最初に確かめて選ぶ。条件を満たすものがなければ、設計の規則を見直す(推奨)
- B. 候補を1つ決めて、Code Generation で、それを前提に進める
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [安全] 入力の大きさの上限

Functional Design で仮に置いた値を、確定するかを確認します。

- A. 入力ファイルの上限は 1 MiB、YAML の展開後の要素数の上限は 100,000 のまま確定する(推奨)
- B. 別の値にする(Other で、値を書く)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [技術] TOML の読み書き

u1-core は、定義ファイルの読み取りに `toml` のライブラリ(位置の情報つき)を使います。

- A. 同じ `toml` のライブラリを使い、読み取りも書き出しも賄う。ワークスペースで、バージョンを揃える(推奨)
- B. 書き出しには、別のライブラリを使う
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
