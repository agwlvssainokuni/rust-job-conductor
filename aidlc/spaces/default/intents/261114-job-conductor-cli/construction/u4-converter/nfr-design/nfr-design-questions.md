# NFR Design(u4-converter)— 質問

u4-converter は library のため、可用性・スケーラビリティ・可観測性の設計は、この Unit の種類には当てはまりません。ここで設計するのは、変換の安全な作り方です。要件と、前のステージの結果から案を作りました。判断が必要な2点を確認します。

## Q1. [設計] 変換の途中の表現

YAML と TOML の木を、互いに写すための途中の表現を、どう持つかを確認します。

- A. この Unit の中に、キーと値の木(文字列・整数・浮動小数・真偽・配列・マップ)だけを持つ、小さな型を定義する。YAML と TOML のライブラリの型は、読み書きの端だけで使い、検査(表せない値、大きさ、深さ)は、この型の上で行う(推奨)
- B. 片方のライブラリの値の型を、そのまま途中の表現として使い、もう片方へ直接変換する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [設計] 深さの超過の防ぎ方

読み取りの前の、再帰を使わない深さの走査(NFR4.4 の (b))を、いつ使うかを確認します。

- A. ライブラリが、再帰の深さの制限を持ち、深さ 100,000 の入力でも失敗で返ることを確かめられたら、それに任せる。確かめられないときだけ、事前の走査を足す(推奨)
- B. ライブラリの制限があっても、常に事前の走査を行う
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
