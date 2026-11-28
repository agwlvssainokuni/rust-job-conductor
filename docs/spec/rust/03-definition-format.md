# 03. 定義ファイルの書式と検証(rust 版 conductor v0.10.4)

この文書は、`conductor` の定義ファイル(`conductor.toml`)の書式と、読み込み時の検証を定める。実装は `crates/core/src/definition/*`(`conductor-core`)で、根拠は実装とそのテスト(`crates/core/tests/definition_*.rs`)である。この文書だけで、同じ受理・拒否・エラーを返す読み込み処理を実装できることを目標とする。

- 定義ファイルの形式は **TOML だけ**である(YAML の定義は読めない。YAML は `convert` で TOML にしてから使う)。
- 検証は、見つけた誤りを**まとめて**返す(ただし、早い段階の誤りは 1 件だけで打ち切る。後述)。
- 定義の内容は、実行も評価もしない。`command` の文字列は、そのまま保持する。
- 確認できなかった点は「未確認」と書く。
- v0.9.0 以降、定義は `env`(環境変数の表)と `env_file`(`.env` ファイルの指定)を持てる(6.6)。定義の読み込み(`load_definition`)は、`env_file` の**パスの書式だけ**を検査し、指す `.env` ファイルは読まない。`.env` の読み取りと検査は、実行の前(`run`・`pipeline`・`--dry-run`)と `validate`・`graph` が別に行い、誤りは `DefinitionError` ではなく `EnvFileError`(6.7、14 種)で返す。終了コードはどちらも 64。
- v0.10.0 以降、`[pipeline]` は `retry`(全ジョブの既定のリトライ `[pipeline.retry]`。6.8)、`log_dir`(ジョブの出力を残すディレクトリ。6.9)、`unset_env`(引き継いだ環境から消す名前。6.10)を、`[[job]]` は `unset_env` と `retry = false`(既定を打ち消す。6.8)を持てる。`env_file` の項目は、文字列のほかに表 `{ path, required }` でも書ける(`required = false` なら、無いファイルを読み飛ばす。6.6、6.7)。
- `format_version` は 1 のままである。新しいキー(v0.9.0 の `env`、`env_file`、v0.10.0 の `retry`(`[pipeline]`)、`log_dir`、`unset_env`)を使わない定義は、変更なしで読める。v0.8.x 以前の `conductor` は v0.9.0 のキーを、v0.9.x の `conductor` は v0.10.0 のキーを、`unknownKey` として拒否する。

## 1. 処理の段階と打ち切り

`load_definition(path, expected_pipeline_name)` は次の順に処理する。前の段階で失敗したら、そこで打ち切り、**その 1 件だけ**を返す(後ろの段階は実行しない)。最後の Walk だけが、複数の誤りを返す。

| 段階 | 失敗の種類(`kind` の識別子) | 返る誤りの数 |
|------|------------------------------|--------------|
| 1. Intake(場所の決定、ファイルの読み取り) | `fileMissing` / `unreadable` / `tooLarge` | 1 |
| 2. Parse(TOML の文法) | `syntax` | 1 |
| 3. Depth(入れ子の深さ) | `syntax` | 1 |
| 4. VersionCheck(`format_version`) | `versionMissing` / `versionUnsupported` | 1 |
| 5. Walk(構造、型、遷移、終了コード、名前) | 下の種類一覧のうち該当するもの | 1 件以上(位置の順) |

`parse_definition(text, file_label, expected_pipeline_name)` は、2〜5 を文字列に対して行う(ファイルに触れない)。`file_label` は、エラーの表示に使うファイル名である。

- `expected_pipeline_name`(`pipeline` コマンドで渡された名前)が `None` のときは、名前の照合をしない。
- 検証に成功したときだけ、`PipelineDefinition`(第 11 節)を返す。誤りが 1 件でもあれば、定義は返さない。
- 版の確認(段階 4)に通らないと、構造の検査(段階 5)は一切行わない。たとえば `format_version` が無い定義に、未知のキーや型の誤りがあっても、返るのは `versionMissing` の 1 件だけである。
- 文法の誤り(段階 2)は版の確認より先である。`format_version = = 1` は `syntax` である。

## 2. 取り込み(Intake)

### 2.1 場所

- 定義ファイルのパスの指定があれば、それを使う。なければ、実行ディレクトリ(カレントディレクトリ)の `conductor.toml`。
- 表示用のファイル名は、パスを文字列にしたもの(与えられた表記のまま)である。

### 2.2 読み取りの手順と失敗

次の順に調べる。最初に当てはまったもので失敗する。

| 順 | 条件 | `kind` | 理由(日本語の原文) |
|----|------|--------|----------------------|
| 1 | パスのメタデータ(`std::fs::metadata`)の取得で「存在しない」 | `fileMissing` | `指定の場所にファイルがありません` |
| 2 | 同、権限がない | `unreadable` | `読み取りの権限がありません` |
| 3 | 同、その他の失敗 | `unreadable` | `読み取りに失敗しました` |
| 4 | パスのメタデータで、通常のファイルでない(ディレクトリ、名前付きパイプなど。**開かない**。大きさはここでは見ない) | `unreadable` | `通常のファイルではありません` |
| 5 | ファイルを開くとき「存在しない」(順 1 のあとに消えた場合) | `fileMissing` | 順 1 と同じ |
| 6 | ファイルを開けない(権限) | `unreadable` | `読み取りの権限がありません` |
| 6' | ファイルを開けない(その他) | `unreadable` | `読み取りに失敗しました` |
| 7 | 開いたハンドルのメタデータを取得できない | `unreadable` | 権限なら `読み取りの権限がありません`、他は `読み取りに失敗しました` |
| 8 | 開いたものが通常のファイルでない(順 4 のあとに差し替わった場合) | `unreadable` | `通常のファイルではありません` |
| 9 | 開いたものの大きさが 1 MiB(1,048,576 バイト)を超える | `tooLarge` | `ファイルの大きさが上限(1 MiB)を超えています` |
| 10 | 読み取りの途中の失敗 | `unreadable` | 権限なら `読み取りの権限がありません`、他は `読み取りに失敗しました` |
| 11 | 読み取り結果が上限を超える(確認のあとに大きくなった場合) | `tooLarge` | 順 9 と同じ |
| 12 | UTF-8 として解釈できない | `unreadable` | `文字コードが UTF-8 ではありません` |

- 上限は **1 MiB = 1,048,576 バイトちょうどまで受理**し、1 バイトでも超えると `tooLarge`。読み取りは「上限 + 1 バイト」までしか行わない。
- 空のファイル(0 バイト)は、空の文字列として読めて、次の段階へ進む(結果は、`format_version` が無いので `versionMissing`)。
- 順 9 の大きさの確認は UTF-8 の確認より先なので、1 MiB を超えて UTF-8 でないファイルは `tooLarge` である。
- 読む権限のない(mode 000 の)ファイルは、大きさにかかわらず `unreadable` である(開けるかの確認が、大きさの確認より先)。v0.9.5 までは、パスのメタデータで大きさを先に見たため、1 MiB を超える読む権限のないファイルは `tooLarge` だった。
- 通常のファイルでないもの(名前付きパイプを含む)は、開かずに `unreadable` である。書き手のいない名前付きパイプでも、待ち続けない(種別を見てから開くまでのあいだに、名前付きパイプに差し替えられる競合を除く)。
- いずれも、行番号は付かない(`line` は `None`)。
- シンボリックリンクは、リンク先で判定する(パスのメタデータの取得も、開く操作も、リンクを辿る)。

## 3. 受理する文法(Parse)

TOML の読み取りは、外部クレート `toml`(バージョン 1.2.1、機能は `parse` と `std`)の `DeTable::parse` に任せる。自前の字句解析・構文解析はない。したがって、受理する文法は、そのクレートの TOML の部分集合ではなく、**そのクレートが受理する TOML そのもの**である。実装時は、TOML の標準的な構文を満たす読み取り器を使えばよい。確認できた振る舞いは次のとおり。

- TOML 1.0 の構文(ベアキー・引用キー・ドットキー、基本文字列・リテラル文字列・複数行文字列、整数(10・16・8・2 進、`_` 区切り)、浮動小数、真偽値、日時、配列、テーブル、インラインテーブル、テーブルの配列、コメント)を受理する。
- TOML 1.1 で加わった構文も受理することを、実行して確認した: インラインテーブルの中の改行と末尾のカンマ(`{x = 1,\n y = 2}`)、エスケープ `\e` と `\xHH`、秒を省略した時刻(`1979-05-27T07:32Z`)。(定義の検証では、これらを使う定義の例は、テストにない。確認は変換器側の実行による。)
- UTF-8 の BOM(先頭の `U+FEFF`)は受理される(BOM つきの定義を `validate` で検証して確認した)。
- 改行は LF と CRLF のどちらも受理する。
- 次は文法の誤り(`syntax`)になる: キーの重複(`a = 1` と `a = 2`)、同じテーブルの見出しの重複(`[a]` を 2 回)、文法に合わない記述。
- `i64` に収まらない整数リテラルは、読み取りの段階では拒否されない(`format_version = 99999999999999999999` は `syntax` ではなく `versionUnsupported` になることを確認した。値を使う段階で、範囲外として拒否される)。
- 日時の値は、読み取りは通る。定義のどのキーも日時を受け付けないので、置いた場所に応じて、型の誤りや未知のキーになる。

文法の誤りの表示は固定で、読み取り器のエラー文は使わない。

| 項目 | 値 |
|------|----|
| `kind` | `syntax` |
| 理由 | `TOML の文法に誤りがあります` |
| 行番号 | 読み取り器が誤りの範囲(バイト位置)を返したときは、その開始位置の行。返さなかったときは行番号なし |

行番号の例: `"a = 1\nb = = 2\n"` は 2 行目。`"format_version = 1\n[pipeline\n"` は 2 行目。`a = 1` を 2 回書いた場合は、2 回目の行。`[a]` を 2 回書くと、2 回目の `[a]` の行(例: 5 行目)。

誤りの文には、入力の値(キー名や値)を含めない。

## 4. 入れ子の深さの制限(Depth)

Parse に成功した木を走査し、入れ子の深さを数える。

