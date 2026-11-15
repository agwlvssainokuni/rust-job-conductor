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

# デモ用のジョブ: extract。終了コードは、環境変数 DEMO_EXTRACT_EXIT で決める(既定 0)。
echo "  [extract] 実行中..."
sleep "${DEMO_SLEEP:-0.5}"
code="${DEMO_EXTRACT_EXIT:-0}"
echo "  [extract] 終了コード $code で終わります"
exit "$code"
