# 07 プロセス起動とシグナル(`command` の分割・起動・待ち・中断)

対象: `conductor` v0.10.0 の機能(`unset_env` の `env_remove`、`log_dir` のログのファイル、`watch --follow` の待ち)を含む現在の実装(Rust 実装)。根拠は `crates/runtime/src/{split,runner,log,wait,types,error,platform/*}.rs`、
`crates/cli/src/{platform/*,ctx,pipeline,run,step,aux_commands,exit,panic_hook,main}.rs`、`crates/core/src/flow/{classify,resume}.rs`、
および `crates/runtime/tests/{runner_*,log_file,wait_interruptible,nfr_*}.rs`、`crates/cli/tests/{signals_*,binary_signal,binary_log}.rs`、
`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`。

この文書は、`command` の文字列を分ける規則、ジョブ(子プロセス)の起動と待ち、シグナルによる中断、
OS ごとの違いを定める。ステートファイルの書式、定義ファイルの検証全体、表示の文言は扱わない(必要な箇所だけ触れる)。

---

## 1. 部品と責務の分担

| 部品 | 場所 | 責務 |
|------|------|------|
| Split(分け方) | `conductor-runtime` の `split.rs` | `command` の文字列を、コマンド名と引数に分ける。文字を 1 つずつ読む状態機械(再帰なし)。3 OS で同じ規則 |
| `check_command` | `split.rs`(公開) | `command` を起動せずに検査する。`Ok` なら分けた結果(`CommandLine`)を返す |
| JobRunner(`run_job`) | `runner.rs`(公開) | 1 つのジョブを、シェルを介さず起動し、終了を待ち、`JobExecution` と `LogOutcome` を返す。パニックしない |
| Copier(v0.10.0 以降) | `runner.rs`(非公開) | `log_dir` があるときだけ、子の標準出力と標準エラーを 1 つずつ読むスレッド(`conductor-log-copier`)。読んだ分を端末とログのファイルへ書く(4.5) |
| RuntimeLog(v0.10.0 以降) | `log.rs`(公開) | ログのファイルの名前(`safe_log_stem`)と作成(`open_log_file`)。`LogTarget`、`LogOutcome` |
| WaitLoop | `runner.rs`(非公開) | `try_wait` を、1 ms から 10 ms の間隔でくり返す。シグナルの転送の判断もここ |
| `wait_for` | `wait.rs`(公開) | リトライ前の待機。中断の合図を短い間隔で確かめる |
| `InterruptSignal` | `types.rs`(公開) | 中断の合図。スレッド間で共有する、読み取り中心の値 |
| `platform`(runtime) | `platform/{mod,unix,windows}.rs` | プロセスグループの指定、シグナルの送信、シグナルによる終了の読み取り、ディレクトリの同期 |
| SignalWatcher | `conductor-cli` の `platform/{mod,unix,windows}.rs` | OS のシグナル(UNIX)/ Ctrl+C(Windows)を受け取り、`InterruptSignal` を立てる |
| Orchestrator | `conductor-cli` の `ctx/run/pipeline/step.rs` | 合図の準備、`run_job` の呼び出し、中断時のステートの記録、終了コード(128 + 番号) |
| 分類 | `conductor-core` の `flow/classify.rs` | `JobExecution` を、成功・警告・エラー・分類なし、と報告する終了コードに変える |

OS ごとの分岐(`cfg(unix)` / `cfg(windows)`)と OS 固有の呼び出しは、runtime と cli のそれぞれの `platform` モジュールの外に
置かない(cli のテストのときだけの単体テスト `src/tests/` を除く。ソース検索のテスト `nfr_source.rs` が確かめる。8 章)。

---

## 2. `command` の形式(定義の側の受理条件)

定義ファイルの `[[job]]` の `command` は、次の 2 つの形式のどちらか(`conductor-core` の `Command` 型)。

| 形式 | TOML での書き方 | 型 | 起動のときの扱い |
|------|-----------------|----|------------------|
| 文字列形式 | `command = "sh scripts/x.sh"` | `Command::Line(String)` | 3 章の規則で、コマンド名と引数に分ける |
| 配列形式(v0.2.0 以降) | `command = ["sh", "-c", "echo hi"]` | `Command::Argv(Vec<String>)` | 要素をそのまま使う(分割・引用符の処理・展開をしない) |

定義の読み込み時の検査(`definition/walk/values.rs` の `command`。v0.10.3 までは `definition/walk/mod.rs`。誤りは定義の誤り、終了コード 64):

| 入力 | 判定 |
|------|------|
| `command` キーがない | 誤り(必須の値がない) |
| 文字列で、`trim()`(Unicode の空白を除く)すると空 | 誤り(`command` が空) |
| 配列が空(`[]`) | 誤り(配列が空) |
| 配列に文字列でない要素がある | 誤り(要素は文字列で書く)。要素ごとに報告する |
| 配列の先頭の要素が `trim()` すると空 | 誤り(先頭の要素が空) |
| 配列の 2 番目以降が空文字列 `""` | 有効(空の引数として渡す) |
| 文字列でも配列でもない(数値・表など) | 誤り(型) |

定義の検査とは別に、実行の前にもう一度、全ジョブの `command` を `check_command` で検査する(3.6 節)。

---

## 3. `command` の分割規則(Split)

### 3.1 入力と出力

- 入力: `command` の文字列(UTF-8)。
- 出力: `CommandLine { program: String, args: Vec<String> }`。`program` は空でない。`args` の各要素が、そのまま 1 つの引数になる。
- シェルは使わない。変数展開(`$HOME`)、グロブ(`*`)、パイプ(`|`)、`;`、`&&`、リダイレクト(`>`)、バッククォート、`~`、
  括弧は **解釈しない**。普通の文字として引数に入る。パイプなどが要るときは、利用者が `sh -c '...'` を command に書く。

### 3.2 文法

文字は Unicode のコードポイント(Rust の `char`)単位で、左から右へ 1 回だけ読む。区切り文字は次の 4 つだけ。

```text
separator := ' ' (U+0020) | '\t' (U+0009) | '\n' (U+000A) | '\r' (U+000D)
```

(U+000B、U+000C、U+00A0 などのほかの空白は区切りではなく、普通の文字。)

状態は 4 つ。`in_word`(語の途中か)と `current`(組み立て中の語)を併せて持つ。

