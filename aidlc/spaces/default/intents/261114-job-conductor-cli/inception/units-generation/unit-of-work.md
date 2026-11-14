# Unit of Work(作業単位の定義)

上流: components(7つの部品)、decisions(ADR-001〜008)、requirements(FR1〜FR7、NFR1〜NFR6)、stories(US1.1〜US7.1)。7つの部品を、4つの Unit にまとめた。[Q1]

`conductor` は単一バイナリであり、すべての Unit は、1つのバイナリに組み込まれる(デプロイは分かれない)。 requirements の NFR2

## Units

| Unit ID | Directory | 説明 | 含む部品 | 種類(kind) | 規模 | デプロイの形 |
|---------|-----------|------|----------|-------------|------|--------------|
| U1 | u1-core | 定義の読み込みと検証、結果の分類・遷移・完了の判断 | DefinitionLoader、Flow | library | M | 組み込み(単一バイナリの一部) |
| U2 | u2-runtime | ジョブの直接起動とシグナル転送、ステートの保持と整合 | JobRunner、StateStore | library | L | 組み込み(単一バイナリの一部) |
| U3 | u3-cli | コマンドとオプションの受け付け、1回の実行の組み立て、表示 | Cli、Reporter | service | L | 組み込み(`conductor` の実行ファイルそのもの) |
| U4 | u4-converter | YAML と TOML の相互変換 | Converter | library | S | 組み込み(単一バイナリの一部) |

- 種類は、`service` = デプロイされる実行ファイル、`library` = 単独の実行を持たない再利用コード。U3 が `conductor` の実行ファイルになり、U1、U2、U4 は、U3 に組み込まれる。
- 規模は、S / M / L / XL の相対的な見積もりである。

## U1 u1-core

- **責務:** 定義ファイル(`conductor.toml`)の読み込みと検証(DefinitionLoader)。終了コードの分類、スケジューラへ返す終了コード、遷移・スキップ・ジャンプ、完了と再開位置の判定(Flow)。
- **所有するエンティティ:** PipelineDefinition、JobDefinition、JobOutcome、NextStep。
- **境界:** Flow は、ファイルとプロセスに触れない。定義の読み込みだけが、ファイルシステムに触れる。
- **実装の注意:** Flow は、入出力を持たない判断ロジックであり、単体テストで網羅する(ADR-005、ADR-008)。型の誤りの事前検知(NFR4)を、DefinitionLoader が担う。
- **対応するストーリー:** US1.1、US1.2、US1.3 の一部、US2.1・US2.2・US2.3 の判断部分、US3.1・US3.2・US4.2・US4.3 の判断部分。

## U2 u2-runtime

- **責務:** ジョブのコマンドの、シェルを介さない直接起動、終了コード・起動失敗・シグナル終了の区別、シグナルの転送(JobRunner)。ステートの原子的な保持、破損と定義との食い違いの検知、初期化(StateStore)。
- **所有するエンティティ:** JobExecution、PipelineState、JobState。
- **境界:** OS のプロセス・シグナルとファイルの扱いを、この Unit に閉じ込める。複数の OS(Linux・macOS・Windows)の違いは、ここで吸収する(NFR3)。
- **実装の注意:** 複数 OS 対応と Rust への不慣れが、この Unit に集中するリスクである(RAID の R1、R2)。ステートの `definitionKey`(仮)の確定は、Functional Design で行う。
- **対応するストーリー:** US2.1、US2.2、US2.5 の一部、US4.1、US4.2、US4.3、US4.4、US5.1、US6.1 の状態の部分。

## U3 u3-cli

- **責務:** `run`、`pipeline`、`watch`、`reset`、`convert` の受け付けと、`--config`、`-v` / `--verbose` の解釈、1回の実行の流れの組み立て(Cli)。表示の組み立てと、ロケールに応じた言語・色の切り替え(Reporter)。シグナルの受け取りと、中断の合図の用意。
- **所有するエンティティ:** DisplayEvent。
- **境界:** 他の Unit の部品を呼び出す、唯一の調整役である(ADR-002)。`run` と `pipeline` の流れは、別々の手順として組み立て、大きくなったら、さらに分ける。
- **実装の注意:** 起動オーバーヘッド(NFR1)を、起動経路で守る。ロケールの判定は、必要になるまで行わない。
- **対応するストーリー:** US1.3、US2.3、US2.4、US2.5 の一部、US3.1、US3.2、US5.1、US6.1、US7.1 のコマンドの部分。

## U4 u4-converter

- **責務:** 定義ファイルを、YAML と TOML のあいだで変換する(Converter)。コメントと記述順は保持しない。
- **所有するエンティティ:** なし。
- **境界:** 他の Unit に依存しない。優先度は Should である。
- **対応するストーリー:** US7.1。

## 優先度

| Unit | 含むストーリーの優先度 |
|------|------------------------|
| U1 | Must |
| U2 | Must |
| U3 | Must(`watch` は Should) |
| U4 | Should(US7.1) |

## Assumptions & Open Questions

- Unit の種類は、`service` の U3 を除き、すべて `library` とした。パッケージングやリリースバイナリの作成は、このあとの CI・デプロイのステージで扱い、独立した Unit にはしなかった。 [assumption]
- 規模の見積もり(S / M / L)は、相対的なものであり、Delivery Planning で見直す。 [assumption]