- テーブル(見出し・ドットキー・インラインテーブルによるもの)、配列、テーブルの配列は、それぞれ**1 段**と数える。根の文書そのものは数えない。
- 根の直下の値の深さを 1 として、**32 段まで受理**し、**33 段目を拒否**する。
- 拒否の誤り: `kind` は `syntax`、理由は `入れ子が深すぎます(上限は 32 段)`、行は、33 段目になった要素の先頭の位置の行。走査は、上限を超えた最初の要素で打ち切る(走査は、テーブルでは各エントリを、そのテーブルが現れた順に、深さ優先で調べる)。
- 数え方の例:
  - `a = [[[...]]]`(`[` を n 個)は n 段。32 個は受理、33 個は拒否。
  - `a = {b = {b = ... 1}}`(`{` を n 個)は n 段。32 個は受理、33 個は拒否。
  - `[a.a.a....a]`(`a` が n 個の見出し)は n 段。32 個は受理、33 個は拒否。
- 数千〜数十万段の入れ子でも、異常終了せず `syntax` の 1 件で返る(配列・インラインテーブル・見出しの 1000 段、100,000 段で確認)。ただし、この段階の拒否になるか、読み取り器の文法の誤り(Parse)になるかは、読み取り器の再帰の制限によるため、どちらの理由文になるかは**未確認**(種類はどちらも `syntax`)。
- 深さが 32 以内でも、根に未知のキーがあれば、後続の段階で別の誤りになる(例: `a = [[...32 段...]]` だけの定義は、版の確認で `versionMissing` になる)。

## 5. 書式の版(VersionCheck)

根の `format_version` を確認する。

| 状況 | `kind` | 行 | 理由 |
|------|--------|----|------|
| キーが無い | `versionMissing` | なし | `` `format_version` が必要です(対応する版は 1 です) `` |
| 整数でない(文字列、浮動小数、真偽値など) | `versionUnsupported` | 値の行 | `` `format_version` は整数で書きます(対応する版は 1 です) `` |
| 整数だが 1 でない(0、-1、2、9、`i64` に収まらない値) | `versionUnsupported` | 値の行 | `この版には対応していません(対応する版は 1 です)` |

- 対応する版は **1 だけ**。`format_version = 1` を受理する。16 進などの書き方(`0x1`)でも、値が 1 なら受理する(確認済み)。
- 版が 1 のとき、`PipelineDefinition.format_version` は 1 になる。

## 6. 定義の構造(Walk)

根のテーブルは、次のキーだけを持てる。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `format_version` | 整数 | 必須 | 第 5 節 |
| `pipeline` | テーブル | 必須 | 第 6.1 節 |
| `job` | テーブルの配列 | 必須(1 つ以上) | 第 6.2 節 |

許されていないキー(上の 3 つ以外)は、`unknownKey` である。以前の版の `[[jobs]]` も未知のキーとして拒否される(`キー `jobs` は定義にありません`)。

### 6.1 `[pipeline]`

| キー | 型 | 必須 | 既定値 | 検査 |
|------|----|------|--------|------|
| `name` | 文字列 | 必須 | なし | 空白だけ(`trim` して空)なら `emptyField` |
| `description` | 文字列 | 任意 | なし(`None`) | 型だけ |
| `run_id_env` | 文字列 | 任意 | なし(`None`。実行時の既定は `CONDUCTOR_RUN_ID`) | 名前の形(下記) |
| `remove_on_complete` | 真偽値 | 任意 | `false` | 型だけ |
| `env`(v0.9.0 以降) | テーブル(値は文字列) | 任意 | 空 | 第 6.6 節 |
| `env_file`(v0.9.0 以降。表は v0.10.0 以降) | 文字列か表 `{ path, required }`、またはその配列 | 任意 | 空 | 第 6.6 節 |
| `retry`(v0.10.0 以降) | テーブル(`[pipeline.retry]`) | 任意 | なし(`None`) | 第 6.8 節 |
| `log_dir`(v0.10.0 以降) | 文字列 | 任意 | なし(`None`。出力を残さない) | 第 6.9 節 |
| `unset_env`(v0.10.0 以降) | 文字列の配列 | 任意 | 空 | 第 6.10 節 |

- 許されたキーは、上の 9 つ(`PIPELINE_KEYS`)。それ以外は `unknownKey`(例: `owner`、`remove_on_complet`)。
- 型の誤り(すべて `typeMismatch`、行は値の行。`name` が無いときは `[pipeline]` のテーブルの先頭の行)の理由の原文:
  - `name` が無い: `` 必須の値 `name` がありません ``。`name` が文字列でない: `` `name` は文字列で書きます ``。
  - `description` が文字列でない: `` `description` は文字列で書きます ``。
  - `run_id_env` が文字列でない: `` `run_id_env` は文字列で書きます ``。
  - `remove_on_complete` が真偽値でない: `` `remove_on_complete` は真偽値(`true` または `false`)で書きます ``。
- `name` が空白だけ: `emptyField`、`name` の値の行、理由 `` `name` が空です ``(名前の照合はしない)。
- `run_id_env` の名前の形の誤り: `invalidEnvName`、値の行、理由 `` `run_id_env` は、英字または `_` で始まり、英数字と `_` だけの名前で書きます ``。
- `run_id_env` の名前の形: 1 文字以上で、先頭が ASCII の英字(`A-Za-z`)または `_`、2 文字目以降が ASCII の英数字(`A-Za-z0-9`)または `_`。受理: `A`、`_`、`_a1`、`GITHUB_RUN_ID`。拒否(`invalidEnvName`): `""`、`1A`、`A-B`、`A B`、`A=B`、`é`、`A.B`、NUL を含む文字列。文字列でない値(整数など)は `typeMismatch`。誤りの文に、値は含めない。
- `remove_on_complete` は真偽値だけ(`true` / `false`)。文字列 `"true"`、整数 `1`、配列 `[true]` は `typeMismatch`。型が違うときの値は、`false` として扱って続行する(誤りは報告済み)。
- `pipeline` の形:
  - 無い: `typeMismatch`、行は **1 行目**(位置 0)、理由 `` 必須の値 `pipeline` がありません ``。
  - テーブルの配列(`[[pipeline]]`): `typeMismatch`、`pipeline` の行、理由 `` `pipeline` は 1 つだけで、テーブルで書きます(1 ファイルに 1 パイプライン) ``。
  - その他(整数など): `typeMismatch`、理由 `` `pipeline` はテーブルで書きます ``。
  - 上の誤りのとき、`[pipeline]` の中身の検査は行わない。
- 名前の照合: `expected_pipeline_name` が与えられ、定義の `name` と**完全に同じでない**(大文字小文字・空白も区別)と、`pipelineNameMismatch`、行は `name` の値の行、理由 `指定されたパイプライン名が、定義の `name` と一致しません`。理由に両方の名前は含めない。`name` が空白だけのときは、`emptyField` だけを返し、照合はしない。`name` が文字列でない・無いときも、照合はしない。

### 6.2 `[[job]]`

`job` は、テーブルの配列でなければならない。

- 無い: `noJobs`、行は 1 行目(位置 0)、理由 `` ジョブ(`job`)が 1 つもありません ``。
- 空の配列(`job = []`): `noJobs`、行は `job` の値の行、理由は同じ。
- 配列でない(テーブルや整数など): `typeMismatch`、理由 `` `job` はテーブルの配列で書きます ``。
- 配列の要素がテーブルでない: `typeMismatch`、理由 `` `job` の要素はテーブルで書きます ``(その要素の検査は行わない)。

各ジョブ(テーブル)のキー:

| キー | 型 | 必須 | 既定値 | 説明 |
|------|----|------|--------|------|
| `id` | 文字列 | 必須 | なし | ジョブID。第 8 節の検査 |
| `command` | 文字列、または文字列の配列 | 必須 | なし | 第 6.3 節 |
| `warning_exit_codes` | 整数の配列 | 任意 | 空(`[]`) | 警告にする終了コード |
| `success_exit_codes` | 整数の配列 | 任意 | 省略(`None`)。意味は `[0]` | 成功にする終了コード |
| `transitions` | テーブルの配列 | 任意 | 空 | 第 6.4 節 |
| `retry` | テーブル、または `false`(v0.10.0 以降) | 任意 | 書かなければ `[pipeline.retry]` の既定(既定も無ければ `None`) | 第 6.5 節、第 6.8 節 |
| `env`(v0.9.0 以降) | テーブル(値は文字列) | 任意 | 空 | 第 6.6 節 |
| `env_file`(v0.9.0 以降。表は v0.10.0 以降) | 文字列か表 `{ path, required }`、またはその配列 | 任意 | 空 | 第 6.6 節 |
| `unset_env`(v0.10.0 以降) | 文字列の配列 | 任意 | 空 | 第 6.10 節 |

許されたキーは、上の 9 つ(`JOB_KEYS`)。それ以外は `unknownKey`(例: `warn_exit_codes`。`log_dir` は `[pipeline]` だけのキーで、ジョブに書くと `unknownKey`)。

- `id` が無い: `typeMismatch`、行はジョブのテーブルの先頭、理由 `` 必須の値 `id` がありません ``。文字列でない: `typeMismatch`、値の行、理由 `` `id` は文字列で書きます ``。
- 終了コードの配列(`warning_exit_codes` と `success_exit_codes`):
  - 配列でない: `typeMismatch`、値の行、理由 `` `<キー名>` は整数のリストで書きます ``(`<キー名>` は実際のキー名)。
  - 要素が整数でない、または `i32` の範囲(-2,147,483,648〜2,147,483,647)に収まらない: 要素ごとに `typeMismatch`、その要素の行、理由 `` `<キー名>` の要素は、整数(32 ビットの範囲)で書きます ``。1 つでも誤りがあれば、そのキー全体を「無効」として扱い、後続の重なりの検査では使わない。
  - 負の値(例: `-1`)は、`i32` の範囲なので**受理**する。重複した値も受理し、除かない(書いた順のまま保持する)。空の配列も受理する(`success_exit_codes = []` は、成功にする終了コードが無い、の意味になる)。
  - 整数は 16 進・8 進・2 進のリテラルでもよい(値で判定する)。
- `success_exit_codes` を省略すると、成功の終了コードは `[0]`(`effective_success_codes`)。書いたら、**書いた値だけ**が成功(`0` を書かなければ 0 でも成功にならない)。
- 要素が浮動小数(`[1.0]`)や、`i64` に収まらない整数(`[99999999999999999999]`)のときも、要素の型の誤り(`typeMismatch`)になる(確認済み)。

### 6.3 `command`

| 書き方 | 受理の条件 | 保持される値 |
|--------|-----------|---------------|
| 文字列 | `trim` して空でない | `Command::Line(文字列)`(そのまま) |
| 文字列の配列 | 空でない配列で、全要素が文字列で、**先頭の要素が `trim` して空でない**。2 番目以降の要素は空文字列でもよい | `Command::Argv(要素の並び)` |

