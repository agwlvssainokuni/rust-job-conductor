# User Stories — 質問

画面を持たない CLI のため、ストーリーは「利用者が `conductor` のコマンドを使って何を得たいか」の形で書きます。ストーリーは INVEST の基準で作り、受け入れ基準は Given/When/Then で書きます。優先度(MoSCoW)は、scope-document と requirements の優先度をそのまま引き継ぎます。

## Q1. [プロダクト] ペルソナの分け方

Target Customer には、運用担当者・開発者(組織内)と、ローカルでデバッグする開発者の2つがあります。定義ファイル(`conductor.toml`)を書く人の扱いを確認します。

- A. 2つのペルソナにする。定義ファイルを書く人は、運用担当者・開発者に含める
- B. 3つのペルソナにする。定義ファイルを書く人(定義の作成者)を、別のペルソナとして分ける
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] ストーリーの分け方

ストーリーをどの単位でまとめるかを確認します。

- A. 機能・コマンドごとにまとめる(定義ファイル、個別実行、一連の実行、ステート、`watch`、`reset`、`convert`)
- B. 利用者の流れ(ワークフロー)ごとにまとめる(定義を書く → 実行する → 失敗から再開する → 状態を確認する)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] ストーリーの細かさ

1つのストーリーの大きさを確認します。

- A. 要件の主なまとまり(FR1〜FR7)ごとに、1〜数個のストーリーにする(合計 12〜18 個程度)
- B. サブ要件(FR1.1 など)に近い細かさで、ストーリーを多めに作る(合計 25〜35 個程度)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [プロダクト] 非機能要件の扱い

起動オーバーヘッド、単一バイナリ、3つのOS対応、テストの基準などの非機能要件の扱いを確認します。

- A. 独立したストーリーにはせず、関係する機能ストーリーの受け入れ基準に取り込む
- B. 利用者に見える非機能要件(起動の速さ、3つのOS対応)は、独立したストーリーにする
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
