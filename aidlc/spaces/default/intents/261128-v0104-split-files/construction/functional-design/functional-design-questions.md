# 機能の設計: 質問

要件(`requirements.md`)の未決の事項のうち、人の判断が要るものだけを質問にします。要件は承認済みで、承認のときにレビューの指摘 R-01〜R-03 を直すと決めています。

次のことは、質問にせず設計の文書に直接書きます。
- 何を数えた数か(R-01)
- 分け終わりを測る基準(R-02)
- コミットの区分(R-03)
- 細かい項目の境目

## Q1. [設計] `render` から分けた 9 つの関数を置く場所

9 つの関数は、どれも `Reporter` の言語と色の判定と、`WatchFrame` の描き直しを使います。

A. 子のファイル `report/render.rs` に、`impl Reporter<'_>` のメソッド(`pub(super)`)として置く。`mod.rs` の `render` は振り分けだけにする(`walk` の `env.rs`・`retry.rs` と同じ形)
B. `mod.rs` の中に、`Reporter` の private のメソッドとして置く(ファイルは増やさない。`mod.rs` は今より大きくは減らない)
X. Other (please specify)

[Answer]: A

## Q2. [設計] 3 つのファイルを分ける順

どのファイルも、「移すだけのコミット」の後に「関数に分けるコミット」を続けます。ファイルの間の順を決めます。

A. `walk`(前例があり、いちばん安全) → `args` → `report`(いちばん大きい)
B. `report`(効き目がいちばん大きい) → `walk` → `args`
X. Other (please specify)

[Answer]: A

## Q3. [設計] 子のファイルの名前

A. 次の名前にする
   - `report/`: `event.rs`(出来事の定義)、`lines.rs`(1 行の組み立て)、`escape.rs`(エスケープと `push_line`)、`render.rs`(Q1 が A のとき)
   - `walk/`: `values.rs`(値の読み取り)、`transition.rs`(遷移)、`cross_check.rs`(ジョブをまたぐ検査)
   - `args/`: `options.rs`(オプションの表と検査)、`tokens.rs`(字句)、`values.rs`(値の解釈)
B. 名前は設計に任せる
X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- `render` から分けた 9 つの関数は、子のファイル `report/render.rs` に、`impl Reporter<'_>` のメソッド(`pub(super)`)として置く。`mod.rs` の `render` は振り分けだけにする(Q1 A)。
- 分ける順は `walk` → `args` → `report`。どのファイルも、移すだけのコミットの後に、関数に分けるコミットを続ける(Q2 A)。
- 子のファイルの名前は次のとおり(Q3 A)。
  - `report/`: `event.rs`、`lines.rs`、`escape.rs`、`render.rs`
  - `walk/`: `values.rs`、`transition.rs`、`cross_check.rs`
  - `args/`: `options.rs`、`tokens.rs`、`values.rs`
- 設計の文書に直接書くことは次のとおり。
  - 35(枝)・37(変種)・40(依頼の数)が、それぞれ何を数えた数か(R-01)
  - 分け終わりを測る基準: `mod.rs` の行数の目安、項目の置き場所、`render` と `build_command` が振り分けだけになっていること(R-02)
  - コミットの区分: 移し・分け・文書・小さな直し・版。FR4.4 は、分けのコミットとは別のコミットにする(R-03)
  - 項目の境目

- Looks correct
- Request changes

[Answer]: Looks correct

## Q4. [設計] 9 種の関数と、`match` の網羅の検査(設計の中で分かった追加の質問)

今の `render` の `match` は、`DisplayEvent` の全変種を網羅しています。変種を足して扱いを書き忘れると、コンパイルが止まります。

9 種の関数はそれぞれ複数の変種を扱い、変種ごとに中身の形が違います。そのため、「1 種 = 1 関数で `&DisplayEvent` を受け取る」形にすると、どの関数にも「担当でない変種」の枝が要ります。

cli の本番のコードには、`unreachable!`・`unwrap`・`expect` が 1 つもありません(パニックの経路を持たない)。

A. 9 種を `render.rs` の中の 9 つの節にする。各節には、変種ごとの小さなメソッド(取り出した中身を受け取る)を並べる。`render` は網羅の `match` で、各枝が 1 行でメソッドを呼ぶ(振り分けだけ)。網羅の検査とパニックの無さを保つ。関数の数は 35 ほどになり、要件の「9 種ごとの関数」は「9 種ごとの節」と読み替える
B. 9 種を 9 つの関数にする。各関数は `Option<String>` を返し、担当でない変種には `None` を返す。`render` は 9 つを順に試す。網羅の検査は失われる(変種を足して書き忘れても、コンパイルが止まらない)
X. Other (please specify)

[Answer]: A
