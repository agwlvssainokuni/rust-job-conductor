# 意図の把握の質問(Intent Capture Questions): 残りの整理と読み込みの順序のそろえ

## Sources

- [desc] Initial description: "4 回目の調査の候補を 1 つの intent で(段 1 は動きを変えない整理、段 2 は利用者から見える変更、最後に版を 0.9.8 に): (段 1) 古くなった説明(runtime の lib.rs の表と runner.rs の run_job の job の記述、read_state_raw・write_state・lib.rs の run ID の関数の説明、messages.rs の U3 のメモ、step.rs・run.rs の経緯の文、types.rs の is_raised、report/mod.rs の escape の説明、dry_run.rs と aux_commands.rs の到達しない守りの理由、仕様書 05 章の apply_interrupt_in)、使われないもの・二重のもの(apply_interrupt_in、EffectiveEnv の job_id と names、core のテストの補助の _job などの使わない引数、模擬の分類の二重の計算、decide の常に真の条件、interrupted_signal、command_line、一時ファイルに書いて置き換える処理の二重、ステートの読み込みの補助の段と read_limited、build_retry の所有権)、公開範囲(core の flow・plan::simulate・envfile と graph の下位モジュールの pub mod と定数、runtime の pub(crate) の狭め残し)、型と名前(escape_path の名前、run ID の有無の bool、converter の行番号の 0 と reason の String)、計算量(定義の env の表の大文字小文字の検査・遷移の重複・env_file の重複の二乗、cli の -v の図の O(N×E)、record_skip の複製)、テスト(src の単体テストの定義の手組み、FakeEnv の言語分け・Deps の組み立て・state_error・runtime の def/put/paths の重複、深さの境界のテストの重複、runtime のテストの run_job の名残)。(段 2) converter の読み込みを形 B に(名前付きパイプで止まらない)、ステートの読み込みも形 B に、空白だけのコマンド名(文字列の \\x27  \\x27)を配列と同じく定義の誤りに。判断が要るもの: cli の Runtime::run_job の _job、core と converter の src のテストの cfg(unix)、計画の型で方式ごとの項目を結ぶこと"
- [scope] Workflow-selected scope: `refactor`.
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Q1. [プロダクト] 進め方(段ごとに止まるか)

背景: 中身は 2 つの段(動きを変えない整理、利用者から見える変更)で、30 コミット前後の見込みです。

- A. 設計とコードの計画は 1 回で承認し、実装は段の順に進める。各段の終わりにテストが通ることを確かめ、結果は最後にまとめて見せる(前回と同じ)
- B. 段ごとに止まり、結果を見せて、続けるかを確かめてから次の段へ進む
- X. Other (please specify)

[Answer]: A

## Q2. [プロダクト] 既存のテストの扱い

背景: 段 1 では、型や関数の形を変えるのに合わせて、テストの組み立てや項目の読み方を直す必要が出ます。また、テストの補助の重複をまとめ、単体テストと統合テストで重なっている深さの境界のテストを減らします(テストを消すと件数が減ります)。

- A. 前回と同じく、組み立て・import・項目の読み方・補助の置き換えは認める。確かめる中身と期待する出力の文字列は変えない。重複したテストを消すときは、消すテストと、同じことを確かめている残るテストを対にして一覧に示す。一覧は検索で網羅し、設計の段で承認を取る
- B. テストを消すことはせず、補助の置き換えと型の変更に伴う直しだけにする(深さの境界のテストの重複は範囲から外す)
- X. Other (please specify)

[Answer]: A

## Q3. [プロダクト] cli の `Runtime::run_job` に残した `_job` の扱い

背景: 前々回、cli のテストの偽物がジョブ ID を記録するために、`Runtime::run_job` に使わない `_job` の引数を残しました。偽物に渡る環境変数には、いつも `CONDUCTOR_JOB_ID` が入っているので、そこからジョブ ID を取ることもできます。

- A. `_job` を外し、本番の `run_job(line, env, signal)` と同じ形にする。偽物は環境変数の `CONDUCTOR_JOB_ID` からジョブ ID を記録する
- B. 前々回の決定のまま残す(範囲から外す)
- X. Other (please specify)

[Answer]: A(詳しい説明と関係するファイル(`crates/cli/src/deps.rs` のトレイトと本番、`retry.rs` の呼び出し、`crates/cli/tests/common/mod.rs` の偽物 2 つ)を聞いたうえで A。説明の途中で B を勧めたが、A を選んだ)

## Q4. [プロダクト] core と converter の src のテストにある `cfg(unix)`

背景: 「`cfg(unix)`/`cfg(windows)` は platform の外に置かない」という決まりがありますが、core と converter の src の単体テストのモジュールの中に `cfg(unix)` があります(権限と名前付きパイプのテスト)。検査(`nfr_source.rs`)は cli と runtime にしかないので、今は違反として検出されません。

- A. そのテストを、公開の入口(`read_definition`、`convert`)で書ける統合テスト(`tests/`)へ移す。src から `cfg(unix)` がなくなる
- B. 決まりに「テストのモジュールは除外する」と書き足し、テストは今の場所に残す
- C. 今回は直さない(範囲から外す)
- X. Other (please specify)

[Answer]: C

## Q5. [プロダクト] 段 2 の利用者から見える変更をどこまで入れるか

背景: 段 2 には 3 つの変更があります。(1)converter の読み込みを定義ファイルと同じ順にする(書き手のいない名前付きパイプを `convert` に渡しても止まらない)。(2)ステートの読み込みも同じ順にする(ステートのパスが名前付きパイプでも止まらない。仕様書 06 章の改訂が要る)。(3)空白だけのコマンド名(`command = "'  '"`)を、配列の `["  "]` と同じく定義の誤り(終了コード 64)にする(今は起動に失敗して 127)。

- A. 3 つとも入れる
- B. (1)と(2)だけ入れる(読み込みの順序だけそろえる)
- C. (1)だけ入れる
- X. Other (please specify)

[Answer]: A

## Q6. [プロダクト] 計算量の改善の確かめ方

背景: 段 1 で、定義の `env` の表の大文字小文字の検査(項目の数に上限が無い)などの二乗の時間を直します。前回は、性能のテストを足し、上限を「直した後の見込みの約 5 倍」にしました(CI で落ちないことを優先)。

- A. 前回と同じく、性能のテストを足し、上限は直した後の見込みの約 5 倍にする。直す前と後の時間を記録する
- B. 性能のテストは足さず、直す前と後の時間を測って記録するだけにする
- X. Other (please specify)

[Answer]: A

## Q7. [プロダクト] 計画の型で、方式ごとの項目を型で結ぶか

背景: 計画の型は、方式(run か pipeline か)、終わり方、次の動きの注記を別々に持ち、方式によって片方だけを読みます。型で結ぶこともできますが、core と cli のテストが変わる割に、得られるものは小さいと見ています。

- A. 今回は直さない(範囲から外す)
- B. 型で結ぶ(段 1 に入れる)
- X. Other (please specify)

[Answer]: A