誤り:

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| キーが無い | `typeMismatch` | ジョブの先頭 | `` 必須の値 `command` がありません `` |
| 文字列が空または空白だけ | `emptyField` | 値の行 | `` `command` が空です `` |
| 配列が空 | `emptyField` | 値の行 | `` `command` の配列が空です `` |
| 配列に文字列でない要素がある | `typeMismatch`(要素ごと) | その要素の行 | `` `command` の配列の要素は、文字列で書きます `` |
| 配列の先頭の要素が空または空白だけ(全要素が文字列のとき) | `emptyField` | 先頭の要素の行 | `` `command` の配列の先頭の要素が空です `` |
| 文字列でも配列でもない(整数・真偽値・テーブル) | `typeMismatch` | 値の行 | `` `command` は文字列、または文字列の配列で書きます `` |

- 配列に型の誤りの要素があるときは、先頭要素の空の検査はしない。
- 受理の例: `command = "echo x"`、`command = ["my prog", "a b", "$HOME", ""]`、`["echo", ""]`。拒否の例: `5`、`true`、`{ a = 1 }`、`["a", 5]`、`[["a"]]`(typeMismatch)、`""`、`"   "`、`[]`、`[""]`、`["  ", "x"]`(emptyField)。
- 文字列の `command` の中の引用符・`$` などは、この段階では解釈しない(実行時の解釈は実行側の仕様)。
- 誤りの文に、入力の値は含めない。

### 6.4 `[[job.transitions]]`

`transitions` は、テーブルの配列(`[[job.transitions]]` の繰り返し、または `transitions = [{ on = "...", next = "..." }, ...]`)。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `on` | 文字列 | 必須 | `success` または `warning` |
| `next` | 文字列 | 必須 | 遷移先のジョブID、または `@end` |

- 許されたキー以外(`when` など)は `unknownKey`。
- `transitions` が配列でない: `typeMismatch`、理由 `` `transitions` はテーブルの配列で書きます ``(遷移は空として扱う)。要素がテーブルでない: `typeMismatch`、理由 `` `transitions` の要素はテーブルで書きます ``。
- `on` が無い: `typeMismatch`、遷移の先頭の行、理由 `` 必須の値 `on` がありません ``。文字列でない: `typeMismatch`、`` `on` は文字列で書きます ``。
- `on` の値: `"success"`、`"warning"` を受理。`"error"` は `errorTransition`、理由 `エラー結果(`error`)には遷移先を書けません`。それ以外の文字列(`"later"`、大文字の `"Success"` など)は `typeMismatch`、理由 `` `on` は `success` または `warning` で書きます ``。
- `next` が無い: `typeMismatch`、遷移の先頭の行、理由 `` 必須の値 `next` がありません ``。文字列でない: `typeMismatch`、`` `next` は文字列で書きます ``。
- 同じ `on`(`success` どうし、`warning` どうし)の遷移が 2 つ以上: 2 つ目以降の遷移に `duplicateTransition`、行は**その遷移(テーブルの先頭)の行**、理由 `同じ結果(`on`)の遷移が 2 つ以上あります`。`success` と `warning` を 1 つずつは受理。`on` が不正(`error` や未知の値)の遷移は、重複判定に入れない。
- 遷移先の存在・順序の検査は、第 8 節。

### 6.5 `[job.retry]`(v0.8.0 以降)

ジョブが決めた終了コードで終わったときに、待って実行し直す設定。書き方は、`[job.retry]` の見出しでも、`retry = { ... }` のインラインテーブルでもよい。**`[job.retry]` の位置は、ジョブのテーブルの中なら、`[[job.transitions]]` の前でも後でもよい**(TOML の意味どおり、直前の `[[job]]` に属する。確認済み)。

許されたキーは次の 5 つだけ。それ以外は `unknownKey`(例: `retries`)。

| キー | 型 | 必須 | 範囲・書式 | 取り込み後 |
|------|----|------|-----------|-------------|
| `exit_codes` | 整数の配列 | 必須 | 空でない。各要素は 1 以上 255 以下の整数 | 昇順に並べ、重複をまとめる(`[75, 3, 75, 3, 12]` → `[3, 12, 75]`) |
| `max` | 整数 | 必須 | 1 以上 100 以下 | `u32` |
| `interval` | 文字列 | 必須 | `<整数><s|m|h>`、1 秒以上 24 時間以下 | 秒(`u64`) |
| `multiplier` | 整数 | 任意 | 2 以上 10 以下 | `u32`(`Option<Backoff>` の `multiplier`) |
| `max_interval` | 文字列 | 任意 | `interval` と同じ書式、`interval` 以上 | 秒(`u64`。`Option<Backoff>` の `max_interval_secs`) |

- 127 などの値も受理する(`[127, 1]` → `[1, 127]`)。
- `multiplier` と `max_interval` は、**両方を書くか、両方とも書かない**か。片方だけなら `retryBackoffIncomplete`。両方なしなら、待ち時間は毎回 `interval`。

#### 間隔の文字列の書式

- 文字列の最後の 1 文字が単位(`s`=秒、`m`=分=60 秒、`h`=時間=3600 秒。小文字だけ)。その前が、**ASCII の数字だけ**の 1 文字以上の列(符号・空白・小数点・`_`・全角数字・16 進(`0x`)は不可)。
- 数値は 1 以上。秒に換算した値が 1 以上 86,400(24 時間)以下。桁あふれ(`u64` に収まらない、または換算の乗算があふれる)は拒否。
- 受理: `1s`、`30s`、`5m`、`2h`、`24h`、`1440m`、`86400s`。拒否: `""`、`s`、`10`、`1.5s`、`-1s`、`+1s`、` 1s`、`1s `、`1 s`、`1d`、`1S`、`１s`、`1ms`、`0x10s`、`1_0s`、`0s`、`0h`、`86401s`、`25h`、`99999999999999999999999s`、`18446744073709551615h`。文字列でない値(整数・真偽値)も拒否。

#### 誤りと検査

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| `retry` がテーブルでも `false` でもない(整数、文字列、`true`、`[[job.retry]]` の配列) | `retryNotTable` | `retry` の値の行(配列のときは、その配列の先頭) | `` `retry` はテーブルか `false` で書きます ``(v0.9.x までは `` `retry` はテーブルで書きます ``) |
| 必須のキー(`exit_codes`、`max`、`interval`)が無い | `retryMissingKey`(欠けたキーごとに 1 件) | `retry` のテーブルの先頭の行 | `` 必須の値 `<キー名>` がありません `` |
| `exit_codes` が配列でない | `retryExitCodeInvalid` | 値の行 | `` `exit_codes` は整数の一覧で書きます `` |
| `exit_codes` が空の配列 | `retryExitCodesEmpty` | 値の行 | `` `exit_codes` が空です `` |
| `exit_codes` の要素が整数でない、`i32` に収まらない、1〜255 の範囲外(0、256、-1、1.5、`"75"`、`true` など) | `retryExitCodeInvalid`(要素ごと) | その要素の行 | `` `exit_codes` の要素は、1 以上 255 以下の整数で書きます `` |
| `max` が整数でない、または 1〜100 の範囲外(0、101、-1、1.5、`"3"`、`true`) | `retryMaxInvalid` | 値の行 | `` `max` は、1 以上 100 以下の整数で書きます `` |
| `interval` / `max_interval` が書式・範囲に合わない | `retryDurationInvalid` | 値の行 | `` `<キー名>` は、整数と単位(`s`、`m`、`h`)で、1 秒以上 24 時間以下に書きます(例: `30s`) ``(`<キー名>` は `interval` か `max_interval`) |
| `multiplier` が整数でない、または 2〜10 の範囲外(1、11、0、-2、`2.0`、`"2"`、`true`) | `retryMultiplierInvalid` | 値の行 | `` `multiplier` は、2 以上 10 以下の整数で書きます `` |
| `multiplier` と `max_interval` の片方だけがある | `retryBackoffIncomplete` | `retry` のテーブルの先頭の行 | `` `multiplier` と `max_interval` は、両方を書くか、両方とも書きません `` |
| `exit_codes` に、そのジョブの成功または警告の終了コードと同じ値がある | `retryExitCodesOverlap` | `exit_codes` の**キー**の行 | `` `exit_codes` に、成功または警告の終了コードと同じ値があります `` |
| `max_interval` が `interval` より小さい(秒で比べる。等しければ受理) | `retryMaxIntervalTooSmall` | `max_interval` の値の行 | `` `max_interval` は、`interval` 以上で書きます `` |

- 整数の読み取りは `i64` で行い、範囲の判定をする(`i64` に収まらない整数は、範囲外の誤りと同じ扱い)。16 進・`_` 区切りのリテラル(`max = 1_0`、`exit_codes = [0x4B, 1_0]`)も、値で判定して受理する(確認済み)。
- 空のテーブル `[job.retry]`(キーなし)は、`retryMissingKey` が 3 件(`exit_codes`、`max`、`interval` の順)返る(行はすべて同じ)。
- `exit_codes` の複数の要素が不正なら、不正な要素ごとに 1 件(例: `[0, 75, 256]` は 2 件)。
- **重なりの検査**(`retryExitCodesOverlap`)は、定義の他の値と照合する。`exit_codes` が有効(エラーなし)のときだけ行う。
  - 成功の終了コード: `success_exit_codes` を**書かなければ `[0]`**。書いて有効なら、その値。書いたが誤り(無効)のときは、成功との重なりを調べない(警告との重なりは調べる)。
  - 警告の終了コード: `warning_exit_codes` が有効なら、その値(無効なら調べない)。
  - どちらかと 1 つでも共通する値があれば、1 件(重なりの値の数にかかわらず 1 件)。
  - 例: `success_exit_codes = [0, 5]` と `exit_codes = [5, 75]` は誤り。`success_exit_codes = [3]` と `exit_codes = [3]` も誤り。`success_exit_codes = [0, 5]`、`warning_exit_codes = [4]`、`exit_codes = [75]` は受理。
- `retry` の有無は、ほかのキーの読み取りの結果に影響しない。
- (v0.10.0 以降)ジョブの `retry` は、書かない・`false`・表の 3 つ。書かなければ `[pipeline.retry]` の既定を当てはめ、`false` なら既定を打ち消してリトライしない。表を書けば、その表を丸ごと使い、既定と混ぜない(第 6.8 節)。
- 誤りの文に、入力の値は含めない(`"SENTINEL..."` のような値が文に出ないことを確認するテストがある)。

