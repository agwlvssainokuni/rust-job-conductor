# Requirements(run-id-state)

## Sources

- Initial description: 実行単位(run ID)ごとにステート(.state)を分けて管理できるようにする。run ID は、オプション --run-id <値> と、環境変数の値で与える。オプションが優先。環境変数の名前は定義ファイルの [pipeline] の run_id_env に書ける(省略時は CONDUCTOR_RUN_ID)。(依頼の全文は、この intent の記録にある) [desc]
- Workflow-selected scope: express [scope]
- 既存コードの調査(`aidlc/spaces/default/codekb/rust-job-conductor/`、この intent の focused scan): ステートのパスは、定義ファイルのパスから `<定義ファイル>.state` として決まる(`StateLocation`)。`watch` と `reset` は、定義ファイルを読まない(壊れた定義でも `reset` が動く性質)。`Env`(環境変数の読み取り)は、いま `Reporter` だけが使う。`reset` は、ステートを読まずに削除する。 [memory:M1]
- 質問への回答: Q1=A(`watch`・`reset` は、定義から `run_id_env` だけをゆるく読む)、Q2=A(`reset --all` は、中身の `run_id` で照合する) [Q1] [Q2]
- 依頼のやり取り(決定済み): 実行単位は「run ID」と呼ぶ。`--run-id` を `run`・`pipeline`・`watch`・`reset` が受け付け、オプションが環境変数に優先する。値の制限は文字 `A-Za-z0-9._-`、64 文字まで、先頭の `.` と `..` は禁止、違反は使い方の誤り(68)。空は未設定と同じで、`--run-id ""` は環境変数があっても run ID なしに固定する。ファイル名は `<定義ファイル>.<run-id>.state` で、ステートの中の `run_id` が食い違えば壊れたステート(65)。後始末は `reset --all`(案 A。自動削除はしない)。置き場所は、これまでのまま(`CONDUCTOR_STATE_DIR` は足さない)。「環境変数は .toml に書けるように」は、環境変数の名前を定義ファイルに書けるという意味(確認済み)。

## Intent analysis

いまのステートは、定義ファイルごとに 1 つ(`<定義ファイル>.state`)で、同じ定義を複数の実行が使うと、お互いのステートを壊す(二重実行、巻き戻り、記録の取りこぼし)。呼び出し側が実行単位(たとえば GitHub Actions の run)を表す文字列を渡し、`conductor` が、実行単位ごとに別のステートを持つようにする。排他制御ではなく、状態の分離で、並列の実行を安全にする。同じ run ID で呼び直すと、止まったジョブから再開できる(GitHub Actions の「失敗したジョブの再実行」に近い)。

## Functional requirements

- **FR1 run ID の与え方と検査**
  - **FR1.1 オプション** `--run-id <値>`(と `--run-id=<値>`)を、`run`・`pipeline`・`watch`・`reset` が受け付ける。`validate` と `convert` は受け付けない(使い方の誤り 68)。2 回以上の指定は、ほかのオプションと同じく使い方の誤り(68)。
    - 受け入れ基準: 4 つのコマンドで `--run-id` が解釈され、`validate --run-id x` は 68。
  - **FR1.2 環境変数** 環境変数の名前は、定義ファイルの `[pipeline].run_id_env` で決める。省略したときは `CONDUCTOR_RUN_ID`。`--run-id` が渡されていれば、環境変数は見ない(オプション優先)。
    - 受け入れ基準: `--run-id a` と環境変数 `b` があれば、run ID は `a`。オプションがなく環境変数があれば、その値。どちらもなければ、run ID なし。
  - **FR1.3 空の値** 環境変数が空(または未定義)なら、run ID なし(未設定と同じ)。`--run-id ""` は、環境変数があっても、run ID なしに固定する。
    - 受け入れ基準: 環境変数 `x` があっても `--run-id ""` では、既定のステート(`<定義ファイル>.state`)を使う。
  - **FR1.4 値の検査** 値は、文字 `A-Za-z0-9._-` だけ、1〜64 文字、先頭が `.` でない(`..` を含む)こと。オプションの値も、環境変数の値も、同じ検査を受ける。違反は使い方の誤り(68)。
    - 受け入れ基準: `a/b`、`..`、`.a`、`a b`、65 文字、`é` などの非 ASCII の文字は 68。`a`、`run-1.2_x`、64 文字は有効。誤りの文に、値は含めない(どこから来たか=オプションか環境変数かだけを示す)。
