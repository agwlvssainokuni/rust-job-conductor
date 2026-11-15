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
echo 完了したので、ステートは空になる:
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
echo.
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

REM DEMO_PAUSE=1 のとき、Enter を待つ
:pause
if "%DEMO_PAUSE%"=="1" set /p "pause_dummy=(Enter で次へ)"
exit /b 0
