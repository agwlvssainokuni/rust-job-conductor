# 契約の概要(Contract Summary): v0.9.0 の機能拡張

出典: ユニットの定義(`units-generation/unit-of-work.md`)、ユニットの依存(`unit-of-work-dependency.md`)、コンポーネントの一覧(`domain-design/components.md`)、要件(`requirements-analysis/requirements.md`)、契約の質問(`contract-design-questions.md`。以下 Q1〜Q4)。

この作業の境界は、同じ実行ファイル(`conductor`)の中の、Rust の公開関数と型(ユニット間)と、利用者が使うコマンドライン、定義ファイル、出力(外部)である。ネットワークの通信はない。

## 契約の表

| # | Provider Unit | Consumer | Mechanism | Owner |
|---|---------------|----------|-----------|-------|
| C1 | u1-env-injection | u3-dry-run | Rust の公開関数と型(shared schema) | u1-env-injection |
| C2 | u2-graph | u3-dry-run | Rust の公開関数と型(shared schema) | u2-graph |
| C3 | u1-env-injection | u4-init | 定義ファイルのキー(shared schema) | u1-env-injection |
| C4 | u1-env-injection | External: 定義ファイルを書く利用者 | 定義ファイルの書式(shared schema) | u1-env-injection |
| C5 | u2-graph; u3-dry-run; u4-init | External: コマンドラインの利用者 | コマンドライン(サブコマンド、オプション、終了コード) | 各ユニット(自分のコマンド) |
| C6 | u2-graph | External: 図を読む利用者と、Mermaid を描画する道具 | text と Mermaid の出力の形 | u2-graph |
| C7 | u3-dry-run | External: コマンドラインの利用者 | `--dry-run` の出力の構造 | u3-dry-run |

契約の範囲は、Q1 の A(コマンドライン、定義のキー、出力の構造)。

### ユニット間の契約の対象外

次の 2 つは、ユニット間の契約ではなく、同じクレート内の編集の調整である。

- 共有する型(`CoreTypes`): u1 が、定義の型に `env` と `env_file` の項目を足す。u2、u3、u4 は、既存の項目を読むだけで、新しい項目には依存しない(新しい項目は、任意の項目で、読まなくても動く)。そのため、依存の図に、u2 から u1 への辺は足さない。型の変更は、u1 のユニットの中で、ほかのユニットのコードが壊れないことを、テストで確かめる。
- メッセージのカタログ(日本語/英語): 各ユニットが、自分の機能のメッセージを、同じファイルに足す。衝突は、ユニットを順に作ることで避ける(ユニットの依存の文書の注意を参照)。

## C1: 渡される環境変数の名前を返す関数(u1 → u3)

```yaml shared-schema
contract: C1
provider: u1-env-injection (component CoreEnvFile)
consumer: u3-dry-run (component CorePlan)
function: 渡される環境変数の名前の取り出し
inputs:
  - definition: PipelineDefinition と JobDefinition の env、env_file の項目
  - env_file_contents: 読み込み済みの .env の内容(文字列。ファイルには触れない)
  - inherited_env_names: 引き継いだ環境の変数の名前
  - context: パイプライン名、ジョブ ID、run ID(任意)、試行の回数
outputs:
  - names: 渡される環境変数の名前の並び(値を含めない)
  - origin: 名前ごとの出どころ(inherited、env_file、pipeline.env、job.env、builtin)
errors:
  - kind: 予約名の拒否(CONDUCTOR_ で始まる名前)
  - kind: .env の書式の誤り(行番号つき。値を含めない)
  - kind: .env のファイルが無い、または読めない(FR1.8。読み取りは runtime が行い、結果を種類つきで渡す)
  - kind: 上限の超過(ファイルの大きさ 1 MiB、項目 10,000)と、NUL を含む値(FR1.9)
guarantees:
  - 値は、結果に含めない(名前だけ)
  - 優先順位は、引き継いだ環境 < pipeline の env_file < pipeline.env < job の env_file < job.env
```

- 持ち主: u1。使う側(u3)は、この入出力に合わせる(Q4: A)。

## C2: 辺の判断の入口(u2 → u3)

