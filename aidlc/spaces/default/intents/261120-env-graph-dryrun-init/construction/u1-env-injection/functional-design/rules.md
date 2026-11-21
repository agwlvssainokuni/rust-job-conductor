# ビジネスルール(Rules): U1 u1-env-injection(環境変数の注入と `.env`)

出典: 要件(`requirements-analysis/requirements.md` の FR1.1〜FR1.13、FR5.1、FR5.2、NFR2、NFR3)、契約(`contract-design/contract-summary.md` の C1、C3、C4)、機能の設計の質問(`functional-design-questions.md`。以下 Q1〜Q8)、`entities.md`。

ルールは `BR{グループ}.{連番}` で番号づけする。グループ: BR1 定義のキー、BR2 `.env` の書式、BR3 `.env` の読み取りと上限、BR4 層の統合、BR5 自動の変数、BR6 実行の前の読み取り、BR7 `validate`、BR8 値の秘匿、BR9 メッセージと文書。

```yaml
rules:
  # --- BR1: 定義のキー ---
  - id: BR1.1
    statement: "`env` の値は文字列だけで、値の中の `${X}` などは展開しない"
    category: validation
    applies_to: [EnvVar]
    trigger: 定義の読み取り(`[pipeline.env]`、`[job.env]`)
    logic: IF 値が文字列でない THEN 型の誤り(typeMismatch)。値は、書いたとおりに渡す
    violation: 定義の誤り(typeMismatch)。終了コード 64
    source: FR1.1

  - id: BR1.2
    statement: "`env` の名前は、`run_id_env` と同じ規則(1 文字以上、先頭は ASCII の英字か `_`、2 文字目以降は ASCII の英数字か `_`)"
    category: validation
    applies_to: [EnvVar]
    trigger: 定義の読み取り
    logic: IF 名前が `is_valid_env_name` を満たさない THEN 誤り
    violation: 定義の誤り(invalidEnvName)。終了コード 64
    source: FR1.2

  - id: BR1.3
    statement: "`CONDUCTOR_` で始まる名前は予約名で、利用者は書けない。判定は、大文字小文字を区別しない(ASCII)"
    category: constraint
    applies_to: [EnvVar, EnvEntry]
    trigger: 定義の読み取り、`.env` の読み取り
    logic: IF 名前を ASCII の大文字にしたものが `CONDUCTOR_` で始まる THEN 誤り
    violation: "`env` の表では定義の誤り(reservedEnvName)、`.env` では envFile の誤り(reservedName)。終了コード 64"
    source: FR1.11
    note: 要件の文面は大文字小文字に触れていない。Windows が区別しないため、すべての OS で同じ結果にするため、区別しない判定にした(要件より厳しい側の補足)

  - id: BR1.4
    statement: 同じ表(`env` の表、または 1 つの `.env` ファイル)の中で、大文字小文字だけが違う名前は誤り
    category: constraint
    applies_to: [EnvVar, EnvEntry]
    trigger: 定義の読み取り、`.env` の読み取り
    logic: IF 表の中に、ASCII の大文字小文字を無視すると同じで、綴りが違う 2 つの名前がある THEN 誤り(完全に同じ名前の重複は、BR2.7 の扱い)
    violation: "`env` の表では定義の誤り(envNameCaseConflict)、`.env` では envFile の誤り(caseConflict)。終了コード 64"
    source: NFR3、質問 Q7

  - id: BR1.5
    statement: 値に NUL を含むものは誤り
    category: validation
    applies_to: [EnvVar, EnvEntry]
    trigger: 定義の読み取り、`.env` の読み取り
    logic: IF 値に U+0000 を含む THEN 誤り
    violation: "`env` の表では定義の誤り(invalidEnvValue)、`.env` では envFile の誤り(nulInValue)。終了コード 64"
    source: FR1.9

  - id: BR1.6
    statement: "`env_file` は、文字列、または文字列の配列。要素は空でない文字列"
    category: validation
    applies_to: [PipelineDefinition, JobDefinition]
    trigger: 定義の読み取り
    logic: IF 値が文字列でも配列でもない、または配列の要素が文字列でない THEN 型の誤り(typeMismatch)。IF 要素が空の文字列 THEN 誤り(emptyField)。空の配列(`env_file = []`)は誤りではなく、何も読まない
    violation: 定義の誤り。終了コード 64
    source: FR1.4

  - id: BR1.7
    statement: "`env_file` の相対パスは、定義ファイルのあるディレクトリを基準にする。絶対パスと `..` を許す。`~` と変数は展開しない"
    category: policy
    applies_to: [RuntimeEnvLoader]
    trigger: "`.env` の読み取り"
    logic: IF パスが相対 THEN (定義ファイルのディレクトリ)/(パス)。IF 絶対 THEN そのまま。シンボリックリンクは OS の規則のまま
    violation: なし(読めないときは BR3.4)
    source: FR1.5、質問 Q6

  - id: BR1.8
    statement: "`format_version` は 1 のまま。キー `pipeline.env`、`pipeline.env_file`、`job.env`、`job.env_file` を許可し、既存のキーの意味は変えない。新しいキーを使わない既存の定義は、変更なしで読める"
    category: constraint
    applies_to: [PipelineDefinition, JobDefinition]
    trigger: 定義の読み取り
    logic: IF キーが許可の一覧にある THEN 読む。IF ない THEN 従来どおり unknownKey。旧版の `conductor` は、新しいキーを unknownKey として拒否する(互換の範囲)
    violation: 従来の unknownKey
    source: FR1.13

  # --- BR2: .env の書式 ---
  - id: BR2.1
    statement: "`.env` の行は、空行、コメント(先頭の空白のあとが `#`)、または `KEY=VALUE`。`KEY` の前の `export `(空白 1 つ以上)は許す"
    category: validation
    applies_to: [EnvFileContent]
    trigger: "`.env` の読み取り"
    logic: "IF 行が空白だけ、または空白に続く `#` THEN 読み飛ばす。IF `export` のあとに空白(スペースまたはタブ)が 1 つ以上続く THEN その部分を取り除く(`export` だけの行、`export=1` は、`KEY` が `export` の項目)。IF `=` がない THEN missingEquals。`KEY` と値は、最初の `=` で分ける(値の中の `=` は、値の文字)。`KEY` の前後の空白は取り除く。例: 受け付ける `A=1`、`export A=1`、`A = 1`、`A=x=y`(値は `x=y`)。誤り `A`(missingEquals)、`=1`(invalidName)、`1A=x`(invalidName)"
    violation: envFile の誤り(missingEquals、invalidName、reservedName。行番号つき)
    source: FR1.6

  - id: BR2.2
    statement: 引用符なしの値は、行末まで。前後の空白を除く。空白に続く `#` から先は、コメント
    category: validation
    applies_to: [EnvEntry]
    trigger: "`.env` の読み取り"
    logic: "`=` の右の空白を読み飛ばしたあと、最初の文字で、引用符つきか、なしかを決める(`KEY= \"x\"` は、二重引用符の値 `x`)。IF 引用符で始まらない THEN 最初の「空白に続く `#`」の手前まで取り、前後の空白を除く(`#` が空白なしで続くときは、値の一部。例: `A=x#y` の値は `x#y`、`A=x #y` の値は `x`)"
    violation: なし
    source: FR1.6

  - id: BR2.3
    statement: 単一引用符の値は、そのまま(エスケープなし)。閉じ引用符のあとは、空白とコメントだけを許す
    category: validation
    applies_to: [EnvEntry]
    trigger: "`.env` の読み取り"
    logic: IF `'` で始まる THEN 次の `'` までを値にする。IF 閉じていない THEN unterminatedQuote。IF 閉じたあとに、空白とコメント以外がある THEN trailingGarbage
    violation: envFile の誤り(unterminatedQuote、trailingGarbage)
    source: FR1.6

  - id: BR2.4
    statement: "二重引用符の値は、エスケープ `\\n`、`\\r`、`\\t`、`\\\"`、`\\\\` だけを解く。ほかの `\\` は誤り。閉じ引用符のあとは、空白とコメントだけを許す"
    category: validation
    applies_to: [EnvEntry]
    trigger: "`.env` の読み取り"
    logic: IF `\"` で始まる THEN 閉じる `\"` までを読み、`\\` に続く文字が上の 5 つなら変換する。IF それ以外 THEN invalidEscape。IF 閉じていない THEN unterminatedQuote。IF 閉じたあとに、空白とコメント以外がある THEN trailingGarbage
    violation: envFile の誤り(invalidEscape、unterminatedQuote、trailingGarbage)
    source: FR1.6

  - id: BR2.5
    statement: 変数の展開と複数行の値は受け付けない。`KEY=` は空の文字列
    category: constraint
    applies_to: [EnvEntry]
    trigger: "`.env` の読み取り"
    logic: "値の中の `$`、`${` は、そのまま値の文字として扱う(展開しない)。引用符が行をまたぐ書き方は、閉じていない引用符(unterminatedQuote)。IF `=` の右が空 THEN 空の文字列"
    violation: unterminatedQuote
    source: FR1.6

  - id: BR2.6
    statement: ファイルの先頭の UTF-8 の BOM は無視する。改行は LF と CRLF の両方を受け付ける(行末の CR は取り除く)
    category: validation
    applies_to: [EnvFileContent]
    trigger: "`.env` の読み取り"
    logic: IF 先頭が U+FEFF THEN 取り除く。行末の `\r` は、行の一部にしない
    violation: なし
    source: FR1.6、質問 Q5

  - id: BR2.7
    statement: 同じ `.env` の中で、完全に同じ名前が 2 回以上出たときは、後ろが勝つ。警告を出す(誤りにしない)。複数のファイルをまたぐ上書きは、警告しない
    category: policy
    applies_to: [EnvFileContent, EnvWarning]
    trigger: "`.env` の読み取り"
    logic: IF 同じ名前の項目が既にある THEN 後ろの値で置き換え、警告(file、後ろの行、名前)を足す
    violation: なし(警告だけ。表示するのは `validate`。BR7.2)
    source: FR1.7

  - id: BR2.8
    statement: 誤りは、ファイルの名前と行番号を付けて報告する。値を含めない。すべて集め、20 件で打ち切り、残りは件数だけを示す
    category: policy
    applies_to: [EnvFileError]
    trigger: "`.env` の読み取りの誤りの表示"
    logic: IF 誤りが 20 件を超える THEN 先頭の 20 件を表示し、「ほか N 件」を添える。行ごとの誤りは、読み取りを止めずに集める
    violation: なし
    source: FR1.6、質問 Q4

  # --- BR3: .env の読み取りと上限 ---
  - id: BR3.1
    statement: "`.env` の大きさの上限は 1 MiB(1,048,576 バイト)。読み取りの前に大きさを調べ、読み取りの途中でも超えないよう、上限 + 1 バイトまでしか読まない"
    category: constraint
    applies_to: [RuntimeEnvLoader]
    trigger: "`.env` の読み取り"
    logic: IF ファイルの大きさ > 上限 THEN tooLarge(中身を読まない)。IF 読み取りの結果が上限を超える THEN tooLarge
    violation: envFile の誤り(tooLarge)。終了コード 64
    source: FR1.9

  - id: BR3.2
    statement: 1 つの `.env` の項目の数の上限は 10,000
    category: constraint
    applies_to: [EnvFileContent]
    trigger: "`.env` の読み取り"
    logic: IF 項目(重複を含む、取り出した行の数)が 10,000 を超える THEN tooManyEntries
    violation: envFile の誤り(tooManyEntries)。終了コード 64
    note: 上限は、ファイルごと。統合した環境の全体の大きさには、上限を設けない(OS の上限を超えて、起動できないときは、従来の「起動の失敗」として扱う。既知の限界として、受け入れる。README に書く)
    source: FR1.9

  - id: BR3.3
    statement: "`.env` は UTF-8 でなければならない"
    category: validation
    applies_to: [RuntimeEnvLoader]
    trigger: "`.env` の読み取り"
    logic: IF バイト列が UTF-8 として正しくない THEN notUtf8
    violation: envFile の誤り(notUtf8)。終了コード 64
    source: FR1.6

  - id: BR3.4
    statement: ファイルが無い、読めない(権限、ディレクトリなど)ときは、誤り
    category: validation
    applies_to: [RuntimeEnvLoader]
    trigger: "`.env` の読み取り"
    logic: IF 存在しない THEN fileMissing。IF 開けない、読めない THEN unreadable
    violation: envFile の誤り。終了コード 64(質問 Q1)
    source: FR1.8、質問 Q1

  # --- BR4: 層の統合 ---
  - id: BR4.1
    statement: "ジョブの環境変数の優先順位は、後ろが勝つ順で、引き継いだ環境 < `[pipeline].env_file` < `[pipeline.env]` < `[job].env_file` < `[job.env]`"
    category: calculation
    applies_to: [EffectiveEnv]
    trigger: ジョブの起動(試行ごと)
    logic: 層を、この順に、前から後ろへ重ねる。引き継いだ環境は、子のプロセスが OS の規則で引き継ぐため、統合の結果には含めない(結果は、足すものだけ)
    violation: なし
    source: FR1.3

  - id: BR4.2
    statement: "`env_file` が配列のときは、書いた順に読み、後ろのファイルが勝つ"
    category: calculation
    applies_to: [EffectiveEnv]
    trigger: ジョブの起動
    logic: 同じ層の中の複数のファイルを、書いた順に重ねる
    violation: なし
    source: FR1.4

  - id: BR4.3
    statement: "統合の規則: 完全に同じ名前は、後ろの層が勝って 1 つにする。大文字小文字だけが違う名前は、別の名前として、層の順に並べる"
    category: calculation
    applies_to: [EffectiveEnv]
    trigger: ジョブの起動
    logic: "名前ごとに、既に同じ名前があれば取り除き、新しい項目を、並びの末尾に足す(後ろの層ほど、並びの後ろ)。起動のときは、並びの順に環境へ足す(Windows は OS が大文字小文字を区別せず、後ろに足した名前が前の名前を置き換える)。**受け入れる OS の違い**: 層をまたぐ大文字小文字だけが違う名前(例: `[pipeline.env]` の `foo` と `[job.env]` の `FOO`)は、Linux と macOS では子に両方が見え、Windows では後ろの `FOO` だけが見える。完全に同じ名前の優先順位は、どの OS でも同じ(後ろの層が勝つ)。この違いは、質問 Q7 の A で受け入れた(NFR3 の例外。README に書く。層をまたぐ場合のテストで確かめる)"
    violation: なし
    source: FR1.3、NFR3、質問 Q7

  - id: BR4.4
    statement: 引き継いだ環境は、書き換えず、足すだけ。値は、`env` と `.env` で、足すか、上書きする。取り除くことはしない
    category: policy
    applies_to: [EffectiveEnv, RuntimeRunner]
    trigger: ジョブの起動
    logic: 子の環境 = 引き継いだ環境 + (統合の結果を、並びの順に、足す)。`unset` はない(要件の前提)
    violation: なし
    source: FR1.3、質問 Q8

  # --- BR5: 自動の変数 ---
  - id: BR5.1
    statement: "ジョブに、`CONDUCTOR_PIPELINE`(パイプライン名)、`CONDUCTOR_JOB_ID`(ジョブ ID)、`CONDUCTOR_RUN_ID`(run ID)、`CONDUCTOR_ATTEMPT`(試行)の 4 つを、自動で渡す"
    category: policy
    applies_to: [EffectiveEnv]
    trigger: ジョブの起動
    logic: 4 つを、層の最後(最も強い)に足す。利用者は、これらの名前を書けない(BR1.3)
    violation: なし
    source: FR1.10

  - id: BR5.2
    statement: "`CONDUCTOR_ATTEMPT` は 1 から始まり、リトライのたびに 1 増える。試行ごとに作り直す"
    category: calculation
    applies_to: [EnvContext]
    trigger: ジョブの試行(リトライを含む)
    logic: attempt = (これまでのリトライの回数) + 1。起動のたびに、環境を作り直す。値は十進の整数の文字列
    violation: なし
    source: FR1.10

  - id: BR5.3
    statement: "run ID が無いときは、`CONDUCTOR_RUN_ID` を渡さない。引き継いだ環境に `CONDUCTOR_RUN_ID` があっても、そのまま(取り除かない)"
    category: policy
    applies_to: [EffectiveEnv]
    trigger: ジョブの起動
    logic: IF run ID が無い THEN `CONDUCTOR_RUN_ID` を結果に足さない。ほかの 3 つ(`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_ATTEMPT`)は、常に足し、引き継いだ値を上書きする
    violation: なし
    source: FR1.10、質問 Q8

  # --- BR6: 実行の前の読み取り ---
  - id: BR6.1
    statement: "`pipeline` は、最初のジョブを起動する前に、`[pipeline].env_file` と、定義にあるすべてのジョブの `env_file` を、すべて読む(実行済みのジョブと、遷移でスキップされるジョブの分も)。`run` は、`[pipeline].env_file` と、そのジョブの `env_file` だけを読む"
    category: policy
    applies_to: [RuntimeEnvLoader, CliFlows]
    trigger: "`pipeline`、`run` の準備(`prepare`)の中。位置は、定義と `command` の検査(`check_definition`)、存在しないジョブの検査(`run` のみ)、run ID の決定のあと、ステートの読み込みの前"
    logic: 読む対象のパスの並びを、定義から作り、解決して、重複を除いて(BR6.4)、すべて読む。位置が後ろなので、使い方の誤り(存在しないジョブ、run ID の誤り。終了コード 68)と `.env` の誤り(64)が同時にあるときは、先に検出する 68 が勝つ。`--dry-run`(U3)も、同じ範囲を、同じ読み取りで読み、同じ終了コード 64 で止まる
    violation: BR6.2
    source: FR1.8、質問 Q2

  - id: BR6.2
    statement: "`.env` の誤りが 1 つでもあるときは、ジョブを 1 つも実行せずに、終了コード 64 で止まる。ステートの作成も更新もしない"
    category: constraint
    applies_to: [CliFlows]
    trigger: BR6.1 の読み取りの結果
    logic: IF 誤りがある THEN 誤りを表示(BR2.8)して、64 を返す。ステートの読み込みの前に止めるため、ステートに触れない
    violation: 終了コード 64
    source: FR1.8、質問 Q1

  - id: BR6.3
    statement: 再開のときも、`.env` を、再び読む
    category: policy
    applies_to: [CliFlows]
    trigger: 再開(ステートが残っている呼び出し)
    logic: ステートに、`.env` の内容も、環境変数も、記録しない。呼び出しのたびに、現在のファイルを読む
    violation: なし
    source: FR1.8

  - id: BR6.4
    statement: 解決した結果が同じパスは、1 回の呼び出しで 1 回だけ読む
    category: policy
    applies_to: [RuntimeEnvLoader]
    trigger: BR6.1 の読み取り
    logic: 解決済みのパスをキーに、読み取りの結果を使い回す。同じファイルの誤りを、重複して報告しない
    violation: なし
    source: FR1.8

  # --- BR7: validate ---
  - id: BR7.1
    statement: "`validate` は、定義の検査と `command` の検査に加えて、すべての `env_file`(BR6.1 の `pipeline` と同じ範囲)を読む"
    category: policy
    applies_to: [CliFlows]
    trigger: "`validate`"
    logic: 定義と command の検査が通ったあと、`.env` をすべて読む。誤りは、実行のときと同じ(BR6.2 と同じ表示と終了コード 64)
    violation: 終了コード 64
    source: FR1.7、質問 Q3

  - id: BR7.2
    statement: 重複キーの警告は、`validate` が、標準エラーに出す。誤りではないので、警告だけのときは、終了コード 0
    category: policy
    applies_to: [CliFlows, CliReport]
    trigger: "`validate`"
    logic: 各ファイルの警告(ファイル、行、名前)を、読み取った順に表示する。`pipeline` と `run` は、警告を表示しない
    violation: なし
    source: FR1.7

  # --- BR8: 値の秘匿 ---
  - id: BR8.1
    statement: 環境変数の値を、`-v`、`--dry-run`、`graph`、ステート、エラーの表示、Debug の出力に出さない。出してよいのは、名前だけ
    category: constraint
    applies_to: [EnvVar, EnvEntry, EffectiveEntry]
    trigger: 表示、記録、ログ、テストの失敗の出力
    logic: 値を持つ型の Debug の出力は、値を伏せ字にする。表示の部品には、値を渡さない(名前だけを渡す)
    violation: なし(守られていないことは、テストで検出する)
    source: FR1.12、NFR2

  - id: BR8.2
    statement: 名前だけの要約(契約 C1)を提供する。名前と出どころの並びで、値を含まない。引き継いだ環境の名前は含めない
    category: policy
    applies_to: [EffectiveEnv]
    trigger: "`--dry-run`(U3)の計画"
    logic: names = [(名前, 出どころ)] を、結果の並びの順に返す
    violation: なし
    source: FR1.12、契約 C1

  - id: BR8.3
    statement: ステートに、環境変数(名前も値も)を書かない
    category: constraint
    applies_to: [PipelineState]
    trigger: ステートの書き込み
    logic: "`PipelineState` の型を変えない(環境変数の属性を足さない)"
    violation: なし
    source: FR1.12、project.md の NEVER

  # --- BR9: メッセージと文書 ---
  - id: BR9.1
    statement: 新しいメッセージ(定義の新しい誤り、`.env` の誤り、警告)は、日本語と英語の両方で用意する
    category: policy
    applies_to: [CliReport]
    trigger: メッセージの表示
    logic: 既存のメッセージの一覧(すべての種類を網羅するテスト)に、新しい種類を足す
    violation: なし
    source: FR5.1

  - id: BR9.2
    statement: README に、新しいキーと動作を、版(v0.9.0 以降)つきで書く。デモに、環境変数の場面を足す
    category: policy
    applies_to: [CliReport]
    trigger: ユニットの完了
    logic: README の必須語句の検査(`docs-check`)に合わせて書き、デモのテストを足す
    violation: なし
    source: FR5.2
