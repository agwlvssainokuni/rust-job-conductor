# Bolt の計画(Bolt Plan): v0.9.0 の機能拡張

「Bolt」は、作りの 1 回のまとまりのことです(作る範囲を 1 つ決め、動くところまで作って終える 1 周)。この計画は、4 つのユニットを、どの順で、どの大きさで作るかを記します。順の理由は、`risk-and-sequencing-rationale.md` にあります。

出典: コンポーネントの一覧(`domain-design/components.md`)、ユニットの定義(`units-generation/unit-of-work.md`)、ユニットの依存(`unit-of-work-dependency.md`)、要件(`requirements-analysis/requirements.md`。FR1.1〜FR5.2)、契約(`contract-design/contract-summary.md`)、チームの慣行(`practices-discovery/team-practices.md`)、納品計画の質問(`delivery-planning-questions.md`。以下 Q1〜Q6)。

## 1. Bolt の順序

| 順 | Bolt | ユニット | 大きさ | 薄い縦切り(walking skeleton) | 依存する Bolt |
|----|------|----------|--------|--------------------------------|----------------|
| 1 | B1 | U1 u1-env-injection(環境変数の注入と `.env`) | L | いいえ | なし |
| 2 | B2 | U2 u2-graph(`conductor graph`) | M | いいえ | なし |
| 3 | B3 | U3 u3-dry-run(`--dry-run` と `--assume`) | M | いいえ | B1、B2 |
| 4 | B4 | U4 u4-init(`conductor init`) | S | いいえ | B1 |

- 1 つのユニットを 1 つの Bolt にする(Q3: A)。各 Bolt は、設計から、コード生成とテストまで、通して終える。
- 1 つずつ順に作る(Q4: A。直列)。順は、決まっている順(Q1: A)。
- 「薄い縦切り(walking skeleton)」は、最初に、動く最小の縦のスライスを作る方法です。チームの方針(`team.md` の Walking Skeleton)により、この作業では作りません。機能を 1 つずつ順に作ります。
- 依存の図(U3 は U1 と U2 に依存、U4 は U1 に依存)は、この順で満たされる。依存の順を崩した Bolt はない。

## 2. 各 Bolt

### B1: 環境変数の注入と `.env`(U1)

- 含むユニット: U1 u1-env-injection。
- 含むコンポーネント(`components.md`): `CoreTypes`、`CoreDefinition`、`CoreEnvFile`(新規)、`RuntimeEnvLoader`(新規)、`RuntimeRunner`、`CliFlows`、`CliReport`。
- 要件: FR1.1〜FR1.13、FR5.1、FR5.2(環境変数の分)。
- 終わりの条件(Definition of Done):
  - 定義の新しいキー(`[pipeline.env]`、`[job.env]`、`env_file`)が読め、予約名、大文字小文字だけが違う名前、文字列でない値が拒否される。
  - `.env` の書式(行番号つきの誤り)、上限(1 MiB、10,000 項目)、NUL を含む値が検査される。
  - 層の優先順位(引き継いだ環境 < pipeline の `env_file` < `pipeline.env` < job の `env_file` < `job.env`)で統合され、組み込みの `CONDUCTOR_*` が渡される。
  - `.env` の誤りのときは、ジョブを 1 つも実行せずに止まる。`validate` が重複キーを警告する。
  - 環境変数の値が、エラー表示とステートに出ない。`format_version` は 1 のまま。
  - 日本語と英語のメッセージ、README、デモの更新。
  - 行カバレッジ 80% 以上。`cargo fmt`、`clippy -D warnings`、テスト(単体と、実バイナリを動かす統合)が通る。既存のテストが通る。
  - 機能の設計で決めること: `.env` の誤りの終了コードの値(要件の未解決事項。契約の Open questions の C4)。
- 信頼の仮説(出荷して分かること): 定義の型の変更(`CoreTypes`)が、既存の多くのテストにどれだけ影響するかが分かる。以後の B2〜B4 が、変更後の型の上に、安全に積めることを確かめる。
- 期待するデモ: 定義に `[pipeline.env]` と `.env` を書き、ジョブが環境変数を受け取ること。`validate` が重複キーを警告すること。

### B2: `conductor graph`(U2)

- 含むユニット: U2 u2-graph。
- 含むコンポーネント(`components.md`): `CoreGraph`(新規)、`CoreFlow`(辺の取り出しの入口)、`CliArgs`、`CliReport`、`CliFlows`。
- 要件: FR2.1〜FR2.7、FR5.1、FR5.2(graph の分)。
- 終わりの条件:
  - `graph` が、`git log --graph` 風の text(既定)と Mermaid(`--format mermaid`)を、標準出力に出す。`-v` で詳細、`--state` でステートの重ね合わせ。
  - 辺は、実行のときに次のジョブを決める判断と同じ関数から作る(図と実行が食い違わない)。
  - 見た目(text と Mermaid)は、期待する出力をそのままテストに書いて、厳密に固定する(契約 C6)。複数の飛び越しがある図を含む。
  - 定義の誤りは `validate` と同じ。環境変数の値は、出力に含めない。
  - 日本語と英語のメッセージ、README、デモの更新。カバレッジ、`fmt`、`clippy`、テストは B1 と同じ。
  - 機能の設計で決めること: 期待出力の具体例(契約の Open questions の C6)、`--run-id` を `--state` なしで許すか(C5)。