| 状態 | 入力の文字 | 動作 | 次の状態 |
|------|-----------|------|----------|
| Plain(引用符の外) | separator | `in_word` なら `current` を語として確定し、空にする。`in_word = false` | Plain |
| Plain | `'` | `in_word = true`(語を開始。まだ何も足さない) | Single |
| Plain | `"` | `in_word = true` | Double |
| Plain | 上記以外(`\` を含む) | `in_word = true`。その文字を `current` に足す | Plain |
| Single(`'...'` の中) | `'` | 何も足さない | Plain |
| Single | 上記以外(`\`、`"`、改行を含む) | その文字を足す | Single |
| Double(`"..."` の中) | `"` | 何も足さない | Plain |
| Double | `\` | 何も足さない(バックスラッシュを保留する) | DoubleBackslash |
| Double | 上記以外 | その文字を足す | Double |
| DoubleBackslash(`"..."` の中で `\` を 1 つ保留中) | `"` | `"` を 1 つ足す(保留の `\` は消える) | Double |
| DoubleBackslash | `\` | 保留していた `\` を 1 つ足す。今読んだ `\` が新しい保留になる | DoubleBackslash |
| DoubleBackslash | 上記以外の文字 c | `\` と c を足す | Double |

入力の終わりでの確定:

1. 状態が Plain でなければ(Single / Double / DoubleBackslash)、**閉じていない引用符**(`UnclosedQuote`)。
2. `in_word` なら `current` を最後の語として確定する。
3. 語の列の先頭がなければ、または先頭の語が空文字列、または空白だけ(`trim` して空)なら、**コマンド名がない**(`Empty`)。
   それ以外は、先頭の語が `program`、残りが `args`(配列の先頭の要素と同じ判定。v0.9.7 までは、文字列の先頭の語が
   空白だけ(`'  '`)なら分けられ、起動の失敗(127)になった)。

要点:

- 引用符で囲んだ部分と囲まない部分は、隣り合っていればつなげて 1 つの語になる(`a"b c"'d e'f` は 1 語 `ab cd ef`)。
- `''` と `""` は、空の語(空の引数)になる(`in_word` が立つため)。ただし先頭の語が空、または空白だけなら `Empty`。
  空白を含むが空白だけでない先頭の語(`' my prog'`)は、そのまま `program`。
- 二重引用符の中で解釈するバックスラッシュは、「直後が `"`」のときだけ(`\"` → `"`)。`\n`、`\t`、`\\` なども、
  **解釈せずそのまま**(`\n` は 2 文字のまま)。ただし `\\` は、1 つ目のバックスラッシュを出力し、2 つ目を新しい保留にする。
  したがって `\\"` は「`\` と、エスケープされた `"`」になる(引用符は閉じない)。`\` の個数で偶奇を数える規則ではなく、
  直前に保留している `\` が 1 つあるかどうかだけで決まるので、上の表を左から機械的に適用すること
  (`"` の直前に `\` が 1 つ以上並んでいれば、`"` は常にエスケープされ、引用符は閉じない)。
- 引用符の外のバックスラッシュは普通の文字(Windows のパス `C:\dir\x.exe` がそのまま書ける)。一重引用符の中も同じ。
- 末尾がバックスラッシュのパスを二重引用符で囲むと `\"` が引用符のエスケープになり、閉じない(誤り)。一重引用符で書く。
- 文字列の長さに上限はなく、入れ子もないため、スタックを使わない(30 万引数の入力でも分割できる)。

### 3.3 受理の例

| 入力(command の文字列) | program | args |
|--------------------------|---------|------|
| `echo a b` | `echo` | `a`, `b` |
| `␣␣echo ⇥ a ⏎ b␣␣`(空白・タブ・改行の混在) | `echo` | `a`, `b` |
| `echo\r\na` | `echo` | `a` |
| `echo 'a b' 'c\d' '"x"'` | `echo` | `a b`, `c\d`, `"x"` |
| `echo "a \" b"` | `echo` | `a " b` |
| `echo "a\nb" "c\\d"` | `echo` | `a\nb`, `c\\d`(バックスラッシュは、そのまま 2 つ) |
| `echo "\\"x"`(`"` `\` `\` `"` `x` `"`) | `echo` | `\"x` |
| `C:\dir\x.exe a\ b` | `C:\dir\x.exe` | `a\`, `b` |
| `\\server\share` | `\\server\share` | (なし) |
| `prog 'C:\dir\'` | `prog` | `C:\dir\` |
| `echo --msg='a b'` | `echo` | `--msg=a b` |
| `echo a"b c"'d e'f` | `echo` | `ab cd ef` |
| `echo '' "" x` | `echo` | (空), (空), `x` |
| `echo ''` | `echo` | (空) |
| `echo a;b && c \| d > e $HOME * `` `x` `` (y)` | `echo` | `a;b`, `&&`, `c`, `\|`, `d`, `>`, `e`, `$HOME`, `*`, `` `x` ``, `(y)` |
| `echo 'あ い' う` | `echo` | `あ い`, `う` |
| `a 'b c'` | `a` | `b c` |

### 3.4 拒否の例

| 入力 | 理由(`CommandErrorKind`) |
|------|---------------------------|
| `echo 'a` | `UnclosedQuote` |
| `echo "a` | `UnclosedQuote` |
| `echo "a\`(末尾が二重引用符の中のバックスラッシュ) | `UnclosedQuote` |
| `echo "a\\" b`(`\\` + `"` は、引用符のエスケープ) | `UnclosedQuote` |
| `prog "C:\dir\"` | `UnclosedQuote` |
| `` (空) | `Empty` |
| `␣⇥⏎`(区切りだけ) | `Empty` |
| `''` | `Empty`(先頭の語が空) |
| `'' x` | `Empty`(先頭の語が空) |
| `'  '` | `Empty`(先頭の語が空白だけ。v0.9.8 以降) |
| `'  ' x` | `Empty`(先頭の語が空白だけ。v0.9.8 以降) |

### 3.5 配列形式の扱い

`Command::Argv(argv)`(`check_command` の配列の枝):

- 要素を一切加工せずに使う。先頭が `program`、残りが `args`。分割、引用符の解釈、展開、前後の空白の除去をしない
  (`"  lead and trail  "` も、そのまま 1 つの引数)。
- 先頭の要素が `trim()` して空(空文字列・空白だけ)、または要素が 0 個のときは `Empty`(定義の検査の二重の備え)。
  先頭の要素が空白を含むが空でないとき(`"my prog"`)は、そのまま `program`(`my prog` というファイル名を起動する)。
- 2 番目以降は空文字列でもよい(空の引数)。

### 3.6 エラーの表示と検査のタイミング

| 項目 | 内容 |
|------|------|
| エラー型 | `CommandError { kind }`。`kind` は `UnclosedQuote` か `Empty`。理由の固定の文は `CommandErrorKind::reason()` |
| `Display` の文 | 固定の文(`reason`)。`UnclosedQuote` は `引用符が閉じていません`、`Empty` は `コマンド名がありません`。**入力の値を含めない**(秘密の値が `command` に入っていても表示に出ない) |
| 利用者への表示(標準エラー) | `conductor: error: <定義ファイルのパス>: job '<ジョブID>': <文>`。文は表示言語に従う。英語は `command has an unclosed quote` / `command is empty`、日本語は `command の引用符が閉じていません` / `command が空です` |
| 検査の時点 | `run` / `pipeline` / `validate` は、定義の読み込みと検証のあと、**全ジョブ**の `command` を `check_command` で検査し、誤りのあるジョブをすべて報告して、終了コード 64 で終わる。ジョブは 1 つも起動せず、ステートにも触れない。分割は、この検査(`Ctx::check_definition`)の 1 回だけで、結果(`CommandLine` の並び)を、起動の失敗の表示(`report_execution` のコマンド名)と `--dry-run` の計画が使い回す |
| 順序 | シグナルの準備(`run`/`pipeline` のみ)→ 定義の読み込み(64)→ `command` の検査(64)→ ジョブID(68)→ run ID(68)→ ステートの読み込み(65)→ 照合(65) |
| 防御 | `run_job` は分割済みの `CommandLine` を受け取り、分け直さない(分けられない `command` は、型の上で `run_job` に届かない)。残る防御は 1 つ: `line.program` が空(空白だけを含む)なら、起動せず `StartFailed` を返す(`CommandLine` のフィールドは公開なので、呼び出し側が直接作れるため)。v0.9.5 までは `run_job` の内側でも分割し直し、失敗したら起動失敗を返した |

---

## 4. プロセスの起動(`run_job`)

### 4.1 手順(時系列)

`run_job(line, env, unset, signal, log) -> (JobExecution, LogOutcome)`(v0.10.0 以降。v0.9.7〜v0.9.9 は `run_job(line, env, signal) -> JobExecution`、v0.9.6 までは `run_job(job, line, env, signal)`。`line: &CommandLine` は、実行の前の検査で `check_command` が分割したコマンド。`env: &[(&str, &str)]` は、引き継いだ環境に足す環境変数(v0.9.0 以降)。`unset: &[&str]` は、足す前に引き継いだ環境から消す名前(v0.10.0 以降。`EffectiveEnv.removals`)。`log: Option<&LogTarget>` は、ログの置き場所の指定(v0.10.0 以降。`log_dir` があるときだけ `Some`)。v0.9.5 までは `run_job(job, env, signal)` で、中で分け直した。v0.8.0 以前は `run_job(job, signal)`)。失敗はすべて戻り値で返し、パニックしない。

1. 返す `JobExecution` の結果(`result: ExecutionResult`)は、下の手順で 3 つの場合のどれか 1 つに決まる(4.3)。v0.9.6 までは、`job` から写した `job_id` と `command` も持った(読まれなかった)。`LogOutcome` は、`log` が `None` のときと、下の 2・3 で返るときは空(3 つとも `None`)。
2. `line.program` が空(空白だけを含む)なら、`StartFailed` で返す(分割はしない。この確認は、次の合図の確認より先)。
3. **起動の前**に合図が立っていれば、起動せずに返す: `NotStarted { signal: 番号 }`。
   (ジョブは実行されない。ログのファイルも作らない。)
4. `std::process::Command::new(program)` に `args` を渡して作り、(v0.10.0 以降)`unset` の各名前を `Command::env_remove(name)` で引き継いだ環境から除き、続けて `env` の各 `(name, value)` を**並びの順に** `Command::env(name, value)` で足し
   (v0.9.0 以降。引き継いだ環境は残り、同じ名前は上書きされる。同じ名前が並びに 2 回あれば、後ろが勝つ)、`platform::configure_spawn` を適用する
   (UNIX: 新しいプロセスグループ。Windows: 何もしない)。`unset` の名前の比べ方は OS(標準ライブラリ)に任せる(8.1)。引き継いだ環境に無い名前は何もしない。
   (v0.10.0 以降)続けて、`log` が `Some` なら、起動の前に `open_log_file` でログのファイルを作る(4.5)。作れなければ、起動せず `StartFailed` と `LogOutcome.log_not_created = Some(パス)` で返す(分類で 127)。作れたら、標準出力と標準エラーをパイプにする(標準入力は引き継ぐ)。
5. `spawn()`。失敗なら `StartFailed` で返す(`log` があるときも、`LogOutcome` は空。作ったログのファイルは空のまま残る)。
   (v0.10.0 以降)`log` があるとき、起動のあとに、標準出力と標準エラーのそれぞれに Copier のスレッドを 1 つずつ始める(4.5)。
6. WaitLoop で終了を待つ(5 章)。時間切れ(タイムアウト)はない。自動のリトライもしない(リトライは cli の層で行う。07 の範囲外)。WaitLoop とシグナルの転送は、ログの有無によらず同じ。
7. 終了を観測したら、`Exited { exit_code, terminated_by_signal, interrupted_by }` を返す:
   - `exit_code = status.code()`。
   - `terminated_by_signal = platform::termination_signal(status)`(UNIX: シグナルで終了したときのシグナル番号。Windows: 常に `None`)。
   - **この時点で** 合図が立っていれば `interrupted_by = Some(番号)`、立っていなければ `None`。
8. `try_wait` が失敗したとき(終了の状態を取得できない)は、`StartFailed` で返す(分類で 127)。
9. (v0.10.0 以降)`log` があるとき、7・8 のあと(中断のときも)、Copier の終わりを、合わせて最大 2 秒待つ(4.5)。`LogOutcome` に、ファイルへの書き込みの失敗(`write_failed`)と待ち切れ(`output_cut_short`)を入れて返す。

### 4.2 起動の条件

| 項目 | 仕様 |
|------|------|
| シェル | 使わない。`sh -c` や `cmd /C` を挟まない(ソース検索のテストで、`"sh"`、`"cmd"`、`"bash"`、`"powershell"`、`"-c"`、`"/C"` の文字列と、`Command::new(&line.program)` 以外の `Command::new` がないことを確かめている) |
| プログラムの探索 | 標準ライブラリ(`std::process::Command`)の規則に従う(`program` に区切り文字がなければ、OS の標準の探索。`PATH` など)。conductor 独自の探索はしない |
| 作業ディレクトリ | 引き継ぐ(`conductor` を起動したときのカレントディレクトリ。定義ファイルの場所に移らない)。変更しない |
| 環境変数 | 引き継ぐ。**v0.8.0 以前**は、追加・削除・上書きをせず、run ID も渡さなかった。**v0.9.0 以降**は、引き継いだ環境に、`run_job` の `env` 引数(定義の `env`・`.env` と自動の変数 `CONDUCTOR_PIPELINE`・`CONDUCTOR_JOB_ID`・`CONDUCTOR_RUN_ID`(run ID があるとき)・`CONDUCTOR_ATTEMPT`。05 の 14 章)を**足す**(同じ名前は上書き)。**v0.10.0 以降**は、足す前に、`run_job` の `unset` 引数の名前(定義の `unset_env`。05 の 14 章)を、引き継いだ環境から `env_remove` で**消す**(`unset` が空なら何も消さない。v0.9.x までは取り除くことをしなかった)。値は、書いたとおりに渡す |
| 標準入力 | 引き継ぐ(`Stdio` を指定しない) |
| 標準出力・標準エラー | `log` が `None`(定義に `log_dir` が無い)なら、引き継ぐ(`Stdio` を指定しない)。ジョブの出力は、`conductor` を介さず、そのまま流れ、`conductor` は読まない。**v0.10.0 以降**、`log` が `Some`(`log_dir` がある)なら、パイプにして Copier が読み、読んだバイト列を、そのまま `conductor` の同じ流れ(標準出力は標準出力、標準エラーは標準エラー)とログのファイルへ書く(4.5)。どちらの場合も、ジョブの出力をステートには書かない |
| 引数の渡し方 | 分割した各引数を、そのまま OS に渡す(メタ文字は注入にならない)。Windows で `.cmd` / `.bat` を起動するときの引数の安全は、標準ライブラリのエスケープ(または起動の拒否)に任せる。どちらでも、続くコマンドは実行されない(Windows のテストで確かめる) |
| 起動できなかった場合 | `spawn` の失敗は、原因(見つからない・実行権限がない・実行できない形式・ディレクトリを指定した、など)を区別せず `StartFailed`。終了コードは 127(分類)。表示は `conductor: error: job '<ID>': cannot start command '<プログラム名>'`(英語)。`StartFailed` は、ジョブの終了コード・シグナルを持たない |
| 時間の上限 | ない。ジョブが終わらなければ、`conductor` は待ち続ける |
| ジョブの強制終了 | しない(SIGKILL を送る機能はない) |

### 4.3 `JobExecution` の各項目

| 項目 | 意味 |
|------|------|
| `result` | 起動の結果(`ExecutionResult`)。下の 3 つの場合のどれか 1 つ |
| `NotStarted { signal }` | 起動の前に合図が立っていた(リトライの待機中に立ったときも、cli がこの場合を作る)。`signal` は**最初に受け取った**シグナルの番号 |
| `StartFailed` | 空のプログラム名(`line.program` が空・空白だけ)、`spawn` の失敗、`try_wait` の失敗のいずれか(v0.9.5 までは、空のプログラム名の代わりに分割の失敗) |
| `Exited { exit_code, .. }` | `status.code()`。UNIX でシグナルによって終了したときは `None`。Windows は常に値がある。UNIX の終了コードは下位 8 ビット(300 を返すと 44)、Windows は 32 ビットのまま(300 のまま) |
| `Exited { terminated_by_signal, .. }` | UNIX のシグナル終了の番号。Windows は常に `None` |
| `Exited { interrupted_by, .. }` | 終了の観測の直後に合図が立っていたとき、**最初に受け取った**シグナルの番号 |

`JobExecution` が持つのは `result` だけ(v0.9.6 までは、読まれない `job_id` と `command` も持った)。組み立ての関数は `JobExecution::new(result)`、`exited(exit_code)`、`not_started(signal)`、`start_failed()`(v0.9.6 までは、どれも先頭に `job` を受け取った)。v0.9.5 までは、
`started`、`exit_code`、`terminated_by_signal`、`start_failure`、`interrupted`、`signal_number` の 6 つの項目で表した(あり得ない組み合わせも表せた)。

### 4.4 分類(終了コードの決定。`conductor-core` の `classify`)

`run_job` の結果は、次の優先順位で、最初に当てはまるものを採る。

| 順 | 条件 | 分類 | 報告する終了コード |
|----|------|------|--------------------|
| 1 | `NotStarted { signal }` | 分類なし(エラーとして記録しない) | `128 + signal` |
| 2 | `Exited { interrupted_by: Some(n), .. }`(起動済みの中断) | エラー | `128 + n`(ジョブ自身の終了コードとシグナルは見ない。ジョブが 0 で終わっていてもエラー) |
| 3 | `StartFailed` | エラー | 127 |
| 4 | `Exited { terminated_by_signal: Some(n), .. }` | エラー | `128 + n` |
| 5 | `Exited { exit_code: None, .. }`(起動して終わったのに値がない) | エラー(`conductor_error`) | 1 |
| 6 | `exit_code` が成功コード(既定は `[0]`、`success_exit_codes` で指定) | 成功 | 0 |
| 7 | `exit_code` が `warning_exit_codes` | 警告 | 0 |
| 8 | `exit_code = 0` で成功コードに含まれない | エラー(`conductor_error`) | 1 |
| 9 | 上記以外 | エラー | `exit_code` のまま |

`conductor` の終了コードは、報告する終了コードを 0〜255 に合わせたもの(範囲内はそのまま。範囲外は下位 8 ビット。エラーで
下位 8 ビットが 0 になるときは 1)。

### 4.5 ログのファイル(`log_dir`。v0.10.0 以降)

定義の `[pipeline].log_dir`(03 の 6.9)があるとき、cli(`step.rs` の `run_job_with_retry`)は、試行ごとに `LogTarget` を作って `run_job` に渡す。

| `LogTarget` の項目 | 値 |
|--------------------|----|
| `dir` | 解決した `log_dir`(相対なら (定義ファイルのディレクトリ)/(`log_dir`)、絶対ならそのまま。`~` と変数は展開しない。`step.rs` の `resolve_log_dir`) |
| `job_id` | 定義のジョブ ID |
| `started_at` | その試行の前に時計から取った時刻(試行の `started_at` と同じ値。05 の 18 章) |
| `attempt` | 1 から数える試行の番号(`CONDUCTOR_ATTEMPT` と同じ。再開で 1 に戻る) |

**ファイルの作成**(`open_log_file(target) -> Result<(File, PathBuf), PathBuf>`):

1. `dir` を、親を含めて作る(`create_dir_all`。権限は既定のまま)。失敗したら、番号なしの名前のパスで `Err`。
2. 名前 `<安全な ID>.<詰めた時刻>.<試行の番号>.log` で、「無いときだけ作る」(`create_new`)で書き込み用に開く。UNIX 系は、作るときに権限 `0600` を付ける(`platform::restrict_to_owner`。Windows は何もしない)。既存のファイルを上書きも追記もしない。
3. 「すでにある」なら、`<安全な ID>.<詰めた時刻>.<試行の番号>-2.log`、`-3` … `-1000` と番号を足して作り直す。ほかの失敗は、その名前のパスで `Err`。`-1000` まで使い切ったら、番号なしの名前のパスで `Err`。
4. `<詰めた時刻>` は `format_compact(started_at)`(`YYYYMMDDTHHMMSSZ`。例 `20261125T091248Z`。UTC)。
5. `conductor` は、ログのファイルを消さない(`reset` も触れない)。

**安全な ID**(`safe_log_stem(job_id)`。どの OS でも同じ結果。ディレクトリの区切りは残らない):

1. `/` `\` `:` `*` `?` `"` `<` `>` `|` と制御文字(U+0000〜U+001F、U+007F〜U+009F)を `_` にする。
2. 先頭の `.` を `_` にする(隠しファイルや `..` を作らない)。
3. 最初の `.` より前の部分から前後の ASCII の空白(` `)を除いた値が、大文字小文字を問わず Windows の予約名(`CON`、`PRN`、`AUX`、`NUL`、`COM1`〜`COM9`、`LPT1`〜`LPT9` の 22 個)なら、先頭に `_` を足す(`NUL.x` → `_NUL.x`、`con` → `_con`、`con ` → `_con_`、`nul .tar.gz` → `_nul .tar.gz`)。空白を除くのは比べるときだけで、名前そのものからは除かない(v0.10.1 以降。v0.10.0 は空白を除かずに比べた)。
4. 100 バイトまでに切る(UTF-8 の文字の途中では切らない)。
5. 末尾の `.` か空白を `_` にする(Windows が名前から落とすもの)。

それ以外(日本語など)はそのまま。名前の全体は、最長でも 255 バイトを超えない。

**出力の写し**(Copier。`copy_stream`):

- 標準出力と標準エラーのそれぞれに、1 つの読み取りのスレッド(名前 `conductor-log-copier`)を作る。8 KiB ずつ読み、読んだバイト列を、そのまま(`conductor` は何も書き足さない)端末(`conductor` の同じ流れ)とログのファイルへ書く。端末へは書くたびに flush する。2 つの流れは、1 つの鍵(`Mutex`)で順にファイルへ書く(1 つのファイルに混ざる)。出力の全体を溜めない。
- 端末への書き込みに失敗したら、端末へ書くのをやめるが、読み続けてファイルへは書く。
- ファイルへの書き込みに失敗したら、以後ファイルには書かず、`write_failed` の印を立てる(端末へは書き続ける)。スレッドを作れなかったときも、その流れの出力をファイルに残せないので、同じ印を立てる。
- 子の終了を観測した後(中断のときも)、読み取りのスレッドの終わりを、合わせて最大 **2 秒**(`DRAIN_LIMIT`)待つ。待ち切れたら(子の子がパイプを開いたまま動き続けるなど)、「止める」印を立て、`output_cut_short` とする。
- 「止める」印は、**読む前**と**読んだ後**に見る(v0.10.1 以降は読む前にも見る)。読む前に立っていれば読まずに終わり、読んだ後に立っていれば、読んだ分をどこにも書かずに終わる。待ち切れたあとに、端末とファイルへ書き足さない。

**`LogOutcome`**(3 つとも、起きたときに、そのログのファイルのパスを持つ。`log` が `None` なら、すべて `None`):

| 項目 | 起きたとき | 表示(08 の 4.14) |
|------|------------|--------------------|
| `log_not_created` | 起動の前に、ディレクトリかファイルを作れなかった(ジョブは起動していない。`StartFailed`、127) | 起動の失敗の 1 行の代わりに、ログを作れなかった 1 行 |
| `write_failed` | 実行中にファイルへの書き込みに失敗し、以後ファイルに書かなかった | 試行の後に警告の 1 行(終了コードは変えない) |
| `output_cut_short` | 子の終了の後、出力の読み取りを待ち切れなかった | 試行の後に警告の 1 行(終了コードは変えない) |

- `validate`・`graph`・`--dry-run` は、ジョブを起動しないので、ディレクトリもファイルも作らない。

---

## 5. 待ち(WaitLoop)

- 専用のスレッドは使わない。`run_job` を呼んだスレッドで、`try_wait` を次の間隔でくり返す。

| 回 | 待ち時間(ミリ秒) |
|----|------------------|
| 1 回目の後 | 1 |
| 2 | 2 |
| 3 | 4 |
| 4 | 8 |
| 5 回目以降 | 10(上限) |

  (前回の 2 倍、上限 10。)
- ジョブを**回収する**(`try_wait` が終了を返す)のは、このくり返しの中だけ。
- 各回の手順: ① `try_wait`。終了なら、その結果を返す。失敗なら `WaitFailed` を返す。② まだ終わっていなければ、
  **まだ転送していない**かつ合図が立っていれば、`forward(pid, シグナル番号)` を **1 回だけ**呼ぶ。③ 間隔だけ眠る。
- 転送の判断は、直前の `try_wait` が「まだ終わっていない」を返した回に限る(終了済み・回収済みのプロセスの pid には送らない)。
  したがって、ジョブが先に終わっていれば、合図が立っていても転送しない。
- 転送は 1 回だけ。2 回目以降の合図(同じシグナルでも別のシグナルでも)で、再度の転送や強制終了はしない。
- 転送した後は、ジョブの終了を待ち続ける(ジョブがシグナルを無視すれば、`conductor` はそのまま待ち続ける)。

---

## 6. 中断の合図(`InterruptSignal`)

| 項目 | 仕様 |
|------|------|
| 実体 | `AtomicI32` 1 つ。未設定を表す予約値 `i32::MIN`、それ以外は最初のシグナルの番号 |
| `new()` | 立っていない |
| `raise(n)` | 立っていなければ `n` を保存(compare-exchange)。**すでに立っていれば何もしない**(最初の番号が残る)。`n == i32::MIN` のときは `i32::MIN + 1` を保存する(立ったことは変わらない) |
| `signal_number()` | 立っているなら最初の番号(`Some`)、立っていなければ `None` |
| 性質 | 立ったら下りない(レベルで持つ)。スレッド間で共有できる(`Sync`)。`cli` は `Arc` で包んで SignalWatcher のスレッドに渡し、runtime は読むだけ |

番号は、`run_job` の転送の許可集合にない値でも(負数や `i32::MAX` を含め)、立ったことは記録され、`NotStarted { signal }` の `signal` か `Exited` の
`interrupted_by` に、そのまま入る。

---

## 7. シグナル処理

### 7.1 準備(両 OS 共通)

- `run` と `pipeline` だけが、**定義の検査より前**に、シグナルの監視を始める(`Ctx::prepare_session` の最初の `start_signals`。v0.9.6 までは `Ctx::prepare_run`・`Ctx::prepare_pipeline`)。
  `watch`・`reset`・`validate`・`convert` は監視せず、OS の既定の動作で終わる。v0.9.0 以降の `graph`・`init` も監視しない。
  ただし、v0.10.0 以降の `watch --follow` だけは、中断の合図で 0 で終わるために、`run`・`pipeline` と同じく**ほかの検査より先**に監視を始める(`run_watch` の最初の `Ctx::start_signals`。失敗は 70。7.5)。`--follow` の無い `watch` は、今までどおり監視しない。
  `run`・`pipeline` の `--dry-run` も、監視を始めない(ジョブを起動しないので、中断の合図を待つものがない。`prepare_session` の代わりに、シグナルを除く `prepare_checked` を呼ぶ。検査の並び `check_job`・`check_pipeline` は本番と共有する)。
- `InterruptSignal` を 1 つ作り、`Arc` で SignalWatcher に渡す。
- 監視の登録(`Signals::start`)を呼ぶのは **プロセスで 1 回**(`prepare_session` の 1 か所。1 回の `run_cli` は
  1 つのコマンドだけを実行し、`main` は `run_cli` を 1 回だけ呼ぶ)。2 回呼んだときの動きは契約の外(UNIX 系は監視が
  2 つになり、Windows は `Failed`)。登録に失敗したとき(`Failed`)は、標準エラーに
  `conductor: error: failed to set up signal handling` を書き、終了コード **70** で終わる(定義の検査もジョブの起動もしない)。
- 「最初の番号」は、`raise` が呼ばれた順の最初。

### 7.2 UNIX 系(Linux、macOS)

実装: `signal-hook`(`default-features = false`、`iterator` 機能)の `Signals` を、`conductor-signals` という名前の専用スレッドで
`forever()` で回す。受け取るシグナルは **SIGHUP(1)、SIGINT(2)、SIGQUIT(3)、SIGTERM(15)** の 4 つ。受け取るたびに
`signal.raise(番号)` を呼ぶ。`low_level` や `unsafe` は使わない。登録後、これらのシグナルでは、OS の既定の動作(終了)をしなくなる。

時系列(ジョブの実行中に、利用者が `kill -TERM <conductor の pid>` したとき):

1. `conductor` が SIGTERM を受け取る。監視のスレッドが `raise(15)`。合図が立つ。
2. メインのスレッドは、WaitLoop の次の `try_wait`(最大 10 ms 後)で「まだ終わっていない」を観測し、合図が立っているのを見る。
3. `platform::forward_signal(ジョブの pid, 15)` を **1 回**呼ぶ。内部では次を検査して `libc::kill(pid, 15)` を呼ぶ。
   - pid が 0 より大きく `i32`(`pid_t`)に収まる(0 や負にならない。プロセスグループへの送信にならない)。
   - シグナルが `{SIGHUP, SIGINT, SIGQUIT, SIGTERM}` のどれか(`SIGKILL`、`SIGSTOP`、`SIGCONT`、`SIGUSR1`、0、負、9999 は送らない)。
   - 送信先は、**ジョブの子プロセスの pid だけ**。ジョブの子孫(`sh -c` が起こした孫など)には、`conductor` は送らない。
   - `kill` の失敗は無視する。
4. ジョブは、受け取ったシグナルの既定の動作で終了するか、自分で処理する。`conductor` は待ち続ける。2 回目以降のシグナルでは、何も追加しない。
5. ジョブの終了を観測する(回収する)。`exit_code`(シグナル終了なら `None`)と `terminated_by_signal` を読み、直後に合図が立っているので
   `interrupted_by = Some(15)` を足す(結果は `Exited`)。
6. 分類で「エラー、`128 + 15 = 143`」となり、Orchestrator はこの結果をステートに**エラーとして**記録する(再開位置はそのジョブ)。
7. `conductor` は **143** で終わる(`pipeline` は次のジョブを起動しない)。

その他:

- ジョブは、`conductor` が起動時に作る **新しいプロセスグループ**(`process_group(0)`。グループ ID は子の pid)で動く。端末の
  Ctrl+C(SIGINT)は、フォアグラウンドのプロセスグループに配られるため、ジョブには直接届かず、`conductor` が受けて 1 回転送する
  (SIGINT のとき、ジョブは `terminated_by_signal = 2` で終わるのが典型)。
- 転送の許可集合にない番号で合図が立った場合(runtime を直接使う場合のみ起こりうる。`conductor` 本体の監視は 4 つだけを受ける)は、
  送信せず、ジョブが自分で終わるのを待ち、終わりを観測した後に `interrupted_by = Some(その番号)` を足す
  (ジョブの終了コードは `exit_code` に残るが、分類では見ない)。
- ほぼ同時に複数のシグナルが届いたとき、`signal-hook` が **番号の小さい順**に返すため、「最初の番号」は到着順でなく小さい方になりうる
  (例: INT(2)と HUP(1)なら HUP の 1)。時間を空けて届いたときは、到着順(INT のあとの HUP でも 2 が残る)。
- 終了コードの例: SIGHUP は 129、SIGINT は 130、SIGQUIT は 131、SIGTERM は 143。

### 7.3 Windows

実装: `ctrlc` クレートの `set_handler`(ライブラリが専用のスレッドでハンドラを呼ぶ)。ハンドラは `signal.raise(2)`
(Ctrl+C の番号 2)。`set_handler` の失敗は、種類によらず `Failed` にする。

時系列(ジョブの実行中に、利用者が Ctrl+C を押したとき):

1. ジョブは `conductor` と **同じコンソール**で起動されている(`configure_spawn` は何もしない。新しいプロセスグループを作らない)。
   Ctrl+C は、OS が、同じコンソールを共有するプロセス(`conductor` とジョブ)の両方に配る。
2. `conductor` のハンドラが `raise(2)`。
3. WaitLoop は転送を試みるが、Windows の `forward_signal` は **何もしない**(追加の送信をしない)。
4. ジョブは、OS から受け取った Ctrl+C で(既定の動作で)終了する。
5. 終了を観測する。`termination_signal` は常に `None`、`exit_code` は OS の終了コード(Ctrl+C による終了コード)。直後に合図が立っているので
   `interrupted_by = Some(2)`。
6. 分類で「エラー、`128 + 2 = 130`」。ステートに**エラーとして**記録し、**130** で終わる。

その他:

- Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(既定の動作)。`ctrlc` が Ctrl+Break を受け取るかは確かめていない(未確認。
  確かめたら記録する、とチェックリストにある)。
- CI の Windows ランナーには対話的なコンソールがなく、Ctrl+C を安定して送る方法がないため、この経路は自動テストにしない。リリース前に
  実機で手動確認する(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`。確認項目: 終了コード 130、ジョブの子プロセスが残らない、
  `watch` で中断したジョブが `error` で次が `pending`、`run -v` の表示に `interrupted by signal 2`)。チェックリストの記録欄は、このリポジトリでは
  空(確認の記録なし)。

### 7.4 合図が立つ時点ごとの動作(Orchestrator)

| 合図が立った時点 | 動作 | ステートの記録 | 終了コード |
|------------------|------|----------------|------------|
| `run_job` の呼び出しの前(`NotStarted`) | ジョブを起動しない | 分類なし。エラーとしては記録しない。再開位置はそのジョブ(`apply_outcome` が、起動していない扱いで記録する) | `128 + 番号` |
| ジョブの実行中 | 1 回転送(UNIX)。ジョブの終了を待つ | そのジョブを**エラー**として記録(ジョブが 0 で終わっても) | `128 + 番号` |
| ジョブの終了の観測の直後(転送が間に合わなかった場合を含む) | 起動済みで `Exited` に `interrupted_by`(上の行と同じ扱い) | エラー | `128 + 番号` |
| `pipeline` のジョブとジョブのあいだ(ジョブの結果は成功か警告で、完了ではない) | 次のジョブを起動しない | `resume_point_on_interrupt`(記述順で、まだ起動していない最初のジョブ。エラーのジョブが先にあればそれ)を再開位置にして、中断を記録。エラーとは記録しない。`-v` で `interrupted by signal N` | `128 + 番号` |
| `pipeline` の最後のジョブが完了した直後(`completed`) | 完了の処理(`--remove-on-complete` ならステートを削除)をして終わる | 完了 | 0(合図は無視される) |
| リトライの待機中(v0.8.0 以降) | 待機を打ち切る。`NotStarted { signal: 番号 }` の実行結果を作る。次の実行はしない | 分類なし、再開位置はそのジョブ。エラーとしては記録しない | `128 + 番号` |
| 定義の検査・ステートの読み込み・照合の途中 | 検査は最後まで進める | (定義の誤り 64、ステートの誤り 65、拒否 66、スキップ 0 の終了コードが、中断の終了コードより優先する) | 各検査の値 |

### 7.5 リトライの待機(`wait_for`)

- `wait_for(secs, signal) -> Elapsed | Interrupted(番号)`。標準ライブラリだけで実装し、OS 固有のコードを持たない。
- 手順: ループで ①合図が立っていれば、すぐ `Interrupted(最初の番号)`。②開始からの経過が `secs` 秒以上なら `Elapsed`。③ `min(残り時間, 50 ms)` 眠る。
- 合図は、待機の開始時と、50 ms 以下の間隔で確かめる。長い待ちでも、合図から 1 秒以内に返る(テスト: 3600 秒の待機に、200 ms 後に合図を立てて、全体が 1.2 秒未満)。
- `secs = 0` は、合図が立っていなければ、すぐ `Elapsed`。合図が立っていれば `Interrupted`。
- 複数の合図は、最初の番号を返す。

### 7.6 `watch --follow` の待ち(v0.10.0 以降)

- `watch --follow` は、1 回描くたびに、cli の `Runtime::wait`(本番は `wait_for`)で待つ。待つ秒は、ステートが有る・無いときは `--interval` の値(1〜3600。既定 2 秒)、ステートが読めない(`unreadable`)ときは、間隔によらず 1 秒。
- 各回の初めに合図が立っていれば、読まずに終わる。待ちが `Interrupted` なら終わる。どちらも終了コード 0(08 の 6 章、10 の 1 章)。
- ジョブを起動しないので、シグナルを転送する相手はいない。合図は、表示し直しを止めるためだけに使う。
- 終わりの判定(完了・止まった・消えた)は 05 の 20 章、表示は 08 の 6 章。

---

## 8. プラットフォーム差と OS 分岐の規則

### 8.1 差の一覧

| 項目 | UNIX 系(Linux、macOS) | Windows |
|------|------------------------|---------|
| ジョブのプロセスグループ | 新しいグループ(`process_group(0)`) | 指定しない(同じコンソール) |
| 監視するシグナル | SIGHUP・SIGINT・SIGQUIT・SIGTERM(`signal-hook`) | Ctrl+C のみ(`ctrlc`。番号 2) |
| ジョブへの転送 | `libc::kill(子の pid, シグナル)` を 1 回(許可集合のみ) | しない(OS が Ctrl+C を配る) |
| シグナル終了の読み取り | `ExitStatusExt::signal()` | なし(`None`。終了はすべて終了コード) |
| 終了コードの範囲 | 下位 8 ビット | 32 ビットのまま(`conductor` が報告する値は、0〜255 に合わせる) |
| ステート書き込み後のディレクトリの同期 | `File::open(dir)` して `sync_all`。失敗は無視。親が空なら `.` | しない |
| 端末の色 | 使える(`color_supported() = true`) | 使わない(色なし) |
| `unset_env` の名前の比べ方(v0.10.0 以降) | `env_remove` は完全一致(大文字小文字を区別する) | `env_remove` は大文字小文字を区別しない(標準ライブラリの規則) |
| ログのファイルの権限(v0.10.0 以降) | 作るときに `0600`(`platform::restrict_to_owner`) | 指定しない(既定の ACL) |
| `watch --follow` の画面(v0.10.0 以降) | 標準出力が端末なら、毎回 `ESC[H ESC[2J` で画面を消して描く(`platform::clear_screen_supported() = true`) | 消さない。2 回目から空の 1 行を前に置いて追記する |
| `unsafe` | `platform/unix.rs` の `libc::kill` 1 か所だけ | なし |
| 依存クレート | `libc`(runtime)、`signal-hook`(cli) | `ctrlc`(cli) |
| デモ・テスト | `demo/run-demo.sh`、`binary_signal.rs`、`signals_*.rs`、`runner_signal.rs` | `demo/windows/run-demo.cmd`、`windows_demo.rs`、手動のチェックリスト |

### 8.2 分岐を置く場所の規則(ソース検索のテストが強制)

- `conductor-runtime`: `cfg(unix)`、`cfg(windows)`、`cfg!(...)`、`target_os`、`target_family`、`os::unix`、`os::windows`、`libc::` は、
  `src/platform/` の外にない。
- `conductor-cli`: `cfg(unix)`、`cfg(windows)`、`cfg!(unix)`、`cfg!(windows)`、`target_os`、`cfg(not(unix))`、`cfg(not(windows))` は、
  `src/platform/` の外にない(`color_supported` の `cfg!(unix)` も `platform/mod.rs` の中)。単体テスト `src/tests/`(`lib.rs` の最後の
  `#[cfg(test)] mod tests;` からだけ読み、本番の成果物に入らない)は、この検査と `panic!(` の検査の外(`unsafe`・ネットワークの検査は対象)。
- `unsafe`: runtime は `#![deny(unsafe_code)]`、許す場所は `platform/unix.rs` の `#![allow(unsafe_code)]` だけで、`unsafe` の呼び出しは 1 回
  (`libc::kill`)。cli(ライブラリと `main.rs` とテスト補助)は `#![forbid(unsafe_code)]` で、`unsafe` と `low_level` の文字列もない。
- `src/` に、`std::net`、`TcpStream`、`TcpListener`、`UdpSocket`、`reqwest`、`hyper`、`ureq`、`tokio`、`curl`、`openssl` がない
  (ネットワークに接続しない)。
- runtime の製品のコード(各ファイルの `#[cfg(test)]` より前)に、`.unwrap()`、`.expect(`、`panic!(`、`unreachable!(`、`todo!(`、`unimplemented!(` がない。

---

## 9. パニックのフック(内部の誤り)

- `main` の最初に `install_panic_hook()`。パニックが起きると、標準エラーに固定の英語の 1 行 `conductor: error: internal error`
  (ロケールを判定しない)を書き、終了コード **70** で `std::process::exit` する。標準エラーへの書き込みが失敗しても 70 で終わる。
- `main` は、`run_cli(...)` の結果で 1 回だけ `std::process::exit` を呼ぶ。`main.rs` は `#![forbid(unsafe_code)]`、パニックを起こすコマンドを持たない。
- ジョブの実行中にパニックすると、子プロセスの終了を待たずに終わり、ジョブが孤児として残りうる。ステートは直前の完全な内容のままで、
  次の実行はそのジョブを再実行しうる(at-least-once の範囲)。
- `conductor` が SIGKILL などで強制終了されたときも、ジョブは残って動き続けうる(検知も停止もしない)。

---

## 10. 検証の観点(この文書の挙動を確かめるテスト)

| 確かめること | テスト |
|--------------|--------|
| 分割の全規則(3 章の表) | `crates/runtime/src/split.rs` の単体テスト |
| メタ文字が引数にそのまま渡る、配列の要素がそのまま渡る、環境変数と作業ディレクトリの引き継ぎ、起動失敗、終了コード(0・3・255・300) | `crates/runtime/tests/runner_basic.rs` |
| `env` 引数が子の環境に足される(足した値が子に見える、引き継いだ環境が残る、同じ名前の上書き、並びの後ろが勝つ。v0.9.0 以降) | `crates/runtime/tests/runner_basic.rs` |
| 起動前の合図で起動しない(`NotStarted`)、終了の観測の直後の合図で `interrupted_by` | 同上 |
| SIGTERM / SIGINT の転送、許可外の番号は送らない、新しいプロセスグループ、転送は 1 回だけ、先に終わったジョブは転送なし | `crates/runtime/tests/runner_signal.rs`(UNIX) |
| 待ちの間隔(1,2,4,8,10,10)、転送 1 回 | `crates/runtime/src/runner.rs` の単体テスト |
| `wait_for` の 1 秒以内の打ち切り | `crates/runtime/tests/wait_interruptible.rs` |
| 実バイナリで SIGTERM → 143、ステートに再開位置、孤児なし。HUP のあとの INT → 129 | `crates/cli/tests/binary_signal.rs`(UNIX) |
| 監視の登録(最初の番号が残る) | `crates/cli/tests/signals_real.rs`、`signals_order.rs`(UNIX。各ファイルにテストは 1 つだけで、登録は 1 回だけ。登録がプロセスで 1 回のため) |
| Windows の Ctrl+C | 手動(`WINDOWS_CTRL_C_CHECKLIST.md`) |
| `cmd` への危険な引数が注入にならない | `runner_basic.rs` の Windows 専用の 3 テスト |
| (v0.10.0 以降)`unset` の名前が子の環境から消え、`env` の値は足される | `crates/runtime/tests/runner_basic.rs` |
| (v0.10.0 以降)ログのファイルの名前・番号・0600・安全な ID・ディレクトリの作成 | `crates/runtime/tests/log_file.rs`、`crates/runtime/src/log.rs` の単体テスト |
| (v0.10.0 以降)出力が端末とファイルの両方に流れる、書き込みの失敗、2 秒の待ち切れ、読む前の「止める」印 | `crates/runtime/tests/runner_log.rs`、`crates/runtime/src/runner.rs` の単体テスト |
| (v0.10.0 以降)実バイナリでのログのファイルと表示 | `crates/cli/tests/binary_log.rs` |

---

## 11. 実装と README の差

README の「command の書き方」「シグナルの扱い」と実装は、大筋で一致する。README が書いていない、または簡略に書いている点(実装の挙動を正とする):

| 項目 | README | 実装 |
|------|--------|------|
| 区切りの文字 | 「空白」 | スペース、タブ、LF、CR の 4 つだけ。ほかの空白(U+00A0 など)は普通の文字 |
| 二重引用符の中のバックスラッシュ | 「中の `\"` だけが二重引用符 1 つになる」 | 左から読む状態機械。`\\` は、1 つ目を出力して 2 つ目を保留にするため、`\\"` は「`\` + エスケープされた `"`」。`\\` の連続は 1 つに縮まらない(そのまま 2 つ) |
| 引用符の外・一重引用符の中のバックスラッシュ | 記載なし | 普通の文字 |
| 配列の先頭の要素 | 「空または空白だけは定義の誤り」 | 起動の側も二重に検査する(`check_command` は `Empty` の誤り。`run_job` は空・空白だけの `program` を起動失敗)。空白を含む先頭の要素(`"my prog"`)は、そのままファイル名として起動する。文字列の先頭の語(`'  '`)も、v0.9.8 以降は配列と同じく空白だけなら `Empty`(v0.9.7 までは起動の失敗 127) |
| シグナルの転送先 | 「実行中のジョブ(の子プロセス)」 | ジョブの子プロセスの pid だけ。プロセスグループ全体や孫には送らない(README は孫に届かないことを明記しない) |
| 起動できなかった場合の 127 | 「ジョブを起動できなかった」 | `spawn` の失敗に加え、`try_wait` の失敗(終了の状態を取得できない)も、起動失敗として 127 にする |
| 許可外の番号 | 記載なし | `run_job` に許可外の番号の合図が来ても送信しない。ただし `conductor` 本体は 4 つのシグナルだけを受けるため、通常は起こらない |
| ジョブが 0 で終わったが合図が立っていたとき | 「ジョブの結果を、エラーとしてステートに記録」 | 終了の観測の直後に合図が立っていれば、ジョブの終了コードが 0 でもエラー(`128 + 番号`)。完了済みの最後のジョブの後に立った合図は無視(0 で終わる) |
| ジョブの出力 | 「ジョブ本体の出力が、そのまま流れます」 | 標準入力・標準出力・標準エラー・環境変数・作業ディレクトリをすべて引き継ぐ(明記は一部)。v0.9.0 以降は、環境変数に、定義の `env`・`.env`・自動の変数を足す(README の「環境変数の設定」の節)。v0.10.0 以降は、足す前に `unset_env` の名前を消し、`log_dir` があれば標準出力と標準エラーをパイプで読んで、端末とログのファイルの両方へ流す(README の「ジョブの出力をファイルに残す」の節) |

## 12. 未確認の事項

- Windows での実機の動作(Ctrl+C の終了コード 130、ジョブの終了の様子、`.cmd` の起動)は、このリポジトリのチェックリストの記録欄が空で、ここでは確認していない。
- `ctrlc` が Windows で Ctrl+Break を受け取るか(README は対象外と明記。チェックリストも未確認)。
- UNIX で、ジョブが新しいプロセスグループにあるため、ジョブが端末から読む(標準入力が端末のとき)場合に、背景グループとして停止するかは、コードとテストから確認できない(テストは標準入力を `/dev/null` にして実行している)。
- Windows の `Command::new` のプログラム探索(カレントディレクトリを含むか等)の詳細は、標準ライブラリの実装に依存し、確認していない。
- この調査では、テストやビルドは実行していない(ソースとテストの読み取りに基づく)。