```

## ルールの要約(`rules` の YAML から作った表)

| 規則 | 内容 | 種類 | 出典 |
|------|------|------|------|
| BR1.1 | `env` の値は文字列だけ(展開しない) | validation | FR1.1 |
| BR1.2 | `env` の名前の規則 | validation | FR1.2 |
| BR1.3 | `CONDUCTOR_` の予約名を拒否(大文字小文字を区別しない) | constraint | FR1.11 |
| BR1.4 | 同じ表の中で、大文字小文字だけが違う名前を拒否 | constraint | NFR3、Q7 |
| BR1.5 | NUL を含む値を拒否 | validation | FR1.9 |
| BR1.6 | `env_file` は、文字列または文字列の配列 | validation | FR1.4 |
| BR1.7 | `env_file` のパスの基準と、絶対パスと `..` の許可 | policy | FR1.5、Q6 |
| BR1.8 | `format_version` は 1 のまま。新しいキーの許可 | constraint | FR1.13 |
| BR2.1 | `.env` の行の種類(`export` を含む) | validation | FR1.6 |
| BR2.2 | 引用符なしの値 | validation | FR1.6 |
| BR2.3 | 単一引用符の値 | validation | FR1.6 |
| BR2.4 | 二重引用符の値とエスケープ | validation | FR1.6 |
| BR2.5 | 展開と複数行を受け付けない | constraint | FR1.6 |
| BR2.6 | BOM と CRLF | validation | FR1.6、Q5 |
| BR2.7 | 重複キーは後ろが勝つ。警告 | policy | FR1.7 |
| BR2.8 | 誤りの報告(行番号、20 件で打ち切り) | policy | FR1.6、Q4 |
| BR3.1 | 大きさの上限 1 MiB | constraint | FR1.9 |
| BR3.2 | 項目の上限 10,000 | constraint | FR1.9 |
| BR3.3 | UTF-8 のみ | validation | FR1.6 |
| BR3.4 | 無い、読めないは誤り | validation | FR1.8、Q1 |
| BR4.1 | 層の優先順位 | calculation | FR1.3 |
| BR4.2 | `env_file` の配列は、後ろが勝つ | calculation | FR1.4 |
| BR4.3 | 統合の規則(完全一致で後ろが勝つ) | calculation | FR1.3、NFR3、Q7 |
| BR4.4 | 引き継いだ環境は、足すだけ | policy | FR1.3、Q8 |
| BR5.1 | 自動の 4 つの変数 | policy | FR1.10 |
| BR5.2 | `CONDUCTOR_ATTEMPT` は試行ごと | calculation | FR1.10 |
| BR5.3 | run ID が無いとき渡さない(引き継ぎはそのまま) | policy | FR1.10、Q8 |
| BR6.1 | 実行の前に読む範囲 | policy | FR1.8、Q2 |
| BR6.2 | 誤りでは何も実行せず 64 | constraint | FR1.8、Q1 |
| BR6.3 | 再開でも読み直す | policy | FR1.8 |
| BR6.4 | 同じパスは 1 回だけ読む | policy | FR1.8 |
| BR7.1 | `validate` が `.env` を読む | policy | FR1.7、Q3 |
| BR7.2 | 重複の警告は `validate` が出す | policy | FR1.7 |
| BR8.1 | 値を出さない | constraint | FR1.12、NFR2 |
| BR8.2 | 名前だけの要約 | policy | FR1.12、C1 |
| BR8.3 | ステートに書かない | constraint | FR1.12 |
| BR9.1 | メッセージは日本語と英語 | policy | FR5.1 |
| BR9.2 | README とデモ | policy | FR5.2 |
