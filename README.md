# rust-job-conductor — Rust製のジョブ実行制御ツール

`conductor` は、既存のジョブスケジューラ(cron など)から呼び出される、ジョブの流れの制御ツールです。
定義ファイルに、ジョブと、結果(成功・警告・エラー)ごとの次の動きを書いておくと、`conductor` が、
ジョブを記述順に実行し、遷移の規則に従ってスキップや終了を行い、実行の状態(ステート)を覚えておきます。
エラーで止まったあとは、エラーのジョブから再開できます。

- 単一の実行ファイル(`conductor`)です。外部のランタイムは要りません。Linux・macOS・Windows で、同じ定義に、同じ結果を返します。
- ネットワークには接続せず、利用状況の収集もしません。
- 表示は、環境のロケールに従って、英語と日本語を切り替えます。

## 目次

- [インストール](#インストール)
  - [ソースからのビルド](#ソースからのビルド)
- [クイックスタート](#クイックスタート)
- [使い方](#使い方)
- [定義ファイルの書き方](#定義ファイルの書き方)
  - [リトライの設定(`[job.retry]`)(v0.8.0 以降)](#リトライの設定jobretryv080-以降)
- [ステートファイルの場所](#ステートファイルの場所)
  - [run ID でステートを分ける(v0.5.0 以降)](#run-id-でステートを分けるv050-以降)
- [終了コードの表](#終了コードの表)
- [保証の範囲(at-least-once)](#保証の範囲at-least-once)
- [シグナルの扱い](#シグナルの扱い)
- [convert の使い方](#convert-の使い方)
- [ロケールと言語](#ロケールと言語)
- [ライセンス](#ライセンス)

## インストール

### 1. ダウンロード

[GitHub Releases](https://github.com/agwlvssainokuni/rust-job-conductor/releases) から、お使いの OS 向けのファイルと、`SHA256SUMS` をダウンロードします。

| OS | ファイル |
|----|----------|
| Linux(x86_64) | `conductor-<版>-x86_64-unknown-linux-musl.tar.gz` |
| macOS(Apple シリコン) | `conductor-<版>-aarch64-apple-darwin.tar.gz` |
| macOS(Intel) | `conductor-<版>-x86_64-apple-darwin.tar.gz`(クロスビルドで作ったもので、動作の確認はしていません) |
| Windows(x86_64) | `conductor-<版>-x86_64-pc-windows-msvc.zip` |

ファイル名の `<版>` は、`v` を付けない版の数字(例: `0.7.0`)です。`<版>` のあとの部分(例: `x86_64-unknown-linux-musl`)を、この README では `<対象>` と書きます。

### 2. SHA-256 の確認

ダウンロードしたファイルが、転送の途中で壊れていないことを、`SHA256SUMS` のチェックサムで確かめます。

```sh
# Linux
sha256sum --check --ignore-missing SHA256SUMS
# macOS
shasum -a 256 --check --ignore-missing SHA256SUMS
```

```powershell
# Windows(PowerShell): 表示された値が、SHA256SUMS の、該当する行の値と同じことを確かめる
Get-FileHash .\conductor-<版>-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

チェックサムが守るのは、**転送の破損の検知まで**です。配布元の改ざんは防げません(この版は、署名をしていません)。

### 3. 展開と PATH

ファイルを展開すると、最上位に `conductor-<版>-<対象>/` のディレクトリができ(zip も同じです)、その中に `conductor`(Windows は `conductor.exe`)と `LICENSE` があります。ディレクトリの中の `conductor` を、`PATH` の通ったディレクトリ(たとえば `/usr/local/bin`)に置きます。

```sh
tar xzf conductor-<版>-<対象>.tar.gz
install -m 755 conductor-<版>-<対象>/conductor /usr/local/bin/conductor
conductor --version
```

### macOS での警告の回避の手順(Gatekeeper)

この版の実行ファイルは、コードサイニングも公証もしていません。ダウンロードしたファイルを初めて実行すると、macOS の Gatekeeper が、「開発元を確認できません」と警告し、実行を止めることがあります。次のどちらかで回避できます。

- Finder で `conductor` を右クリックして「開く」を選び、確認のダイアログで「開く」を選ぶ(1 回だけ)。
- ターミナルで、ダウンロードの印(隔離属性)を外す。

  ```sh
  xattr -d com.apple.quarantine /usr/local/bin/conductor
  ```

### ソースからのビルド

必要なものは、Rust(1.100 以降)と `git` です。

```sh
git clone https://github.com/agwlvssainokuni/rust-job-conductor.git
cd rust-job-conductor
cargo build --release --locked --bin conductor
```

できた実行ファイルは、`target/release/conductor`(Windows は `target\release\conductor.exe`)です。ビルドしたあとの確認には、テストが使えます。

```sh
cargo test --workspace --locked
```

`PATH` の通ったディレクトリに置くには、次のどちらかを使います。

```sh
# ビルドした実行ファイルを置く
install -m 755 target/release/conductor /usr/local/bin/conductor
# または、cargo で入れる(~/.cargo/bin/conductor ができる)
cargo install --path crates/cli --locked --bin conductor
```

## クイックスタート

3 つのジョブを順に動かし、途中で失敗させて、失敗したジョブから再開するまでを、確かめます。`conductor` は、インストールして `PATH` に置いたものとします。

作業用のディレクトリを作り、次の内容の `conductor.toml` を保存します。`step2` は、環境変数 `FAIL` の値の終了コードで終わります(未設定なら 0 です)。

```sh
mkdir hello-conductor && cd hello-conductor
```

```toml
format_version = 1

[pipeline]
name = "hello"

[[job]]
id = "step1"
command = ["sh", "-c", "echo step1"]

[[job]]
id = "step2"
command = ["sh", "-c", "echo step2; exit ${FAIL:-0}"]

[[job]]
id = "step3"
command = ["sh", "-c", "echo step3"]
```

Windows では、`command` を `cmd /c` で動く形に書き換えてください(例: `["cmd", "/c", "echo step1"]`)。環境変数は、`set FAIL=3` で渡します。

以下は、`LANG=C` で実行したときの出力です(`[conductor]` で始まる行は標準エラー、それ以外はジョブの出力と、`watch` の表です)。日本語のロケールでは、`conductor` 自身の表示が日本語になります(「ロケールと言語」を参照)。

1. 定義を検査します(ジョブは実行しません)。何も表示せず、終了コードは 0 です。

   ```text
   $ conductor validate
   $ echo $?
   0
   ```

2. `step2` を失敗させて、パイプラインを実行します。`step2` の終了コード 3 が、そのまま `conductor` の終了コードになり、`step3` は実行されません。

   ```text
   $ FAIL=3 conductor pipeline -v hello
   step1
   [conductor] job=step1 result=success (exit code 0)
   step2
   [conductor] job=step2 result=error (exit code 3)
   $ echo $?
   3
   ```

3. ステートを見ます。止まった位置(`step2` の `error`)が残っています。

   ```text
   $ conductor watch
   Pipeline: hello
   JOB    STATUS   NOTE
   step1  success  exit code 0
   step2  error    exit code 3
   step3  pending
   ```

4. 同じコマンドを、`FAIL` なしで、もう一度実行します。成功済みの `step1` は飛ばし、`step2` から再開して、最後まで進みます。

   ```text
   $ conductor pipeline -v hello
   [conductor] job=step1 skipped (already executed)
   step2
   [conductor] job=step2 result=success (exit code 0)
   step3
   [conductor] job=step3 result=success (exit code 0)
   $ echo $?
   0
   ```

5. 最後まで終わりました。ステートは残っていて、`watch` は、表の下に、完了を示す行(`completed`)を出します(v0.7.0 以降)。

   ```text
   $ conductor watch
   Pipeline: hello
   JOB    STATUS   NOTE
   step1  success  exit code 0
   step2  success  exit code 0
   step3  success  exit code 0
   completed
   ```

6. 完了したステートが残っていても、同じコマンドをもう一度実行すると、先頭の `step1` から実行します。

   ```text
   $ conductor pipeline -v hello
   step1
   [conductor] job=step1 result=success (exit code 0)
   step2
   [conductor] job=step2 result=success (exit code 0)
   step3
   [conductor] job=step3 result=success (exit code 0)
   $ echo $?
   0
   ```

完了のときにステートを消したいときは、`--remove-on-complete` を付けます(「[ステートファイルの場所](#ステートファイルの場所)」を参照)。

次は、「使い方」と「定義ファイルの書き方」へ進んでください。

## 使い方

```text
conductor run [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <ジョブID>
conductor pipeline [-v] [--config <パス>] [--run-id <ID>] [--remove-on-complete] <パイプライン名>
conductor watch [--config <パス>] [--run-id <ID>]
conductor reset [--config <パス>] [--run-id <ID> | --all]
conductor validate [-v] [--config <パス>] [<パイプライン名>]
conductor convert <ファイル> --to <yaml|toml>
conductor --help | --version
```

| コマンド | 動き |
|----------|------|
| `run <ジョブID>` | ジョブ 1 つを実行する(個別実行)。起動前の判断(実行・スキップ・拒否)に従う |
| `pipeline <パイプライン名>` | 定義の記述順に、ジョブを 1 つずつ実行する(一括実行)。エラーで止まる。完了しても、ステートは残る(v0.7.0 以降) |
| `watch` | ステートを、ジョブ・状態・補足の 3 列の表で、1 回だけ表示する。完了したステートには、表の下に完了を示す行を足す(v0.7.0 以降) |
| `reset` | ステートを消して、最初からやり直す(壊れたステートの回復と、完了したステートの片づけにも使う) |
| `validate [<パイプライン名>]` | 定義ファイルを検証し、全ジョブの `command` を検査する(v0.4.0 以降) |
| `convert <ファイル> --to <形式>` | 定義ファイルを、YAML と TOML のあいだで変換し、結果を標準出力に書く |

| オプション | 意味 |
|------------|------|
| `--config <パス>`(`--config=<パス>`) | 定義ファイルのパス。省略時は、実行ディレクトリの `conductor.toml`(`run`・`pipeline`・`watch`・`reset`・`validate`) |
| `-v`、`--verbose` | ジョブの結果、スキップ、遷移を、標準エラーに表示する(`run`・`pipeline`・`validate` だけ。1 段階) |
| `--run-id <ID>`(`--run-id=<ID>`) | ステートを分ける run ID(`run`・`pipeline`・`watch`・`reset` だけ)。規則は「[run ID でステートを分ける](#run-id-でステートを分けるv050-以降)」を参照 |
| `--remove-on-complete` | `run`・`pipeline` だけ(v0.7.0 以降)。値は取りません。完了のときに、ステートファイルを消す。指定しなければ、定義の `[pipeline]` の `remove_on_complete`(既定は `false`)に従う。`watch`・`reset`・`validate`・`convert` に付けると、使い方の誤り(68) |
| `--all` | `reset` だけ(v0.5.0 以降)。その定義のステートを、run ID の有無によらず、すべて消す |
| `--to <yaml\|toml>` | `convert` の変換後の形式(必須) |
| `-h`、`--help` | 使い方を表示する |
| `-V`、`--version` | 版を表示する |

- `--` は、オプションの終わりを表します(先頭が `-` のジョブIDは、`--` のあとに書きます)。
- 既定では、`conductor` 自身の出力は、ありません(ジョブ本体の出力が、そのまま流れます)。エラーのときだけ、標準エラーに原因を表示します。

### コマンドの詳しい動き

- **`watch`**: ステートを、ジョブ・状態・補足の 3 列の表で、1 回だけ表示します。run ID があるときは、パイプライン名の次の行に `Run ID: <ID>` を表示します。完了したステートには(v0.7.0 以降)、表の下に、完了を示す 1 行を足します(英語は `completed`、日本語は `完了`。状態の列の語は、英語のままです)。`no state` を表示するのは、ステートファイルがないとき、または長さが 0 のときだけです。終了コードは 0 です。
- **`reset`**: ステートを(完了したものも)消します。`--run-id` で、その run ID のステートだけを消します。`--all` で、その定義のステートを、run ID の有無によらず、すべて消します(`--run-id` との併用は、使い方の誤り(68)です)。run ID と `--all` の規則は、「[run ID でステートを分ける](#run-id-でステートを分けるv050-以降)」にあります。
- **`validate`(v0.4.0 以降)**: 定義ファイルを検証し、全ジョブの `command` を検査します。ジョブは実行せず、ステートを読まず、書かず、消しません。パイプライン名を書くと、定義の名前と照合します(省略すると照合しません)。成功は何も表示しません。`-v` を付けると、検査が通ったときに 1 行(ジョブの数)だけ、標準エラーに表示します。`--run-id` は使えません(使い方の誤り(68))。

### 例

```sh
conductor run extract                  # extract だけを実行する
conductor run -v extract               # 結果と遷移を表示しながら
conductor pipeline nightly --config /etc/batch/conductor.toml
conductor watch                        # 実行の状況を見る
conductor reset                        # 最初からやり直す
conductor pipeline nightly --run-id "$GITHUB_RUN_ID"   # 実行単位(run ID)ごとに、ステートを分ける
conductor reset --all                  # その定義のステートを、すべて消す
conductor pipeline nightly --remove-on-complete   # 完了したら、ステートファイルを消す
conductor validate                     # 定義を実行せずに検査する(成功は何も出さない)
conductor validate -v nightly --config /etc/batch/conductor.toml   # 名前も照合し、結果を 1 行表示
conductor convert conductor.yaml --to toml > conductor.toml
```

動きを一通り見るには、[デモ](demo/README.md)(UNIX 系は `demo/run-demo.sh`、Windows は `demo\windows\run-demo.cmd`)が使えます。

## 定義ファイルの書き方

既定の名前は `conductor.toml` です(`--config` で変えられます)。

```toml
format_version = 1

[pipeline]
name = "nightly"
description = "夜間バッチ"
run_id_env = "GITHUB_RUN_ID"   # 省略可。run ID を読む環境変数の名前(省略すると CONDUCTOR_RUN_ID)
remove_on_complete = false     # 省略可。true にすると、完了のときにステートファイルを消す(省略すると false)

[[job]]
id = "extract"
command = "python scripts/extract.py"
warning_exit_codes = [1]

[[job.transitions]]
on = "warning"
next = "notify"            # 警告のとき、間のジョブ(transform、load)をスキップして移る

[[job]]
id = "transform"
command = "python scripts/transform.py"
success_exit_codes = [0, 3]

[[job]]
id = "load"
command = "python scripts/load.py"

[[job.transitions]]
on = "success"
next = "@end"              # 成功したら、ここでパイプラインを終える

[[job]]
id = "notify"
command = "python scripts/notify.py --msg='Warning in extraction'"
```

| キー | 意味 |
|------|------|
| `format_version` | 書式の版。`1` だけ(必須) |
| `[pipeline]` の `name` | パイプラインの名前(必須。`conductor pipeline <名前>` で照合する)。`description` は省略可 |
| `[pipeline]` の `run_id_env` | 省略可(v0.5.0 以降)。run ID を読む環境変数の名前で、形の規則は「[run ID でステートを分ける](#run-id-でステートを分けるv050-以降)」を参照 |
| `[pipeline]` の `remove_on_complete` | 省略可(v0.7.0 以降)。真偽値(`true` または `false`)で、省略すると `false`。`true` のとき、完了でステートファイルを消す(`--remove-on-complete` を付けた実行も同じ)。真偽値でないと、定義の誤り(64。`validate` も同じ)です |
| `[[job]]` | ジョブ。**書いた順**に実行する。1 つ以上 |
| `id` | ジョブID(必須。空でなく、定義の中で重複せず、`@` で始めない) |
| `command` | 実行するコマンド(必須)。文字列、または文字列の配列(下の「command の書き方」) |
| `success_exit_codes` | 成功にする終了コード。**省略すると `[0]`**。書いたら、書いた値だけが成功(`0` を書かなければ、`0` でも成功にならない) |
| `warning_exit_codes` | 警告にする終了コード。`success_exit_codes` と重ならず、`0` を含まない |
| `[job.retry]` | 省略可(v0.8.0 以降)。ジョブが、決めた終了コードで終わったときに、待ってから実行し直す設定。書き方と動きは、「[リトライの設定](#リトライの設定jobretryv080-以降)」を参照 |
| `[[job.transitions]]` | 結果ごとの遷移。`on` は `success` か `warning`(エラーには書けない)。同じ `on` は 1 つだけ |
| `next`(遷移先) | 記述順で**後ろ**のジョブID、または `@end`(以降のジョブをスキップして終える) |

- 遷移を書かない成功・警告は、記述順で次のジョブに進みます。
- 終了コードが、成功にも警告にも含まれなければ、エラーです。
- 以前の版の `[[jobs]]` は、`[[job]]` に変わりました。`[[jobs]]` と書くと、定義の誤りになります。

### リトライの設定(`[job.retry]`)(v0.8.0 以降)

一時的な失敗(たとえば、相手のサービスが一瞬つながらない)で、ジョブが決まった終了コードで終わったときに、少し待ってから、同じジョブを自動で実行し直せます。ジョブの `[job.retry]` に、次のように書きます。`[job.retry]` は、そのジョブの、`id`・`command` などのキーのあと(`[[job.transitions]]` の前)に書きます。

```toml
format_version = 1

[pipeline]
name = "nightly"

[[job]]
id = "extract"
command = "python scripts/extract.py"

[job.retry]
exit_codes = [75, 111]
max = 5
interval = "10s"
multiplier = 2
max_interval = "60s"
```

| キー | 意味 |
|------|------|
| `exit_codes` | 必須。リトライする終了コードの一覧。空でない整数の一覧で、各値は 1〜255。同じ値を重ねて書いても誤りにならず、1 つとして扱う。そのジョブの成功の終了コード・警告の終了コードと、重ならないこと |
| `max` | 必須。リトライの最大回数(1〜100)。最初の実行に加えて、最大この回数だけ実行し直す(合計で、最大 `max + 1` 回) |
| `interval` | 必須。最初の待ち時間。整数と単位(`s` 秒、`m` 分、`h` 時間)の文字列(`"30s"`、`"2m"`、`"1h"`)で、1 秒〜24 時間 |
| `multiplier` | 省略可。待ち時間にかける倍率(2〜10 の整数)。`max_interval` と一緒に書く |
| `max_interval` | 省略可。待ち時間の上限。`interval` と同じ書き方で、`interval` 以上。`multiplier` と一緒に書く |

- `multiplier` と `max_interval` は、**両方を書くか、両方とも書かない**かのどちらかです。両方を書かないと、待ち時間は、毎回 `interval` のままです。
- 1 回目のリトライの前の待ち時間は `interval` で、2 回目以降は、直前の待ち時間に `multiplier` をかけた値です(`max_interval` を超えません)。上の例(`10s`、倍率 2、上限 `60s`、`max = 5`)では、待ち時間は、10 秒、20 秒、40 秒、60 秒、60 秒です。
- ここに書いた誤り(`exit_codes` が空、値が範囲外、成功・警告の終了コードと重なる、必須のキーがない、知らないキーがある、`max` や間隔が範囲外、`multiplier` と `max_interval` の片方だけ、`max_interval` が `interval` より小さい、など)は、定義の誤り(64)です。`validate` も、同じ誤りを検出します。ジョブは実行しません。

#### リトライの動き

- **リトライするとき**: ジョブ**自身**の終了コードが、`exit_codes` の一覧にあるときだけです。ジョブを起動できなかったとき(終了コード 127 を、`conductor` が付けた場合)、ジョブがシグナルで終了したとき、`conductor` が中断されたときは、リトライしません。ジョブ自身が 127 で終わった場合は、`exit_codes` に 127 を書いていれば、リトライします。
- **使い切ったとき**: 最大回数まで実行し直しても、まだ一覧の終了コードで終わるときは、通常のエラーと同じです。最後の実行の終了コードで終わり、再開位置はそのジョブになります。実行し直した結果が、成功や警告の終了コードなら、通常どおり、遷移の規則に従って進みます。
- **リトライの前の知らせ**: 待つ前に、`-v` がなくても、標準エラーに 1 行を表示します(エラーではありません)。表示は、リトライごとに 1 回で、途中の結果の `-v` の行は出さず、最終の結果の行が、1 回出ます。

  ```text
  conductor: retry: job 'extract' exited with code 75; retry 2 of 5 in 20 seconds
  conductor: retry: ジョブ 'extract' が終了コード 75 で終わりました。20 秒後にリトライします(2/5 回目)
  ```

  上が英語、下が日本語の表示です(実際は、ロケールに合う方の 1 行だけが出ます)。
- **待っているあいだのシグナル**: すぐに待つのをやめ、次の実行はしません。詳しくは、「[シグナルの扱い](#シグナルの扱い)」を参照してください。
- **ステートファイル**: リトライの回数は、ステートファイルに記録しません(書式は、変わりません)。止まったあとの再開は、そのジョブを、最初の実行から数え直します。
- **同じ処理が複数回動く**: リトライするジョブは、同じ処理が複数回走ります。再実行されても困らない作りにしてください(「[保証の範囲(at-least-once)](#保証の範囲at-least-once)」を参照)。
- **`[job.retry]` を書かないジョブ**: これまでと、まったく同じ動きです。

### command の書き方

`command` は、シェルを介さずに、直接起動します。空白で、コマンド名と引数に分け、`'...'`(中は、そのまま)と `"..."`(中の `\"` だけが、二重引用符 1 つになる)で、空白を含む引数を書けます。変数(`$HOME`)、`*`、`|`、`;`、`&&`、`>` は、解釈されません(普通の文字として、引数に入ります)。パイプやリダイレクトが必要なら、`sh -c '...'` を、`command` に書いてください。

#### 配列で書く(v0.2.0 以降)

引数に空白や引用符が入るときは、`command` を文字列の配列で書けます。

```toml
command = ["python", "scripts/notify.py", "--msg=Warning in extraction", ""]
```

- **1 要素が 1 引数**です。先頭の要素がプログラム、残りが引数になります。文字列の `command` のような分割や引用符の処理はせず、`$HOME` や `*` なども展開しません(要素の文字が、そのまま引数になります)。
- 先頭の要素が、空または空白だけのときと、配列が空のときは、定義の誤りです(終了コード 64)。要素が文字列でないときも、定義の誤りです。
- 2 番目以降の要素は、空の文字列(`""`)でも有効で、空の引数として渡します。文字列の `command` が空または空白だけのときは、これまでどおり定義の誤りです。
- `format_version` は、`1` のままです。配列の `command` を書いた定義は、v0.1.x では読めません(`command` の型の誤りとして拒否されます)。v0.1.x でも使う定義は、文字列で書いてください。
- `convert` は、配列をそのまま TOML と YAML のあいだで変換します。

## ステートファイルの場所

ステート(どのジョブが、どう終わったか)は、**定義ファイルと同じディレクトリ**の `<定義ファイル名>.state` に保存されます(例: `conductor.toml` なら `conductor.toml.state`)。

- 最後のジョブの完了、`@end`、スキップでの完了のあとも、ステートは**残ります**(v0.7.0 以降。v0.6.x までは、完了で消されました)。完了したステートは、`completed = true` のまま残り、書式(`format_version = 1`)は変わりません。`watch` は、これに完了を示す行を足して表示します。次の実行は、先頭から始まります(完了したステートは、空として扱います。先頭以外のジョブを `run` で単独に呼ぶと、順序外(66)です)。
- 完了のときにステートを消すには、`--remove-on-complete`(`run`・`pipeline`)を付けるか、定義の `[pipeline]` に `remove_on_complete = true` を書きます。オプションを付けたら消し、付けなければ定義の値に従います(環境変数は見ません)。消すのに失敗したときは、65 で終わります。残ったステートの片づけは、`conductor reset`(`reset --all`)で行います。
- エラーで止まったときは、エラーのジョブが、再開位置として残ります。同じコマンドを、もう一度実行すると、成功済みのジョブを飛ばして、エラーのジョブから再開します。定義に `[job.retry]` を書かなければ、**自動の再試行はしません**(書いたときは、「[リトライの設定](#リトライの設定jobretryv080-以降)」に従って、ジョブの実行の中で実行し直します)。リトライの回数は、ステートに記録しません。止まったあとの再開は、そのジョブを、最初の実行から数え直します。
- ステートが読めない・壊れている・定義と食い違っているときは、65 で終わります。`conductor reset` で消してください(`reset` は、ステートを読まず、検査もしないため、どんな状態でも動きます)。
- 同じ定義に、複数の `conductor` を同時に動かしても、排他制御はしません(後に書いた方が、勝ちます)。同時の実行は、避けてください。同じ定義を別々の実行単位で並行して使うときは、次の run ID で、ステートを分けてください。

### run ID でステートを分ける(v0.5.0 以降)

run ID を与えると、ステートは `<定義ファイル名>.<run-id>.state`(例: `conductor.toml.12345.state`)に分かれ、run ID ごとに独立します。run ID を与えないときは、これまでどおり `<定義ファイル名>.state` で、形式もメッセージも変わりません。同じ run ID で呼び直すと、止まったジョブから再開します(完了した run のステートは、run ID なしと同じく、v0.7.0 以降は残ります。`--remove-on-complete` か、定義の `remove_on_complete = true` で、消せます)。

- **与え方**: `--run-id <ID>`、または環境変数。環境変数の名前は、定義ファイルの `[pipeline]` の `run_id_env` で決めます(省略すると `CONDUCTOR_RUN_ID`)。`run_id_env` は、`A-Za-z_` で始まり、`A-Za-z0-9_` が続く 1 文字以上の文字列です。文字列でない、または形が違うと、定義の誤り(64。`validate` も同じ)です。`--run-id` があれば、環境変数は見ません。環境変数が未定義または空なら、run ID なしです。`--run-id ""` は、環境変数があっても、run ID なしに固定します。
- **値の規則**: 文字は `A-Za-z0-9._-`、1〜64 文字、先頭が `.` でないこと(`..` も不可)。規則に合わない値は、オプションでも環境変数でも、使い方の誤り(68)です。誤りの文に、値は含めません。`validate`・`convert` への `--run-id`、`reset` 以外への `--all` も、使い方の誤り(68)です。
- **中身の `run_id`**: run ID つきのステートには、キー `run_id` を書きます(run ID なしのステートには書きません)。ファイル名の run ID と中身の `run_id` が食い違うと(たとえば、別の run ID のファイルをコピーしたとき)、壊れたステート(65)として扱います。`conductor reset --run-id <ID>` で消してください。
- **`watch`・`reset`**: `--run-id` があれば、定義ファイルを読みません。なければ、定義から `run_id_env` だけを、ゆるく読みます(定義が壊れている、キーがない、型や形が違う、のときは `CONDUCTOR_RUN_ID`。ほかの誤りは無視するので、壊れた定義でも `reset` は動きます)。
- **`reset --all`**: 定義ファイルと同じディレクトリの、次のものを、すべて消します。
  - `<定義ファイル名>.state`(run ID なし): 中身は読まずに、通常のファイルなら消します。シンボリックリンクは消しません。
  - `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が規則に合い、中身の `run_id` がその値と一致するものだけを消します。

  次のものは、消しません。一時ファイル(`.<ステート名>.tmp.*`)、別の定義のステート(run ID つきで、中身に `run_id` がない、または値が違うもの)、ディレクトリ、シンボリックリンクです。run ID つきで、読めない・壊れているファイルは、消さずに、ファイル名を標準エラーに警告します(`conductor reset --run-id <ID>` で個別に消せます。終了コードは 0)。消すのに失敗したときは、65 で終わります。消すものがなければ、何も出力せず 0 で終わります。ステートは、自動では消えません(古い run のステートは、`reset --all` で片づけてください)。
- **大文字小文字を区別しないファイルシステム**(macOS の既定、Windows)では、`A` と `a` は同じファイルになります。run ID は、大文字小文字だけで区別しないでください(食い違いは、中身の `run_id` との照合で、壊れたステート(65)として検出されます)。
- 同じ run ID で、`conductor` を同時に動かすことは、避けてください(排他制御はしません)。

## 終了コードの表

| 終了コード | 意味 |
|------------|------|
| 0 | 成功、警告、スキップ(実行済み・遷移・`@end` によるスキップ)、`watch`・`reset`・`validate`・`convert` の成功 |
| ジョブの値 | エラーになったジョブの終了コードを、そのまま(0〜255) |
| 1 | 成功コードに含まれない `0` で終わったジョブ(`success_exit_codes` に `0` がない)。原因を標準エラーに表示する |
| 64 | 定義の誤り(ファイルがない・読めない・検証の誤り・パイプライン名の不一致・`command` の誤り・`run_id_env` の型や名前の形の誤り・`remove_on_complete` の型の誤り・`[job.retry]` の誤り)。ジョブは実行しない。`validate` の検査の失敗も、この値 |
| 65 | ステートの誤り(読めない・壊れている・版の不一致・定義との食い違い・中身の `run_id` とファイル名の食い違い・書き込みや初期化の失敗、`reset --all` で消すのに失敗したとき)、`watch` の標準出力に書けない |
| 66 | 拒否(順序外。前のジョブが終わっていない) |
| 67 | 変換の誤り、`convert` の標準出力に書けない |
| 68 | 使い方の誤り(知らないコマンド、引数の不足、余分な引数、知らないジョブIDなど。`validate` に名前を 2 個以上渡した場合を含む)。run ID の値の誤り(`--run-id` の値、または環境変数の値が、規則に合わない)、`validate`・`convert` への `--run-id`、`reset` 以外への `--all`、`--all` と `--run-id` の併用、`run`・`pipeline` 以外への `--remove-on-complete` も、この値 |
| 70 | 内部の誤り(パニック、シグナルの準備の失敗) |
| 127 | ジョブを起動できなかった |
| 128 + 番号 | ジョブがシグナルで終了した、または、`conductor` が中断された(例: SIGTERM = 143、Ctrl+C = 130)。リトライの待機中に中断された場合も含む |

- 終了コードは、どの OS でも、0〜255 の範囲に合わせます。範囲外の値(255 を超える、または負)は、下位 8 ビットです。エラーのジョブで、下位 8 ビットが 0 になる場合は、1 にします(エラーを成功に見せないため)。元の値は、`-v` の表示に、そのまま示します。
- 64〜68 と 70 は、ジョブ自身の終了コードと重なりえます。区別は、標準エラーの `conductor: error: ...` の行で行います。
- ジョブの結果の書き込みに失敗したときは、ジョブの終了コードではなく 65 で終わり、「ジョブは実行されました」と表示します(次の「保証の範囲」を参照)。
- run ID に関する 65・68 の詳細は、「[run ID でステートを分ける](#run-id-でステートを分けるv050-以降)」を参照してください。

## 保証の範囲(at-least-once)

**ジョブの終了と、その結果のステートへの記録のあいだに、`conductor` が落ちる(kill される・電源が切れるなど)と、ジョブは終わっているのに、ステートに記録されません。** 次の呼び出しでは、そのジョブは未実行として扱われ、**再実行されうる**(ジョブが 2 回動きうる)ことになります。冪等でないジョブは、2 回動きうるため、ジョブを、再実行されても困らない作りにしてください。

- ステートが壊れた内容になることはありません(一時ファイルに書いてから、置き換えます)。完了のステートも、同じ手順で書きます。書くのに失敗したときは、65 で終わります。
- 結果の書き込みに失敗したときは、65 で終わり、「ジョブは実行されました」と表示します。ジョブは、次の呼び出しで、再実行されえます。
- `conductor` が SIGKILL などで強制終了されると、実行中のジョブは、残って動き続けうります(`conductor` は、検知も停止もしません)。
- ジョブの実行中に、`conductor` が内部の誤り(パニック)で終わると、ジョブが孤児として残りうります(終了コード 70)。
- `[job.retry]`(v0.8.0 以降)を書いたジョブは、同じ処理が複数回走ります(最大 `max + 1` 回)。再実行されても困らない作りにしてください。リトライの回数は、ステートに記録しないため、`conductor` が落ちたあとの再開では、そのジョブを、最初の実行から数え直します(リトライ済みの分も、また動きます)。
- 時間の上限(タイムアウト)はありません。ジョブが終わらなければ、`conductor` は待ち続けます。

## シグナルの扱い

`run` と `pipeline` は、定義の検査より前に、シグナルを受け取る準備をします(`watch`・`reset`・`validate`・`convert` は、既定の動作で終わります)。

- **UNIX 系**: `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM` を受け取ると、実行中のジョブ(の子プロセス)に、同じシグナルを**1 回だけ**転送し、ジョブの終了を待ちます。ジョブの結果を、エラーとしてステートに記録し、**128 + シグナル番号**で終わります。2 回目以降のシグナルは、追加の動作をしません(強制終了しません)。終了コードに使う「最初の番号」は、**受け取った順**の最初です。ただし、ほぼ同時に複数のシグナルが届いたときは、シグナルの監視(`signal-hook`)が、番号の小さい順に返すため、小さい方の番号(たとえば、INT(2)と HUP(1)なら HUP)が「最初」になりえます。時間を空けて届いたときは、先に届いた方です。
- **Windows**: Ctrl+C(番号 2)を受け取ります(終了コード 130)。Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(既定の動作)です。
- ジョブを起動する前、または、`pipeline` のジョブのあいだに、シグナルを受け取ったときは、次のジョブを起動せず、再開位置をステートに記録して、128 + 番号で終わります(エラーとしては記録しません)。次の実行は、そのジョブから続きます。
- リトライの待機中(v0.8.0 以降)にシグナルを受け取ったときは、すぐに待機をやめ、次の実行はせずに、再開位置をそのジョブとしてステートに記録して、128 + 番号で終わります(エラーとしては記録しません)。待機の打ち切りは、間隔の長さにかかわらず、シグナルを受けてから 1 秒以内です。次の実行は、そのジョブを、最初の実行から数え直して始めます。
- 検査やステートの確認の途中でシグナルを受け取っても、検査は最後まで進めます。定義の誤り(64)や、ステートの誤り(65)、拒否(66)、スキップ(0)の終了コードが、中断の終了コードより優先します。

## convert の使い方

定義ファイルを、YAML と TOML のあいだで変換します。入力の形式は、`--to` の**反対**とみなします(拡張子は見ません)。

```sh
conductor convert conductor.yaml --to toml > conductor.toml
conductor convert conductor.toml --to yaml
```

- 変換結果を、全部作ってから、標準出力に、1 回で書きます。出力先の指定オプションはありません(ファイルには、シェルのリダイレクトで書きます)。失敗したとき(文法の誤り、読めない、表せない値)は、標準出力は空で、標準エラーに原因を表示し、終了コード 67 で終わります。
- コメントと、マップのキーの記述順は、保持されません。YAML のアンカー・エイリアス・マージキーは、展開されます。`null`・日時などの、相手の形式で表せない値は、失敗になります。
- 変換した結果が、`conductor` の定義として正しいかは、検査しません(`run` などの読み込み時に検査されます)。
- 入力ファイルは 1 MiB までです。

## ロケールと言語

`conductor` 自身の表示は、環境変数 `LC_ALL`、`LC_MESSAGES`、`LANG` を、この順に見て、最初に空でない値が `ja` で始まれば日本語、それ以外(または、どれもない)は英語です。ジョブ本体の出力は、言語に関わらず、そのままです。

- 状態を表す語(`success`・`warning`・`skipped`・`error`・`pending`)と、`-v` の `job=`・`result=` は、言語に関わらず英語です。
- 定義の誤りなどの、原因の文の一部(キー名を含む固定の文)は、日本語です。
- `watch` の表の色は、標準出力が端末で、UNIX 系で、環境変数 `NO_COLOR` が未設定または空のときだけ付きます。ファイルやパイプへの出力には、制御文字を含めません。状態は、色に関わらず、文字でも表します。
- 入力に由来する文字列(ジョブID、キー名、パス)は、制御文字などを `\u{..}` に直して表示します(ジョブID・キー名は 64 文字まで)。

## ライセンス

[LICENSE](LICENSE) を参照してください。
