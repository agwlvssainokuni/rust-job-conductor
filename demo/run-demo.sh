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

# 利用者の環境に左右されないよう、run ID の環境変数を消す(場面 8、9 で、デモが自分で設定する)。
unset DEMO_RUN_ID CONDUCTOR_RUN_ID

# 表示の言語(既定は日本語。DEMO_LANG=en_US.UTF-8 で英語)。LC_ALL と LC_MESSAGES は、LANG より優先されるので空にする。
unset LC_ALL LC_MESSAGES
LANG="${DEMO_LANG:-ja_JP.UTF-8}"; export LANG

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cp "$here/conductor.toml" "$work/"
cp "$here/retry.toml" "$here/log.toml" "$work/"
cp "$here/env.toml" "$here/demo.env" "$work/"
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
# ステートファイルの一覧を見せる(無ければ「(なし)」)
states() {
  printf '\033[36m$ ls conductor.toml*.state\033[0m\n'
  if ls conductor.toml*.state 2>/dev/null; then :; else echo "(なし)"; fi
}
# 環境変数の指定を、1 行で見せる(指定そのものは、続くサブシェルの中で行う)
setenv() {
  printf '\033[36m$ export %s\033[0m\n' "$1"
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
echo "止まった位置と、ジョブごとの所要時間を見る(watch。所要時間の列は v0.10.0 以降):"
cmd "$CONDUCTOR" watch
pause

scene "4. 原因を直して、同じコマンドをもう一度: 失敗した load から再開する(transform は再実行しない)"
cmd "$CONDUCTOR" pipeline -v nightly
echo
echo "完了しても、ステートは残る。watch は、表の下に「完了」を示す行を出す:"
cmd "$CONDUCTOR" watch
states
echo
echo "--remove-on-complete を付けると、完了でステートファイルを消す(次に watch すると、ステートはない):"
cmd "$CONDUCTOR" pipeline -v --remove-on-complete nightly
states
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
pause

scene "7. 定義を検査する(validate): ジョブは実行せず、定義の誤りだけを見つける"
cmd "$CONDUCTOR" validate -v
echo
echo "command の中の引用符(')が閉じていない定義(bad.toml)を作って、検査する:"
printf 'format_version = 1\n\n[pipeline]\nname = "bad"\n\n[[job]]\nid = "extract"\ncommand = "sh -c '"'"'echo hi"\n' > bad.toml
cat bad.toml
cmd "$CONDUCTOR" validate --config bad.toml
echo
echo "ジョブは実行されず(出力なし)、ステートも作られない:"
states
pause

scene "8. run ID でステートを分ける: run ID ごとに、別のステートファイルになる"
echo "run ID build-1 で実行し、load で止める(--run-id):"
( DEMO_LOAD_EXIT=2; export DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v --run-id build-1 nightly )
echo
echo "別の run ID build-2 を、環境変数 DEMO_RUN_ID で指定して実行する(定義の run_id_env = \"DEMO_RUN_ID\"):"
setenv DEMO_RUN_ID=build-2
( DEMO_RUN_ID=build-2; export DEMO_RUN_ID; cmd "$CONDUCTOR" pipeline -v nightly )
echo
echo "build-2 は最後まで進み、build-1 のステートは、そのまま残る:"
states
echo
echo "build-1 の止まった位置を見る:"
cmd "$CONDUCTOR" watch --run-id build-1
echo
echo "同じ run ID で同じコマンドをもう一度: load から再開する:"
cmd "$CONDUCTOR" pipeline -v --run-id build-1 nightly
echo
echo "完了しても、ステートファイルは残る(watch は完了を示す):"
states
cmd "$CONDUCTOR" watch --run-id build-1
echo
echo "片づけは reset(run ID を指定して、1 つずつ消す):"
cmd "$CONDUCTOR" reset --run-id build-1
cmd "$CONDUCTOR" reset --run-id build-2
states
pause

scene "9. 環境変数と後始末: --run-id は環境変数に勝つ。reset --run-id と reset --all"
echo "環境変数だけで run ID env-1 を指定して、load で止める:"
setenv DEMO_RUN_ID=env-1
( DEMO_RUN_ID=env-1; DEMO_LOAD_EXIT=2; export DEMO_RUN_ID DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v nightly )
echo
echo "環境変数が env-1 でも、--run-id opt-1 が勝つ(別のステートになる):"
setenv DEMO_RUN_ID=env-1
( DEMO_RUN_ID=env-1; DEMO_LOAD_EXIT=2; export DEMO_RUN_ID DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v --run-id opt-1 nightly )
echo
echo "--run-id \"\"(空)は、環境変数があっても、run ID なしに固定する:"
setenv DEMO_RUN_ID=env-1
( DEMO_RUN_ID=env-1; DEMO_LOAD_EXIT=2; export DEMO_RUN_ID DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v --run-id "" nightly )
echo
echo "止まった run をもう 1 つ(opt-2):"
( DEMO_LOAD_EXIT=2; export DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline -v --run-id opt-2 nightly )
echo
echo "止まった run が、別々のステートファイルになっている:"
states
echo
echo "run ID を指定して、1 つだけ消す:"
cmd "$CONDUCTOR" reset --run-id opt-1
states
echo
echo "reset --all で、残りをすべて消す:"
cmd "$CONDUCTOR" reset --all
states
cmd "$CONDUCTOR" watch
pause

scene "10. リトライ: 一時的な失敗(終了コード 75)なら、間隔を空けて、実行し直す"
echo "定義(retry.toml)。[job.retry] に、リトライする終了コードと、最大の回数、間隔を書く:"
cat retry.toml
echo
echo "flaky は、2 回失敗してから成功する。実行し直す前に、conductor が知らせを出す(標準エラー):"
rm -f flaky.count
cmd "$CONDUCTOR" pipeline -v --config retry.toml retry-demo
echo
echo "(回数を使い切っても失敗するときは、通常のエラーと同じ: 最後の終了コードで止まり、同じコマンドで、そのジョブから再開できる)"
pause

scene "11. 環境変数: [pipeline.env]・[job.env]・.env(env_file)を、ジョブに足す"
echo "定義(env.toml)と .env(demo.env)。env_file の相対パスは、定義ファイルのあるディレクトリが基準:"
cat env.toml
echo
cat demo.env
echo
echo "ジョブには、引き継いだ環境に足した環境変数が渡る(優先順位: .env < [pipeline.env] < [job.env])。"
echo "CONDUCTOR_* は、conductor が自動で渡す(run ID がなければ CONDUCTOR_RUN_ID は渡さない)。"
echo "unset_env の DEMO_PROXY は、親に置いてあっても、ジョブには渡らない:"
setenv DEMO_PROXY=from-parent
( DEMO_PROXY=from-parent; export DEMO_PROXY; cmd "$CONDUCTOR" pipeline -v --config env.toml env-demo )
echo
echo "--dry-run は、足す名前と並べて、消す名前も出す:"
cmd "$CONDUCTOR" pipeline --dry-run --config env.toml env-demo
echo
echo "値は、conductor 自身の出力(標準出力・標準エラー・ステート)には出ない。.env が読めないときは、ジョブを動かさずに 64 で止まる:"
printf 'format_version = 1\n\n[pipeline]\nname = "bad-env"\nenv_file = "missing.env"\n\n[[job]]\nid = "x"\ncommand = "sh scripts/show-env.sh"\n' > bad-env.toml
cmd "$CONDUCTOR" pipeline --config bad-env.toml bad-env
echo
echo "validate は、.env も検査する。同じ名前が 2 回あると、警告を出す(終了コードは 0):"
printf 'A=1\nA=2\n' > dup.env
printf 'format_version = 1\n\n[pipeline]\nname = "dup-env"\nenv_file = "dup.env"\n\n[[job]]\nid = "x"\ncommand = "sh scripts/show-env.sh"\n' > dup-env.toml
cmd "$CONDUCTOR" validate --config dup-env.toml
echo
scene "12. 遷移を図にする: graph は、ジョブを実行せず、定義の遷移を図にする"
echo "定義(conductor.toml)の遷移を図にする(extract が警告なら notify へ飛ぶ。右の線が、飛び越しを表す):"
cmd "$CONDUCTOR" graph
echo
echo "-v で、ジョブごとに、コマンドと、飛び越しの注記を足す:"
cmd "$CONDUCTOR" graph -v
echo
echo "--format mermaid は、Mermaid の図を出す(Markdown や Web ページに貼れる):"
cmd "$CONDUCTOR" graph --format mermaid
echo
echo "--format dot は、Graphviz の DOT を出す(dot -Tsvg などで画像にできる):"
cmd "$CONDUCTOR" graph --format dot
echo
echo "--state で、ジョブの状態と再開位置を重ねる。まず、load で止める:"
( DEMO_LOAD_EXIT=2; export DEMO_LOAD_EXIT; cmd "$CONDUCTOR" pipeline nightly )
cmd "$CONDUCTOR" graph --state
echo
echo "graph は、ステートを読むだけ(書かない)。後片付け:"
cmd "$CONDUCTOR" reset
pause

scene "13. 実行計画: --dry-run は、ジョブを実行せず、実行したら何が起こるかを見せる"
echo "--dry-run で、実行計画を出す(ジョブは実行されず、ステートも作られない):"
cmd "$CONDUCTOR" pipeline nightly --dry-run
echo
echo "--assume で、ジョブの終了コードを仮定する(extract が警告=終了コード 1 なら、transform と load は飛ばされる):"
cmd "$CONDUCTOR" pipeline nightly --dry-run --assume extract=1
echo
echo "load が終了コード 2 で失敗すると仮定すると、そこで停止する:"
cmd "$CONDUCTOR" pipeline nightly --dry-run --assume load=2
echo
echo "run でも使える(次の動きを示す):"
cmd "$CONDUCTOR" run extract --dry-run
echo
echo "実際には何も実行されず、ステートも作られていない:"
states
pause

scene "14. ひな形: init は、定義のひな形を作る(既にあるファイルは壊さない)"
echo "init で、定義のひな形を作る(名前は省略すると my-pipeline。ここでは starter):"
cmd "$CONDUCTOR" init --config new.toml starter
echo
echo "作ったひな形は、validate に通る:"
cmd "$CONDUCTOR" validate --config new.toml starter
echo
echo "もう一度 init すると、既にあるので、何も変えず、終了コード 69 で終わる:"
cmd "$CONDUCTOR" init --config new.toml starter
echo
echo "--force を付けると、上書きする:"
cmd "$CONDUCTOR" init --config new.toml --force starter
echo
echo "--config - なら、ファイルを作らずに、ひな形を標準出力に出す:"
cmd "$CONDUCTOR" init --config - starter
rm -f new.toml
pause

scene "15. ログのファイル: [pipeline] log_dir で、ジョブの出力を端末に出しつつ、ファイルにも残す"
echo "定義(log.toml)。log_dir に、ログのディレクトリを書く:"
cat log.toml
echo
echo "flaky は、2 回失敗してから成功する。端末の出力は、log_dir が無いときと同じ:"
rm -f flaky.count
cmd "$CONDUCTOR" pipeline --config log.toml log-demo
echo
echo "試行ごとに 1 つのファイル(<ジョブ ID>.<開始の時刻(UTC)>.<試行の番号>.log)。本人だけが読み書きできる(0600):"
echo "$ ls -l logs"
ls -l logs
for f in logs/*.log; do
  echo "--- $f"
  cat "$f"
done
echo
echo "conductor は、古いログを消さない。片づけは利用者が行う:"
echo "$ rm -r logs"
rm -r logs
cmd "$CONDUCTOR" reset --config log.toml
pause

echo "デモは以上です。"
