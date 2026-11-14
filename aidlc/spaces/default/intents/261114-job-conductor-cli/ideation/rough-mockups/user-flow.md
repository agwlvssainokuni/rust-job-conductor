# User Flow

上流: intent-statement、scope-document、intent-backlog。ここで描く流れは概念であり、確定した仕様ではない。

## 1. 主な流れ: 個別実行(最優先)

最優先の利用場面は、既存スケジューラから `conductor run <job-id>` が1ステップずつ呼ばれる場面である。 [Q1]

```
スケジューラ が conductor run <job-id> を呼ぶ
        |
        v
 定義ファイルを読む --(誤りあり)--> 原因を表示して異常終了(ジョブは実行しない)
        |(誤りなし)
        v
 状態を参照: このジョブはスキップ対象か?
   |                       |
  はい                    いいえ
   |                       |
   v                       v
 何もせず                ジョブを実行する
 終了コード 0             |
                          v
                   終了ステータスを判定(成功 / 警告 / 失敗)
                          |
                          v
                   遷移ルールに従い、後続のスキップ・ジャンプを状態に書き込む
                          |
                          v
                   スケジューラへ終了コード(0 / 非0)を返す
```

- 定義ファイルに誤りがあるときは、ジョブを一切実行せず、原因を表示して異常終了する。 [Q5]
- スキップ対象のジョブは、実処理をせず終了コード 0 で終わる。 intent-statement の Problem Statement、scope-document の In Scope
- スケジューラに返せるのは 0 / 非0 の2値のみである。 constraint-register の C2
- `conductor` 自身の表示は、オプションで有効にしない限り出さない。 [Q2, Q3]

## 2. 副次的な流れ

| 流れ | 概要 | Source |
|------|------|--------|
| 状態の確認 | `conductor watch` で現在の状態を一覧で1回表示して終了する | [Q4] |
| 状態の初期化 | `conductor reset` で保持している状態を初期化する | scope-document の In Scope |
| 一連の実行 | `conductor pipeline <pipeline-name>` で定義済みの流れ全体を `conductor` が駆動する | scope-document の In Scope |
| 定義ファイルの変換 | `conductor convert` で YAML ⇔ TOML を変換する | scope-document の In Scope |

## Assumptions & Open Questions

None.
