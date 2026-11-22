# ビジネスルール(Rules): run と pipeline の重複をなくすリファクタリング

```yaml
rules:
  - id: BR1.1
    statement: 1 つのジョブを処理する手順(判断、スキップの記録、初期ステートの書き込み、リトライつきの実行、分類、報告、記録)を、`crates/cli/src/step.rs` の共通の関数にまとめ、`run` と `pipeline` はそれを呼ぶ。
    category: policy
    applies_to: crates/cli/src/run.rs、pipeline.rs、step.rs
    trigger: 常に
    logic: functional-spec.md の §3
    violation: 同じ手順が 2 か所に残る
    source: intent-statement.md(Q1: B)
  - id: BR1.2
    statement: 完了のあとの削除の処理(`remove_on_complete` が真なら `reset`)は共通にし、いつ消すかの判断は、呼び出し側に残す(run と pipeline で違うため)。
    category: policy
    applies_to: run.rs、pipeline.rs、step.rs
    trigger: 完了
    logic: functional-spec.md の §2 の 1、2
    violation: 削除の時機が変わり、動きが変わる
    source: functional-spec.md §2
  - id: BR2.1
    statement: 利用者から見える動き(出力、終了コード、ステートの書き込みの順序・Durability・文脈・内容)は 1 文字も変えない。既存のテストは変えずに、すべて通る。
    category: constraint
    applies_to: run、pipeline
    trigger: 常に
    logic: 既存のテストで確かめる
    violation: 回帰
    source: intent-statement.md(Q1、Q3: A)
  - id: BR3.1
    statement: docs/spec/rust の 05 の §5.2 と §5.3 を、共通の手順を指す形に書き直し、共通の手順を 1 回だけ書く。02 のモジュールの表に step.rs を足す。
    category: policy
    applies_to: 文書
    trigger: 実装の完了
    logic: functional-spec.md の §4
    violation: 文書と実装の食い違い、同じ手順の二重の記述
    source: intent-statement.md(Q1: B)
  - id: BR4.1
    statement: cargo fmt --check、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 が通る。新しい依存、unsafe、cfg は足さない。
    category: validation
    applies_to: 全体
    trigger: 実装の完了
    logic: CI と同じ検査
    violation: マージの関門で止まる
    source: intent-statement.md(合格の条件)、project.md
```