### 6.6 `env` と `env_file`(v0.9.0 以降)

`[pipeline]` と各 `[[job]]` に、同じ形の 2 つの任意のキーを書ける。意味(層の順、ジョブへの足し方)は 05 の 14 章。

```toml
[pipeline]
name = "nightly"
env_file = ".env"                 # 文字列か表 { path, required }、またはその配列
[pipeline.env]                    # 全ジョブに足す環境変数
TZ = "Asia/Tokyo"

[[job]]
id = "extract"
command = "python scripts/extract.py"
env_file = ["extract.env", "secret.env"]
[job.env]
LOG_LEVEL = "debug"
```

`env` の書き方は、`[pipeline.env]` / `[job.env]` の見出しでも、`env = { A = "1" }` のインラインテーブルでもよい。

#### `env`(テーブル)の検査

`env` がテーブルでなければ、`typeMismatch`、値の行、理由 `` `env` は表で書きます ``(`env` は空として扱う)。テーブルなら、項目を**キーの位置の順(書いた順)**に 1 つずつ、次の順に検査する。当てはまった最初の誤りを報告して、その項目の検査を終える(次の項目へ進む)。

| 順 | 条件 | `kind` | 行 | 理由(`<名前>` は `quote_key` で整形した名前) |
|----|------|--------|----|------|
| 1 | 名前が `is_valid_env_name` を満たさない(第 6.1 節の `run_id_env` と同じ規則。空、数字で始まる、`A-B`、`A B`、非 ASCII など) | `invalidEnvName` | **キー**の行 | `` `env` の名前 <名前> は、英字または `_` で始まり、英数字と `_` だけで書きます `` |
| 2 | 名前が予約名(ASCII の大文字にして `CONDUCTOR_` で始まる。`conductor_x`、`Conductor_X` も) | `reservedEnvName` | キーの行 | `` `env` の名前 <名前> は、`CONDUCTOR_` で始まるため書けません `` |
| 3 | すでに検査を通った名前のうち、ASCII の大文字小文字を無視すると同じで、綴りが違う名前がある(`foo` と `FOO`)。**完全に同じ綴りの重複は、TOML の文法の誤り(`syntax`)で、ここには来ない** | `envNameCaseConflict` | キーの行(2 つ目の名前) | `` `env` の名前 <名前> は、大文字小文字だけが違う名前が、すでにあります `` |
| 4 | 値が文字列でない(整数、配列など) | `typeMismatch` | **値**の行 | `` `env` の名前 <名前> の値は、文字列で書きます `` |
| 5 | 値の文字列に NUL(U+0000)を含む | `invalidEnvValue` | 値の行 | `` `env` の名前 <名前> の値に、NUL は書けません `` |

- 順 1〜2 を通らなかった名前は、順 3 の「すでにある名前」の集合に入れない。順 1〜3 を通った名前は、値の誤り(順 4・5)があっても、集合に入る。
- 値は、**書いたとおり**に保持する(`${X}`、`$X`、`~`、空文字列、空白、改行を含めて展開・変換をしない)。空の文字列は、有効な値である。
- 受理: `A = "1"`、`_x = ""`、`path = "$HOME:/bin"`。拒否: `"1A" = "x"`(`invalidEnvName`)、`CONDUCTOR_JOB_ID = "x"`(`reservedEnvName`)、`foo` と `FOO` の併記(`envNameCaseConflict`)、`A = 1`(`typeMismatch`)、`A = "a\u0000b"`(`invalidEnvValue`)。
- 検査を通った項目は、`Vec<EnvVar>`(`{ name, value }`)に、書いた順で入る。

#### `env_file` の検査

項目は、文字列か表 `{ path, required }`(v0.10.0 以降)。項目 1 つでも、項目の配列でも書け、配列には文字列と表を混ぜて書ける(`env_file = [".env", { path = "local.env", required = false }]`)。文字列 `"a.env"` は `{ path = "a.env", required = true }` と同じ。

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| 文字列・表・その配列のどれでもない(整数、真偽値など) | `typeMismatch` | 値の行 | `` `env_file` は、文字列かテーブル、またはその配列で書きます ``(v0.9.x までは `` `env_file` は、文字列、または文字列の配列で書きます ``) |
| 配列の要素が文字列でも表でもない | `typeMismatch`(要素ごと) | その要素の行 | `` `env_file` の配列の要素は、文字列かテーブルで書きます ``(v0.9.x までは `` `env_file` の配列の要素は、文字列で書きます ``) |
| 文字列(または配列の要素、表の `path`)が空の文字列 | `emptyField`(要素ごと) | その値(要素)の行 | `` `env_file` に空の文字列は書けません `` |
| 表に `path`・`required` 以外のキーがある(v0.10.0 以降) | `unknownKey` | キーの行 | `` キー `<キー名>` は定義にありません `` |
| 表に `path` が無い(v0.10.0 以降) | `typeMismatch` | 表の先頭の行 | `` 必須の値 `path` がありません `` |
| 表の `path` が文字列でない(v0.10.0 以降) | `typeMismatch` | 値の行 | `` `path` は文字列で書きます `` |
| 表の `required` が真偽値でない(v0.10.0 以降) | `typeMismatch` | 値の行 | `` `required` は真偽値(`true` または `false`)で書きます `` |

- 項目 1 つは、要素 1 つの並びとして保持する(`Vec<EnvFileRef>`。`EnvFileRef { path: String, required: bool }`。v0.9.x までは `Vec<String>`)。表の `required` を省くと `true`。空の配列 `env_file = []` は誤りではなく、何も読まない。空白だけの文字列は、空ではないので受理する(読み取りのときに、ファイルが無い誤りになる)。
- 表の中の検査の順は、キー検査 → `path` → `required`。配列の複数の誤りは、すべて集める(例: `[{ path = "" }, true, { path = "a", required = 1, x = 2 }]` は、位置の順に `emptyField`、`typeMismatch`、`typeMismatch`、`unknownKey`)。
- `required = false` の意味(無いファイルを読み飛ばす)は 6.7。
- 同じパスを 2 回書いても、受理する(書いた順のまま保持する。読み取りは 1 回で済む。6.7)。
- 定義の読み込みでは、パスの存在も内容も調べない。`~` と変数は展開しない。

### 6.7 `.env` ファイルの読み取りと書式(v0.9.0 以降)

`env_file` が指す `.env` ファイルの読み取りは、`conductor-runtime` の `load_env_files(definition_path, refs)` が行い、解析は `conductor-core` の純粋な関数 `parse_env_file(text, file)` が行う。読む範囲(どのコマンドが、どの `env_file` を読むか)は 05 の 14 章。

#### 読み取りの手順(`load_env_files`)

入力: 定義ファイルのパス、読む対象の `env_file` の記述(`EnvFileRef` の並び。`env_file_refs` が作る。同じパスの文字列は 1 つにまとめ、`required` は論理和にする)。出力: すべて読めて解析できたときは `LoadedEnvFiles`(記述の文字列 → 内容を引ける。同じ解決済みのパスは、1 つの内容を共有)。1 つでも誤りがあれば、**すべてのファイル分の誤り**を集めた `Vec<EnvFileError>`(記述の出現順)。

1. 基準のディレクトリ = 定義ファイルの親ディレクトリ(親が無ければ空 = 実行ディレクトリ)。**実行ディレクトリではなく、定義ファイルのあるディレクトリが基準**である。
2. 記述ごとに、パスを解決する: 絶対パスならそのまま、相対パスなら (基準)/(記述)。`..` を許す。`~` と変数は展開しない。シンボリックリンクは OS の規則どおりにたどる。
3. 解決した結果が、すでに読んだパスと同じなら、読み直さない(内容を共有し、誤りも重複して報告しない)。そのパスが必ず要るかは、そこに解決されるすべての記述の `required` の論理和で決める(v0.10.0 以降。どれか 1 つでも必ず要るなら必ず要る)。
   (v0.10.0 以降)必ず要らない(`required = false`)パスが**無い**(下の順 1 または順 4 の「存在しない」)ときは、誤りにも警告にもせず、そのパスの記述を引き当て(`LoadedEnvFiles`)に載せない。無いことの判断は「存在しない」だけで行い、あって読めない・大きすぎる・UTF-8 でない・書式の誤りは、`required` によらず誤りである。
4. 1 つのファイルを、次の順に調べる(最初に当てはまったもので失敗し、そのファイルは終わる)。表示するファイル名は、解決済みのパスの文字列。行番号は付かない。

| 順 | 条件 | `kind` |
|----|------|--------|
| 1 | パスのメタデータの取得で「存在しない」 | `fileMissing` |
| 2 | パスのメタデータの取得で、その他の失敗(権限など) | `unreadable` |
| 3 | 通常のファイルでない(ディレクトリ、名前付きパイプなど。開かない) | `unreadable` |
| 4 | 開けない(「存在しない」は `fileMissing`、読む権限がないなどその他は `unreadable`) | `fileMissing` / `unreadable` |
| 5 | 開いたもののメタデータが取れない、または通常のファイルでない | `unreadable` |
| 6 | 開いたもののメタデータの大きさが 1 MiB(1,048,576 バイト)を超える(中身は読まない) | `tooLarge` |
| 7 | 読み取りの途中の失敗 | `unreadable` |
| 8 | 読み取り(「上限 + 1 バイト」までしか読まない)の結果が上限を超える | `tooLarge` |
| 9 | UTF-8 として解釈できない | `notUtf8` |
| 10 | 解析(下記)の誤り | 下の表 |

- 上限は、**ちょうど 1 MiB まで受理**し、1 バイトでも超えると `tooLarge`。空のファイル(0 バイト)は、項目 0 個として受理する。
- 順は、定義ファイルの取り込み(2.2)と同じ(種別をパスで見る → 開く → 開いたものから大きさ → 上限 + 1 まで読む → UTF-8)。読む権限のない(mode 000 の)`.env` は、大きさにかかわらず `unreadable`。v0.9.6 までは、パスのメタデータで大きさを先に見たため、1 MiB を超える読む権限のない `.env` は `tooLarge` だった。
- 通常のファイルでないもの(名前付きパイプを含む)は、開かずに `unreadable`(書き手のいない名前付きパイプで待ち続けない)。
- 順 1〜8 は、ステートの読み込み(06 の §5.1 の 1〜4)と同じ関数 `bounded_read::read_regular_file` で行う。
- 定義の取り込み(2.2)と違い、`.env` の誤りの理由は、**言語に従って**表示される(08 の 4.12)。

