# Unit of Work — 正常終了しなかったジョブの自動リトライ

入力: `requirements.md`(FR1〜FR8、NFR1〜NFR7)、`units-generation-questions.md`(Q1〜Q3)。新しい構成要素は増やさず、既存の 4 つの crate の中を、機能の段階で 3 つに分ける。(Q1: A)

## Unit 一覧

| Unit ID | Directory | 名前 | 種類(kind) | 規模 | デプロイの形 |
|---|---|---|---|---|---|
| U1 | u1-retry-config | リトライの設定と検証 | library | M | 組み込み(単一の実行ファイルに入る) |
| U2 | u2-retry-execution | リトライの実行と、途中の知らせ | service | L | 組み込み(単一の実行ファイルに入る) |
| U3 | u3-retry-display-docs | 表示の文言の確定と文書 | service | S | 組み込み(単一の実行ファイルに入る) |

3 つとも、配布物は `conductor` の単一の実行ファイルのままで、独立してデプロイする Unit はない。Unit の大きさは、レビューしやすい範囲に収める。(Q2: A)

種類(kind)の理由:

- U1 は `library`: 実行時の動作を持たない、定義の型と検証のコード(core の型と純粋な検査)だから。
- U2 は `service`: 実行ファイルとして動く流れ(ジョブの実行し直し、待機、シグナル、表示)を変えるから。
- U3 は `service`: 利用者に見える動作(表示される文言、README が説明する動き)の仕上げであり、`packaging`(配布物)でも `spec`(契約)でも `ui`(画面)でもないから。`conductor` の標準出力と標準エラーの表示は、実行ファイルの動作に含まれる。

## U1 u1-retry-config — リトライの設定と検証

- 責務: ジョブの定義に、リトライの設定(終了コードの一覧、最大回数 N、間隔、倍率、上限)を書けるようにし、読み取りと検証をする。誤りは定義の誤り(64)として扱い、`validate` が検出する。`convert`(YAML と TOML の相互変換)が、新しい項目を失わないことを確かめる。
- 主な変更: `core/src/types.rs` の `JobDefinition` への項目の追加、`core/src/definition/walk.rs`(許可するキー、型の検査、関係の検査、組み立て)、`core/src/error.rs` の誤りの種類、`cli/src/report/messages.rs` の誤りの文言(英語と日本語)、`converter` の確認(通常は変更不要)、テストの定義のリテラルの更新。
- 担当する要件: FR1.1〜FR1.9、FR7.1、FR7.2。
- 制約: 値は整数と文字列で表す(小数は使わない)。リトライの設定がない定義は、これまでと同じ動作のままにする。

## U2 u2-retry-execution — リトライの実行と、途中の知らせ

- 責務: ジョブが、リトライの終了コードで終わったとき、設定した間隔を空けて、最大 N 回、同じジョブを実行し直す。固定の間隔と、回ごとに伸ばす方式の両方に対応する。`run` と `pipeline` の両方で動く。間隔の待機中に中断のシグナルを受けたら、すぐに待機をやめる。回数はステートファイルに記録しない。待機を差し込める形にして、テストで実際に待たずに済むようにする。
- リトライの前の知らせ: 何回目のリトライか、最大回数、次の実行までの待ち時間を、表示の仕組み(`DisplayEvent`)に出す。表示の種類(`DisplayEvent` の新しい種類と、その表示のレベル、描画)は、実行の流れが呼ぶ側なので、この Unit に含める。コンパイルに必要な最初の文言(英語と日本語)も、この Unit で書く。
- 主な変更: `cli/src/run.rs`、`cli/src/pipeline.rs`(共通の関数にまとめるかは設計で決める)、`cli/src/deps.rs` の時間と待機の差し込み口、`cli/src/report/mod.rs` の `DisplayEvent`・`level`・描画と `report/messages.rs` の最初の文言、`runtime` の待機とシグナル、`core/src/flow/*`(必要な場合)。
- 担当する要件: FR1.2 の動き、FR1.6 の動き、FR2.1、FR2.2、FR3.1〜FR3.6、FR4.1、FR4.2、FR5.1、FR5.2、FR6.1、NFR3、NFR7。
- 制約: `unsafe` の禁止、`platform/` の外での OS 別の `cfg` の禁止、ネットワークを使わないこと、新しい外部クレートを追加しないこと。ステートファイルの形式は変えない。

## U3 u3-retry-display-docs — 表示の文言の確定と文書

- 責務: U2 が出す「リトライの前の知らせ」の、英語と日本語の文言を確定し(ロケールに従う)、表示のレベル(`-v` なしで出すか)を確認する。README に、リトライの項目の書き方、動き、at-least-once との関係を書く。
- 主な変更: `cli/src/report/messages.rs`(文言の確定)、`README.md`、`demo/`(必要なら)。
- 担当する要件: FR6.2、FR8.1。
- 制約: README は、CI の `docs-check`(必須の語の検査)を通す。

## Unit をまたぐ要件

後方互換(NFR1)、既存の検査の維持(NFR2)、カバレッジ(NFR4)、3 つの OS(NFR5)、新しい外部クレートを増やさないこと(NFR6)は、3 つの Unit のすべてで守る。担当は `unit-of-work-story-map.md` に示す。
