# 03. 定義ファイルの書式と検証(rust 版 conductor v0.8.0)

この文書は、`conductor` の定義ファイル(`conductor.toml`)の書式と、読み込み時の検証を定める。実装は `crates/core/src/definition/*`(`conductor-core`)で、根拠は実装とそのテスト(`crates/core/tests/definition_*.rs`)である。この文書だけで、同じ受理・拒否・エラーを返す読み込み処理を実装できることを目標とする。

- 定義ファイルの形式は **TOML だけ**である(YAML の定義は読めない。YAML は `convert` で TOML にしてから使う)。
- 検証は、見つけた誤りを**まとめて**返す(ただし、早い段階の誤りは 1 件だけで打ち切る。後述)。
- 定義の内容は、実行も評価もしない。`command` の文字列は、そのまま保持する。
- 確認できなかった点は「未確認」と書く。

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
| 1 | メタデータの取得で「存在しない」 | `fileMissing` | `指定の場所にファイルがありません` |
| 2 | メタデータの取得で、権限がない | `unreadable` | `読み取りの権限がありません` |
| 3 | メタデータの取得で、その他の失敗 | `unreadable` | `読み取りに失敗しました` |
| 4 | 通常のファイルでない(ディレクトリなど) | `unreadable` | `通常のファイルではありません` |
| 5 | メタデータの大きさが 1 MiB(1,048,576 バイト)を超える | `tooLarge` | `ファイルの大きさが上限(1 MiB)を超えています` |
| 6 | ファイルを開けない(権限) | `unreadable` | `読み取りの権限がありません` |
| 6' | ファイルを開けない(その他) | `unreadable` | `読み取りに失敗しました` |
| 7 | 読み取りの途中の失敗 | `unreadable` | 権限なら `読み取りの権限がありません`、他は `読み取りに失敗しました` |
| 8 | 読み取り結果が上限を超える(確認のあとに大きくなった場合) | `tooLarge` | 順 5 と同じ |
| 9 | UTF-8 として解釈できない | `unreadable` | `文字コードが UTF-8 ではありません` |

- 上限は **1 MiB = 1,048,576 バイトちょうどまで受理**し、1 バイトでも超えると `tooLarge`。読み取りは「上限 + 1 バイト」までしか行わない。
- 空のファイル(0 バイト)は、空の文字列として読めて、次の段階へ進む(結果は、`format_version` が無いので `versionMissing`)。
- 順 5 の大きさの確認は UTF-8 の確認より先なので、1 MiB を超えて UTF-8 でないファイルは `tooLarge` である。
- いずれも、行番号は付かない(`line` は `None`)。
- シンボリックリンクは、リンク先のメタデータで判定する(`std::fs::metadata` を使うため)。

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

- 許されたキー以外は `unknownKey`(例: `owner`、`remove_on_complet`)。
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
| `retry` | テーブル | 任意 | なし(`None`) | 第 6.5 節 |

許されたキー以外は `unknownKey`(例: `warn_exit_codes`)。

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
| `multiplier` | 整数 | 任意 | 2 以上 10 以下 | `Option<u32>` |
| `max_interval` | 文字列 | 任意 | `interval` と同じ書式、`interval` 以上 | 秒(`Option<u64>`) |

- 127 などの値も受理する(`[127, 1]` → `[1, 127]`)。
- `multiplier` と `max_interval` は、**両方を書くか、両方とも書かない**か。片方だけなら `retryBackoffIncomplete`。両方なしなら、待ち時間は毎回 `interval`。

#### 間隔の文字列の書式

- 文字列の最後の 1 文字が単位(`s`=秒、`m`=分=60 秒、`h`=時間=3600 秒。小文字だけ)。その前が、**ASCII の数字だけ**の 1 文字以上の列(符号・空白・小数点・`_`・全角数字・16 進(`0x`)は不可)。
- 数値は 1 以上。秒に換算した値が 1 以上 86,400(24 時間)以下。桁あふれ(`u64` に収まらない、または換算の乗算があふれる)は拒否。
- 受理: `1s`、`30s`、`5m`、`2h`、`24h`、`1440m`、`86400s`。拒否: `""`、`s`、`10`、`1.5s`、`-1s`、`+1s`、` 1s`、`1s `、`1 s`、`1d`、`1S`、`１s`、`1ms`、`0x10s`、`1_0s`、`0s`、`0h`、`86401s`、`25h`、`99999999999999999999999s`、`18446744073709551615h`。文字列でない値(整数・真偽値)も拒否。