#### `unset_env` との重なりの検査(v0.10.0 以降。`check_unset_conflicts(def, job_id, files)`)

すべての `.env` が読めて解析できたあとに、cli(`Ctx::load_env_files`)が、`conductor-core` の純粋な関数 `check_unset_conflicts` で、読めた `.env` の項目の名前と、`unset_env`(6.10)の名前の重なりを調べる。重なりは、`.env` の誤りと同じく表示し、終了コードは 64。

- 範囲は `env_file_refs` と同じ(`job_id` が `None` ならすべてのジョブ、`Some` ならそのジョブ)。範囲のジョブごとに、`[pipeline].env_file` とそのジョブの `env_file` で読めたファイルの項目の名前を、`[pipeline].unset_env` とそのジョブの `unset_env` の名前と比べる。
- 比べ方は、どの OS でも ASCII の大文字小文字を区別しない。
- 重なった項目ごとに、そのファイルと項目の行で `unsetConflict` を 1 つ返す。同じファイルを複数のジョブが読んでも、「ファイルと行」で 1 つにまとめる。並びは、ファイルの読み取り順、行の順。誤りに値も名前も入れない。
- 読み飛ばした(`required = false` で無い)ファイルは、調べない。

#### 書式と解析(`parse_env_file(text, file)`)

出力: `EnvFileContent { entries: Vec<EnvEntry { name, value }>, warnings: Vec<EnvWarning { file, line, name, kind: DuplicateKey }> }`、または `Vec<EnvFileError>`(行ごとの誤りは**止めずにすべて集める**。1 件でもあれば、項目は返さない)。純粋な関数で、ファイルにも環境変数にも触れない。

1. 先頭の BOM(U+FEFF)があれば、1 つだけ取り除く。
2. 文字列を LF(`\n`)で分けて行にする。各行の末尾の `\r` を 1 つ取り除く(CRLF の対応)。行番号は 1 始まり(末尾の LF のあとの空の行も 1 行と数える)。
3. 各行について:
   1. 行頭の空白(Unicode の空白)を取り除いた結果が、空、または `#` で始まるなら、読み飛ばす(空行とコメント)。
   2. `export` で始まり、その直後の 1 文字が空白(スペースまたはタブ)なら、`export` と続く空白を取り除く(取り除いたあとの先頭の空白も取り除く)。`export` のあとに空白が続かなければ、取り除かない(`export=1` は名前が `export` の項目。`exportX=1` は名前 `exportX`。`export` だけの行は、`=` が無いので `missingEquals`)。
   3. 最初の `=` で、`KEY` と残り(値の部分)に分ける。`=` が無ければ `missingEquals`。`KEY` の前後の空白を取り除く(`A = 1` は受理)。値の中の `=` は、値の文字である(`A=x=y` の値は `x=y`)。
   4. `KEY` が `is_valid_env_name` を満たさなければ(空を含む)`invalidName`。予約名(ASCII の大文字小文字を区別せず `CONDUCTOR_` で始まる)なら `reservedName`。
   5. 値を読む(下記)。誤りがあれば、その種類(`unterminatedQuote`、`invalidEscape`、`trailingGarbage`)で、その行を誤りにする。
   6. 値に NUL があれば `nulInValue`。
   7. ここまで通った行を数える。数が **10,000 を超えたら**、その行に `tooManyEntries` を報告して、**解析を止める**(それ以降の行は読まない)。数えるのは、通った行(重複を含む)で、誤りの行は数えない。
   8. 重複の扱い: すでに通った名前のうち、ASCII の大文字小文字を無視すると同じものがあるとき、綴りが**完全に同じ**なら、前の項目を取り除き、後ろの項目を並びの**末尾**に足し、警告 `DuplicateKey`(`file`、後ろの行、名前)を足す(後ろが勝つ)。綴りが**違う**なら(`foo` と `FOO`)、その行を `caseConflict` の誤りにして、項目に足さない。なければ、項目を並びの末尾に足す。
4. 誤りが 1 件もなければ、項目と警告を返す。

値の読み方(`=` の右側。`KEY` を切り出したあとの残り):

| 先頭の文字(`=` の右の空白を読み飛ばしたあと) | 値 |
|-----------------------------------------------|----|
| `'` | 次の `'` までを、そのまま値にする(エスケープなし)。`'` が無ければ `unterminatedQuote`。閉じたあとに、空白とコメント(`#` で始まる)以外が続けば `trailingGarbage` |
| `"` | 次の閉じる `"` までを読む。`\n`(LF)、`\r`(CR)、`\t`(タブ)、`\"`、`\\` の 5 つだけを解く。それ以外の `\` の組は `invalidEscape`。行末で閉じていなければ(行の末尾の `\` を含む)`unterminatedQuote`。閉じたあとの規則は、単一引用符と同じ |
| それ以外 | 行末まで。ただし、**空白に続く `#`** から先はコメントで、その手前までとする。前後の空白を取り除く。`#` が `=` の直後(空白なし)や、空白でない文字に続くとき(`A=x#y`)は、値の一部。`KEY=` は空の文字列 |

- 受理の例: `A=1`、`export A=1`、`A = 1`、`A=x=y`(値 `x=y`)、`A=x #y`(値 `x`)、`A=x#y`(値 `x#y`)、`A= "x"`(値 `x`)、`A='no $expansion'`(値 `no $expansion`)、`A="a\nb"`(値は `a`、LF、`b`)、`A=`(値は空)、`A=  # c`(値は空)。
- 拒否の例: `A`(`missingEquals`)、`=1` と `1A=x`(`invalidName`)、`CONDUCTOR_X=1`(`reservedName`)、`A='x`(`unterminatedQuote`)、`A="x\q"`(`invalidEscape`)、`A="x" y`(`trailingGarbage`)、`foo=1` と `FOO=2`(`caseConflict`)。
- `$`、`${` は展開しない(値の文字)。複数行の値は受け付けない(行をまたぐ引用符は `unterminatedQuote`)。

#### `.env` の誤りの種類(14 種。v0.10.0 で `unsetConflict` を追加)

識別子は、`EnvFileErrorKind` の変種の名前の先頭を小文字にしたもの。日本語の種類の文は `label()`。理由(`reason`)の日本語は `reason()`(英語は 08 の 4.12)。

| 識別子 | 日本語の種類の文 | 日本語の理由 | 行 |
|--------|------------------|--------------|----|
| `fileMissing` | 環境変数ファイルが存在しません | `` `env_file` に書かれたファイルが、見つかりません `` | なし |
| `unreadable` | 環境変数ファイルを読めません | `` `env_file` に書かれたファイルを、読めません `` | なし |
| `tooLarge` | 環境変数ファイルが大きすぎます | `ファイルの大きさが、上限(1 MiB)を超えています` | なし |
| `notUtf8` | 環境変数ファイルが UTF-8 ではありません | `ファイルは UTF-8 で書きます` | なし |
| `tooManyEntries` | 環境変数ファイルの項目が多すぎます | `項目の数が、上限(10,000)を超えています` | 10,001 個目の項目の行 |
| `invalidName` | 環境変数の名前が正しくありません | `` 名前は、英字または `_` で始まり、英数字と `_` だけで書きます `` | 行 |
| `reservedName` | 予約された環境変数の名前です | `` `CONDUCTOR_` で始まる名前は、書けません `` | 行 |
| `caseConflict` | 大文字小文字だけが違う環境変数の名前があります | `大文字小文字だけが違う名前が、同じファイルにあります` | 後ろの行 |
| `missingEquals` | `` `KEY=VALUE` の形ではありません `` | `` `KEY=VALUE` の形で書きます `` | 行 |
| `unterminatedQuote` | 引用符が閉じていません | `引用符が、同じ行で閉じていません` | 行 |
| `invalidEscape` | 許されないエスケープです | `` 二重引用符の中で使えるエスケープは、`\n`、`\r`、`\t`、`\"`、`\\` だけです `` | 行 |
| `trailingGarbage` | 閉じた引用符のあとに余計な文字があります | `閉じた引用符のあとは、空白とコメントだけを書けます` | 行 |
| `nulInValue` | 値に NUL が含まれています | `値に NUL(U+0000)は書けません` | 行 |
| `unsetConflict`(v0.10.0 以降) | 消す環境変数の名前と重なっています | `` `unset_env` の名前と同じ名前があります `` | 重なった項目の行 |

- 誤りに、値を含めない(理由は固定の文)。表示の書式は `<ファイル>[:<行>]: <種類の文>: <理由>`(ライブラリの `Display`)。CLI の表示は 08 の 4.12。
- 表示の件数: 先頭の最大 **20 件**を表示し、超えた分は「ほか N 件」の 1 行にする(`summarize_errors(errors) -> (&[EnvFileError], usize)`。crate の中の定数 `MAX_REPORTED_ERRORS = 20`)。
- 重複キーの警告(`EnvWarning`)は誤りではない。`validate` と `graph` が表示する(04 の 7.5、7.7)。

### 6.8 `[pipeline.retry]` と `retry = false`(v0.10.0 以降)

`[pipeline]` に `retry` の表を書くと、全ジョブの既定のリトライになる。書き方は `[pipeline.retry]` の見出しでも、`retry = { ... }` のインラインテーブルでもよい。表の中身(キー、型、範囲、誤り)は、`[job.retry]`(第 6.5 節)と同じ手続きで読む(キーは `exit_codes`・`max`・`interval`・`multiplier`・`max_interval` の 5 つ。誤りの種類も同じで、位置は既定の表の中)。

```toml
[pipeline]
name = "nightly"
[pipeline.retry]                  # 全ジョブの既定
exit_codes = [75]
max = 3
interval = "10s"

[[job]]
id = "extract"
command = "extract"               # retry を書かない: 既定を当てはめる

[[job]]
id = "notify"
command = "notify"
retry = false                     # 既定を打ち消し、リトライしない

[[job]]
id = "load"
command = "load"
[job.retry]                       # 自分の表を丸ごと使う(既定の倍率などを混ぜない)
exit_codes = [111]
max = 5
interval = "30s"
```

| ジョブの `retry` | 実効のリトライ(`JobDefinition.retry`) |
|------------------|----------------------------------------|
| 書かない | 既定の複製(既定も無ければ `None`) |
| `false` | `None`(既定が無くても誤りにならない) |
| 表 | その表(既定と混ぜない) |

