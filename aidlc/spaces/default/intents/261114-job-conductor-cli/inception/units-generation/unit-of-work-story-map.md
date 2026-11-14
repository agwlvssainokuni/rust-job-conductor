# Unit of Work Story Map(ストーリーと Unit の対応)

上流: unit-of-work、stories(US1.1〜US7.1)、components。各行は、ストーリーを実装する Unit(ID とディレクトリ名)を示す。複数の Unit は `;` で区切る。Unit は、そのストーリーの一部でも実装する場合に、記載する。

| Story | Unit | Directory |
|-------|------|-----------|
| US1.1 | U1 | u1-core |
| US1.2 | U1 | u1-core |
| US1.3 | U1;U3 | u1-core;u3-cli |
| US2.1 | U1;U2 | u1-core;u2-runtime |
| US2.2 | U1;U2 | u1-core;u2-runtime |
| US2.3 | U1;U3 | u1-core;u3-cli |
| US2.4 | U3 | u3-cli |
| US2.5 | U2;U3 | u2-runtime;u3-cli |
| US3.1 | U1;U3 | u1-core;u3-cli |
| US3.2 | U1;U2;U3 | u1-core;u2-runtime;u3-cli |
| US4.1 | U2 | u2-runtime |
| US4.2 | U1;U2 | u1-core;u2-runtime |
| US4.3 | U1;U2 | u1-core;u2-runtime |
| US4.4 | U2 | u2-runtime |
| US5.1 | U2;U3 | u2-runtime;u3-cli |
| US6.1 | U2;U3 | u2-runtime;u3-cli |
| US7.1 | U3;U4 | u3-cli;u4-converter |

`traceability.json` では、複数の Unit にまたがる行は、主に実装を担う Unit を1つだけ対象に記す(上の表にその Unit も載っている)。主な Unit は、US1.3 が U3、US2.1 が U2、US2.2・US2.3・US3.2・US4.2 が U1、US2.5・US4.3・US6.1 が U2、US3.1・US5.1 が U3、US7.1 が U4。他の Unit は、この表に残る。

## 複数の Unit にまたがる関心事

- 表示とエラーの文言(Reporter): U3 が持つが、すべての Unit のエラーの種類を受け取って表示する。エラーの種類は、各 Unit が返す値として定義する(Contract Design で定める)。
- 複数 OS 対応(NFR3): U2 が OS ごとの違いを閉じ込めるが、U1(定義の読み込みのパス)と U3(シグナルの受け取り、ロケール)にも影響する。CI(GitHub Actions)で、3つの OS を検証する。
- 起動オーバーヘッド(NFR1): U3 の起動経路が、U1(定義の読み込み)、U2(ステートの読み込み)、U3(ロケールの判定)をすべて通る。
- 型の誤りの事前検知(NFR4): U1(定義)と U2(ステート)が、それぞれ受け持つ。
- 共有する型の定義: U1 が置く。U2、U3 は、これを使う(unit-of-work-dependency を参照)。

## 各 Unit の中での実装の順序

| Unit | 実装の順序(Unit の内側) |
|------|---------------------------|
| U1 | 定義の読み込みと検証 → 結果の分類 → 遷移・スキップの判断 → 完了と再開位置の判定 |
| U2 | ステートの保持(原子的な書き込み)→ 破損・食い違いの検知 → ジョブの起動 → シグナルの転送 |
| U3 | 表示(Reporter)→ `run` の流れ → `pipeline` の流れ → `watch`、`reset`、`convert` |
| U4 | YAML と TOML の相互変換 |

Unit どうしの順序(何から作るか)は、Delivery Planning で決める。

## カバレッジの確認

- 17本のストーリーのすべてが、少なくとも1つの Unit に割り当てられている。
- 4つの Unit のすべてが、1つ以上のストーリーを持つ(U1: 10、U2: 10、U3: 9、U4: 1)。

## Assumptions & Open Questions

None.
