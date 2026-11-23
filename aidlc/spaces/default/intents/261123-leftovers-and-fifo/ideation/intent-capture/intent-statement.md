# 意図の記述(Intent Statement): 残りの整理と読み込みの順序のそろえ

## Sources

- [desc] Initial description: "4 回目の調査の候補を 1 つの intent で(段 1 は動きを変えない整理、段 2 は利用者から見える変更、最後に版を 0.9.8 に): (段 1) 古くなった説明(runtime の lib.rs の表と runner.rs の run_job の job の記述、read_state_raw・write_state・lib.rs の run ID の関数の説明、messages.rs の U3 のメモ、step.rs・run.rs の経緯の文、types.rs の is_raised、report/mod.rs の escape の説明、dry_run.rs と aux_commands.rs の到達しない守りの理由、仕様書 05 章の apply_interrupt_in)、使われないもの・二重のもの(apply_interrupt_in、EffectiveEnv の job_id と names、core のテストの補助の _job などの使わない引数、模擬の分類の二重の計算、decide の常に真の条件、interrupted_signal、command_line、一時ファイルに書いて置き換える処理の二重、ステートの読み込みの補助の段と read_limited、build_retry の所有権)、公開範囲(core の flow・plan::simulate・envfile と graph の下位モジュールの pub mod と定数、runtime の pub(crate) の狭め残し)、型と名前(escape_path の名前、run ID の有無の bool、converter の行番号の 0 と reason の String)、計算量(定義の env の表の大文字小文字の検査・遷移の重複・env_file の重複の二乗、cli の -v の図の O(N×E)、record_skip の複製)、テスト(src の単体テストの定義の手組み、FakeEnv の言語分け・Deps の組み立て・state_error・runtime の def/put/paths の重複、深さの境界のテストの重複、runtime のテストの run_job の名残)。(段 2) converter の読み込みを形 B に(名前付きパイプで止まらない)、ステートの読み込みも形 B に、空白だけのコマンド名(文字列の \\x27  \\x27)を配列と同じく定義の誤りに。判断が要るもの: cli の Runtime::run_job の _job、core と converter の src のテストの cfg(unix)、計画の型で方式ごとの項目を結ぶこと"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(型の変更に伴う直しと補助の置き換えは認める。期待の文字列は変えない。重複したテストを消すときは、残るテストと対にして一覧に示す。一覧は検索で網羅し、設計で承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(cli の `Runtime::run_job` の `_job` を外し、本番と同じ形にする。偽物は環境変数 `CONDUCTOR_JOB_ID` からジョブ ID を記録する)
- [Q4] `intent-capture-questions.md` Q4: C(core と converter の src のテストの `cfg(unix)` は今回は直さない)
- [Q5] `intent-capture-questions.md` Q5: A(段 2 の 3 つの変更をすべて入れる)
- [Q6] `intent-capture-questions.md` Q6: A(性能のテストを足し、上限は直した後の見込みの約 5 倍。前後の時間を記録する)
- [Q7] `intent-capture-questions.md` Q7: A(計画の型で方式ごとの項目を結ぶことは、今回は直さない)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Problem Statement

- 直前の変更の後で、説明が古くなった所、使われなくなった関数や引数、二重の処理、狭め残しの公開範囲、名前と使い方が合わない所が残っている。読む人を誤らせ、直すときの手間を増やす。[desc]
- 定義の `env` の表の検査などに、入力の大きさの二乗の時間がかかる所がある。テストの補助にも重複が残っている。[desc]
- converter とステートの読み込みは、定義ファイルや `.env` の読み込みの順(形 B)とそろっておらず、名前付きパイプで止まりうる。空白だけのコマンド名は、配列と文字列で扱いが食い違う。[desc]
- 解決したいのは、前の 2 つを利用者から見える動きを変えずに直し、最後の 1 つを利用者に分かる変更としてそろえて、版を上げることである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]
- 段 2 については、`convert` とステートを使う利用者と、定義を書く利用者(まれな場合の誤りの種類と終了コードが変わる)。[desc] [Q5]