- **FR2 定義ファイル**
  - **FR2.1** `[pipeline]` に、任意のキー `run_id_env`(文字列)を足す。文字列でなければ、定義の誤り(64)。キーがなければ、`CONDUCTOR_RUN_ID`。
  - **FR2.2** 変数名の形は、`A-Za-z_` で始まり `A-Za-z0-9_` が続く、1 文字以上。違反は、定義の誤り(64)。`validate` も、同じ検査をする。
  - **FR2.3** `format_version` は 1 のまま。
    - 受け入れ基準: `run_id_env = "GITHUB_RUN_ID"` を持つ定義が読め、`run_id_env = 5`・`run_id_env = "1A"`・`run_id_env = ""` は 64。`validate` も同じ結果。
- **FR3 ステートの分離**
  - **FR3.1 ファイル名** run ID があるときのステートは `<定義ファイル>.<run-id>.state`、なければ、従来の `<定義ファイル>.state`。置き場所は、定義ファイルと同じディレクトリ。
  - **FR3.2 中身の `run_id`** run ID があるときは、ステートの中にキー `run_id` を書く。run ID なしのステートには書かない。読み込むとき、ファイル名の run ID と中身の `run_id` が食い違えば(中身にない、値が違う、run ID なしで中身にある)、壊れたステート(65)。この検査は、完了の判定(`completed`)より前に行う。
    - 受け入れ基準: `x.state` を `x.a.state` にコピーして `--run-id a` で読むと、65。
  - **FR3.3 独立** run ID が違えば、ステートは独立する。ある run ID の読み書き・削除は、別の run ID(と run ID なし)のステートに影響しない。
    - 受け入れ基準: run ID `a` と `b` の `pipeline` を並行して動かしても、互いの再開位置が変わらない。
  - **FR3.4 完了と再開** 完了した run のステートは、従来どおり消える。止まった run は、同じ run ID で呼び直すと、止まったジョブから再開する(従来のステートと同じ規則)。
- **FR4 `watch` と `reset`**
  - **FR4.1 `watch`** 選ばれた run ID のステートを表示する。run ID があるときは、表の前に、run ID の行(英語 `Run ID: <値>`、日本語 `Run ID: <値>`)を 1 行足す [assumption]。run ID なしのときの表示は、変えない。
  - **FR4.2 `reset`** 選ばれた run ID のステートだけを、読まずに削除する(従来どおり。壊れたステートの回復にも使える)。存在しなければ、成功。
  - **FR4.3 変数名の決め方** [Q1: A] `--run-id` が渡されたら、`watch`・`reset` は、定義ファイルを読まない。渡されなければ、定義から `[pipeline].run_id_env` だけを、ゆるく読む(ほかの誤りは無視する)。定義が読めない(壊れている、ない)、キーがない、値が文字列でない、変数名の形が違う、のどれでも、既定の `CONDUCTOR_RUN_ID` を使う。壊れた定義でも、`reset` は動く。
    - 受け入れ基準: 壊れた定義と、`CONDUCTOR_RUN_ID` の環境変数があるとき、`reset` が、その run のステートを消す。`run_id_env = "X_ID"` を持つ定義では、`watch` と `reset` が `X_ID` の値を使う。
- **FR5 `reset --all`**
  - **FR5.1 対象** `conductor reset --all [--config <パス>]` は、その定義のステートをすべて消す: 定義ファイルと同じディレクトリの、`<定義ファイル>.state`(run ID なし)と、`<定義ファイル>.<run-id>.state` のうち、名前の `<run-id>` が有効な値で、中身の `run_id` がその値と一致するもの [Q2: A]。
  - **FR5.2 消さないもの** 一時ファイル(`.<ステート名>.tmp.*`)、別の定義のステート(名前が重なっても、中身に `run_id` がない、または値が一致しない)、読めないもの(壊れている)は、消さない。読めなくて残したものは、標準エラーに、ファイル名を警告として表示する(壊れたものは `reset --run-id <ID>` で個別に消せる)。
  - **FR5.3 併用の禁止** `--all` は `reset` だけが受け付ける。`--all` と `--run-id` の併用は、使い方の誤り(68)。環境変数があっても、`--all` では見ない。
  - **FR5.4 結果** 消すものがなければ、成功(0)。警告があっても、削除に失敗していなければ 0。削除に失敗した(権限など)ときは、既存の失敗(65)。既定では、何も出力しない(警告だけ、標準エラーに出す)。
    - 受け入れ基準: `x.toml` に、`x.toml.state`・`x.toml.a.state`・`x.toml.b.state`(中身は run ID が合う)と、別の定義 `x.toml.a` の `x.toml.a.state`(中身に `run_id` なし)と、壊れた `x.toml.c.state` がある状態で `reset --all` すると、最初の 3 つだけが消え、残りの 2 つのうち壊れたものが警告に出る。一時ファイルは消えない。
