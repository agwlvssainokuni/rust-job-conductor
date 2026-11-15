#!/bin/sh
# デモ用のジョブ: notify。終了コードは、環境変数 DEMO_NOTIFY_EXIT で決める(既定 0)。
echo "  [notify] 実行中..."
sleep "${DEMO_SLEEP:-0.5}"
code="${DEMO_NOTIFY_EXIT:-0}"
echo "  [notify] 終了コード $code で終わります"
exit "$code"