## Success Metrics

- 段 1(動きを変えない整理): 初期の記述の (段 1) の項目のそれぞれについて、説明が実装と一致し、使われないものと二重のものがなくなり、公開範囲が狭まり、名前と型が使い方に合う。利用者から見える動きは変わらない。[desc]
  - cli の `Runtime::run_job` から `job` の引数を外し、runtime の本物と同じ形にする。テストの偽物は、環境変数 `CONDUCTOR_JOB_ID` からジョブ ID を記録し、今と同じ記録と台本の引き当てを保つ。[Q3]
  - 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形、終わったと判断する基準を一覧にし、実装の後にその場所を 1 つずつ確かめる(Q2 の「検索で網羅して設計の段で承認を取る」と同じ一覧に含める)。[Q1] [Q2]
- 計算量: 初期の記述の二乗の時間の項目が、入力の大きさに比例する時間(またはそれに近い時間)で終わる。性能のテストを足し、上限は直した後の見込みの約 5 倍にし、直す前と後の時間を記録する。[desc] [Q6]
  - 見込みは、設計の段で入力の大きさ(上限の定数などの根拠つき)を決め、直す前の時間を実際に測ってから見積もり、上限の数と一緒に書く。[Q6]
- 段 2(利用者から見える変更): 初期の記述の (段 2) の 3 つを入れる。[desc] [Q5]
  - converter とステートの読み込みが、定義ファイルと同じ「種類を先に見る → 開く → 開いたものから大きさを見る」の順になり、書き手のいない名前付きパイプでも止まらずに「読めない」と報告する。ほかの場合の誤りの種類は変わらないことを、テストで確かめる。[Q5]
  - 空白だけのコマンド名は、文字列で書いても配列と同じく定義の誤り(終了コード 64)になる。[Q5]
  - 変更を仕様書に書き、前の intent と同じく「v0.9.7 までは…」と注記する。誤りの表示に入力の値を含めない性質は保つ(`aidlc/spaces/default/memory/project.md` の Forbidden の決まり)。[Q5]
- 版: 最後のコミットで v0.9.8 に上げる。[desc]
  - タグは `main` の上に付け、リリースは下書きとして作って人が確認してから公開する(チームの決まり)。この intent ではタグとリリースはせず、終わった後に頼まれたら行う。[memory:M6]
- 既存のテストの扱い: 型の変更に伴う直し、補助の置き換えは変えてよい。確かめる中身と期待する出力の文字列は変えない。重複したテスト(深さの境界など)を消すときは、消すテストと同じことを確かめて残るテストを対にして示す。一覧は検索で網羅して設計の段で示し、承認を取る。[Q2]
  - 段 2 で期待が変わるテスト(版の表示など)も、設計の段で一覧に挙げて承認を取る。[Q5] [desc]
  - 確かめる手段: Q1 の「各段の終わりにテストが通ることを確かめる」に従い、段の前と後でテストの件数も記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。基準は、実装を始める前の HEAD で `cargo test --locked --workspace` を macOS で実行した通過の件数とする。[Q1] [Q2] [Q6]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- v0.9.7 の後のリファクタリングの候補の調査(4 回目)で挙がったものを、1 つの intent で片付ける。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (段 1) と (段 2)。段はこの順で、利用者から見える変更の段を最後に置く。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 次の 3 つ。[Q4] [Q7] [Q2]
  - core と converter の src のテストの `cfg(unix)`(今回は直さない)。[Q4]
  - 計画の型で方式ごとの項目を結ぶこと(今回は直さない)。[Q7]
  - 直す仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` とし、凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は前の intent `261122-internal-cleanup` の記録の決定に従い変えない(前の決定の引き継ぎ)。[Q2]

## Assumptions & Open Questions

None.