```yaml shared-schema
contract: C2
provider: u2-graph (component CoreFlow が公開する関数)
consumer: u3-dry-run (component CorePlan)
function: 流れの判断の公開(次のジョブの決定、起動前の判断、再開位置の決定、辺の取り出し)
inputs:
  - definition: 検証済みの定義
  - state: ステートの内容(各ジョブの状態、再開位置)
  - job_index: ジョブの位置
  - classification: 成功、警告、エラー(結果の分類)
  - mode: 実行の方式(pipeline、個別の run)
outputs:
  - next_step: 次のジョブ(なし、または @end)、スキップするジョブ、@end に到達したか、完了したか、再開位置
  - pre_launch_decision: 起動前の判断(実行する、スキップする、拒否する)と、その理由の種類
  - resume_job_id: 再開位置(あるとき)
  - edges: ジョブから出る辺の並び(行き先、種類(成功、警告、既定)、飛び越すジョブ、定義に書かれた遷移か)
errors: なし(純粋な関数。失敗しない。ファイルにも、プロセスにも触れない)
guarantees:
  - 実行のときの判断と、同じ関数から出る(図と模擬が、実行と食い違わない)
  - 起動前の判断、次の動き、再開位置は、すでに公開されている既存の関数である(この契約は、辺の取り出しの入口を、新しく公開する)
```

- 持ち主: u2。使う側(u3)は、この入出力に合わせる(Q4: A)。

## C3、C4: 定義ファイルの新しいキー(u1 → u4、利用者)

```yaml shared-schema
contract: C3 と C4
provider: u1-env-injection
consumers: [u4-init(ひな形の例), 定義ファイルを書く利用者]
format_version: 1  # 変更しない
keys:
  pipeline.env:   { type: 文字列の表, value: 文字列, name_rule: "run_id_env と同じ" }
  pipeline.env_file: { type: 文字列 または 文字列の配列, element: 文字列, base: 定義ファイルのあるディレクトリ }
  job.env:        { type: 文字列の表, value: 文字列, name_rule: "run_id_env と同じ" }
  job.env_file:   { type: 文字列 または 文字列の配列, element: 文字列, base: 定義ファイルのあるディレクトリ }
rejects:
  - CONDUCTOR_ で始まる名前(予約名)
  - 同じ表の中で、大文字小文字だけが違う名前
  - 文字列でない値
  - env_file の配列の、文字列でない要素
  - 名前の規則に合わない名前(pipeline.env と job.env の両方)
compat:
  - 旧版(v0.8.0)の conductor は、新しいキーを「未知のキー」として拒否する
  - 新しいキーを使わない既存の定義は、変更なしで読める
```

## C5: コマンドライン

```yaml shared-schema
contract: C5
commands:
  graph:
    synopsis: "conductor graph [--config <パス>] [--format text|mermaid] [-v] [--state] [--run-id <ID>] [<パイプライン名>]"
    exit: { ok: 0, definition_error: "64(validate と同じ)" }
  init:
    synopsis: "conductor init [--config <パス>] [--force] [<パイプライン名>]"
    exit: { ok: 0, already_exists: 69 }
    path_dash: 標準出力にひな形を出す(ファイルを作らない)
  pipeline_and_run:
    added_option: "--dry-run"
    added_option_assume: "--assume <ジョブID>=<終了コード>(複数回指定できる)"
    exit: { plan_ok: 0, others: "本実行と同じ" }
changes_to_existing: なし(既存のオプションと終了コードの意味は、変えない)
env_file_errors:
  exit: 決定待ち(要件 FR1.8 の未解決事項。機能の設計で決める。Open questions の C4 の行)
  note: .env のファイルが無い、書式の誤りのときは、ジョブを 1 つも実行せずに止まる(非 0)
option_combinations:
  - "--assume は --dry-run と一緒でないと、使い方の誤り(終了コード 68)とする"
  - "--run-id は graph で --state と一緒でないと、使い方の誤り(終了コード 68)とする(決定待ち。機能の設計で確定)"
  - "--force は init だけで使える。init 以外では、使い方の誤り"
  - "graph の --format は text と mermaid だけ。ほかの値は、使い方の誤り"
```

- 既存の終了コード(64〜68、70 など)の意味は、変えない。新しい値は、69(`init`)だけである。
- 契約の変更の方針(Q3: A): 新しいキーやオプションを足すのは、互換のある変更。既存の意味を変えるのは、破壊的な変更として、リリースノートと README に明記する。

## C6: `graph` の出力の形

