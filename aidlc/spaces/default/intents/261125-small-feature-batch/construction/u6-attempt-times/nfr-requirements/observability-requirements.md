# 観測の要件: u6-attempt-times

出典: 要件(`requirements.md` の FR6.5、FR6.8)、機能の仕様(`functional-spec.md` の 3、4)、ビジネスルール(`rules.md` の BR3.1、BR4.1、BR4.2)、契約(`contract-summary.md` の C4、C5)、技術スタック(`technology-stack.md`)。

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR1.5 | `watch` の表の所要時間の列は、英日の見出し(`duration`、`所要時間`)、単位つきの短い形(`17s`、`1m05s`、`2h03m04s`)で、列の幅を今の表と同じ方法でそろえる | `watch` の表のテスト(英日、0 秒、59 秒、60 秒、3600 秒、空) | FR6.5、BR4.2 |
| NFR6.4 | 古い形・壊れたステートの 65 は、今の壊れたステートの表示(種類の文、理由の文、`reset` の案内)をそのまま使い、4 つのコマンド(`run`、`pipeline`、`watch`、`graph --state`)で同じに出る | 各コマンドの流れのテスト | FR6.7、FR6.8、BR3.1 |
