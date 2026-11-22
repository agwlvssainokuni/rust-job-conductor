# ビジネスルール(Rules): `--dry-run` の模擬の重複をなくすリファクタリング

```yaml
rules:
  - id: BR1.1
    statement: ステートの更新の関数(initial_state、apply_outcome、apply_skip、apply_interrupt)を、core の flow/record.rs に置く(runtime の state/record.rs から、単体のテストごと移す)。
    category: policy
    applies_to: crates/core/src/flow/record.rs、crates/runtime/src/state/record.rs
    trigger: 常に
    logic: functional-spec.md §3 の 1
    violation: 更新の規則が 2 か所に残る
    source: intent-statement.md(Q2: B)
  - id: BR1.2
    statement: runtime は、これらを conductor_core から再エクスポートし、公開の名前を保つ。cli の use は変えない。
    category: constraint
    applies_to: crates/runtime/src/lib.rs
    trigger: 常に
    logic: functional-spec.md §3 の 2
    violation: 呼び出し側の変更が要る
    source: intent-statement.md(成功の指標)
  - id: BR1.3
    statement: 模擬(plan_pipeline)は、PipelineState を 1 つ持ち、スキップを apply_skip、結果を apply_outcome で記録する。apply_result と、別に持つ resume と end_reached の変数はなくす。plan_run は変えない。
    category: calculation
    applies_to: crates/core/src/plan/simulate.rs
    trigger: --dry-run の pipeline
    logic: functional-spec.md §3 の 3
    violation: 模擬と本番の更新がずれる
    source: intent-statement.md(Q1: A)
  - id: BR2.1
    statement: 利用者から見える動き(本番の出力・終了コード・ステート、--dry-run の出力)は変えない。既存のテストは変えずにすべて通る。
    category: constraint
    applies_to: 全体
    trigger: 常に
    logic: 既存のテスト
    violation: 回帰
    source: intent-statement.md(Q3: A)
  - id: BR2.2
    statement: 今の模擬と新しい模擬の --dry-run の出力を、ランダムな定義とステートで比べ、違いが 0 件であること(設計の段階で 90 万例、違い 0 件を確認済み)。
    category: validation
    applies_to: 設計の記録
    trigger: 設計
    logic: functional-spec.md §2、plan-compare.rs.txt
    violation: 承認のない出力の変化
    source: intent-statement.md(Q1: A、Q3: A)
  - id: BR2.3
    statement: apply_skip を、定義の位置で状態を引く形にし、1 回を O(n) にする。結果は変えない。終点の後に 1,000 個近いジョブをスキップする --dry-run の計画が 1 秒以内。
    category: constraint
    applies_to: crates/core/src/flow/record.rs、crates/core/tests/plan_perf.rs
    trigger: 常に
    logic: functional-spec.md §3 の 5
    violation: --dry-run が遅くなる
    source: 設計の確認(計測 1.13 秒)
  - id: BR3.1
    statement: docs/spec/rust の 05、02、12 を、更新の規則を 05 §3.3〜§3.5 に 1 回だけ書き、模擬の節がそれを指す形に直す。language-agnostic は変えない。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: functional-spec.md §4
    violation: 文書と実装の食い違い
    source: intent-statement.md(Q2: B)
  - id: BR4.1
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、cfg を足さない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(合格の条件)
```
