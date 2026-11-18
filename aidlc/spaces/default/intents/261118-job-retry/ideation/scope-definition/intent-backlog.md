# Intent Backlog — 正常終了しなかったジョブの自動リトライ

根拠: `scope-document.md`、`intent-capture/intent-statement.md`。優先度は MoSCoW(Must / Should)。次の段階で、要件に分解する。

| ID | 項目(proto-Unit) | 優先度 | 根拠 | 依存 |
|---|---|---|---|---|
| IB1 | リトライの設定(ジョブごとに、終了コード、最大回数 N、固定の間隔を書ける) | Must | Q1: A、B、C / Q2: A | なし |
| IB2 | リトライの実行(設定した終了コードのとき、間隔を空けて、最大 N 回まで同じジョブを実行し直す) | Must | Q1、`intent-statement.md` の Problem Statement | IB1 |
| IB3 | 設定しない定義ファイルは、これまでと同じ動作のまま | Must(制約) | `intent-statement.md` の Success Metrics | IB1、IB2 |
| IB4 | リトライの途中の様子の表示(何回目か、次まで何秒か) | Must | Q3: A | IB2 |
| IB5 | 回ごとに間隔を伸ばす方式(指数バックオフ) | Should(最初の版に入れる) | Q4: B | IB1、IB2 |

## 進める順番

期限がないため、必須の項目から先に作る。(Q5: A) IB1 → IB2 → IB3 の確認 → IB4 → IB5 の順を、依存から導いた目安とする。

## 範囲外

出力の内容での判定、外部への通知。(`scope-document.md` の範囲外)

## 未確認事項・前提

None.
