#!/bin/sh
# conductor のデモ。一時ディレクトリで実行するので、リポジトリは汚れない。
#   使い方: demo/run-demo.sh            (リポジトリの根から)
#   DEMO_PAUSE=1 を付けると、各場面のあとで Enter を待つ。
#   CONDUCTOR=/path/to/conductor で、使う実行ファイルを指定できる。
set -u
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)

if [ -z "${CONDUCTOR:-}" ]; then
  CONDUCTOR="$root/target/release/conductor"
  if [ ! -x "$CONDUCTOR" ]; then
    echo "conductor をビルドします(cargo build --release --bin conductor)..."
    (cd "$root" && cargo build --release --locked --bin conductor) || exit 1
  fi
fi

# 表示の言語(既定は日本語。DEMO_LANG=en_US.UTF-8 で英語)
LANG="${DEMO_LANG:-ja_JP.UTF-8}"; export LANG

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$here/conductor.toml" "$work/"
cp -R "$here/scripts" "$work/"
cd "$work" || exit 1

scene() {
  printf '\n\033[1m=== %s ===\033[0m\n' "$1"
}
cmd() {
  printf '\033[36m$ %s\033[0m\n' "$*"
  "$@"
  code=$?
  printf '\033[33m→ 終了コード %s\033[0m\n' "$code"
}
pause() {
  if [ "${DEMO_PAUSE:-0}" = 1 ]; then printf '(Enter で次へ)'; read -r _; fi
}

scene "0. 定義ファイル(conductor.toml)"
cat conductor.toml
pause

scene "1. 全部が成功する: pipeline は、書いた順に、ジョブを 1 つずつ実行する"
cmd "$CONDUCTOR" pipeline -v nightly
pause

scene "2. 警告で分岐する: extract が警告(終了コード 1)→ transform と load を飛ばして notify へ"
( DEMO_EXTRACT_EXIT=1; export DEMO_EXTRACT_EXIT; cmd "$CONDUCTOR" pipeline -v nightly )
pause

scene "3. エラーで止まる: load が失敗(終了コード 2)。conductor の終了コードも 2"
( DEMO_LOAD_EXIT=2; export DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v nightly )
echo
echo "止まった位置を見る(watch):"
cmd "$CONDUCTOR" watch
pause

scene "4. 原因を直して、同じコマンドをもう一度: 失敗した load から再開する(transform は再実行しない)"
cmd "$CONDUCTOR" pipeline -v nightly
echo
echo "完了したので、ステートは空になる:"
cmd "$CONDUCTOR" watch
pause

scene "5. ジョブを 1 つだけ実行する(個別実行)"
cmd "$CONDUCTOR" run -v extract
cmd "$CONDUCTOR" reset
pause

scene "6. 定義ファイルを YAML に変換する(convert)"
cmd "$CONDUCTOR" convert conductor.toml --to yaml
echo
echo "(YAML から TOML へも変換できます)"
echo
echo "デモは以上です。"
