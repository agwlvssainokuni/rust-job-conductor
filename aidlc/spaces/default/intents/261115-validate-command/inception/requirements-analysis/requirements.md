# Requirements(validate-command)

## Sources

- Initial description: 定義ファイルの誤りを、実行せずに検査できるように、validate サブコマンドを追加する。あわせて、PipelineDefinition の Rust のフィールド名 jobs を job に改名し、定義ファイルのキー(job)と揃える。converter の単体テストのサンプルデータのキー "jobs" も "job" に直す。 [desc]
- Workflow-selected scope: express [scope]
- 既存コードの調査(`aidlc/spaces/default/codekb/rust-job-conductor/`、この intent の focused scan): `Ctx::prepare`(`crates/cli/src/ctx.rs`)が、シグナルの準備、定義の検証、パイプライン名の照合、command の検査、ジョブIDの確認、ステートの読み込みと照合を一続きで行う。`Runtime` trait の `load_definition` と `check_command` で足りる。引数解釈(`args.rs`)は「パイプライン名が 1 つ必須」か「0 個」の 2 通りだけ。使い方の文言は `messages.rs` と README の 2 か所。`PipelineDefinition.jobs` の全使用箇所は、調査の引き継ぎにファイル:行で列挙済み。 [memory:M1]
- 質問への回答: Q1=A(command のプログラムの実在は調べない)、Q2=A(パイプライン名は任意)、Q3=B(成功は、既定は出力なし、`-v` のときだけ標準エラーに 1 行) [Q1] [Q2] [Q3]
- 依頼のやり取り: README のタイトルを `# rust-job-conductor — Rust製のジョブ実行制御ツール` にする(文言まで、依頼で決定済み)。「jobs を job に修正したけれど、プログラムは jobs のままっぽい。他のフィールドが定義ファイルと揃っているので、ここだけ気になる」。

## Intent analysis

(1) 定義ファイルの誤りは、いまは、`run` や `pipeline` を実行しないと分からない。実行の前に行う検査(定義の読み込みと検証、全ジョブの `command` の検査)だけを行う `validate` サブコマンドを足し、何も実行せずに誤りを拾えるようにする。(2) Rust の `PipelineDefinition` のフィールド `jobs` が、定義ファイルのキー `job` と揃っていないので、`job` に揃える。(3) README のタイトルが `conductor` だけで、リポジトリ名(`rust-job-conductor`)と合っていないので、直す。

## Functional requirements

- **FR1 `validate` サブコマンド**
  - **FR1.1 書式** `conductor validate [-v] [--config <パス>] [<パイプライン名>]`。`--config` を省略すると、実行ディレクトリの `conductor.toml`(`run`・`pipeline` と同じ)。`--config=<パス>` の形も受け付ける。`--help` の使い方と README の使い方の表に、`validate` を載せる。
    - 受け入れ基準: `conductor validate --config <正しい定義>` が終了コード 0。`--help` の出力に `validate` がある。
  - **FR1.2 検査の内容** `run`・`pipeline` が実行の前に行う検査と、同じ検査を、同じ規則と同じメッセージで行う: 定義の読み込みと検証(構文、キー、型、遷移の整合、終了コードの重なり、`command` の型と形)と、全ジョブの `command` の検査(引用符が閉じていること、先頭が空でないこと)。誤りは、複数あれば、まとめて表示する。
    - 受け入れ基準: `run` が定義の誤り(終了コード 64)で拒否する定義は、`validate` も 64 で拒否し、同じ誤りの文を表示する。
  - **FR1.3 何も実行しない** ジョブを起動しない。ステートファイル(`.state`)を、読まず、書かず、作らない。`validate` の前後で、ステートファイルの有無と内容が変わらない。
    - 受け入れ基準: 副作用のあるジョブ(ファイルを作る)を含む定義で `validate` を実行しても、ファイルは作られず、`.state` も作られない。壊れた `.state` が隣にあっても、`validate` は(定義が正しければ)0 を返す。
  - **FR1.4 パイプライン名** 名前は任意 [Q2: A]。渡したときは、定義の `[pipeline].name` と照合し、一致しなければ定義の誤り(64)。省略したときは、照合しない。
    - 受け入れ基準: 名前を省略した `validate` が成功し、正しい名前では成功し、違う名前では 64。
  - **FR1.5 終了コード** 成功は 0。定義の誤りは 64(`run` と同じ)。使い方の誤り(未知のオプション、名前が 2 つ以上、など)は 68。内部のエラー(パニック)は 70。`validate` は、66(拒否)・65(ステートの誤り)・67(変換の失敗)を返さない。
    - 受け入れ基準: 上の各場合の終了コードが、README の終了コードの表と一致する。
  - **FR1.6 表示** [Q3: B] 成功したときは、既定では何も出力しない。`-v` を付けたときだけ、標準エラーに 1 行(定義が正しいこと、ジョブの数)を表示する。表示は、日本語と英語の両方(ロケールに従う)。誤りは、常に標準エラーに表示する。
    - 受け入れ基準: `validate` は標準出力に何も書かない。`-v` なしで標準エラーも空。`-v` ありで 1 行(ジョブ数を含む)。
  - **FR1.7 command の実在は調べない** [Q1: A] `command` のプログラムが PATH にあるか、ファイルが存在するかは、調べない(`run`・`pipeline` と同じ)。
    - 受け入れ基準: 存在しないプログラムを `command` に書いた、ほかは正しい定義が、`validate` で 0 を返す。
  - **FR1.8 シグナルの監視を始めない** `validate` は、実行する処理がないので、中断のためのシグナルの監視(`run`・`pipeline` が行うもの)を始めない。
