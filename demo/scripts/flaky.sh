#!/bin/sh
# Copyright 2026 agwlvssainokuni
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# デモ用のジョブ: flaky(リトライの場面で使う)。
# 実行した回数を、カレントディレクトリの flaky.count に覚え、
# 環境変数 DEMO_FLAKY_FAILS 回(既定 2 回)までは、終了コード 75 で失敗する。
n=$(cat flaky.count 2>/dev/null || echo 0)
n=$((n + 1))
echo "$n" > flaky.count
echo "  [flaky] ${n} 回目の実行..."
sleep "${DEMO_SLEEP:-0.5}"
if [ "$n" -le "${DEMO_FLAKY_FAILS:-2}" ]; then
  echo "  [flaky] 終了コード 75 で終わります(一時的な失敗)"
  exit 75
fi
echo "  [flaky] 終了コード 0 で終わります"
exit 0
