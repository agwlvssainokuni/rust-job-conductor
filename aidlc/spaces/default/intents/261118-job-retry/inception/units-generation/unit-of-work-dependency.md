# Unit of Work Dependency — 正常終了しなかったジョブの自動リトライ

このファイルは、何が何に依存するか(形)だけを示す。作る順番の推奨や、最も長い経路の特定は、Delivery Planning の仕事である。

## 依存の関係(DAG)

```mermaid
graph LR
  U1[u1-retry-config] --> U2[u2-retry-execution]
  U2 --> U3[u3-retry-display-docs]
```

文章での説明:

- u2-retry-execution は、u1-retry-config に依存する。実行は、u1 が定義に追加したリトライの設定の型(終了コード、最大回数、間隔、倍率、上限)を読むため、u1 のコードがないとビルドもテストも通らない。
- u3-retry-display-docs は、u2-retry-execution に依存する。表示は、実行の流れが出す「リトライの前」の知らせを受けて出すため、u2 のコードがないとビルドもテストも通らない。
- u1 は、ほかの Unit に依存しない。

## Unit 間の接点

| 接点 | 提供する Unit | 使う Unit | 内容 |
|---|---|---|---|
| リトライの設定の型(`JobDefinition` の新しい項目) | u1 | u2 | 終了コードの一覧、最大回数、間隔、倍率、上限。整数と、整数にした間隔で表す |
| リトライの前の知らせ(表示のための情報) | u2 | u3 | 何回目か、最大回数、次までの待ち時間 |

## 並行して進められる組

なし。3 つの Unit は一列に依存している。いずれにしても、1 つずつ順に進める。(`units-generation-questions.md` Q3: A)

## 機械で読む辺の定義

```yaml
units:
  - name: u1-retry-config
    kind: library
    depends_on: []
  - name: u2-retry-execution
    kind: service
    depends_on: [u1-retry-config]
  - name: u3-retry-display-docs
    kind: service
    depends_on: [u2-retry-execution]
```
