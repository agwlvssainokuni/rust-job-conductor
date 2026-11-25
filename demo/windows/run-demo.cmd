@echo off
REM Copyright 2026 agwlvssainokuni
REM
REM Licensed under the Apache License, Version 2.0 (the "License");
REM you may not use this file except in compliance with the License.
REM You may obtain a copy of the License at
REM
REM     http://www.apache.org/licenses/LICENSE-2.0
REM
REM Unless required by applicable law or agreed to in writing, software
REM distributed under the License is distributed on an "AS IS" BASIS,
REM WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
REM See the License for the specific language governing permissions and
REM limitations under the License.

REM conductor のデモ(Windows 版)。一時ディレクトリで実行するので、リポジトリは汚れない。
REM   使い方: demo\windows\run-demo.cmd   (どこから呼んでもよい)
REM   DEMO_PAUSE=1 を付けると、各場面のあとで Enter を待つ。
REM   CONDUCTOR=C:\path\to\conductor.exe で、使う実行ファイルを指定できる。
REM   DEMO_LANG=en_US で、表示を英語にする(既定は ja_JP.UTF-8)。
REM   DEMO_SLEEP=<整数の秒> で、各ジョブの待ち時間を変える(既定 1)。
setlocal EnableExtensions

for %%I in ("%~dp0..\..") do set "root=%%~fI"
set "here=%~dp0"

if not defined CONDUCTOR (
  set "CONDUCTOR=%root%\target\release\conductor.exe"
)
if not exist "%CONDUCTOR%" (
  echo conductor をビルドします^(cargo build --release --bin conductor^)...
  pushd "%root%"
  cargo build --release --locked --bin conductor
  if errorlevel 1 (
    popd
    exit /b 1
  )
  popd
)

REM 利用者の環境に左右されないよう、run ID の環境変数を消す(場面 8、9 で、デモが自分で設定する)。
set "DEMO_RUN_ID="
set "CONDUCTOR_RUN_ID="

REM 表示の言語(既定は日本語。DEMO_LANG=en_US で英語)。conductor は、LC_ALL、LC_MESSAGES、LANG の順に見る。
if not defined DEMO_LANG set "DEMO_LANG=ja_JP.UTF-8"
set "LC_ALL="
set "LC_MESSAGES="
set "LANG=%DEMO_LANG%"

REM 日本語を正しく出すため、コードページを UTF-8 にする(終わりに元へ戻す)。
set "old_cp="
for /f "tokens=2 delims=:" %%a in ('chcp') do set "old_cp=%%a"
set "old_cp=%old_cp: =%"
chcp 65001 >nul

set "work=%TEMP%\conductor-demo-%RANDOM%%RANDOM%"
mkdir "%work%"
copy /y "%here%conductor.toml" "%work%\" >nul
copy /y "%here%retry.toml" "%work%\" >nul
copy /y "%here%log.toml" "%work%\" >nul
copy /y "%here%env.toml" "%work%\" >nul
copy /y "%here%demo.env" "%work%\" >nul
xcopy /e /i /q /y "%here%scripts" "%work%\scripts" >nul
cd /d "%work%"

call :scene "0. 定義ファイル(conductor.toml)"
type conductor.toml
call :pause

call :scene "1. 全部が成功する: pipeline は、書いた順に、ジョブを 1 つずつ実行する"
call :runcmd pipeline -v nightly
call :pause

call :scene "2. 警告で分岐する: extract が警告(終了コード 1)→ transform と load を飛ばして notify へ"
setlocal
set "DEMO_EXTRACT_EXIT=1"
call :runcmd pipeline -v nightly
endlocal
call :pause

call :scene "3. エラーで止まる: load が失敗(終了コード 2)。conductor の終了コードも 2"
setlocal
set "DEMO_LOAD_EXIT=2"
call :runcmd pipeline -v nightly
endlocal
echo.
echo 止まった位置を見る(watch):
call :runcmd watch
call :pause