- **FR2 内部の名前の改名** Rust の `PipelineDefinition` のフィールド `jobs` を `job` に改名する。使用箇所(`conductor-core`・`conductor-runtime`・`conductor-cli` の製品コードとテスト)を、すべて追従させる。`converter` の単体テストのサンプルデータのキー `"jobs"` も `"job"` に直す。動作、ファイル形式、メッセージは変えない。
  - 受け入れ基準: ビルドが通り、改名の前後で、全テストが成功する。`PipelineDefinition` の Rust のフィールドに `jobs` が残らない。`converter` のテストのサンプルのキーに `"jobs"` が残らない(定義ファイルの旧書式の拒否のテスト `[[jobs]]` は、そのまま)。
- **FR3 文書**
  - **FR3.1** README のタイトルを `# rust-job-conductor — Rust製のジョブ実行制御ツール` にする。コマンド名の `conductor` は、本文では変えない。
    - 受け入れ基準: README の 1 行目がそのタイトルで、CI の `docs-check`(README の必須の節の検索)が成功する。
  - **FR3.2** README の使い方の表・書式・例・終了コードの表に、`validate` を追加する(`validate` が実行しないこと、ステートに触れないこと、`-v` の動作を含む)。
    - 受け入れ基準: README に `validate` の行と節がある。

## Non-functional requirements

- **NFR1 後方互換** `run`・`pipeline`・`watch`・`reset`・`convert` の動作、終了コード、メッセージ、ステートの形式、定義ファイルの形式は、変わらない。
- **NFR2 安全** `validate` の誤りの文にも、入力の値は含めない(種類と場所だけ。`run` と同じ規則を使う)。`unsafe` を増やさない。
- **NFR3 品質** 行カバレッジ 80% 以上を保つ。既存のテストは、使い方の文言(`--help`)と引数の文法を固定するテストの期待値の更新と、`jobs` の改名の追従を除き、そのまま成功する。`cargo fmt --check`、`cargo clippy -D warnings`、Rust 1.85 でのビルドとテスト、3 OS の CI が成功する。
- **NFR4 版** 機能追加のため、`Cargo.toml` の `version` を 0.4.0 に上げる。[assumption]

## Constraints

- Rust 1.85 で、ビルドできること(let チェーンなど、1.88 以降の構文は使わない)。
- ライブラリの追加はしない。
- CI の `docs-check` が README の必須の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)を検索する。タイトルの変更や `validate` の追加で、これらを消さない。
- チームの実践(test-after、`rustfmt`、`clippy`、`main` への取り込みで履歴を残す)に従う。

## Assumptions

- 版を 0.4.0 に上げる。タグ付けと公開は、この作業の範囲外(あとの Deployment Execution で、人が決める)。[assumption]
- `-v` の 1 行の文言は、`定義は正しいです(ジョブ N 個)` / `definition is valid (N jobs)` を目安とし、正確な文は Code Generation で決める。[assumption]
- `validate` の `-v` は、標準エラーへの 1 行の表示だけで、`run`・`pipeline` の `-v` のような、ジョブごとの表示はない。[assumption]

## Out of scope

- `command` のプログラムの実在の確認(Q1: A)。`validate` のステート(`.state`)の検査(ステートは、`watch` で見る)。
- デモ(`demo/`)への `validate` の場面の追加。
- ステートファイルと定義ファイルの形式の変更。`format_version` の引き上げ。
- `jobs` 以外の内部の名前(`JobState`、`job_states` など)の改名。

## Open questions

None.

## Assumptions & Open Questions

None.
