# 意図の記述(Intent Statement): 動きを変えない小さな整理とテストの補助

## Sources

- [desc] Initial description: "動きを変えない小さな整理とテストの補助(調査の 1 と 2): (1) 文言の定数の重複(cli の messages.rs、core の環境変数の名前の規則・CONDUCTOR_・format_version・間隔の上下限)、runtime の一時ファイルの作り方の二重と境界、cli の push_line・job の場所・reset の案内の重複、到達しない分岐と効かない処理(decide の規則 5、args の lexopt の枝、retry.rs の既定値、simulate の unwrap_or と walk の try_from)、文字列での比較(コマンド名の逆引き、誤りの対象の直書き、reset --all の理由の文の比較)、説明のずれ(types.rs の生成元、sanitize のテスト名、args.rs の文法の表、仕様書 08 章の拒否の理由の表)、使われない program_name、walk の整理(success_declared、build_jobs の clone、integer_of の重複、depth の分岐)、(2) テストの補助の整理(README の読み取りと必須語句、run/kinds・median・SECRET・ステートの包み・status_of・send の重複、core の tests/common、JobExecution の手組みを exited などへ、cli の偽物の writes・台本の消費の順・分割済みのコマンドの記録・Default の derive、runtime の state/consistency.rs を core へ)。既存のテストの期待は変えない"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(補助の置き換え・移動・偽物の中身の変更は認める。テストの名前、件数、確かめる中身、期待する出力の文字列は変えない。変えるファイルの一覧を検索で網羅して設計で承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(整理で確かめられるようになったものに限って、少数の新しいテストを足す)
- [Q4] `intent-capture-questions.md` Q4: A(`program_name` とその単体テストを消し、仕様書の 1 行も直す。件数の減りは理由と一緒に記録する)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- 本体のコードに、同じ文言の定数の重複、同じ処理の重複、到達しない分岐や結果に効かない処理、文字列の一致での判定、実装とずれた説明、使われないメソッドが残っている。読む人を誤らせ、直すときに同じ所を何度も直す必要がある。[desc]
- テストのコードに、同じ補助の関数が複数のファイルに写されている所、今の型の作り方を使わない古い組み立て、使われない状態を持つ偽物がある。テストを直すときの手間が増え、偽物では確かめられないことがある。[desc]
- 解決したいのは、利用者から見える動きと、既存のテストが確かめている期待を変えずに、これらを直すことである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- (1) 本体の整理: 初期の記述の (1) に挙げた項目のそれぞれについて、重複が 1 か所にまとまり、到達しない分岐と効かない処理がなくなり、文字列の一致での判定が型や定数での判定になり、説明が実装と一致する。`program_name` はなくなる。[desc] [Q4]
  - 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形を一覧にし、実装の後に、その場所に重複・ずれ・到達しない分岐・文字列での判定が残っていないことを確かめる。[desc] [Q1]
- (2) テストの補助: 初期の記述の (2) に挙げた重複が共通の場所の補助にまとまり、古い組み立ては今の型の作り方になり、cli の偽物は使われない状態を持たず、渡されたコマンドを記録する。[desc]
- 新しいテスト: 整理で確かめられるようになったもの(たとえばコマンド名の往復、cli の層で分割済みのコマンドを渡して分け直さないこと)に限って、少数を足す。足すテストは設計の段で一覧に示す。[Q3]
- 利用者から見える動きは変わらない。[desc]
  - 前の同種の intent(`261122-internal-cleanup-2` の記録)と同じく、出力、終了コード、ステートファイル、`--dry-run` の出力を動きとみなし、動きが変わらないので版は上げない。[desc]
- (2) の確かめる手段: (1) と同じく、設計の段で重複している補助の場所(ファイルと行)と、まとめる先の一覧を作り、実装の後に、その場所に写しが残っていないことを確かめる。[desc] [Q2]
- 既存のテストの扱い: テストのファイルは、補助の置き換え・移動・偽物の中身の変更に限って変えてよい。テストの名前、件数(移動したものも数える)、確かめる中身、期待する出力の文字列は変えない。例外は `program_name` の単体テスト 1 件の削除だけである。変えるファイルと理由の一覧は、該当する行を検索で網羅して設計の段で示し、承認を取る。[Q2] [Q4]
  - 確かめる手段: Q1 の「各段の終わりにテストが通ることを確かめる」に従い、段の前と後でテストの件数も記録する。合格は、失敗 0 で、件数が「基準 − 1(`program_name`)+ 足したテスト」に一致すること。基準は、実装を始める前の HEAD で `cargo test --locked --workspace` を macOS で実行した通過の件数とする。移動したテストは、移動先で 1 件として数える(移動では件数は変わらない)。[Q1] [Q2] [Q3] [Q4]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- リファクタリングの候補の調査で挙がったもののうち、初期の記述の「調査の 1 と 2」(動きを変えないもの)を片付ける。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (1) と (2)。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 調査の 3(型と構造)、4(計算量)、5(利用者から見える変更)。これらは初期の記述に含めず、別の intent で扱う。[desc]
- 初期の記述の「仕様書 08 章」と仕様書の 1 行(Q4)は `docs/spec/rust` の仕様書を指す。凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)は、前の intent `261122-internal-cleanup` の記録の決定に従い変えない。[desc] [Q4]

## Assumptions & Open Questions

None.
