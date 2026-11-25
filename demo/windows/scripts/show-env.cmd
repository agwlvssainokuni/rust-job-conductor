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

REM デモ用のジョブ: show-env(環境変数の場面で使う)。conductor が渡した環境変数を表示する。
echo   [show-env] DEMO_GREETING=%DEMO_GREETING%
echo   [show-env] DEMO_FROM_FILE=%DEMO_FROM_FILE%
echo   [show-env] DEMO_QUOTED=%DEMO_QUOTED%
echo   [show-env] DEMO_STAGE=%DEMO_STAGE%
echo   [show-env] CONDUCTOR_PIPELINE=%CONDUCTOR_PIPELINE%
echo   [show-env] CONDUCTOR_JOB_ID=%CONDUCTOR_JOB_ID%
if defined CONDUCTOR_RUN_ID (echo   [show-env] CONDUCTOR_RUN_ID=%CONDUCTOR_RUN_ID%) else (echo   [show-env] CONDUCTOR_RUN_ID=^(なし^))
echo   [show-env] CONDUCTOR_ATTEMPT=%CONDUCTOR_ATTEMPT%
if defined DEMO_PROXY (echo   [show-env] DEMO_PROXY=%DEMO_PROXY%) else (echo   [show-env] DEMO_PROXY=^(なし^))
exit /b 0