- text(既定): `git log --graph` 風。ジョブは `*`、終点(`@end`)は `o`。遷移で飛び越すジョブは、`|\` と `|/` の分岐と合流で表す。成功と警告の行き先が同じなら、1 つの辺にまとめる。`-v` では、コマンド、リトライの設定、飛び越しの注記を出す。`--state` では、`[OK]`、`[--]`、`[NG]`、`[  ]` を付ける。
- Mermaid(`--format mermaid`): `flowchart`。
- **見た目は、テストで厳密に固定する**(期待する出力を、そのままテストに書く)。見た目を変えるときは、破壊的な変更として扱う(Q2: A)。
- 環境変数の値は、出力に含めない。

```yaml shared-schema
contract: C6
text:
  node: "ジョブは「* <ジョブ ID>」の 1 行。終点は「o @end」"
  fork_and_merge: "遷移で飛び越すジョブがあるとき、ジョブの次の行に「|\\」、飛び越される区間はレーンを 1 つ右にずらして「| * <ジョブ ID>」、合流の行に「|/」"
  order: 上から下が、実行の順(定義の記述順)
  merged_edge: 成功と警告の行き先が同じなら、辺を 1 つにまとめる
  verbose: "-v では、ジョブの行にコマンドを、続く行に、リトライの設定と、飛び越しの注記を出す"
  state: "--state では、ジョブの行頭に [OK]、[--]、[NG]、[  ] を付け、再開位置を示す"
mermaid:
  header: "flowchart TD"
  node: ジョブ ID を節にし、終点 @end を節にする
  edge: 成功と警告の辺を、種類を示す辺のラベルで区別する
```

- 具体の期待出力(複数の飛び越しがある図を含む)は、機能の設計で決める(Open questions の C6 の行)。

## C7: `--dry-run` の出力の構造

- 何も実行せず、ステートを作成も更新も削除もしない。環境変数の値は、出力に含めない。
- 出力の構造(どの項目が、どの順に出るか)を、契約として固定する(Q1: A)。

```yaml shared-schema
contract: C7
output_order:
  - header: パイプライン名(と run ID があるとき run ID)
  - resume: 再開位置(あるとき。なければ「最初から」)と、その理由(前回の停止、完了済み、など)
  - assumed: --assume で仮定した終了コードの一覧(あるとき)
  - jobs: ジョブごとに、定義の順で 1 件ずつ
job_entry:
  - job_id
  - disposition: 実行する | スキップする(実行済み) | スキップする(遷移による) | 再開位置
  - command_words: 分割後のコマンド(コマンドの誤りのときは、誤りの種類と理由。本実行と同じ誤り)
  - env_names: 渡される環境変数の名前の並び(値は含めない)
  - retry: リトライの設定(あるとき)
on_error:
  - 定義の誤り、ステートの誤り、コマンドの誤りは、本実行と同じ誤りと終了コードにする
  - 計画を出せたときの終了コードは 0
```

- 項目の具体の文言と書式(表の形、記号)は、機能の設計で決める(Open questions の C7 の行)。

## 契約の持ち主の規則

- 提供するユニット(u1、u2)が、契約の仕様を持つ。使うユニット(u3)は、それに合わせる(Q4: A)。
- 契約の変更は、ユニットの設計の中で、提供するユニットと使うユニットの両方を確認して行う。
- 追加の変更(新しい項目)は、使う側が、知らない項目を無視すれば、安全である。破壊的な変更は、リリースノートと README に明記する(Q3: A)。

## Open questions

| Contract | Question | Blocks |
|----------|----------|--------|
| C4 | `.env` の誤り(ファイルが無い、書式の誤り)の終了コードの具体の値(要件 FR1.8 の未解決事項) | u1-env-injection |
| C1、C7 | コマンドの分割(`runtime::split`)を、`core` の計画から使う方法(`CliFlows` が分割して渡すか)。分割の誤りが、どこで出るか(ドメイン設計のレビューの指摘) | u3-dry-run |
| C5 | `--assume` の不正な入力(存在しないジョブ ID、書式の誤り、重複、範囲外の終了コード)のメッセージと終了コード(要件分析のレビューの指摘) | u3-dry-run |
| C5 | `init` の端の動き(パイプライン名の検証、`-` と `--force` の併用、親ディレクトリが無いとき、パスがディレクトリのとき) | u4-init |
| C6 | text と Mermaid の期待出力の具体例(複数の飛び越しがある図を含む) | u2-graph |
| C7 | 出力の項目の具体の文言と書式(表の形、記号、日本語と英語の見出し) | u3-dry-run |
| C5 | `graph --run-id` を `--state` なしで許すか(併用の規則の確定) | u2-graph |
| C1 | `CONDUCTOR_ATTEMPT` を、`--dry-run` の環境変数の名前に含めるときの値(リトライの何回目として示すか) | u1-env-injection、u3-dry-run |
