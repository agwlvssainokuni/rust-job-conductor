# 規則: v0.10.1 の整理・修正

出典: 要件(`requirements.md`、承認済み。要件の承認で直すと決めた R-07 を含む)、設計の質問(`functional-design-questions.md` の Q1 と、それを見直した Q2)、コードの知識(`aidlc/spaces/default/codekb/rust-job-conductor/`)。

```yaml
rules:
  - id: BR1.1
    statement: README の「エラーの後に続くジョブがあるときの注意」を、起きる使い方(エラーで止まったステートを watch --follow で見ながら再開するとき)に限って書き、「パイプラインを始める前から見ておけます」の使い方では起きないことが読み取れるようにする
    category: policy
    applies_to: README.md の watch --follow の節
    trigger: 文書の更新
    logic: IF 読み手が始める前から見ている THEN 注意は当てはまらない。IF エラーで止まったステートを見ながら再開する THEN 最初の描画で 0 で終わることがある、と読める
    violation: 文書の検査(docs-check、src/tests/docs.rs)が落ちる、またはレビューで差し戻す
    source: FR1.1
  - id: BR1.2
    statement: 注意に、遷移で次のジョブへ続く実行の例(再開の位置のジョブが試行を書く前の最初の描画で終わる例と、その避け方)を載せる
    category: policy
    applies_to: README.md の watch --follow の節
    trigger: 文書の更新
    logic: 例は、どのコマンドを、どの順で、どの端末で動かすかと、何が起きるか、どうすれば最後まで見られるかを含む
    violation: レビューで差し戻す
    source: FR1.2
  - id: BR1.3
    statement: watch --follow の「止まった」の判定(core の flow/activity.rs)は変えない
    category: constraint
    applies_to: crates/core/src/flow/activity.rs
    trigger: 変更のとき
    logic: IF activity.rs に差分がある THEN 範囲の外の変更として戻す
    violation: コード生成の確かめで差分を見つけたら戻す
    source: FR1.3
  - id: BR2.1
    statement: ログの名前で、最初の "." より前の部分を予約名と比べるとき、その部分の前後の ASCII の空白を除いた値で比べる。名前そのものは今の規則で作る
    category: validation
    applies_to: runtime の safe_log_stem
    trigger: ログのファイルを作るとき
    logic: IF 置き換え後の名前の最初の "." より前の部分から前後の空白を除いた値が、予約名(CON、PRN、AUX、NUL、COM1〜COM9、LPT1〜LPT9、大文字小文字を問わない)のどれかと等しい THEN 名前の先頭に "_" を足す
    violation: 該当なし(名前を作るだけで、誤りにしない)
    source: FR2.1
  - id: BR2.2
    statement: BR2.1 の結果は、要件の FR2.1 の表(con、console、"con "、" con"、" con "、"con .x"、"nul .tar.gz")のとおりになる。README の「名前の置き換え」に空白の例を足し、runtime の log_file.rs の名前の表に同じ行を足す
    category: constraint
    applies_to: safe_log_stem、README.md、crates/runtime/tests/log_file.rs、リリースの注記
    trigger: 変更のとき
    logic: IF 表のどれかの行と結果が違う THEN テストが落ちる
    violation: テストが落ちる
    source: FR2.2
  - id: BR2.3
    statement: 出力を写す読み取りのスレッドは、read を呼ぶ前に「止める」印を見て、立っていたら読まずに終わる
    category: policy
    applies_to: runtime の copy_stream
    trigger: 読みの繰り返しの各回の初め
    logic: IF 印が立っている THEN read を呼ばずに繰り返しを抜ける
    violation: 単体テスト(印が立った状態で呼ぶと read が 1 回も呼ばれない)が落ちる
    source: FR2.3
  - id: BR2.4
    statement: 読んだ後に印を見て、立っていたら読んだ分をどこにも書かずに終わる今の確かめは、動きを変えずに残す
    category: constraint
    applies_to: runtime の copy_stream
    trigger: read が返った後
    logic: IF read が返った後に印が立っている THEN 読んだ分を端末にもファイルにも書かずに抜ける(今と同じ)
    violation: 既存のテスト(runner_log.rs など)が落ちる
    source: FR2.3
  - id: BR3.1
    statement: plan_simulate_perf.rs の上限 150ms と、テストの中身は変えない
    category: constraint
    applies_to: crates/core/tests/plan_simulate_perf.rs
    trigger: 変更のとき
    logic: IF LIMIT や計測の形が変わる THEN 範囲の外の変更として戻す
    violation: コード生成の確かめで差分を見つけたら戻す
    source: FR3.1, NFR8
  - id: BR3.2
    statement: 直す前に、ジョブの数 250・500・1,000・2,000 で、通常の cargo test とカバレッジの計測の下の両方の所要時間を測り、code-summary.md に記録する。時間のかかる関数も特定する。測るための仕組みは使い捨てとし、リポジトリに残さない
    category: policy
    applies_to: コード生成の作業
    trigger: F3 の作業の初め
    logic: 各ジョブの数で、計画を作る時間の中央値を記録する
    violation: 記録が無ければ、F3 の直しに進まない
    source: FR3.2
  - id: BR3.3
    statement: カバレッジの計測の下で、ジョブの数を 2 倍にしたときの所要時間が、どの段でも 3 倍以上に伸びたら、二乗に近い比較が原因と判定する
    category: calculation
    applies_to: BR3.2 の測定の結果
    trigger: 測定の後
    logic: IF 250→500、500→1,000、1,000→2,000 のすべてで伸びが 3 倍以上 THEN 原因と判定して BR3.4 に進む。ELSE BR3.6 に進む
    violation: 該当なし(判定の規則)
    source: FR3.3
  - id: BR3.4
    statement: 直すときは、crates/core/src/flow/ の中に、ジョブの位置(と最初の済んでいないジョブの位置の目印)を受け取る内部の入口(pub(crate))を足し、計画の模擬はそれを呼ぶ。判断と記録の規則は flow の中の 1 か所のままにし、plan に書き写さない。今の公開の入口は、位置を自分で求めて同じ本体を呼ぶ。共有の公開の関数 job_index・record_of そのものと、実行の流れの動きと結果、計画の結果は変えない
    category: constraint
    applies_to: crates/core/src/flow/、crates/core/src/plan/simulate.rs
    trigger: BR3.3 で原因と判定したとき
    logic: IF plan に判断や記録の規則の写しがある、または job_index・record_of の中身が変わる、または既存の流れ・計画のテストが落ちる THEN 戻す
    violation: コード生成の確かめで見つけたら戻す。既存の流れと計画のテストが落ちる
    source: FR3.3
  - id: BR3.5
    statement: 直した後、カバレッジの計測の下で 1,000 ジョブの計画が 75ms 以下で作れ、ジョブの数を 2 倍にしたときの伸びが 2.5 倍以下になる
    category: validation
    applies_to: 直した後の測定
    trigger: 直した後
    logic: IF 75ms を超える、または伸びが 2.5 倍を超える THEN 直しを戻して BR3.6 に進む(直しの繰り返しはしない)
    violation: Build and Test の確かめで不合格
    source: FR3.3
  - id: BR3.6
    statement: 原因と判定しなかったとき、または直しても BR3.5 に届かず戻したときは、コードもテストも変えない。測った結果の表と、直さない理由(どの段で 3 倍に届かなかったか、または届かなかった直しの結果)を build-and-test-summary.md に書く(要件の FR3.3 のとおり)。code-summary.md の測定の表はそのまま残す
    category: policy
    applies_to: この intent の記録
    trigger: BR3.3 で原因と判定しなかったとき、または BR3.5 に届かなかったとき
    logic: 測定の表と理由を build-and-test-summary.md に書く
    violation: 記録が無ければ Build and Test を終えない
    source: FR3.3
  - id: BR4.1
    statement: build-and-test-summary.md に節「原因不明の失敗の記録の手順」を置き、残すもの(実行の番号か手元の日時、落ちたテストの名前、失敗の出力、OS、コミット)と、調べ始めるときの最初の手がかり(実時計を挟んで比べるテスト)を書く
    category: policy
    applies_to: construction/build-and-test/build-and-test-summary.md
    trigger: Build and Test のとき
    logic: IF 節が無い、または 5 つの残すもののどれかが無い THEN 未完了
    violation: Build and Test の確かめで不合格
    source: FR4.1
  - id: BR5.1
    statement: UNIX 版のデモの場面 4 で、DEMO_PAUSE=1 のときは、watch --follow を呼ぶ前に今の pause の関数で Enter を待つ
    category: policy
    applies_to: demo/run-demo.sh
    trigger: 場面 4 で watch --follow の説明を出した後、呼ぶ前
    logic: IF DEMO_PAUSE が "1" THEN "(Enter で次へ)" を出して 1 行を読む。ELSE 何もしない
    violation: BR5.5 のテストが落ちる
    source: FR5.1
  - id: BR5.2
    statement: DEMO_PAUSE が無いときの、デモの出力と動きは変えない。conductor の動きは変えない
    category: constraint
    applies_to: demo/run-demo.sh、conductor
    trigger: 変更のとき
    logic: IF 既存の unix_demo.rs の DEMO_PAUSE の無いテストが落ちる THEN 戻す
    violation: 既存のテストが落ちる
    source: FR5.2
  - id: BR5.3
    statement: Windows 版のデモ(demo/windows/run-demo.cmd)は変えない。Windows の watch --follow は画面を消さずに足していくため
    category: constraint
    applies_to: demo/windows/run-demo.cmd
    trigger: 変更のとき
    logic: IF run-demo.cmd に差分がある THEN 範囲の外として戻す
    violation: コード生成の確かめで差分を見つけたら戻す
    source: FR5.3
  - id: BR5.4
    statement: demo/README.md の DEMO_PAUSE の説明に、UNIX 版では場面 4 の watch --follow の前でも待つことを書き足す
    category: policy
    applies_to: demo/README.md
    trigger: BR5.1 と同じコミットの近く
    logic: 説明の「各場面のあとで Enter を待つ」に 1 文を足す
    violation: レビューで差し戻す
    source: FR5.4
  - id: BR5.5
    statement: unix_demo.rs に、DEMO_PAUSE=1 を足してデモを動かすテストを足す。標準入力は今の補助 run_limited の既定(空)のままにし、read は EOF ですぐ戻る。場面 4 で、1 回の watch の完了の行の後、watch --follow を呼ぶコマンドの行の前に、"(Enter で次へ)" が出ることを確かめる。今の補助は変えない
    category: validation
    applies_to: crates/cli/tests/unix_demo.rs
    trigger: テストのとき
    logic: 場面 4 の範囲の出力で、"(Enter で次へ)" の位置が、watch --follow のコマンドの行より前で、1 回の watch の完了の行より後
    violation: テストが落ちる
    source: FR5.5
  - id: BR6.1
    statement: crates/runtime/src/lib.rs の契約の表の run_job の行に、unset の引数(消す環境変数の名前の並び)を書き足す
    category: policy
    applies_to: crates/runtime/src/lib.rs
    trigger: 文書の更新
    logic: 表の引数の並びを、run_job の今の引数(line、env、unset、signal、log)に揃える
    violation: レビューで差し戻す
    source: FR6.1
  - id: BR6.2
    statement: DefinitionErrorKind の今の数を指す「33 種」を「34 種」に直す。対象は要件の FR6.2 の一覧と、要件の承認で加えた crates/cli/src/tests/report_messages.rs の 1225 行の注釈。12-rebuild-guide.md の 105 行(履歴)と 285 行(入れ子の深さ)は残す
    category: constraint
    applies_to: docs/spec/rust/、crates/core/tests/env_types.rs、crates/cli/src/tests/report_messages.rs
    trigger: 文書の更新
    logic: grep -rn '33 種' docs/spec/rust/ crates/ の結果が 12-rebuild-guide.md の 105 行だけになる
    violation: Build and Test の確かめで不合格
    source: FR6.2
  - id: BR7.1
    statement: すべてを片づけてから、crates/cli/Cargo.toml の版を 0.10.1 にする。件名は「版を 0.10.1 にする」の 1 コミット
    category: policy
    applies_to: crates/cli/Cargo.toml、Cargo.lock
    trigger: 他のすべての変更の後
    logic: IF 他の変更が残っている THEN 版を上げない
    violation: 順序の誤りとしてレビューで差し戻す
    source: FR7.1
```