- 既定は、組み立て後の定義には残さない(各ジョブの `retry` に複製して入れる)。既定を当てはめたジョブは、同じ値の表を書いたジョブと同じ定義になる。
- 誤り:

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| `[pipeline]` の `retry` が表でない(`false`、`true`、整数、文字列を含む) | `retryNotTable` | `retry` の値の行 | `` `[pipeline.retry]` はテーブルで書きます `` |
| ジョブの `retry` が表でも `false` でもない | `retryNotTable` | 値の行 | `` `retry` はテーブルか `false` で書きます ``(第 6.5 節) |
| `retry` を書かないジョブで、既定の `exit_codes` が、そのジョブの成功または警告の終了コードと同じ値を持つ | `retryExitCodesOverlap`(ジョブごとに 1 件) | そのジョブの表(`[[job]]`)の先頭の行 | `` 既定のリトライ(`[pipeline.retry]`)の `exit_codes` に、このジョブの成功または警告の終了コードと同じ値があります `` |
| 既定の `max_interval` が `interval` より小さい | `retryMaxIntervalTooSmall`(ジョブの数によらず 1 件) | 既定の `max_interval` の値の行 | `` `max_interval` は、`interval` 以上で書きます `` |

- 既定との重なりの比べ方は、第 6.5 節の重なりと同じ(成功の終了コードを書かなければ `[0]`、書いて誤りなら成功とは比べない、警告は有効なときだけ)。既定の `exit_codes` に誤りがあれば、重なりは調べない。`retry = false` のジョブと、自分の表を書いたジョブは、既定と比べない(自分の表は、第 6.5 節のとおり自分の成功・警告と比べる)。
- `[pipeline]` の `name` に誤りがあっても、既定の誤りと既定との重なりの誤りは報告する(既定は `[pipeline]` の他の値とは別に読む)。

### 6.9 `log_dir`(v0.10.0 以降)

`[pipeline]` の `log_dir` に、ジョブの出力を残すディレクトリを書く。書けば、各試行の標準出力と標準エラーを、端末へ流すのと同時に、そのディレクトリの中の試行ごとのファイルへ書く(05 の 19 章、07 の 4.5)。書かなければ、出力は今までどおり引き継ぐだけで、ファイルには残さない。

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| 文字列でない | `typeMismatch` | 値の行 | `` `log_dir` は文字列で書きます `` |
| 空の文字列 | `emptyField` | 値の行 | `` `log_dir` が空です `` |

- 値は、書いた文字列のまま `PipelineDefinition.log_dir`(`Option<String>`)に持つ。相対パスなら、**定義ファイルのあるディレクトリ**が基準(解決は使う側。`.env` と同じ)。`~` と変数は展開しない。
- 定義の読み込みでは、ディレクトリの存在を調べない(無ければ、試行の前に親を含めて作る。07 の 4.5)。
- `log_dir` は `[pipeline]` だけのキーで、`[[job]]` に書くと `unknownKey`。

### 6.10 `unset_env`(v0.10.0 以降)

`[pipeline]` と各 `[[job]]` に、引き継いだ環境から消す環境変数の名前を、文字列の配列で書く。`[pipeline]` の名前はすべてのジョブで、`[[job]]` の名前はそのジョブで消す。消したうえで、`env`・`env_file`・自動の変数を足す(05 の 14 章)。

```toml
[pipeline]
name = "nightly"
unset_env = ["HTTP_PROXY", "HTTPS_PROXY"]

[[job]]
id = "extract"
command = "extract"
unset_env = ["SECRET_TOKEN"]
```

配列の要素を、書いた順に 1 つずつ、次の順に検査する。当てはまった最初の誤りを報告して、その要素の検査を終える。

| 順 | 条件 | `kind` | 行 | 理由(`<名前>` は `quote_key` で整形した名前) |
|----|------|--------|----|------|
| 0 | `unset_env` が配列でない(文字列、表など) | `typeMismatch` | 値の行 | `` `unset_env` は、文字列の配列で書きます ``(`unset_env` は空として扱う) |
| 1 | 要素が文字列でない | `typeMismatch` | その要素の行 | `` `unset_env` の要素は、文字列で書きます `` |
| 2 | 空の文字列 | `emptyField` | その要素の行 | `` `unset_env` に空の文字列は書けません `` |
| 3 | 名前が `is_valid_env_name` を満たさない(第 6.1 節の規則) | `invalidEnvName` | その要素の行 | `` `unset_env` の名前 <名前> は、英字または `_` で始まり、英数字と `_` だけで書きます `` |
| 4 | 予約名(ASCII の大文字にして `CONDUCTOR_` で始まる) | `reservedEnvName` | その要素の行 | `` `unset_env` の名前 <名前> は、`CONDUCTOR_` で始まるため書けません `` |

- 完全に同じ名前は誤りにせず、最初の 1 つだけを残す(`["HTTP_PROXY", "A", "HTTP_PROXY"]` は `["HTTP_PROXY", "A"]`)。大文字小文字だけが違う名前(`SECRET_TOKEN` と `a` と `A` など)は、まとめずに別の名前として残す。空の配列は、書かないのと同じ。
- 検査を通った名前は、`PipelineDefinition.unset_env` と `JobDefinition.unset_env`(`Vec<String>`)に、書いた順で入る。
- **`env` との重なり**(全ジョブにまたがる検査。第 7 節の 4 の最後): `[pipeline].unset_env` の名前は、`[pipeline.env]` とすべてのジョブの `[job.env]` の名前と比べる。ジョブの `unset_env` の名前は、`[pipeline.env]` とそのジョブの `[job.env]` の名前と比べる(ほかのジョブの `env` とは比べない)。比べ方は、どの OS でも ASCII の大文字小文字を区別しない(`Path` と `PATH` も重なり)。重なれば、その `unset_env` の名前の要素の行で、名前ごとに 1 件の `unsetEnvConflict`、理由 `` `unset_env` の名前が、`env` の名前と重なっています ``(値も名前も入れない)。
- `.env` の名前との重なりは、定義の読み込みではなく、`.env` を読んだあとに `unsetConflict` として調べる(6.7)。

## 7. 検査の順序とエラーの集約・並び

Walk の中の処理順(誤りは、見つかった順に集める):

1. 根のキー検査(`unknownKey`)。
2. `[pipeline]`: (v0.10.0 以降)既定のリトライ `retry`(6.8。表の検査は 3 の `retry` と同じ順で、続けて既定の上限と間隔の大小) → キー検査 → `name` → `description` → `run_id_env`(型の誤り、続けて名前の形の誤り) → `remove_on_complete` → `env`(v0.9.0 以降。6.6) → `env_file`(同) → `log_dir`(v0.10.0 以降。6.9) → `unset_env`(同。6.10) → `name` の空の検査 → 名前の照合。
3. `job`: 無い/空/型の誤りの処理。配列なら、各ジョブを順に、次の順で検査する: キー検査 → `id` → `command` → `warning_exit_codes` → `success_exit_codes` → `transitions`(各遷移について、キー検査 → `on` → `next` → 重複) → `retry`(`false` なら検査なし。表なら、キー検査 → `exit_codes` → `max` → `interval` → `multiplier` → `max_interval` → 倍率と上限の片方だけの検査) → `env`(v0.9.0 以降) → `env_file`(同) → `unset_env`(v0.10.0 以降)。
4. 全ジョブにまたがる検査(第 8 節): ジョブIDの検査 → 各ジョブについて、遷移先 → 終了コード(`zeroAsWarning`、`overlappingExitCodes`) → `retry` の重なり・上限と間隔の大小(`retry` を書かないジョブは、既定との重なり。6.8) → (v0.10.0 以降)全ジョブのあとに、`unset_env` と `env` の重なり(6.10。`[pipeline]` の `unset_env`、続けて各ジョブの `unset_env`)。
5. 誤りが 1 件もなく、`[pipeline]` とすべてのジョブが組み立てられるときだけ、`PipelineDefinition` を返す。

返す誤りの順序: すべての誤りを、**誤りの位置(ファイル先頭からのバイト位置)の昇順**に並べて返す。同じ位置のものは、上の検査の順(集めた順)を保つ(安定な並べ替え)。行番号の昇順になるが、行番号が同じでもバイト位置が違えば、バイト位置の順である。

- 型の誤りなどで検査できなくなった値は、その値に依存する後続の検査だけを省略する(他の検査は続ける)。
- 位置がファイルの先頭(位置 0)の誤りは、行 1 として扱われる(`pipeline` が無い、`job` が無い)。

複数の誤りの例(`tests/fixtures/multiple_errors.toml`):

```toml
format_version = 1
unknown_top = true          # 2 行目: unknownKey

[pipeline]
name = "p"

[[job]]
id = "a"
command = "x"
success_exit_codes = ["zero"]   # 10 行目: typeMismatch

[[job]]
id = "b"
command = "y"
colour = "red"              # 15 行目: unknownKey

[[job.transitions]]
on = "success"
next = "nowhere"            # 19 行目: unknownTarget
```

返る誤り: `unknownKey`(2)、`typeMismatch`(10)、`unknownKey`(15)、`unknownTarget`(19)。

## 8. ジョブ間の検査(cross-check)

### 8.1 ジョブID

`id` が文字列であるジョブごとに、記述順に検査する。

- `trim` して空、または先頭が `@` → `invalidJobId`、`id` の値の行、理由 `ジョブID(`id`)は、空でなく、`@` で始まらない文字列で書きます`。この ID は、遷移先の索引に入らない(後続のジョブから、遷移先として参照できない)。
- 既出の ID と完全に同じ(大文字小文字・前後の空白も区別。`" a "` と `"a"` は別) → `duplicateJobId`、2 つ目以降の `id` の値の行、理由 `同じジョブID(`id`)が、すでに定義されています`。
- 上限の長さ、使える文字(空白・記号)の制限は、これ以外にない(未確認の制限は、見つかっていない)。

### 8.2 遷移先

`on` と `next` がともに有効な遷移について、`next` を検査する(`on` が不正な遷移は、検査しない)。

- `next == "@end"`: 受理(以降のジョブをスキップして終える)。
- ID の索引に無い(存在しない、または無効な ID): `unknownTarget`、`next` の値の行、理由 `遷移先(`next`)のジョブが、定義にありません`。
- 記述順で、自分自身または前のジョブ(位置 ≦ 自分の位置): `backwardOrSelfTarget`、`next` の値の行、理由 `遷移先(`next`)は、記述順で後ろのジョブまたは `@end` で書きます`。
- 後ろのジョブの ID は受理する。

### 8.3 終了コード

