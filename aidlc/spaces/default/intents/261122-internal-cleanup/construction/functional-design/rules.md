# ビジネスルール(Rules): 動きを変えない整理のリファクタリング

詳細の場所は `inventory.md`。

```yaml
rules:
  - id: BR1.1
    statement: run ID の決定と、ステートの読み取りと照合を、ctx.rs の 2 つの関数(definition_run_id、read_checked_state)にまとめ、prepare、graph、watch が使う。prepare と graph の検査の順序と、read_state と read_state_raw の呼び分けを変えない。
    category: policy
    applies_to: crates/cli/src/ctx.rs、graph.rs、aux_commands.rs
    trigger: 常に
    logic: inventory.md 1a
    violation: 重複が残る、または誤りの順序が変わる
    source: intent-statement.md(段 1)
  - id: BR1.2
    statement: コマンドの分割は定義の検査の 1 回だけにし、結果を Checked と Session が持つ。dry_run と report_execution は使い回し、通らない枝を消す。誤りは全件を集めて表示する。
    category: policy
    applies_to: crates/cli/src/ctx.rs、dry_run.rs、step.rs、run.rs、pipeline.rs
    trigger: 常に
    logic: inventory.md 1b
    violation: 分割が複数回残る
    source: intent-statement.md(段 1)
  - id: BR1.3
    statement: check_definition_and_env を ctx.rs に置く。
    category: policy
    applies_to: crates/cli/src/ctx.rs、aux_commands.rs、graph.rs
    trigger: 常に
    logic: inventory.md 1c
    violation: 置き場所と中身が合わない
    source: intent-statement.md(段 1)
  - id: BR1.4
    statement: run と pipeline の入口を共通にし、prepare を部品に分けて、run はジョブの位置を prepare から受け取る。prepare の順序(シグナル、定義、command、ジョブ、--assume、run ID、.env、ステート)を保つ。
    category: policy
    applies_to: crates/cli/src/run.rs、pipeline.rs、ctx.rs、dry_run.rs
    trigger: 常に
    logic: inventory.md 1d
    violation: 入口の重複や通らない枝が残る、または順序が変わる
    source: intent-statement.md(段 1)
  - id: BR2.1
    statement: 分類と遷移を、文字列でなく型(Option<Classification>、TransitionOn)で DisplayEvent に渡し、文字列にするのは表示の側だけにする。分類から遷移への写しは core の 1 か所。
    category: policy
    applies_to: crates/cli/src/ctx.rs、report/mod.rs、crates/core/src/types.rs、flow/next_step.rs
    trigger: 常に
    logic: inventory.md 2a、functional-spec.md §3
    violation: 文字列の比較が残る
    source: intent-statement.md(段 2)
  - id: BR2.2
    statement: JobStatus と SkipReason の文字列の対応(as_str、parse)を core の型に置き、writer、reader、watch が使う。
    category: policy
    applies_to: crates/core/src/types.rs、crates/runtime/src/state/{writer,reader}.rs、crates/cli/src/report/watch.rs
    trigger: 常に
    logic: inventory.md 2b
    violation: 対応表が複数残る
    source: intent-statement.md(段 2)
  - id: BR2.3
    statement: check_consistency と ConsistencyResult、MismatchKind を core に置き、runtime は再エクスポートする。runtime の単体テスト 4 件はその場に残す。
    category: policy
    applies_to: crates/core、crates/runtime/src/state/consistency.rs、crates/runtime/src/types.rs
    trigger: 常に
    logic: inventory.md 2c
    violation: 公開の道が変わる、または単体テストが動く
    source: intent-statement.md(段 2)
  - id: BR2.4
    statement: ステートの上限つきの読み取りを read_bounded の 1 か所にし、今の誤りの写像を保つ。
    category: policy
    applies_to: crates/runtime/src/state/reader.rs
    trigger: 常に
    logic: inventory.md 2d
    violation: 重複が残る、または誤りの種類が変わる
    source: intent-statement.md(段 2)
  - id: BR3.1
    statement: walk.rs を walk/{mod,retry,env}.rs に分ける(移すだけ。呼ぶ順序と公開の道は変えない)。
    category: policy
    applies_to: crates/core/src/definition/walk.rs
    trigger: 常に
    logic: inventory.md 3a
    violation: 動きが変わる
    source: intent-statement.md(段 3)
  - id: BR3.2
    statement: 既定の成功コード、終了コードの読み取り、入れ子の上限と表示の上限、空の次の動きの重複をなくす。誤りの文は 1 文字も変えない。
    category: policy
    applies_to: crates/core/src/{types.rs,definition/walk,definition/depth.rs,error.rs,envfile/mod.rs,flow/next_step.rs}、crates/converter/src/error.rs
    trigger: 常に
    logic: inventory.md 3b〜3e
    violation: 重複が残る、または文が変わる
    source: intent-statement.md(段 3)
  - id: BR3.3
    statement: JobExecution::exited(job, code) を足し、本番(simulate)で使う。テストでの使用は段 4。
    category: policy
    applies_to: crates/core/src/types.rs、plan/simulate.rs
    trigger: 常に
    logic: inventory.md 3f
    violation: 手の組み立てが残る
    source: intent-statement.md(段 3)
  - id: BR4.1
    statement: graph のテストを形ごとのファイルに組み直し、補助を graph_common に置く。入力と期待の全文が同じ 5 組と、同じ性能のテスト 1 件をまとめ、一覧を記録する。ほかのテストは入力と期待を落とさずに移す。
    category: policy
    applies_to: crates/core/tests/graph_*.rs
    trigger: 段 4
    logic: inventory.md 4a
    violation: 確かめていた入力や期待が落ちる
    source: intent-statement.md(段 4、Q3: A)
  - id: BR4.2
    statement: converter のテスト名を日本語にする。env_probe_child の起動の文字列と、仕様書の参照も合わせる。
    category: policy
    applies_to: crates/converter
    trigger: 段 4
    logic: inventory.md 4b
    violation: テストが起動できない、参照が切れる
    source: intent-statement.md(段 4)
  - id: BR4.3
    statement: テストの偽物の wait を共通の関数にし、run_job の土台に JobExecution::exited を使う。テストのフィールドは残す。
    category: policy
    applies_to: crates/cli/tests/common/mod.rs
    trigger: 段 4
    logic: inventory.md 4c
    violation: テストの変更が広がる
    source: intent-statement.md(段 4)
  - id: BR4.4
    statement: README の終了コード 68 の行を実装に合わせる。必須の語句は残す。
    category: policy
    applies_to: README.md
    trigger: 段 4
    logic: inventory.md 4d
    violation: README と実装の食い違い、docs-check の失敗
    source: intent-statement.md(段 4)
  - id: BR4.5
    statement: go、python、typescript、csharp、powershell の 00-README.md の見出しの直後に、v0.8.0 時点の凍結版で最新は rust と language-agnostic、と書く。中身は変えない。
    category: policy
    applies_to: docs/spec/{go,python,typescript,csharp,powershell}/00-README.md
    trigger: 段 4
    logic: inventory.md 4e
    violation: どれが最新か分からない
    source: intent-statement.md(Q1: A)
  - id: BR5.1
    statement: 利用者から見える動き(出力、終了コード、ステート、--dry-run の出力)を変えない。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテスト
    violation: 回帰
    source: intent-statement.md
  - id: BR5.2
    statement: 段 1〜3 は既存のテストを変えずに通す。例外は report_messages.rs の組み立ての値(functional-spec.md §3。ユーザーの承認が要る)。
    category: constraint
    applies_to: テスト
    trigger: 段 1〜3
    logic: functional-spec.md §3、§4
    violation: 承認のないテストの変更
    source: intent-statement.md(Q3: A)
  - id: BR5.3
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、cfg を足さない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(合格の条件)
```
