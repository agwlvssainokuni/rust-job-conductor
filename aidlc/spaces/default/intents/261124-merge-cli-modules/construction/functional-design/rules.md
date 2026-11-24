# ビジネスルール(Rules): cli の本番の小さなモジュールの統合

詳細の場所は `inventory.md`(項目ごとの今の形とまとめた後の形、名前の衝突、可視性、仕様書の S1〜S24、写しでの検証)、承認が要るテストの変更(T 表)、件数の見込み、コミットの並び、決めてほしい点は `functional-spec.md` の §4、§6、§8、§10。D1〜D7 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。下の規則の D の箇所は、その決定のとおりの形を書いている。T 表(T1)は、設計の承認で認めてもらう(承認待ち)。

```yaml
rules:
  # ---- 項目 1: retry.rs を step.rs へ(Q2: A) ----
  - id: BR1.1
    statement: crates/cli/src/retry.rs の run_job_with_retry を、本体を変えずに crates/cli/src/step.rs の run_step の直後へ移し(D4。決定: 推奨のとおり。ユーザー、2026-11-25)、retry.rs を消す。可視性は、使うのが同じモジュールの run_step だけなので、非公開の fn にする(D1。決定: 推奨のとおり。ユーザー、2026-11-25)。元のモジュールの説明(規則の番号を含む)は関数の説明の先頭の段落にし、呼ぶ側の記述は実際に呼ぶ run_step に直す。元の関数の説明と規則の番号は、すべて残す。step.rs のモジュールの説明の関数の表に 1 行を足す。
    category: constraint
    applies_to: crates/cli/src/step.rs、crates/cli/src/retry.rs
    trigger: コミット 1
    logic: functional-spec.md §2.1、inventory.md §1
    violation: retry.rs が残る、関数の本体が変わる、規則の番号が消える、または pub(crate) のまま残る
    source: intent-statement.md(SM1)
  - id: BR1.2
    statement: retry モジュールを指す所を直す。lib.rs の mod retry; を消し、モジュールの表の「流れ」の行から retry を消して、step が共通の手順とリトライを含む実行を持つと書く。run.rs のモジュールの説明の retry::run_job_with_retry を step の run_job_with_retry にする。step.rs の use crate::retry::run_job_with_retry を消し、移した関数が使う名前(EnvContext、RetryDecision、merge_env、retry_decision、InterruptSignal、WaitOutcome)を use に足す。
    category: constraint
    applies_to: crates/cli/src/lib.rs、crates/cli/src/run.rs、crates/cli/src/step.rs
    trigger: コミット 1
    logic: inventory.md §1.2
    violation: crate::retry か retry:: の参照が残る、またはモジュールの表が実装と合わない
    source: intent-statement.md(SM1)

  # ---- 項目 2: report/escape.rs を report/mod.rs へ(Q3: A) ----
  - id: BR2.1
    statement: crates/cli/src/report/escape.rs の MAX_DISPLAY_CHARS(非公開)、escape_name、escape_full を、本体と説明を変えずに crates/cli/src/report/mod.rs の push_line の直前へ移し(D5。決定: 推奨のとおり。ユーザー、2026-11-25)、escape.rs を消す。escape_name・escape_full の可視性は pub(crate) にする(今の実効の範囲 crate の中と同じで、広げも狭めもしない。D2。決定: 推奨のとおり。ユーザー、2026-11-25)。元のモジュールの説明(本体は core の escape_display、薄い包み、converter の写しと同じ期待値の表、規則の番号)は、report/mod.rs のモジュールの説明のエスケープの箇条へ写す。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs、crates/cli/src/report/escape.rs
    trigger: コミット 2
    logic: functional-spec.md §2.2、inventory.md §2
    violation: escape.rs が残る、関数の本体か上限の値が変わる、規則の番号が消える、または今の実効の範囲より広い可視性になる
    source: intent-statement.md(SM2)
  - id: BR2.2
    statement: escape モジュールを指す所を直す。report/mod.rs の pub mod escape; と use escape::{…} を消し、conductor_core の use に escape_display を足す。下位のモジュール(plan_text.rs、graph_text.rs、watch.rs)は use super::escape_full; / use super::escape_name; にする(use super::push_line と同じ形)。単体テスト src/tests/escape_table.rs は use crate::report::{escape_full, escape_name}; の 1 行だけを直す(T1)。
    category: constraint
    applies_to: crates/cli/src/report/mod.rs、crates/cli/src/report/plan_text.rs、crates/cli/src/report/graph_text.rs、crates/cli/src/report/watch.rs、crates/cli/src/tests/escape_table.rs
    trigger: コミット 2
    logic: inventory.md §2.2、functional-spec.md §4.1
    violation: report::escape か super::escape:: の参照が残る、またはテストで use の 1 行のほかが変わる
    source: intent-statement.md(SM2)

  # ---- 項目 3: graph.rs と init.rs を aux_commands.rs へ(Q4: A) ----
  - id: BR3.1
    statement: crates/cli/src/graph.rs の run_graph と read_overlay、crates/cli/src/init.rs の run_init を、本体を変えずに crates/cli/src/aux_commands.rs の run_convert の後へ、run_graph・read_overlay・run_init の順で移し(D6。決定: 推奨のとおり。ユーザー、2026-11-25)、graph.rs と init.rs を消す。可視性は変えない(run_graph・run_init は pub(crate)、read_overlay は非公開)。元のモジュールの説明(流れの番号と規則の番号を含む)は、run_graph・run_init の説明へ移し、各関数の説明の規則の番号をすべて保つ。use は 3 つのファイルの和集合にし、重なる名前は 1 つにする。
    category: constraint
    applies_to: crates/cli/src/aux_commands.rs、crates/cli/src/graph.rs、crates/cli/src/init.rs
    trigger: コミット 3
    logic: functional-spec.md §2.3、inventory.md §3
    violation: graph.rs か init.rs が残る、関数の本体が変わる、または元のモジュールの説明の規則の番号が消える
    source: intent-statement.md(SM3)
  - id: BR3.2
    statement: aux_commands.rs のモジュールの説明の見出しに、graph と init と、その流れの番号を足し、シグナルの準備をしない補助のコマンドは aux_commands.rs、準備をするコマンド(run、pipeline、--dry-run)は 1 コマンド 1 ファイル、という分け方を 1 文で書く。lib.rs の mod graph; と mod init; を消し、CommandKind::Graph と CommandKind::Init の呼び出しを aux_commands::run_graph・aux_commands::run_init にする。lib.rs のモジュールの表の「流れ」の行を、この分け方に合わせる。
    category: constraint
    applies_to: crates/cli/src/aux_commands.rs、crates/cli/src/lib.rs
    trigger: コミット 3
    logic: inventory.md §3.2
    violation: crate::graph・crate::init・graph::run_graph・init::run_init の参照が残る、またはモジュールの説明か表がコマンドの置き場所と合わない
    source: intent-statement.md(SM3)

  # ---- 項目 4: platform/mod.rs の二重の中継(Q5: A) ----
  - id: BR4.1
    statement: crates/cli/src/platform/mod.rs の中継の関数(start_signal_watcher と、OS ごとの spawn_watcher の 2 つ)と、それだけが使う use を消し、runtime の platform/mod.rs と同じ形で、#[cfg(unix)] mod unix; の直後に #[cfg(unix)] pub(crate) use unix::spawn_watcher as start_signal_watcher; を置く(Windows も同じ。D3。決定: 推奨のとおり。ユーザー、2026-11-25)。再公開のために、unix.rs・windows.rs の spawn_watcher の可視性を pub(super) から pub(crate) にする(名前・本体・説明は変えない)。中継の関数の説明(プロセスで 1 回だけ呼ぶ、2 回目は契約の外)は、モジュールの説明へ移す。呼ぶ側(deps.rs の platform::start_signal_watcher)は変えない。
    category: constraint
    applies_to: crates/cli/src/platform/mod.rs、crates/cli/src/platform/unix.rs、crates/cli/src/platform/windows.rs
    trigger: コミット 4
    logic: functional-spec.md §2.4、inventory.md §4
    violation: 中継が 2 段のまま残る、OS ごとの関数の本体が変わる、deps.rs が変わる、または 1 回だけ呼ぶという契約の説明が消える
    source: intent-statement.md(SM4)
  - id: BR4.2
    statement: platform のディレクトリと mod.rs・unix.rs・windows.rs の名前は変えない。cfg(unix)/cfg(windows) は platform の中だけに置き、unsafe・low_level・#[path・include の 3 つのマクロを足さない。cli の nfr_source.rs の 6 件が、数と中身を変えずに通る。Windows の対象(x86_64-pc-windows-msvc)で、全体と本番の形(-p conductor-cli --lib --bins)の clippy -D warnings が通る。
    category: validation
    applies_to: crates/cli/src/platform/、crates/cli/tests/nfr_source.rs
    trigger: コミット 4 と実装の完了
    logic: inventory.md §0.2、§4.2、functional-spec.md §7
    violation: platform の名前が変わる、platform の外に OS の分岐が入る、nfr_source のどれかが落ちる、または Windows の対象でコンパイルが通らない
    source: intent-statement.md(SM4、SM7)

  # ---- 仕様書と、確かめの検索 ----
  - id: BR5.1
    statement: docs/spec/rust の、動かしたファイルとモジュールのパスを指す記述を、そのファイルを動かすコミットで直す(02 章のモジュールの表・crate の中の関数の表・シーケンスの図、04 章の実装の所在、05 章の実装の所在、07 章の根拠と部品の表、08 章の実装と 9 章、12 章の作るもの。S1〜S24)。消えたファイルの行は消し、まとめ先の行に中身を足す。言語に依らない仕様書は、ファイル名もモジュールのパスも書いていないので変えない。凍結した仕様書は変えない。版の履歴の注記は足さない(D7。決定: 足さない。推奨のとおり。ユーザー、2026-11-25)。
    category: constraint
    applies_to: docs/spec/rust/02-architecture.md、04-cli.md、05-execution-flow.md、07-process-and-signals.md、08-output-and-i18n.md、12-rebuild-guide.md
    trigger: コミット 1〜4
    logic: inventory.md §5、functional-spec.md §8 の各コミット
    violation: 無くなったファイルかモジュールのパスを指す記述が残る、まとめ先の行が中身を書かない、または決めた所の外が変わる
    source: intent-statement.md(SM8)
  - id: BR5.2
    statement: 実装の完了のとき、functional-spec.md §7 の検索で、無くなる 4 ファイル(retry.rs、report/escape.rs、graph.rs、init.rs)が無いこと、無くなったモジュールのパス(crate::retry、retry::、report::escape、super::escape::、crate::graph、crate::init、graph::run_graph、init::run_init)の参照がコードに 0 件であること、仕様書 rust と言語に依らない仕様書に無くなったファイルの名前とパスが 0 件であることを確かめる。
    category: validation
    applies_to: crates/cli、docs/spec
    trigger: 実装の完了
    logic: functional-spec.md §7
    violation: 検索の結果が見込みと違う
    source: intent-statement.md(SM5、SM8)

  # ---- 全体 ----
  - id: BR9.1
    statement: 利用者から見える動き(出力、終了コード、ステートファイル、--dry-run の出力、シグナルの扱い)を変えない。移す関数の本体は変えない(変わるのは可視性の語だけ: run_job_with_retry の pub(crate) を外す、escape_name・escape_full の pub を pub(crate) に、spawn_watcher の pub(super) を pub(crate) に)。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: functional-spec.md §7(本体の比べ)、全テストが期待を変えずに通る
    violation: 関数の本体が変わる、または既存のテストの期待を変えないと通らない
    source: intent-statement.md(SM9)
  - id: BR9.2
    statement: テストの変更は functional-spec.md §4 の T1(escape_table.rs の use の 1 行)だけにする。一覧は、対象のモジュールのパスを grep で網羅し、写しの木で全項目を当てて作る。テストの名前・確かめの行・期待の値は変えない。消すテスト・足すテスト・名前を変えるテストは無い(★ なし)。実装で増減や変更が出たら、その数と理由を記録し、名前を変える・消すなら承認を取る。
    category: constraint
    applies_to: テスト
    trigger: 常に
    logic: functional-spec.md §4、inventory.md §0.2、§6
    violation: 一覧に無いテストのファイルの変更、確かめの行や期待の値の変化、テスト名の変化、または理由の無い増減
    source: intent-statement.md(SM6)
  - id: BR9.3
    statement: 実装の前と後でテストの件数をクレートごとに記録する。合格は失敗 0 で、全体が「基準 1,149 − 消したテスト 0 + 足したテスト 0 = 1,149」(と無視 1。cli は単体 162、統合 294)に一致すること(macOS)。テスト名の一覧(パスつき)の差は 0。
    category: validation
    applies_to: テスト、完了の判定
    trigger: 各コミット(件数)と実装の完了
    logic: functional-spec.md §6、§7、§8 の件数の列
    violation: 件数の説明できない増減、またはテスト名の一覧の差が 0 でない
    source: intent-statement.md(SM6)
  - id: BR9.4
    statement: cargo fmt --check、clippy -D warnings(macOS と x86_64-pc-windows-msvc の対象、本番の形 -p conductor-cli --lib --bins も両方)、全テスト、行カバレッジ 80% 以上、MSRV 1.100、cargo doc の警告が基準の 5 件のまま、が通る。新しい依存、unsafe、platform の外の cfg(unix)/cfg(windows) を足さない。ドキュメントコメントは日本語。
    category: validation
    applies_to: 全体
    trigger: 各コミットと実装の完了
    logic: CI と同じ検査。functional-spec.md §7、§8 の各コミットの確かめ
    violation: マージの関門で止まる
    source: intent-statement.md(SM10)
  - id: BR9.5
    statement: 版を上げない(Cargo.toml と Cargo.lock の版の表記は 0.9.8 のまま)。タグとリリースはしない。凍結した仕様書と CI の設定は変えない。1 項目 1 コミットの小さな並び(4 コミット)にし、どのコミットでもビルドとテストが通る。移す先は既にあるファイルなので、同じコミットで中身を足して元を git rm し、コミットの本文に元のパスを書く(履歴は git log -- <元のパス> と git blame -w -C -C で追う)。
    category: constraint
    applies_to: crates/*/Cargo.toml、Cargo.lock、.github、docs/spec、コミットの並び
    trigger: 常に
    logic: functional-spec.md §1、§8、§8.1
    violation: 版が変わる、凍結した仕様書か CI が変わる、途中のコミットでビルドかテストが止まる、または同じ関数が 2 つある・無いコミットがある
    source: intent-statement.md(SM9)
  - id: BR9.6
    statement: cli の製品のコードの検査(nfr_source.rs の 6 件)は、テストの数と中身を変えずに、各コミットで通る。ソースのファイルが 4 つ減っても、検査の条件(10 ファイル以上)を満たす。
    category: validation
    applies_to: crates/cli/tests/nfr_source.rs
    trigger: 各コミットと実装の完了
    logic: functional-spec.md §7、inventory.md §0.2
    violation: 検査のテストの数か中身が変わる、またはどれかが落ちる
    source: intent-statement.md(SM7)
```
