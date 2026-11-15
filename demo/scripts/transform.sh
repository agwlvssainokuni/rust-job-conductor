#!/bin/sh
# デモ用のジョブ: transform。終了コードは、環境変数 DEMO_TRANSFORM_EXIT で決める(既定 0)。
echo "  [transform] 実行中..."
sleep "${DEMO_SLEEP:-0.5}"
code="${DEMO_TRANSFORM_EXIT:-0}"
echo "  [transform] 終了コード $code で終わります"
exit "$code"