各ジョブについて(`warning_exit_codes` と `success_exit_codes` が有効なとき):

- `warning_exit_codes` に 0 を含む → `zeroAsWarning`、`warning_exit_codes` の**キー**の行、理由 `0 は警告の終了コード(`warning_exit_codes`)にできません`。(`success_exit_codes` の誤りの有無に関わらず検査する。)
- 両方を書き、有効で、共通する値がある → `overlappingExitCodes`、`warning_exit_codes` の**キー**の行、理由 `` `warning_exit_codes` と `success_exit_codes` に、同じ値があります ``。ただし、共通の値が 0 だけで、すでに `zeroAsWarning` を報告した場合は、重ねて報告しない(例: `warning_exit_codes = [0]` と `success_exit_codes = [0]` は `zeroAsWarning` の 1 件だけ)。
- `success_exit_codes` を書かないとき、成功は `[0]` だが、`warning_exit_codes` との重なりは(書かれていない側は)調べない。警告に 0 を入れた場合は、`zeroAsWarning` で拒否される。

### 8.4 `retry` との関係

第 6.5 節(重なり、上限と間隔の大小)。既定のリトライとの重なりは第 6.8 節(v0.10.0 以降)。

### 8.5 `unset_env` と `env` の重なり(v0.10.0 以降)

第 6.10 節。

## 9. エラーの種類と表示

### 9.1 エラーの構造

| フィールド | 内容 |
|------------|------|
| `file` | 表示用のファイル名。ライブラリ側の `Display` では、制御文字をエスケープし、256 文字までに切る。CLI の表示は、エスケープだけで切らない(08 章 9 章) |
| `line` | 行番号(1 始まり)。位置が決まらないときは無し |
| `kind` | 種類(下の表) |
| `reason` | 固定の文から組み立てた理由。入力の値は含めない。入力由来はキー名だけ |

ライブラリの表示(`Display`)の書式:

```
<file>:<line>: <種類の文>: <reason>      行があるとき
<file>: <種類の文>: <reason>             行が無いとき
```

例: `conductor.toml:7: 定義にないキーです: キー `x` は定義にありません`。

`conductor` コマンドの表示(`crates/cli`)は、`conductor: error: <file>[:<line>]: <種類の文(言語による)>: <reason>` の 1 行で、`reason` は日本語のまま(言語によらない)。種類の文は、ロケールが日本語のとき日本語(下表の「日本語」)、それ以外は英語(下表の「英語」)。`<file>` と `reason` は、改行・制御文字・書式文字をエスケープして表示する(`\n`、`\r`、`\t`、その他は `\u{16進}`)。例:

```
conductor: error: bad.toml:7: unknown key: キー `typo` は定義にありません
conductor: error: conductor.toml: definition file does not exist: 指定の場所にファイルがありません
```

誤りの扱い(終了コード 64)は、CLI の仕様(別文書)に従う。

### 9.2 キー名の表示