call :scene "4. 原因を直して、同じコマンドをもう一度: 失敗した load から再開する(transform は再実行しない)"
call :runcmd pipeline -v nightly
echo.
echo 完了しても、ステートは残る。watch は、表の下に「完了」を示す行を出す:
call :runcmd watch
call :states
echo.
echo --remove-on-complete を付けると、完了でステートファイルを消す^(次に watch すると、ステートはない^):
call :runcmd pipeline -v --remove-on-complete nightly
call :states
call :runcmd watch
call :pause

call :scene "5. ジョブを 1 つだけ実行する(個別実行)"
call :runcmd run -v extract
call :runcmd reset
call :pause

call :scene "6. 定義ファイルを YAML に変換する(convert)"
call :runcmd convert conductor.toml --to yaml
echo.
echo (YAML から TOML へも変換できます)
call :pause

call :scene "7. 定義を検査する(validate): ジョブは実行せず、定義の誤りだけを見つける"
call :runcmd validate -v
echo.
echo command の中の引用符(')が閉じていない定義(bad.toml)を作って、検査する:
> bad.toml echo format_version = 1
>> bad.toml echo.
>> bad.toml echo [pipeline]
>> bad.toml echo name = "bad"
>> bad.toml echo.
>> bad.toml echo [[job]]
>> bad.toml echo id = "extract"
>> bad.toml echo command = "sh -c 'echo hi"
type bad.toml
call :runcmd validate --config bad.toml
echo.
echo ジョブは実行されず(出力なし)、ステートも作られない:
call :states
call :pause

call :scene "8. run ID でステートを分ける: run ID ごとに、別のステートファイルになる"
echo run ID build-1 で実行し、load で止める(--run-id):
setlocal
set "DEMO_LOAD_EXIT=2"
call :runcmd pipeline -v --run-id build-1 nightly
endlocal
echo.
echo 別の run ID build-2 を、環境変数 DEMO_RUN_ID で指定して実行する(定義の run_id_env = "DEMO_RUN_ID"):
setlocal
set "DEMO_RUN_ID=build-2"
echo $ set DEMO_RUN_ID=build-2
call :runcmd pipeline -v nightly
endlocal
echo.
echo build-2 は最後まで進み、build-1 のステートは、そのまま残る:
call :states
echo.
echo build-1 の止まった位置を見る:
call :runcmd watch --run-id build-1
echo.
echo 同じ run ID で同じコマンドをもう一度: load から再開する:
call :runcmd pipeline -v --run-id build-1 nightly
echo.
echo 完了しても、ステートファイルは残る^(watch は完了を示す^):
call :states
call :runcmd watch --run-id build-1
echo.
echo 片づけは reset^(run ID を指定して、1 つずつ消す^):
call :runcmd reset --run-id build-1
call :runcmd reset --run-id build-2
call :states
call :pause

call :scene "9. 環境変数と後始末: --run-id は環境変数に勝つ。reset --run-id と reset --all"
echo 環境変数だけで run ID env-1 を指定して、load で止める:
setlocal
set "DEMO_RUN_ID=env-1"
set "DEMO_LOAD_EXIT=2"
echo $ set DEMO_RUN_ID=env-1
call :runcmd pipeline -v nightly
endlocal
echo.
echo 環境変数が env-1 でも、--run-id opt-1 が勝つ(別のステートになる):
setlocal
set "DEMO_RUN_ID=env-1"
set "DEMO_LOAD_EXIT=2"
echo $ set DEMO_RUN_ID=env-1
call :runcmd pipeline -v --run-id opt-1 nightly
endlocal
echo.
echo --run-id ""(空)は、環境変数があっても、run ID なしに固定する:
setlocal
set "DEMO_RUN_ID=env-1"
set "DEMO_LOAD_EXIT=2"
echo $ set DEMO_RUN_ID=env-1
call :runcmd pipeline -v --run-id "" nightly
endlocal
echo.
echo 止まった run をもう 1 つ(opt-2):
setlocal
set "DEMO_LOAD_EXIT=2"
call :runcmd pipeline -v --run-id opt-2 nightly
endlocal
echo.
echo 止まった run が、別々のステートファイルになっている:
call :states
echo.
echo run ID を指定して、1 つだけ消す:
call :runcmd reset --run-id opt-1
call :states
echo.
echo reset --all で、残りをすべて消す:
call :runcmd reset --all
call :states
call :runcmd watch
call :pause

call :scene "10. リトライ: 一時的な失敗(終了コード 75)なら、間隔を空けて、実行し直す"
echo 定義^(retry.toml^)。[job.retry] に、リトライする終了コードと、最大の回数、間隔を書く:
type retry.toml
echo.
echo flaky は、2 回失敗してから成功する。実行し直す前に、conductor が知らせを出す^(標準エラー^):
if exist flaky.count del flaky.count
call :runcmd pipeline -v --config retry.toml retry-demo
echo.
echo ^(回数を使い切っても失敗するときは、通常のエラーと同じ: 最後の終了コードで止まり、同じコマンドで、そのジョブから再開できる^)
call :pause

call :scene "11. 環境変数: [pipeline.env]・[job.env]・.env(env_file)を、ジョブに足す"
echo 定義^(env.toml^)と .env^(demo.env^)。env_file の相対パスは、定義ファイルのあるディレクトリが基準:
type env.toml
echo.
type demo.env
echo.
echo ジョブには、引き継いだ環境に足した環境変数が渡る^(優先順位: .env ^< [pipeline.env] ^< [job.env]^)。
echo CONDUCTOR_* は、conductor が自動で渡す^(run ID がなければ CONDUCTOR_RUN_ID は渡さない^):
call :runcmd pipeline -v --config env.toml env-demo
echo.
echo 値は、conductor 自身の出力^(標準出力・標準エラー・ステート^)には出ない。.env が読めないときは、ジョブを動かさずに 64 で止まる:
> bad-env.toml echo format_version = 1
>> bad-env.toml echo.
>> bad-env.toml echo [pipeline]
>> bad-env.toml echo name = "bad-env"
>> bad-env.toml echo env_file = "missing.env"
>> bad-env.toml echo.
>> bad-env.toml echo [[job]]
>> bad-env.toml echo id = "x"
>> bad-env.toml echo command = 'scripts\show-env.cmd'
call :runcmd pipeline --config bad-env.toml bad-env
echo.
echo validate は、.env も検査する。同じ名前が 2 回あると、警告を出す^(終了コードは 0^):
> dup.env echo A=1
>> dup.env echo A=2
> dup-env.toml echo format_version = 1
>> dup-env.toml echo.
>> dup-env.toml echo [pipeline]
>> dup-env.toml echo name = "dup-env"
>> dup-env.toml echo env_file = "dup.env"
>> dup-env.toml echo.
>> dup-env.toml echo [[job]]
>> dup-env.toml echo id = "x"
>> dup-env.toml echo command = 'scripts\show-env.cmd'
call :runcmd validate --config dup-env.toml
echo.
call :scene "12. 遷移を図にする: graph は、ジョブを実行せず、定義の遷移を図にする"
echo 定義^(conductor.toml^)の遷移を図にする^(extract が警告なら notify へ飛ぶ。右の線が、飛び越しを表す^):
call :runcmd graph
echo.
echo -v で、ジョブごとに、コマンドと、飛び越しの注記を足す:
call :runcmd graph -v
echo.
echo --format mermaid は、Mermaid の図を出す^(Markdown や Web ページに貼れる^):
call :runcmd graph --format mermaid
echo.
echo --format dot は、Graphviz の DOT を出す^(dot -Tsvg などで画像にできる^):
call :runcmd graph --format dot
echo.
echo --state で、ジョブの状態と再開位置を重ねる。まず、load で止める:
setlocal
set "DEMO_LOAD_EXIT=2"
call :runcmd pipeline nightly
endlocal
call :runcmd graph --state
echo.
echo graph は、ステートを読むだけ^(書かない^)。後片付け:
call :runcmd reset
call :pause

call :scene "13. 実行計画: --dry-run は、ジョブを実行せず、実行したら何が起こるかを見せる"
echo --dry-run で、実行計画を出す^(ジョブは実行されず、ステートも作られない^):
call :runcmd pipeline nightly --dry-run
echo.
echo --assume で、ジョブの終了コードを仮定する^(extract が警告=終了コード 1 なら、transform と load は飛ばされる^):
call :runcmd pipeline nightly --dry-run --assume extract=1
echo.
echo load が終了コード 2 で失敗すると仮定すると、そこで停止する:
call :runcmd pipeline nightly --dry-run --assume load=2
echo.
echo run でも使える^(次の動きを示す^):
call :runcmd run extract --dry-run
echo.
echo 実際には何も実行されず、ステートも作られていない:
call :states
call :pause

call :scene "14. ひな形: init は、定義のひな形を作る(既にあるファイルは壊さない)"
echo init で、定義のひな形を作る^(名前は省略すると my-pipeline。ここでは starter^):
call :runcmd init --config new.toml starter
echo.
echo 作ったひな形は、validate に通る:
call :runcmd validate --config new.toml starter
echo.
echo もう一度 init すると、既にあるので、何も変えず、終了コード 69 で終わる:
call :runcmd init --config new.toml starter
echo.
echo --force を付けると、上書きする:
call :runcmd init --config new.toml --force starter
echo.
echo --config - なら、ファイルを作らずに、ひな形を標準出力に出す:
call :runcmd init --config - starter
del /q new.toml >nul 2>&1
call :pause

call :scene "15. ログのファイル: [pipeline] log_dir で、ジョブの出力を端末に出しつつ、ファイルにも残す"
echo 定義^(log.toml^)。log_dir に、ログのディレクトリを書く:
type log.toml
echo.
echo flaky は、2 回失敗してから成功する。端末の出力は、log_dir が無いときと同じ:
if exist flaky.count del flaky.count
call :runcmd pipeline --config log.toml log-demo
echo.
echo 試行ごとに 1 つのファイル^(^<ジョブ ID^>.^<開始の時刻^(UTC^)^>.^<試行の番号^>.log^):
echo $ dir /b logs
dir /b logs
for %%f in (logs\*.log) do (
  echo --- %%f
  type "%%f"
)
echo.
echo conductor は、古いログを消さない。片づけは利用者が行う:
echo $ rmdir /s /q logs
rmdir /s /q logs
call :runcmd reset --config log.toml
call :pause

echo デモは以上です。

REM 後片付け(一時ディレクトリを消し、コードページを戻す)
cd /d "%root%"
rmdir /s /q "%work%"
if defined old_cp chcp %old_cp% >nul
endlocal
exit /b 0

REM ---- サブルーチン ----

REM 場面の見出しを出す
:scene
echo.
echo === %~1 ===
exit /b 0

REM conductor を実行し、終了コードを出す(引数は、そのまま conductor へ渡す)
:runcmd
echo $ conductor %*
"%CONDUCTOR%" %*
set "code=%errorlevel%"
echo → 終了コード %code%
exit /b 0

REM ステートファイルの一覧を見せる(無ければ「(なし)」)
:states
echo $ dir /b conductor.toml*.state
dir /b conductor.toml*.state 2>nul
if errorlevel 1 echo (なし)
exit /b 0

REM DEMO_PAUSE=1 のとき、Enter を待つ
:pause
if "%DEMO_PAUSE%"=="1" set /p "pause_dummy=(Enter で次へ)"
exit /b 0
