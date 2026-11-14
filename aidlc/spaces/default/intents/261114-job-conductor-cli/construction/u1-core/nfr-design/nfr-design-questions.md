# NFR Design(u1-core)— 質問

u1-core は library のため、可用性・スケーラビリティ・可観測性の設計は、この Unit の種類には当てはまりません。ここで設計するのは、定義ファイルの読み取りの安全性です。前のステージの要件と、レビューからの引き継ぎから、設計の案を作りました。判断が必要な3点を確認します。(案は security-design.md、logical-components.md にあります)

## Q1. [設計] TOML の入れ子の深さの上限

深すぎる入れ子は、読み取りのときに、プログラムを異常終了させるおそれがあります。実際の定義(ジョブと遷移の、浅い構造)には、十分に大きい値を上限にします。

- A. 32段を上限とし、超えたものは、文法の誤りとして拒否する(推奨)
- B. 上限は置かず、ライブラリの既定に任せる
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q2. [設計] 定義の誤りの種類の追加

「文法の誤り」と「ファイルが大きすぎる」を、既存の種類(型の誤り、読めない)に含めず、新しい種類として分けます。

- A. `syntax`(文法の誤り・深すぎる入れ子)と `tooLarge`(大きさの上限の超過)を、新しい種類として足す(推奨)
- B. 新しい種類は足さず、既存の `typeMismatch` と `unreadable` に含める(種類は増えないが、意味が広がる)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q3. [運用] 依存のライセンス・脆弱性の確認

依存のライブラリが、ライセンスの問題や、既知の脆弱性を持たないかを確認する仕組み(`cargo-deny`、`cargo-audit`)を、どう扱うかを確認します。

- A. CI で確認する仕組みとして、後の CI Pipeline のステージで導入を決める(今は、候補として記録するだけ)(推奨)
- B. 今回の実装から、必ず導入する
- C. 導入しない
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