`unknownKey` の理由に入るキー名は、`` ` `` で囲み、次の整形をする。

- 改行を `\n`、復帰を `\r`、タブを `\t` に、その他の制御文字と書式文字(`U+200B`〜`U+200F`、`U+2028`、`U+2029`、`U+202A`〜`U+202E`、`U+2060`〜`U+2064`、`U+2066`〜`U+2069`、`U+FEFF`)を `\u{16進小文字}` にエスケープする。
- エスケープ後の文字数が **64 文字**を超えるときは、64 文字までで切って末尾に `…` を付ける(エスケープの途中では切らない)。ちょうど 64 文字は切らない。

### 9.3 種類の一覧(34 種)

「識別子」は機械が読む名前、「日本語」はライブラリの表示、「英語」は CLI が英語ロケールで出す文。

| 識別子 | 日本語 | 英語 |
|--------|--------|------|
| `fileMissing` | 定義ファイルが存在しません | definition file does not exist |
| `unreadable` | 定義ファイルを読めません | cannot read definition file |
| `tooLarge` | 定義ファイルが大きすぎます | definition file is too large |
| `syntax` | 文法の誤りです | syntax error |
| `typeMismatch` | 値の型が正しくありません | wrong value type |
| `unknownKey` | 定義にないキーです | unknown key |
| `versionMissing` | 書式の版がありません | format_version is missing |
| `versionUnsupported` | 対応していない書式の版です | unsupported format_version |
| `unknownTarget` | 遷移先のジョブが定義にありません | transition target job is not defined |
| `errorTransition` | エラー結果には遷移先を書けません | no transition can be written for the error result |
| `backwardOrSelfTarget` | 遷移先は、記述順で後ろのジョブまたは @end のみです | transition target must be a later job or @end |
| `overlappingExitCodes` | 成功と警告の終了コードが重なっています | success and warning exit codes overlap |
| `zeroAsWarning` | 0 は警告の終了コードにできません | 0 cannot be a warning exit code |
| `pipelineNameMismatch` | パイプライン名が定義と一致しません | pipeline name does not match the definition |
| `duplicateJobId` | ジョブIDが重複しています | duplicate job id |
| `invalidJobId` | ジョブIDが正しくありません | invalid job id |
| `noJobs` | ジョブが 1 つもありません | no jobs are defined |
| `emptyField` | 必須の値が空です | required value is empty |
| `duplicateTransition` | 同じ結果の遷移が重複しています | duplicate transition for the same result |
| `invalidEnvName` | 環境変数の名前が正しくありません | invalid environment variable name |
| `retryNotTable` | retry は表で書きます | retry must be a table |
| `retryMissingKey` | retry の必須のキーがありません | a required key of retry is missing |
| `retryExitCodesEmpty` | retry の終了コードの一覧が空です | retry exit_codes is empty |
| `retryExitCodeInvalid` | retry の終了コードが正しくありません | retry exit_codes has an invalid exit code |
| `retryExitCodesOverlap` | retry の終了コードが、成功または警告の終了コードと重なっています | retry exit_codes overlap the success or warning exit codes |
| `retryMaxInvalid` | retry の最大回数が正しくありません | retry max is invalid |
| `retryDurationInvalid` | retry の間隔が正しくありません | retry interval is invalid |
| `retryBackoffIncomplete` | retry の倍率と間隔の上限は、両方を書くか、両方とも書きません | retry multiplier and max_interval must be given together or not at all |
| `retryMultiplierInvalid` | retry の倍率が正しくありません | retry multiplier is invalid |
| `retryMaxIntervalTooSmall` | retry の間隔の上限が、間隔より小さくなっています | retry max_interval is smaller than interval |
| `reservedEnvName`(v0.9.0 以降) | 予約された環境変数の名前です | reserved environment variable name |
| `envNameCaseConflict`(v0.9.0 以降) | 大文字小文字だけが違う環境変数の名前があります | environment variable names differ only in letter case |
| `invalidEnvValue`(v0.9.0 以降) | 環境変数の値が正しくありません | invalid environment variable value |
| `unsetEnvConflict`(v0.10.0 以降) | 消す環境変数の名前が、足す環境変数の名前と重なっています | a name in unset_env is also set by env |

(`retryDurationInvalid` の英語は、`max_interval` の誤りにも同じ文が使われる。`invalidEnvName` は、v0.9.0 以降、`run_id_env` と `env` の名前の両方に使い、v0.10.0 以降は `unset_env` の名前にも使う。`reservedEnvName` も、v0.10.0 以降は `unset_env` の名前に使う。`retryNotTable`・`retryExitCodesOverlap`・`retryMaxIntervalTooSmall` は、v0.10.0 以降、`[pipeline.retry]` の誤りにも使う(6.8)。`.env` の読み取りの誤りは、別の型 `EnvFileErrorKind`(14 種。6.7)で、この表に含めない。)

理由(`reason`)は、第 2〜6 節の各表の日本語の原文のとおり。英語の理由文は存在しない(理由は常に日本語)。

## 10. 行番号の出し方

- 行番号は 1 始まり。入力の先頭からのバイト位置を、「各行の先頭のバイト位置の表(先頭 0、および各 `\n` の次の位置)」で二分探索して求める。位置がちょうど行頭ならその行、そうでなければ、その位置以下で最大の行頭を持つ行。範囲外の位置は、最後の行。
- 改行は LF の数で数える。CRLF でも、`\r` は直前の行に属し、LF のあとが次の行(CRLF の定義で、LF と同じ行番号になることを確認済み)。
- 位置の取り方: 型の誤りや範囲の誤りは、**誤った値の先頭**。未知のキーは、**キーの先頭**。必須のキーの欠落は、**それを含むテーブルの先頭**(見出し `[pipeline]`、`[[job]]`、`[job.retry]`、`[[job.transitions]]` の行。インラインテーブルなら `{` の位置、ドットキーなどで暗黙に作られたテーブルの位置は**未確認**)。`pipeline` や `job` 自体が無いときは、位置 0(1 行目)。
- 終了コードの重なり(`overlappingExitCodes`、`zeroAsWarning`、`retryExitCodesOverlap`)は、対象のキーの行(値ではなく**キー**の先頭)。ただし、既定のリトライとの重なり(v0.10.0 以降。6.8)は、そのジョブの表(`[[job]]`)の先頭の行。
- `unset_env` と `env` の重なり(`unsetEnvConflict`。v0.10.0 以降)は、`unset_env` の名前の要素の行。
- 配列の誤りは、要素ごとに、その要素の先頭の行(複数行の配列なら、要素の書かれた行)。
- Parse の文法の誤りは、読み取り器が示す位置の行(第 3 節)。深さの誤りは、33 段目の要素の先頭の行。
- 取り込みの誤りと `versionMissing` は、行なし。
- 複数行の文字列の内側など、値の「先頭」は、値のトークンの最初の文字(引用符を含む)。

## 11. 取り込み後のデータモデル

検証に成功すると、次のデータを返す(Rust の型名で示す)。

```
PipelineDefinition {
    name: String,                      // 空白だけでない
    description: Option<String>,
    run_id_env: Option<String>,        // 名前の形は検査済み。None なら実行時に CONDUCTOR_RUN_ID
    remove_on_complete: bool,          // 省略時 false
    format_version: u32,               // 1
    job: Vec<JobDefinition>,           // 記述順を保つ。1 つ以上
    env: Vec<EnvVar>,                  // v0.9.0 以降。[pipeline.env]。書いた順。省略時 []
    env_file: Vec<EnvFileRef>,         // v0.9.0 以降(v0.9.x までは Vec<String>)。[pipeline].env_file。省略時 []
    log_dir: Option<String>,           // v0.10.0 以降。空でない。書いた文字列のまま。省略時 None
    unset_env: Vec<String>,            // v0.10.0 以降。書いた順。完全に同じ名前は最初の 1 つ。省略時 []
}
JobDefinition {
    id: String,                        // 空白だけでなく、@ で始まらず、定義内で一意
    command: Command,                  // Line(String) | Argv(Vec<String>)
    warning_exit_codes: Vec<i32>,      // 省略時 []。0 を含まない。書いた順のまま
    success_exit_codes: Option<Vec<i32>>, // 省略時 None(意味は [0])
    transitions: Vec<Transition>,      // 書いた順。on ごとに高々 1 つ
    retry: Option<RetryPolicy>,        // 実効のリトライ。表ならその表、書かなければ [pipeline.retry] の複製、
                                       // false なら None(v0.10.0 以降。既定から来たかは持たない)
    env: Vec<EnvVar>,                  // v0.9.0 以降。書いた順。省略時 []
    env_file: Vec<EnvFileRef>,         // v0.9.0 以降。書いた順。項目 1 つは要素 1 つ。省略時 []
    unset_env: Vec<String>,            // v0.10.0 以降。形は PipelineDefinition.unset_env と同じ
}
EnvVar { name: String, value: String }   // Debug は value を "***" と表示する
EnvFileRef { path: String, required: bool }   // v0.10.0 以降。文字列の項目は required = true
Transition { on: Success | Warning, next: Job(String) | End }   // next が "@end" のとき End
RetryPolicy {
    exit_codes: Vec<i32>,              // 昇順・重複なし・各 1〜255・空でない
    max: u32,                          // 1〜100
    interval_secs: u64,                // 1〜86400
    backoff: Option<Backoff>,          // multiplier と max_interval の両方を書いたときだけ Some
}
Backoff {                              // Copy。片方だけの組は作れない
    multiplier: u32,                   // 2〜10
    max_interval_secs: u64,            // interval_secs 以上
}
```

- 成功として扱う終了コード = `success_exit_codes` があればそれ、なければ `[0]`。
- ジョブIDから記述順の位置(0 始まり)を引く操作がある。

## 12. `run_id_env` のゆるい読み取り

`watch` と `reset` は、`--run-id` が無いとき、定義全体を検証せず、`[pipeline].run_id_env` だけを取り出す(`peek_run_id_env`)。

- TOML として読めない、`pipeline` がテーブルでない、キーが無い、文字列でない、名前の形(第 6.1 節)に合わない、のどれでも `None`(実行時は `CONDUCTOR_RUN_ID`)。
- ほかの誤り(`format_version` の誤り、未知のキーなど)は無視する。壊れた入力でもパニックしない。
- 例: `[pipeline]\nname = "p"\nrun_id_env = "X_ID"\n` → `X_ID`。`run_id_env = 5`、`"1A"`、`""` → `None`。`pipeline = 3` → `None`。空の入力 → `None`。

## 13. 受理・拒否の例

最小の定義(受理):

```toml
format_version = 1

[pipeline]
name = "p"

[[job]]
id = "a"
command = "c"
```

全キーを使った定義(受理):

```toml
format_version = 1

[pipeline]
name = "nightly"
description = "夜間バッチ"
run_id_env = "GITHUB_RUN_ID"
remove_on_complete = true

[[job]]
id = "extract"
command = ["python", "scripts/extract.py", ""]
warning_exit_codes = [4]
success_exit_codes = [0, 5]

[job.retry]
exit_codes = [75, 111]
max = 5
interval = "10s"
multiplier = 2
max_interval = "60s"

[[job.transitions]]
on = "warning"
next = "notify"

[[job]]
id = "load"
command = "load"

[[job.transitions]]
on = "success"
next = "@end"

[[job]]
id = "notify"
command = "notify --channel ops"
```

`env` と `env_file` を使った定義(受理。v0.9.0 以降):

```toml
format_version = 1

[pipeline]
name = "nightly"
env_file = ".env"
[pipeline.env]
TZ = "Asia/Tokyo"

[[job]]
id = "extract"
command = "python scripts/extract.py"
env_file = ["extract.env", "secret.env"]
[job.env]
LOG_LEVEL = "debug"
```

v0.10.0 のキーを使った定義(受理):

```toml
format_version = 1

[pipeline]
name = "nightly"
log_dir = "logs"
unset_env = ["HTTP_PROXY"]
env_file = [".env", { path = "local.env", required = false }]

[pipeline.retry]
exit_codes = [75]
max = 3
interval = "10s"

[[job]]
id = "extract"
command = "python scripts/extract.py"
unset_env = ["SECRET_TOKEN"]

[[job]]
id = "notify"
command = "notify --channel ops"
retry = false
```

拒否の例(各行の行番号は、ヘッダ 3 行を除く、`with_header` の続きとしての値。表の「行」は、テスト `definition_errors.rs` / `definition_retry.rs` で確認した値):

| 入力(ヘッダ `format_version = 1` / `[pipeline]` / `name = "p"` の次から) | 結果 |
|------|------|
| `[[job]]` `id = "a"` `command = "c"` `[[job.transitions]]` `on = "success"` `next = "zzz"` | `unknownTarget` 9 行目 |
| 同、`on = "error"` `next = "@end"` | `errorTransition` 8 行目 |
| ジョブ a の `next = "a"`(自分自身) | `backwardOrSelfTarget` 9 行目 |
| `warning_exit_codes = [2, 3]` と `success_exit_codes = [0, 3]` | `overlappingExitCodes` 7 行目 |
| `warning_exit_codes = [0]` | `zeroAsWarning` 7 行目 |
| `warning_exit_codes = ["x"]` | `typeMismatch` 7 行目 |
| `id = 1` | `typeMismatch` 5 行目 |
| `command` が無い(`[[job]]` と `id` だけ) | `typeMismatch` 4 行目 |
| `warn_exit_codes = [1]` | `unknownKey` 7 行目 |
| `id = ""` / `id = "@end"` | `invalidJobId` 5 行目 |
| 同じ `id` の 2 つ目 | `duplicateJobId`(2 つ目の `id` の行) |
| `command = "  "` | `emptyField` 6 行目 |
| ヘッダのみ(ジョブなし) | `noJobs` 1 行目 |
| `[job.retry]` に `exit_codes = [0]`、`max = 0`、`interval = "0s"`、`multiplier = 1`、`max_interval = "0s"`、`unknown = 1`(7 行目が `[job.retry]`) | 8〜13 行目に、`retryExitCodeInvalid`、`retryMaxInvalid`、`retryDurationInvalid`、`retryMultiplierInvalid`、`retryDurationInvalid`、`unknownKey`(この順) |
| `[pipeline.env]` に `CONDUCTOR_X = "1"`(v0.9.0 以降) | `reservedEnvName`(キーの行) |
| `[job.env]` に `foo = "1"` と `FOO = "2"` | `envNameCaseConflict`(2 つ目のキーの行) |
| `[job.env]` に `A = 1`、または `"1A" = "x"` | `typeMismatch`(値の行)、または `invalidEnvName`(キーの行) |
| `env_file = ""`、`env_file = ["a", 3]`、`env_file = 5` | `emptyField`、`typeMismatch`(3 の行)、`typeMismatch` |
| `env_file = { required = false }`、`env_file = { path = "a", optional = true }`(v0.10.0 以降) | `typeMismatch`(`` 必須の値 `path` がありません ``)、`unknownKey` |
| `[pipeline]` に `log_dir = 1`、`log_dir = ""`(v0.10.0 以降) | `typeMismatch`、`emptyField`(値の行) |
| `[[job]]` に `log_dir = "logs"` | `unknownKey` |
| `unset_env = "HTTP_PROXY"`、`unset_env = ["", "1A", 2, "CONDUCTOR_A"]`(v0.10.0 以降) | `typeMismatch`、および位置の順に `emptyField`、`invalidEnvName`、`typeMismatch`、`reservedEnvName` |
| ジョブに `unset_env = ["foo"]` と `[job.env]` の `FOO = "x"` | `unsetEnvConflict`(`unset_env` の名前の行) |
| `[pipeline]` に `retry = false`(v0.10.0 以降) | `retryNotTable`(`` `[pipeline.retry]` はテーブルで書きます ``) |
| ジョブに `retry = true` | `retryNotTable`(`` `retry` はテーブルか `false` で書きます ``) |

テストの確認済みの実行例: 行番号は `\r\n` の改行でも同じ。複数のジョブにまたがる誤り(`command = ""`、`max = 0`、同じ `id`、`retry` の必須キー欠落)は、位置の順に `emptyField`(6)、`retryMaxInvalid`(9)、`duplicateJobId`(12)、`retryMissingKey`(14)で返る。

## 14. 実装と README の差

README の記述と実装は、確認した範囲では矛盾しない。補足・差は次のとおり。

- README は、`[job.retry]` を「`id`・`command` などのキーのあと(`[[job.transitions]]` の前)に書く」と述べる。実装は、`[job.retry]` を `[[job.transitions]]` の**後**に書いても受理する(TOML の意味どおり、直前のジョブに属する)。README の位置の指定は、推奨の書き方であり、検証の制約ではない。
- README は、定義ファイルの大きさの上限(1 MiB)、入れ子の深さの上限(32 段)、整数のリテラルの形式を述べていない。実装は、上限 1 MiB、深さ 32 段を課す。
- README の表は、`warning_exit_codes` の値に負の数や大きな数への言及がない。実装は、`i32` の範囲の整数(負の値を含む)を受理し、範囲外を `typeMismatch` にする。
- README の説明では、`success_exit_codes` と `warning_exit_codes` の重なりは「重ならず、`0` を含まない」とあるのみ。実装は、0 が両方に書かれたときは `zeroAsWarning` だけを報告する。

## 15. 未確認事項

- 深さ 33 段〜100,000 段の入力で、拒否の理由文が `入れ子が深すぎます(上限は 32 段)` か `TOML の文法に誤りがあります` のどちらになるか(種類は常に `syntax`)。
- TOML の読み取りで受理される構文の全範囲(使う読み取り器 `toml` 1.2.1 の仕様に従う)。TOML 1.1 の構文の受理は、変換器経由の実行で確認した。
- 暗黙に作られるテーブル(ドットキーで作られた `retry.exit_codes = ...` など)でキーが欠けたときの、誤りの行。
- Windows など他の OS での、ファイルの権限・シンボリックリンクに関わる `unreadable` の判定の細部(実装は、`std::fs` が返す種類だけで判断する)。
