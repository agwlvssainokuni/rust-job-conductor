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

REM デモ用のジョブ: transform。終了コードは、環境変数 DEMO_TRANSFORM_EXIT で決める(既定 0)。
REM 待ち時間は、環境変数 DEMO_SLEEP(整数の秒。既定 1)で決める。
setlocal EnableExtensions
if not defined DEMO_SLEEP set DEMO_SLEEP=1
if not defined DEMO_TRANSFORM_EXIT set DEMO_TRANSFORM_EXIT=0
echo   [transform] 実行中...
set /a wait_count=DEMO_SLEEP+1
ping -n %wait_count% 127.0.0.1 >nul
echo   [transform] 終了コード %DEMO_TRANSFORM_EXIT% で終わります
exit /b %DEMO_TRANSFORM_EXIT%
