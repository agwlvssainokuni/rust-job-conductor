# 意図の記述(Intent Statement): テストの都合で曲げられた本番のコードと、残りの整理

## Sources

- [desc] Initial description: "5 回目の調査の候補を 1 つの intent で(動きを変えない。版は上げない): (1) テストの都合で本番のコードが曲げられている所を直す(テストのコードの都合で本番のコードを曲げない、という前の intent の判断と同じ観点): cli の Reporter::language_resolved と color_resolved、runtime の ResetAllReport.removed、core の graph::is_lane_drawable、core の LoadedEnvFiles::add、runtime の remove_stale_temps の引数 now、core と converter の誤りの種類の code()、誤りの種類の一覧 ALL(5 つ。CommandKind::ALL を除く)、cli のパニックのフックの差し替え口(on_panic、install_panic_hook_with)、cli のシグナルの AlreadyRegistered と登録済みの印、core の graph::node_state_of と flow_edges の公開。(2) ふつうのリファクタリング: 古くなった説明(.env の重複キーの警告を graph も出す、run_validate の .env の検査、runtime の lib.rs のモジュール表、自動の再試行はしないという言い方 4 か所、converter のモジュール表、env_file_refs、Reporter::show)、ステートの definition_key と job_order(型から消し、ファイルの形は保つ)、runtime の CommandError.reason をメソッドに、ExitCodeClass、小さい整理(非公開にできる定数と関数、常に真の条件、テストだけの use の置き場所、Path の包み直し)、ステートが無いときの判断の重複(cli と計画)。範囲の外: converter の testing モジュール、cli のライブラリの公開範囲(別の intent)、上限つき読み取りの core と runtime の共有、skip_reason_kind、エスケープの確保、JobExecution の包み、辺の間のジョブの複製"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(書き方を変える・移す・消すテストを認める。利用者から見える動きと期待の文字列は変えない。一覧は検索で網羅し、消すものは理由つきで設計で承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(パニックのフックの差し替え口をなくす。本体は非公開の関数にして単体テストへ。「フックを設定すると 70 で終わる」の 1 件は消し、確かめられなくなることを記録する)
- [Q4] `intent-capture-questions.md` Q4: A(シグナルの `AlreadyRegistered` と登録済みの印をなくす。テストは 1 つで 1 回だけ登録する形に直す)
- [Q5] `intent-capture-questions.md` Q5: A(誤りの種類の一覧 `ALL` をテストの側へ移し、網羅の `match` を付ける)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"

## Problem Statement

- 本番のコードに、テストのコードの都合で曲げられている所(初期の記述の (1) の項目)がある。[desc]
- 説明が古くなった所、使われない型の項目、二重の処理が残っている。[desc]
- 解決したいのは、利用者から見える動きを変えずに、これらを直すことである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]

## Success Metrics

- (1) テストの都合で曲げられた所: 初期の記述の (1) に挙げた項目のそれぞれについて、本番のコードから、テストのためだけの関数・項目・引数・公開範囲・差し替え口がなくなる。テストは、公開の入口から確かめる、偽物の側で記録する、テストの補助に同じ処理を書く、単体テストへ移す、のどれかで代わりを持つ。[desc] [Q2]
  - パニックのフックは差し替え口をなくし、本体を単体テストで確かめる。「フックを設定すると 70 で終わる」ことは確かめられなくなり、そのことを記録する。[Q3]
  - シグナルの「すでに登録済み」の種類と印をなくし、テストは 1 つで 1 回だけ登録する形にする。[Q4]
  - 誤りの種類の一覧はテストの側へ移し、網羅の `match` で種類の増加にコンパイルで気づけるようにする。[Q5]
- (2) ふつうの整理: 初期の記述の (2) に挙げた項目のそれぞれについて、説明が実装と一致し、使われない型の項目と二重の処理がなくなる。ステートのファイルの形は変わらない。[desc]
- 確かめる手段: 設計の段で、項目ごとに今の場所(ファイルと行)と目標の形、終わったと判断する基準(たとえば「本番のコードから参照が 0 件」)を一覧にし、実装の後に検索で確かめる。この一覧は、Q2 のテストの一覧と一緒に設計の段で承認を取る。[Q1] [Q2]
- 利用者から見える動きは変わらない。版は上げない。[desc]
  - 前の同種の intent の記録と同じく、出力、終了コード、ステートファイル、`--dry-run` の出力を動きとみなす。[Q2]
- 既存のテストの扱い: 書き方を変える・移す・消すテストは認める。利用者から見える動きの確かめと期待する出力の文字列は変えない。一覧は検索で網羅し、消すテストは理由を添えて、設計の段で承認を取る。[Q2]
  - 確かめる手段: Q1 の「各段の終わりにテストが通ることを確かめる」に従い、段の前と後でテストの件数も記録する。合格は、失敗 0 で、件数が「基準 − 消したテスト + 足したテスト」に一致すること。基準は、実装を始める前の HEAD で `cargo test --locked --workspace` を macOS で実行した通過の件数とする。[Q1] [Q2]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- リファクタリングの候補の調査(5 回目)で挙がったもののうち、初期の記述の (1) と (2) を片付ける。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 初期の記述の (1) と (2)。段はこの順。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 初期の記述の「範囲の外」に挙げたもの(converter の `testing` モジュール、cli のライブラリの公開範囲、上限つき読み取りの共有、`skip_reason_kind`、エスケープの確保、`JobExecution` の包み、辺の間のジョブの複製)。[desc]

## Assumptions & Open Questions

None.