- 信頼の仮説: レーンの配置が、複雑な流れ(複数の飛び越し、合流)でも崩れないことが分かる。辺の取り出しの入口が、B3 の計画の生成にも使えることを確かめる(契約 C2)。
- 期待するデモ: 飛び越しのあるパイプラインの `graph` の出力(text と Mermaid)。`--state` で、実行済みのジョブに印が付くこと。

### B3: `--dry-run` と `--assume`(U3)

- 含むユニット: U3 u3-dry-run。前提: B1(渡される環境変数の名前)と B2(辺の判断の入口)が終わっている。
- 含むコンポーネント(`components.md`): `CorePlan`(新規)、`CliArgs`、`CliReport`、`CliFlows`。
- 要件: FR3.1〜FR3.4、FR5.1、FR5.2(dry-run の分)。
- 終わりの条件:
  - `pipeline` と `run` の `--dry-run` が、ステートを読み、再開位置と、ジョブごとの扱いを示す。何も実行せず、ステートを作成も更新も削除もしない。
  - `--assume <ジョブID>=<終了コード>` で、終了コードを仮定した流れを模擬する。不正な入力(存在しないジョブ ID、書式の誤り、重複、範囲外の終了コード)は、使い方の誤りとして扱う。
  - 出力は、契約 C7 の順(ヘッダー、再開位置、仮定、ジョブごとの項目)で、環境変数の値を含めない。
  - 日本語と英語のメッセージ、README、デモの更新。カバレッジ、`fmt`、`clippy`、テストは B1 と同じ。
  - 機能の設計で決めること: コマンドの分割(`runtime::split`)の置き場所と誤りの出し方、`--assume` の不正な入力のメッセージと終了コード、出力の文言と書式、`CONDUCTOR_ATTEMPT` の値(契約の Open questions)。
- 信頼の仮説: 計画の生成が、実行と同じ判断を使い、実行と食い違わないことが分かる。`core` と `runtime` の境界(`core` はファイルに触れない)を保ったまま、コマンドの分割を計画に含められることを確かめる。
- 期待するデモ: 失敗して止まったあとの `--dry-run` が、再開位置を示すこと。`--assume` で、後続のジョブが飛ぶ流れを模擬すること。

### B4: `conductor init`(U4)

- 含むユニット: U4 u4-init。前提: B1(新しいキー)が終わっている。
- 含むコンポーネント(`components.md`): `RuntimeFileCreate`(新規)、`InitTemplate`(新規)、`CliArgs`、`CliReport`、`CliFlows`。
- 要件: FR4.1〜FR4.5、FR5.1、FR5.2(init の分)。
- 終わりの条件:
  - `init` が、定義のひな形を作る。既にあるときは終了コード 69。`--force` で上書き。`-` で標準出力にだけ出す(ファイルを作らない)。
  - ひな形は、ロケールに従い、`[pipeline.env]` と `[job.retry]` の例を含み、必ず `validate` に通る(テストで確かめる)。
  - 排他的な作成(`runtime`)は、ファイルが既にあるときに上書きしない。
  - 日本語と英語のメッセージ、README、デモの更新。カバレッジ、`fmt`、`clippy`、テストは B1 と同じ。
  - 機能の設計で決めること: `init` の端の動き(パイプライン名の検証、`-` と `--force` の併用、親ディレクトリが無いとき、パスがディレクトリのとき)(契約の Open questions の C5)。
- 信頼の仮説: ひな形が、B1 の新しいキーと矛盾せず、`validate` に通ることが分かる。新しい利用者が、`init` から、動く定義まで、迷わず進めることを確かめる。
- 期待するデモ: 空のディレクトリで `init` を実行し、続けて `validate` と `pipeline` が通ること。もう一度 `init` を実行すると終了コード 69 になること。

## 3. 共通の終わりの条件(全 Bolt)

- 新しい外部ライブラリ(依存)を足さない(`project.md`)。
- `unsafe` は `platform/unix.rs` の外に書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない(`nfr_source.rs`)。
- エラーの表示に、入力の値を含めない。ステートに環境変数を書かない。
- コミットは小さな単位(型の追加、読み取りと検証、表示、テスト、README など)で、直接 `main` に作る(`team.md`)。
- リリース(版 0.9.0、タグ、README と仕様書の更新)は、4 つの Bolt のあとの、別の依頼とする(この計画の範囲の外)。

## 4. Construction の設定

- 反復の順: ユニットごとに順に(unit-major)。実行: 直列(serial)。チェックポイント: 有効。いずれも、この計画の Q3、Q4 と合う。変更は不要。
- 検証のコマンドは、この段階の最後に、人の承認を得て記録する(下の手順)。
