# Unit of Work Dependency(依存関係)

上流: unit-of-work、components(部品どうしは Cli を通してのみ呼び合う。ADR-002)。この文書は、トポロジー(何が何に依存するか)だけを記述する。推奨の実装順序や、クリティカルパスは、Delivery Planning で決める。

## 依存の DAG

- U2(u2-runtime)は、U1(u1-core)に依存する。U2 は、U1 が持つ型(定義の PipelineDefinition・JobDefinition、および共有する型)を使うため。
- U3(u3-cli)は、U1(u1-core)、U2(u2-runtime)、U4(u4-converter)に依存する。Cli が、他の部品を呼び出す唯一の調整役であるため(ADR-002)。
- U1 と U4 は、互いに依存しない。U4 は、U1・U2 のどちらにも依存しない。
- 循環はない。

```
U1 u1-core <------ U2 u2-runtime <-----\
   ^                                     \
   |                                      U3 u3-cli
   +------------------------------------/
U4 u4-converter ----------------------->/
```

Text fallback: u2-runtime は u1-core に依存する。u3-cli は、u1-core、u2-runtime、u4-converter のすべてに依存する。u4-converter は、他のどの Unit にも依存しない。

```yaml
units:
  - name: u1-core
    kind: library
    depends_on: []
  - name: u4-converter
    kind: library
    depends_on: []
  - name: u2-runtime
    kind: library
    depends_on: [u1-core]
  - name: u3-cli
    kind: service
    depends_on: [u1-core, u2-runtime, u4-converter]
```

`kind: service` は「デプロイされる実行ファイル」、`kind: library` は「単独の実行を持たない再利用コード」を表す。U3 は `conductor` の実行ファイルそのものであり、U1・U2・U4 は、U3 に組み込まれる。

## 共有する型の置き場(R-01 への対応)

Flow(U1)は、JobState と JobExecution の形を引数として受け取り、StateStore・JobRunner(U2)は、PipelineDefinition と JobDefinition の形を参照する。このままでは、U1 と U2 が型の上で互いに依存し、依存の辺を超えた循環が生じる(別のクレートにすれば、実際の循環になる)。これを避けるため、次のように決める。

- **共有する型(データの形だけの定義)は、すべて U1(u1-core)に置く。** 対象は、PipelineDefinition、JobDefinition、JobExecution、JobState、PipelineState、JobOutcome、NextStep(DisplayEvent は U3 が持つ)。
- **エンティティの所有は、components.md のまま変えない。** 型の置き場が U1 であることは、振る舞いの所有(たとえば、JobState の読み書きは StateStore)を変えない。
- U2 は、U1 の型を使うため、U1 に依存する。U1 は、U2 に依存しない。
- 型の定義は、データの形だけを持ち、入出力や OS への依存を持たない。これにより、U1 は、ファイルとプロセスに触れない部分を保てる。

## Unit どうしの接点

| 接点 | 提供する Unit | 使う Unit | 内容 |
|------|---------------|-----------|------|
| 共有する型の定義 | U1 | U2、U3 | PipelineDefinition、JobDefinition、JobExecution、JobState、PipelineState、JobOutcome、NextStep の、データの形 |
| 定義の読み込みと検証 | U1 | U3 | PipelineDefinition、JobDefinition の取り出しと、検証の結果 |
| 結果の分類と次の動きの判断 | U1 | U3 | JobOutcome、NextStep(終了コード、次に実行・スキップするジョブ、完了と再開位置) |
| ジョブの起動と結果 | U2 | U3 | JobExecution(終了コード、起動失敗、シグナル終了、中断)。起動するコマンドと、成功・警告の終了コードの扱いは、JobDefinition から受け取る |
| ステートの読み書きと照合 | U2 | U3 | PipelineState、JobState、定義との食い違いの結果。照合には、PipelineDefinition のジョブIDと記述順を使う |
| 変換 | U4 | U3 | 入力ファイルから、指定した形式への変換の結果 |

## Unit の独立性

- 各 Unit は、自分のコードと、依存する Unit だけで、ビルドとテストが通るように作る。[Q2]
  - U1: 依存なし。
  - U4: 依存なし。
  - U2: U1 の型だけを使う。
  - U3: U1、U2、U4 を使う。
- U2 が使う U1 の型は、データの形だけであり、U1 のロジック(Flow、DefinitionLoader)を呼ぶ必要はない。

## 並行にできる Unit

- U1 と U4 は、互いに依存しないため、DAG の上では並行にできる。U2 は、U1 の型の定義が済んでいれば、U1 のロジックの完成を待たずに進められる。複数の正しい順序が存在する。
- 実際の進め方は、1人で、依存関係の順に1つずつ進める。ただし、並行にできる余地を残す。[Q2]

## Assumptions & Open Questions

- 共有する型を、U1 の中でどのまとまり(モジュールなど)に置くか、および公開する範囲は、Contract Design で決める。このステージでは、置き場が U1 であることだけを確定する。 [assumption]
- DisplayEvent は、共有する型には含めず、U3 が持つ。U3 は、U1 に置かれた型(JobOutcome、JobState など)を、表示の元データとして参照する。 [assumption]
