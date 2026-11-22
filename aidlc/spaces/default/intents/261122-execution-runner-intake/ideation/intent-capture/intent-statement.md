# 意図の記述(Intent Statement): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

## Sources

- [desc] Initial description: "残りの整理を 1 つの intent で: (1) JobExecution の列挙型化(動きを変えない。ステートファイルの書式は変えない)、(2) F: run_job(crates/runtime/src/runner.rs)に分割済みのコマンドを渡す(動きを変えない。Runtime トレイトと仕様書の 4 つの章)、(3) B3: core の定義ファイルの読み込みの順序(crates/core/src/definition/intake.rs の read_definition。利用者から見える変更で、仕様書と版の更新を伴う)。段はこの順で、B3 を最後に置く"
- [scope] Workflow-selected scope: `refactor`.
- [Q1] `intent-capture-questions.md` Q1: A(承認は 1 回、段の順に進め、結果は最後にまとめる)
- [Q2] `intent-capture-questions.md` Q2: A(公開の型とトレイトの変更に伴う既存のテストの変更は、組み立て、import、項目の読み方、偽物の `run_job` の引数だけ認める。設計で一覧を示して承認を取る)
- [Q3] `intent-capture-questions.md` Q3: A(core を converter の順、つまり先に開く順に揃える)
- [Q4] `intent-capture-questions.md` Q4: B(この intent の最後のコミットで v0.9.6 に上げる。タグとリリースは後で頼まれたら行う)
- [memory:M1] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)"
- [memory:M2] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)"
- [memory:M3] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)"
- [memory:M4] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)"
- [memory:M5] `aidlc/spaces/default/memory/project.md#Forbidden`: "NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)"
- [memory:M6] `aidlc/spaces/default/memory/project.md#Mandated`: "ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)"

## Problem Statement

- ジョブの 1 回の起動の結果(`JobExecution`)は、起動したか、起動に失敗したか、中断されたかを 3 つの真偽値で持つため、あり得ない組み合わせを表せてしまう。[desc]
- ジョブを起動する関数(`run_job`)は、実行の前の検査で分けたコマンドを受け取らず、リトライのたびに分け直している。[desc]
- 定義ファイル(core)と変換の入力(converter)で、ファイルを読む手順の順序が違う。そのため、読む権限のない大きなファイルのような、まれな場合の誤りの種類が 2 つで食い違う。[desc] [Q3]
- 解決したいのは、前の 2 つを利用者から見える動きを変えずに直し、最後の 1 つを利用者に分かる変更として揃えることである。[desc]

## Target Customer

- このコマンドラインツールのコードとテストを保守し、機能を足す開発者。仕様書を読む実装者。[desc]
- 段 3 については、定義ファイルを指定してツールを使う利用者(まれな場合に見る誤りの種類が変わる)。[desc] [Q3]

## Success Metrics

- 段 1(`JobExecution` の型): 起動の結果が、あり得る場合だけを表す型になる。ステートファイルの書式、出力、終了コードは変わらない。[desc]
  - あり得る場合の一覧(たとえば「起動しなかった」「起動に失敗した」と、中断の有無の組み合わせ)は、設計の段で、今のコードが実際に作る値から決めて示す。今のどの値も、新しい型のどれか 1 つに対応し、分類の結果が同じであることを確かめる。[desc] [Q2]
- 段 2(`run_job` の入力): `run_job` が分けたコマンドを受け取り、リトライのたびに分け直さない。`Runtime` トレイトと仕様書の該当の章が、新しい形に合う。出力、終了コード、ステートファイルは変わらない。[desc]
  - 確かめる手段: `run_job` の引数が分けたコマンドの型になり、`run_job` の中にコマンドを分ける呼び出しが残らないことを、設計の段の一覧と実装の後の確認で示す。分けられないときの扱い(今は `run_job` の中の防御)がどこへ移るかも、設計の段で示す。[desc] [Q2]
- 段 3(読み込みの順序): core の定義ファイルの読み込みが、converter と同じく「先に開き、開いたものから大きさを見る」順になる。読む権限のない大きなファイルは「読めない」と報告される。この変更を仕様書に書き、テストで確かめる。[desc] [Q3]
  - 変わるのは誤りの種類だけで、表示に入力の値を含めない性質は保つ。[desc] [Q3]
  - ファイルが無い、通常のファイルでない、読めるが上限を超える、UTF-8 でない場合の誤りの種類は変わらないことを、テストで確かめる。[Q3]
  - converter の読み込みは変えない(揃える先であるため)。[Q3]
- 版: この intent の最後のコミットで v0.9.6 に上げる。タグとリリースは、終わった後に頼まれたら行う。[Q4] [memory:M6]
- 既存のテストは変えずに通す。例外として、段 1・2 の公開の型とトレイトの変更で、組み立て、import、項目の読み方、偽物の `run_job` の引数を直す必要があるテストは変えてよい。そのとき、確かめる中身と期待する出力の文字列は変えない。変えるテストは、設計の段で一覧と理由を示して承認を取る。[Q2]
  - 段 3 で期待が変わるテストがあれば、それも設計の段で一覧に挙げ、承認を取る。[desc] [Q3]
  - 確かめる手段: Q1 の「各段の終わりにテストが通ることを確かめる」に従い、段の前と後でテストの件数も記録する。合格は、失敗 0 で、件数が「基準 + 足したテスト」に一致すること。[Q1]
- 合格の条件: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る [memory:M4]。行カバレッジ 80% 以上 [memory:M2]。MSRV 1.100 でのテストが通る [memory:M3]。新しい依存を足さない [memory:M5]。

## Initiative Trigger

- 初期の記述の「残りの整理」を、1 つの intent でまとめて片付ける。3 つは、前の intent(`261122-internal-cleanup-2`)の記録で範囲の外とした B3、F、`JobExecution` の列挙型化にあたる。[desc]

## Initial Scope Signal

- Workflow-selected scope: `refactor`(workflow-selected)[scope]
- 範囲: 段 1〜3。段はこの順で、利用者から見える変更の段 3 を最後に置く。[desc]
- 進め方: 設計とコードの計画は 1 回で承認し、実装は段の順に、小さなコミットで進める。結果は最後にまとめて見せる。[Q1] [memory:M1]
- 範囲の外: 辺の一覧(`EdgeList`)の削除、凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)の中身。直す仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` だけ(前の intent `261122-internal-cleanup-2` の記録の決定に従う)。[desc]
- converter の読み込みの順序は変えない。[Q3]

## Assumptions & Open Questions

None.
