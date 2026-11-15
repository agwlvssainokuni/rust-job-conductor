#!/bin/sh
# デモ用のジョブ: extract。終了コードは、環境変数 DEMO_EXTRACT_EXIT で決める(既定 0)。
echo "  [extract] 実行中..."
sleep "${DEMO_SLEEP:-0.5}"
code="${DEMO_EXTRACT_EXIT:-0}"
echo "  [extract] 終了コード $code で終わります"
exit "$code"
