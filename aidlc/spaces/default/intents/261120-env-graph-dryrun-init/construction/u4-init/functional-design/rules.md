# ビジネスルール(Rules): U4 u4-init(`conductor init`)

出典: 要件(`requirements-analysis/requirements.md` の FR4.1〜FR4.5、FR5.1、FR5.2、NFR5、NFR6)、契約(`contract-design/contract-summary.md` の C3、C4、C5)、U1 の設計、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q4)、`entities.md`、`functional-spec.md`(ひな形の原本)。

ルールは `BR{グループ}.{連番}`。グループ: BR1 コマンドと作成、BR2 ひな形、BR3 誤りと終了コード、BR4 メッセージと文書。

```yaml
rules:
  # --- BR1: コマンドと作成 ---
  - id: BR1.1
    statement: "`conductor init [--config <パス>] [--force] [<パイプライン名>]` は、定義のひな形を作る。パスの既定は `conductor.toml`(カレントディレクトリ)。`--config` の解決は、ほかのコマンドと同じ"
    category: policy
    applies_to: [CliArgs, CliFlows]
    trigger: "`init`"
    logic: "パスを決め、ひな形を作って、出力先に書く。定義の読み込み、`.env` の読み取り、ステートの読み取り、シグナルの準備は、しない"
    violation: なし
    source: FR4.1

  - id: BR1.2
    statement: 作る先にファイルが既にあるときは、何も変更せず、終了コード 69 で終わる。標準エラーに、「<パス> は既にあります(上書きするには --force)」を出す
    category: constraint
    applies_to: [RuntimeFileCreate, CliFlows]
    trigger: "`init`(`--force` なし)"
    logic: "排他的な作成(BR1.3)が `already_exists` を返したら、表示して 69"
    violation: 終了コード 69
    source: FR4.2、契約 C5

  - id: BR1.3
    statement: ファイルの作成は、排他的に行う(同時に作られても、既存のファイルを壊さない)。新規は「ファイルがなければ作る」を 1 回の操作(`create_new`)で行い、`--force` の上書きは、同じディレクトリに一時ファイルを書いて、置き換える(ステートの書き込みと同じ流儀)
    category: constraint
    applies_to: [RuntimeFileCreate]
    trigger: "`init` のファイルの書き込み"
    logic: "新規: `create_new` が失敗したら、パスを `symlink_metadata` で調べて分類する(OS によって、ディレクトリへの `create_new` の誤りの種類が違うため): ディレクトリなら `is_directory`、ファイルまたはシンボリックリンク(宙に浮いたリンクを含む)なら `already_exists`、`NotFound` なら `parent_missing`、権限の誤りなら `permission_denied`、そのほか(パスの末尾が `/`、親がファイル、など)は `other_io`。`--force`: 一時ファイル(`state/writer.rs` の一時ファイルの名前の作り方を再利用)に書き、置き換える。置き換え先がディレクトリなら `is_directory`。シンボリックリンクは、リンクそのものを置き換える(リンク先は変えない)。一時ファイルは、失敗したとき、残さない。新規で作ったファイルへの書き込みに失敗したときは、そのファイルを消す。ひな形は、UTF-8、LF の改行(Windows でも)。既知の限界: 新規の作成は、作ったあとに書くので、その短い間と、強制終了のあとには、空のファイルが見えうる(2 回目の `init` は 69 になる。`--force` で直せる)。`--force` の置き換えは、Windows では、読み取り専用の置き換え先を置き換えられない(UNIX は置き換えられる)。この OS の違いは、README に書く。置き換えたファイルは、新しい権限と所有者になる"
    violation: なし(テストで確かめる)
    source: FR4.3

  - id: BR1.4
    statement: "`--force` を付けると、既にあるファイルを上書きする。成功したら、標準エラーに「<パス> を作成しました」(置き換えたときは「置き換えました」)を出して、終了コード 0"
    category: policy
    applies_to: [CliFlows]
    trigger: "`init --force`"
    logic: 作成の結果(`created`、`replaced`)に応じたメッセージを出す
    violation: なし
    source: FR4.2

  - id: BR1.5
    statement: "パスに `-` を指定すると(`--config -`)、ファイルを作らずに、標準出力にひな形だけを出す。成功のメッセージは出さない(標準エラーにも)。このとき、`--force` は、意味がないので、誤りにせず、無視する"
    category: policy
    applies_to: [CliFlows]
    trigger: "`init --config -`"
    logic: ひな形の全体を作ってから、標準出力に 1 回で書く。標準出力に書けないときは、終了コード 65
    violation: 終了コード 65
    source: FR4.4、質問 Q4

  - id: BR1.6
    statement: 親のディレクトリが無いときは、作らず、誤りにする(`init` は、ファイルだけを作る)
    category: constraint
    applies_to: [RuntimeFileCreate]
    trigger: "`init`"
    logic: "`parent_missing` を返す(終了コード 65。BR3.2)"
    violation: 終了コード 65
    source: 質問 Q4

  # --- BR2: ひな形 ---
  - id: BR2.1
    statement: "ひな形は 1 種類で、コメントだけがロケール(日本語/英語)に従う。中身: `format_version = 1`、`[pipeline]`(名前と説明)、`[pipeline.env]`(例: `GREETING = \"hello\"`)、`[[job]]` が 2 つ(`hello`: `echo`、`flaky`: `[job.retry]` の例)"
    category: policy
    applies_to: [InitTemplate]
    trigger: ひな形の作成
    logic: "`functional-spec.md` の日本語と英語のひな形(原本)に、1 文字も違わず一致する。`[pipeline.env]` と `[job.retry]` は、有効な行(コメントアウトしない)。コマンドは `echo` で、Windows では `cmd /c echo` に直すことを、コメントで案内する(OS ごとに変えない)"
    violation: なし
    source: FR4.5

  - id: BR2.2
    statement: "パイプライン名の既定は固定の `my-pipeline`。名前に使える文字は、定義の検査と同じ規則(空でない。空白だけでもない)。空、または空白だけの名前は、使い方の誤り(68)。名前は、TOML の基本文字列として、`\"` と `\\` と制御文字(U+0000〜U+001F、U+007F)をエスケープして書く(制御文字は `\\uXXXX`)"
    category: validation
    applies_to: [CliArgs, InitTemplate]
    trigger: ひな形の作成
    logic: エスケープで、どの名前でも、TOML として正しいひな形になる。名前は、コメントには書かない(改行で、コメントが壊れないため)。空または空白だけの名前は、`InvalidValue`(固定の対象、名前の値を含めない)にする。位置引数が 2 つ以上のときの `UnexpectedArgument` と、UTF-8 でない値の `InvalidValue` は、既存の規則どおり、エスケープした対象を表示する(既存の動き。名前の値を新しく表示する経路は、足さない)
    violation: 使い方の誤り(68)
    source: FR4.1、質問 Q1、Q2

  - id: BR2.3
    statement: "ひな形に、利用者の環境変数の値、`.env` の内容、ステートの内容は、入らない。環境変数の例の値は、固定の文字列 `hello` だけ"
    category: constraint
    applies_to: [InitTemplate]
    trigger: ひな形の作成
    logic: ひな形の入力は、ロケールと名前だけ
    violation: なし
    source: FR1.12、project.md の NEVER

  - id: BR2.4
    statement: 作ったひな形は、必ず `validate` に通る(日本語と英語の両方。名前を変えても。特殊な文字の名前を含む)。この性質を、テストで確かめる
    category: constraint
    applies_to: [InitTemplate]
    trigger: テスト
    logic: "ひな形から、定義を読み込み(`parse_definition`)、`command` を分け(`check_command`)、`validate` の `.env` の検査(`env_file` なし)を通す。`[pipeline.env]` の例が、U1 の検査(名前の規則、予約名でない)に通る。`[job.retry]` の例が、リトライの検査に通る"
    violation: テストの失敗
    source: FR4.5

  # --- BR3: 誤りと終了コード ---
  - id: BR3.1
    statement: "終了コード: 作成できた、または標準出力に出せたら 0。既にあれば 69。使い方の誤り(許されないオプション、位置引数が 2 つ以上、名前が空)は 68"
    category: policy
    applies_to: [CliFlows]
    trigger: "`init`"
    logic: "`--force` は、`init` だけで使える(それ以外のコマンドでは、許されないオプション 68)。`init` では、`--config` と `--force` 以外のすべてのオプション(`--dry-run`、`--assume`、`--format`、`--state`、`--run-id`、`--remove-on-complete`、`-v`、`--all`、`--to` など)は、許されないオプション(68)。引数の誤り(68)は、ファイルに触れる前に返す(名前が空で、ファイルが既にあるときは、68 が勝つ)"
    violation: 終了コード 68、69
    source: FR4.2、契約 C5

  - id: BR3.2
    statement: "ファイルを作れなかったとき(親のディレクトリが無い、書く権限がない、作る先がディレクトリ、そのほかの入出力の誤り)は、終了コード 65(既存の「書き込みの失敗」と同じ)。標準エラーに、誤りの種類の固定の文と、パスを出す。OS のエラーの文は含めない"
    category: constraint
    applies_to: [CliFlows]
    trigger: "`init`"
    logic: "`CreateError`(`already_exists` 以外)を、固定の文に対応づける(日本語と英語)。パスは、制御文字をエスケープして、切り詰めて表示する"
    violation: 終了コード 65
    source: 質問 Q3

  # --- BR4: メッセージと文書 ---
  - id: BR4.1
    statement: 新しいメッセージ(作成した、置き換えた、既にある、作れなかった各種、使い方)は、日本語と英語の両方で用意する
    category: policy
    applies_to: [CliReport]
    trigger: メッセージの表示
    logic: 既存のメッセージの網羅のテストに、新しい種類を足す
    violation: なし
    source: FR5.1

  - id: BR4.2
    statement: README に、`init` の使い方と、ひな形の例と、終了コード 69 を、版(v0.9.0 以降)つきで書く。デモに、`init` の場面を足す
    category: policy
    applies_to: [CliReport]
    trigger: ユニットの完了
    logic: README の必須語句の検査(`docs-check`)に合わせて書き、デモのテストを足す。README の例は、テストが、実際のひな形と照合する
    violation: なし
    source: FR5.2
```

## ルールの要約(`rules` の YAML から作った表)

| 規則 | 内容 | 出典 |
|------|------|------|
| BR1.1 | コマンドの形、パスの既定、定義を読まない | FR4.1 |
| BR1.2 | 既にあれば 69 | FR4.2、C5 |
| BR1.3 | 排他的な作成、上書きは一時ファイルから置き換え | FR4.3 |
| BR1.4 | `--force` で上書き、成功のメッセージ | FR4.2 |
| BR1.5 | `-` で標準出力、`--force` は無視 | FR4.4、Q4 |
| BR1.6 | 親のディレクトリは作らない | Q4 |
| BR2.1 | ひな形の中身(1 種類、コメントだけロケール) | FR4.5 |
| BR2.2 | 名前の既定と規則、エスケープ | FR4.1、Q1、Q2 |
| BR2.3 | 利用者の値は入らない | FR1.12 |
| BR2.4 | 必ず `validate` に通る | FR4.5 |
| BR3.1 | 終了コード(0、68、69) | FR4.2、C5 |
| BR3.2 | 作れなかったときは 65 | Q3 |
| BR4.1 | メッセージは日本語と英語 | FR5.1 |
| BR4.2 | README とデモ | FR5.2 |
