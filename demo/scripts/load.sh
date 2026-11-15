#!/bin/sh
# デモ用のジョブ: load。終了コードは、環境変数 DEMO_LOAD_EXIT で決める(既定 0)。
echo "  [load] 実行中..."
sleep "${DEMO_SLEEP:-0.5}"
code="${DEMO_LOAD_EXIT:-0}"
echo "  [load] 終了コード $code で終わります"
exit "$code"
