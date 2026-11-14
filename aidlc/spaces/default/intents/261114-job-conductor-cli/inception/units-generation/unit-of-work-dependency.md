# Unit of Work Dependency(依存関係)

上流: unit-of-work、components(部品どうしは Cli を通してのみ呼び合う。ADR-002)。この文書は、トポロジー(何が何に依存するか)だけを記述する。推奨の実装順序や、クリティカルパスは、Delivery Planning で決める。

## 依存の DAG

- U3(u3-cli)は、U1(u1-core)、U2(u2-runtime)、U4(u4-converter)に依存する。Cli が、他の部品を呼び出す唯一の調整役であるため(ADR-002)。
- U1、U2、U4 は、互いに依存しない。
- 循環はない。

```
U1 u1-core ------\
U2 u2-runtime ----+--> U3 u3-cli
U4 u4-converter -/
```

Text fallback: u3-cli は、u1-core、u2-runtime、u4-converter のすべてに依存する。この3つは、互いに依存しない。

```yaml
units:
  - name: u1-core
    kind: library
    depends_on: []
  - name: u2-runtime
    kind: library
    depends_on: []
  - name: u4-converter
    kind: library
    depends_on: []
  - name: u3-cli
    kind: service
    depends_on: [u1-core, u2-runtime, u4-converter]
```

## Unit どうしの接点

| 接点 | 提供する Unit | 使う Unit | 内容 |
|------|---------------|-----------|------|
| 定義の読み込みと検証 | U1 | U3 | PipelineDefinition、JobDefinition の取り出しと、検証の結果 |
| 結果の分類と次の動きの判断 | U1 | U3 | JobOutcome、NextStep(終了コード、次に実行・スキップするジョブ、完了と再開位置) |
| ジョブの起動と結果 | U2 | U3 | JobExecution(終了コード、起動失敗、シグナル終了、中断) |
| ステートの読み書きと照合 | U2 | U3 | PipelineState、JobState、定義との食い違いの結果 |
| 変換 | U4 | U3 | 入力ファイルから、指定した形式への変換の結果 |
| データ型のみの依存 | U1、U2 | U1、U2、U3 | Flow(U1)は、JobState(U2)・JobExecution(U2)の形を引数として受け取る。U2 は、PipelineDefinition(U1)の、ジョブIDと記述順だけを受け取る。この型の共有のしかたは、Contract Design で定める |

## データ型のみの依存と、Unit の独立性

- 「データ型のみの依存」により、U1 は U2 のエンティティの形を、U2 は U1 のエンティティの形を参照する。このため、2つの Unit の型の定義の共有が必要になる。共有のしかたは、Contract Design(次のステージ)で、型の所有と合わせて決める。
- 各 Unit は、自分のコードと、依存する Unit だけで、ビルドとテストが通るように作る。[Q2] U1 と U2 のあいだの共有する型は、依存の辺ではなく、共有された型の定義として扱う。

## 並行にできる Unit

- U1、U2、U4 は、互いに依存しないため、DAG の上では並行にできる。複数の正しい順序が存在する。
- 実際の進め方は、1人で、依存関係の順に1つずつ進める。ただし、並行にできる余地を残す。[Q2]

## Assumptions & Open Questions

- U1 と U2 のあいだの共有する型(JobExecution、JobState、PipelineDefinition の形など)を、どちらの Unit の定義にするか、または共有の定義を別に置くかは、Contract Design で決める。決まるまで、この文書では「データ型のみの依存」として扱う。 [assumption]
