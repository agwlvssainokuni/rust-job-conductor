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

REM デモ用のジョブ: flaky(リトライの場面で使う)。
REM 実行した回数を、カレントディレクトリの flaky.count に覚え、
REM 環境変数 DEMO_FLAKY_FAILS 回(既定 2 回)までは、終了コード 75 で失敗する。
REM 待ち時間は、環境変数 DEMO_SLEEP(整数の秒。既定 1)で決める。
setlocal EnableExtensions EnableDelayedExpansion
if not defined DEMO_SLEEP set DEMO_SLEEP=1
if not defined DEMO_FLAKY_FAILS set DEMO_FLAKY_FAILS=2
set n=0
if exist flaky.count set /p n=<flaky.count
set /a n=n+1
>flaky.count echo !n!
echo   [flaky] !n! 回目の実行...
set /a wait_count=DEMO_SLEEP+1
ping -n %wait_count% 127.0.0.1 >nul
if !n! LEQ %DEMO_FLAKY_FAILS% goto fail
echo   [flaky] 終了コード 0 で終わります
exit /b 0
:fail
echo   [flaky] 終了コード 75 で終わります^(一時的な失敗^)
exit /b 75