#### 誤りと検査

| 条件 | `kind` | 行 | 理由 |
|------|--------|----|------|
| `retry` がテーブルでない(整数、文字列、`[[job.retry]]` の配列) | `retryNotTable` | `retry` の値の行(配列のときは、その配列の先頭) | `` `retry` はテーブルで書きます `` |
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
- 誤りの文に、入力の値は含めない(`"SENTINEL..."` のような値が文に出ないことを確認するテストがある)。

## 7. 検査の順序とエラーの集約・並び

Walk の中の処理順(誤りは、見つかった順に集める):

1. 根のキー検査(`unknownKey`)。
2. `[pipeline]`: キー検査 → `name` → `description` → `run_id_env`(型の誤り、続けて名前の形の誤り) → `remove_on_complete` → `name` の空の検査 → 名前の照合。
3. `job`: 無い/空/型の誤りの処理。配列なら、各ジョブを順に、次の順で検査する: キー検査 → `id` → `command` → `warning_exit_codes` → `success_exit_codes` → `transitions`(各遷移について、キー検査 → `on` → `next` → 重複) → `retry`(キー検査 → `exit_codes` → `max` → `interval` → `multiplier` → `max_interval` → 倍率と上限の片方だけの検査)。
4. 全ジョブにまたがる検査(第 8 節): ジョブIDの検査 → 各ジョブについて、遷移先 → 終了コード(`zeroAsWarning`、`overlappingExitCodes`) → `retry` の重なり・上限と間隔の大小。
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

第 6.5 節(重なり、上限と間隔の大小)。

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

### 9.3 種類の一覧(30 種)

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

(`retryDurationInvalid` の英語は、`max_interval` の誤りにも同じ文が使われる。)

理由(`reason`)は、第 2〜6 節の各表の日本語の原文のとおり。英語の理由文は存在しない(理由は常に日本語)。

## 10. 行番号の出し方

- 行番号は 1 始まり。入力の先頭からのバイト位置を、「各行の先頭のバイト位置の表(先頭 0、および各 `\n` の次の位置)」で二分探索して求める。位置がちょうど行頭ならその行、そうでなければ、その位置以下で最大の行頭を持つ行。範囲外の位置は、最後の行。
- 改行は LF の数で数える。CRLF でも、`\r` は直前の行に属し、LF のあとが次の行(CRLF の定義で、LF と同じ行番号になることを確認済み)。
- 位置の取り方: 型の誤りや範囲の誤りは、**誤った値の先頭**。未知のキーは、**キーの先頭**。必須のキーの欠落は、**それを含むテーブルの先頭**(見出し `[pipeline]`、`[[job]]`、`[job.retry]`、`[[job.transitions]]` の行。インラインテーブルなら `{` の位置、ドットキーなどで暗黙に作られたテーブルの位置は**未確認**)。`pipeline` や `job` 自体が無いときは、位置 0(1 行目)。
- 終了コードの重なり(`overlappingExitCodes`、`zeroAsWarning`、`retryExitCodesOverlap`)は、対象のキーの行(値ではなく**キー**の先頭)。
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
}
JobDefinition {
    id: String,                        // 空白だけでなく、@ で始まらず、定義内で一意
    command: Command,                  // Line(String) | Argv(Vec<String>)
    warning_exit_codes: Vec<i32>,      // 省略時 []。0 を含まない。書いた順のまま
    success_exit_codes: Option<Vec<i32>>, // 省略時 None(意味は [0])
    transitions: Vec<Transition>,      // 書いた順。on ごとに高々 1 つ
    retry: Option<RetryPolicy>,        // 書かなければ None
}
Transition { on: Success | Warning, next: Job(String) | End }   // next が "@end" のとき End
RetryPolicy {
    exit_codes: Vec<i32>,              // 昇順・重複なし・各 1〜255・空でない
    max: u32,                          // 1〜100
    interval_secs: u64,                // 1〜86400
    multiplier: Option<u32>,           // 2〜10。max_interval_secs と同時に Some
    max_interval_secs: Option<u64>,    // interval_secs 以上。multiplier と同時に Some
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