- **FR6 `validate`** `validate` は、`run_id_env` の型と形を検査する(FR2)。`--run-id` は受け付けない(FR1.1)。環境変数の値は検査しない(`validate` はステートに触れず、run ID を使わない)。
- **FR7 文書** README に、`--run-id`、`run_id_env`、ステートのファイル名、`reset --all`、`watch` の run ID の行を、書く(使い方の表、オプションの表、定義ファイルの書き方、ステートファイルの場所、終了コードの表)。`--help` の使い方にも載せる。
  - 受け入れ基準: README と `--help` に、それぞれの記述がある。CI の `docs-check` の語句が残る。

## Non-functional requirements

- **NFR1 後方互換** run ID を使わない(オプションも環境変数もない)場合は、動作、ステートのファイル名と形式(`run_id` キーを書かない)、終了コード、メッセージが、変わらない。既存のステートは、そのまま読める。
- **NFR2 安全** run ID は、ファイル名の一部になるので、FR1.4 の検査を通ったものだけを使う(パス区切り、`..`、空白、制御文字、非 ASCII を、受け付けない)。`reset --all` は、定義ファイルと同じディレクトリの、通常のファイルだけを対象にし、シンボリックリンクを辿らず、ディレクトリの外に触れない。エラーの文に、入力の値(run ID、環境変数の値)を含めない。`unsafe` を増やさない。
- **NFR3 品質** 行カバレッジ 80% 以上を保つ。名前の検査と `reset --all` の対象判定には、境界の入力(`..`、`a/b`、先頭の `.`、長さ 64 と 65、大文字小文字、別の定義と名前が重なるもの、壊れたもの、一時ファイル)を並べたテストを必ず含める。`cargo fmt --check`、`cargo clippy -D warnings`、Rust 1.85 でのビルドとテスト、3 OS の CI が成功する。
- **NFR4 版** 機能追加のため、`Cargo.toml` の `version` を 0.5.0 に上げる。[assumption]

## Constraints

- Rust 1.85 で、ビルドできること(let チェーンなど、1.88 以降の構文は使わない)。
- ライブラリの追加はしない。
- Windows でも動くこと(ファイル名の制約:使える文字を `A-Za-z0-9._-` に絞るので、予約名の `CON` などは、拡張子付きのファイル名 `x.toml.CON.state` になり、問題にならない)。
- CI の `docs-check` が README の必須の語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)を検索する。消さない。
- チームの実践(test-after、`rustfmt`、`clippy`、`main` への取り込みで履歴を残す)に従う。

## Assumptions

- `watch` の run ID の行は、`Run ID: <値>` とする(表の前に 1 行)。[assumption]
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` は同じファイルになる。この場合、中身の `run_id` の照合で、食い違いが壊れたステート(65)として検出される。run ID は、大文字小文字だけで区別しない運用を、README に書く。[assumption]
- 版を 0.5.0 に上げる。タグ付けと公開は、この作業の範囲外(あとの Deployment Execution で、人が決める)。[assumption]
- 同じ run ID の同時起動の排他制御はしない(従来どおり。呼び出し側が抑止する)。[assumption]

## Out of scope

- ステートの置き場所の指定(`CONDUCTOR_STATE_DIR`)。古いステートの自動削除。同じ run ID の排他制御。
- `validate` での run ID の検査。`convert` への `--run-id`。
- デモ(`demo/`)への run ID の追加。
- 定義ファイルの形式の変更(`run_id_env` の追加以外)と、`format_version` の引き上げ。

## Open questions

None.

## Assumptions & Open Questions

None.