## 規則のまとめ

| ID | 種類 | 対象 | 要約 | 出典 |
|---|---|---|---|---|
| BR1.1 | policy | README | 注意を起きる使い方に限って書く | FR1.1 |
| BR1.2 | policy | README | 遷移で続く実行の例を載せる | FR1.2 |
| BR1.3 | constraint | activity.rs | 判定は変えない | FR1.3 |
| BR2.1 | validation | safe_log_stem | 前後の空白を除いて予約名と比べる | FR2.1 |
| BR2.2 | constraint | 名前の表 | 要件の表のとおり。README・テスト・注記を揃える | FR2.2 |
| BR2.3 | policy | copy_stream | read の前に印を見る | FR2.3 |
| BR2.4 | constraint | copy_stream | 読んだ後の確かめは変えない | FR2.3 |
| BR3.1 | constraint | perf のテスト | 上限とテストを変えない | FR3.1、NFR8 |
| BR3.2 | policy | 測定 | 4 つのジョブの数で 2 つの条件を測る。仕組みは残さない | FR3.2 |
| BR3.3 | calculation | 判定 | 2 倍で 3 倍以上なら原因 | FR3.3 |
| BR3.4 | constraint | flow・plan | flow に位置を受け取る内部の入口。規則は 1 か所 | FR3.3 |
| BR3.5 | validation | 直した後 | 75ms 以下、伸び 2.5 倍以下。届かなければ戻す | FR3.3 |
| BR3.6 | policy | 記録 | 直さない・戻したときは記録だけ | FR3.3 |
| BR4.1 | policy | 記録 | 失敗の記録の手順の節 | FR4.1 |
| BR5.1 | policy | run-demo.sh | 場面 4 の follow の前で待つ | FR5.1 |
| BR5.2 | constraint | デモ | 待たないときは変えない | FR5.2 |
| BR5.3 | constraint | run-demo.cmd | Windows 版は変えない | FR5.3 |
| BR5.4 | policy | demo/README.md | 待つ所を書き足す | FR5.4 |
| BR5.5 | validation | unix_demo.rs | 待ちの表示の位置を確かめる | FR5.5 |
| BR6.1 | policy | runtime lib.rs | 契約の表に unset | FR6.1 |
| BR6.2 | constraint | 仕様書ほか | 33 種を 34 種に。履歴は残す | FR6.2 |
| BR7.1 | policy | 版 | 最後に 0.10.1 | FR7.1 |
