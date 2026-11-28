# 規則: v0.10.4 の大きなファイルの分割

出典: 要件(`requirements.md`、承認済み。R-01〜R-03)、設計の質問(`functional-design-questions.md` の Q1〜Q4)、引き渡し(`developer-scan.md` の 1〜6)、project.md の Forbidden。

```yaml
rules:
  - id: BR1.1
    statement: report/mod.rs の出来事の定義、1 行の組み立て、エスケープを、event.rs、lines.rs、escape.rs に移す(functional-spec の 4.1)
    category: constraint
    applies_to: crates/cli/src/report/
    trigger: 作業のとき
    logic: 移すだけで、中身の行は変えない
    violation: レビューで差し戻す
    source: FR1.1
  - id: BR1.2
    statement: render の網羅の match は残し、各枝は render.rs の変種ごとのメソッドを 1 行で呼ぶだけにする。render.rs は表示の形の 9 つの節に分ける
    category: constraint
    applies_to: crates/cli/src/report/render.rs
    trigger: 作業のとき
    logic: IF 枝に 2 行以上の組み立てが残る、または unreachable!・unwrap・expect を足す THEN 直す
    violation: レビューで差し戻す
    source: FR1.2
  - id: BR1.3
    statement: 言語は、必要なメソッドの中で初めて判定する。固定の文と Graph・Plan・InitTemplate は判定しない。色は watch_table だけが判定する。watch_frame は self.render を呼び直す
    category: constraint
    applies_to: crates/cli/src/report/render.rs
    trigger: テストのとき
    logic: src/tests/report_messages.rs の 694・726 行のテストが通る
    violation: テストが落ちる
    source: FR1.3
  - id: BR1.4
    statement: 今の道筋(crate::report::…、子モジュールの super::…、messages.rs の crate::report::StateContext)を pub use で保つ。report/messages.rs は動かさない
    category: constraint
    applies_to: crates/cli/src/report/mod.rs
    trigger: ビルドのとき
    logic: 呼ぶ側の use の行を変えずにビルドが通る
    violation: ビルドが落ちる
    source: FR1.4
  - id: BR1.5
    statement: 文の中身、改行、format! の形を変えない
    category: constraint
    applies_to: crates/cli/src/report/
    trigger: テストのとき
    logic: report_messages.rs と cli の統合テストが、期待の値を変えずに通る
    violation: テストが落ちる
    source: NFR1
  - id: BR2.1
    statement: walk/mod.rs から、値の読み取り、遷移、ジョブをまたぐ検査を values.rs、transition.rs、cross_check.rs に移す(functional-spec の 2.1)。形は env.rs・retry.rs と同じ(子の impl Walker と pub(super))
    category: constraint
    applies_to: crates/core/src/definition/walk/
    trigger: 作業のとき
    logic: 移すだけで、中身の行と呼ぶ順は変えない
    violation: レビューで差し戻す
    source: FR2.1
  - id: BR2.2
    statement: キーの表 4 つは walk/mod.rs に残す。const PIPELINE_KEYS・const JOB_KEYS を含む行を、今の定義より前に置かない
    category: constraint
    applies_to: crates/core/src/definition/walk/mod.rs
    trigger: テストのとき
    logic: spec_numbers.rs が変更なしで通る
    violation: テストが落ちる
    source: FR2.2
  - id: BR2.3
    statement: env.rs・retry.rs の use super::{…} を変えずに引けるよう、mod.rs に use を置く
    category: constraint
    applies_to: crates/core/src/definition/walk/
    trigger: ビルドのとき
    logic: env.rs・retry.rs に差分が無い
    violation: レビューで差し戻す
    source: FR2.3
  - id: BR2.4
    statement: walk の誤りの順(同じ位置の誤りは積んだ順)を変えない
    category: constraint
    applies_to: crates/core/src/definition/walk/
    trigger: テストのとき
    logic: core の definition の統合テストが、期待の値を変えずに通る
    violation: テストが落ちる
    source: NFR1
  - id: BR3.1
    statement: args.rs を args/mod.rs にし(git mv)、オプションの表と検査、字句、値の解釈を options.rs、tokens.rs、values.rs に移す(functional-spec の 3.1)
    category: constraint
    applies_to: crates/cli/src/args/
    trigger: 作業のとき
    logic: git mv のコミットは中身を変えない。移しのコミットは中身の行を変えない
    violation: レビューで差し戻す
    source: FR3.1
  - id: BR3.2
    statement: build_command を、functional-spec の 3.2 の 6 つの関数を順に呼ぶだけにする。Convert と Init の枝は convert_operand、init_operand に分ける
    category: constraint
    applies_to: crates/cli/src/args/mod.rs
    trigger: 作業のとき
    logic: build_command が 30 行以下
    violation: レビューで差し戻す
    source: FR3.2
  - id: BR3.3
    statement: 返す誤りの種類、対象、最初に返す誤りの順を変えない
    category: constraint
    applies_to: crates/cli/src/args/
    trigger: テストのとき
    logic: src/tests/args_grammar.rs の 69 のテストが通る
    violation: テストが落ちる
    source: FR3.2, NFR1
  - id: BR3.4
    statement: 外の入口は parse_args だけのまま、crate::args::parse_args で引ける
    category: constraint
    applies_to: crates/cli/src/args/mod.rs
    trigger: ビルドのとき
    logic: lib.rs と args_grammar.rs の use を変えずにビルドが通る
    violation: ビルドが落ちる
    source: FR3.3
  - id: BR4.1
    statement: rust 版の仕様書の 02・07・08・12 章と文書コメントのモジュールの記述を、分けた後の形に直す。12 章の段の時点の記述は残す
    category: policy
    applies_to: docs/spec/rust/、crates/*/src/lib.rs
    trigger: 作業のとき
    logic: grep で、分けた後に存在しないパスを今の形として書く行が無い
    violation: レビューで差し戻す
    source: FR4.1
  - id: BR4.2
    statement: team.md の「使い方の表の 3 か所」の行だけを、計画の承認を経て直す
    category: constraint
    applies_to: aidlc/spaces/default/memory/team.md
    trigger: コード生成のとき
    logic: IF 差分がその 1 行の外にある THEN 戻す
    violation: レビューで差し戻す
    source: FR4.2
  - id: BR4.3
    statement: core/src/error.rs:176 の古い名前 report::escape を今の名前に直す
    category: constraint
    applies_to: crates/core/src/error.rs
    trigger: 作業のとき
    logic: 文書コメントの 1 行だけ
    violation: レビューで差し戻す
    source: FR4.3
  - id: BR4.4
    statement: with_all_events に DefinitionValid と ResetAllKept を足し、分けのコミットとは別にする
    category: constraint
    applies_to: crates/cli/src/tests/report_messages.rs
    trigger: 作業のとき
    logic: そのテストが通り、#[test] の数は変わらない
    violation: テストが落ちる
    source: FR4.4
  - id: BR5.1
    statement: すべてを片づけてから、版を 0.10.4 にする(ad775bc と同じ 22 ファイル)。件名は「版を 0.10.4 にする」
    category: policy
    applies_to: 版
    trigger: 最後
    logic: IF 他の変更が残る THEN 版を上げない
    violation: 差し戻す
    source: FR5.1
  - id: BR6.1
    statement: 公開の範囲を変えない。テストのために pub を足さない。新しい依存を足さない。#[path]・include!・unreachable!・unwrap・expect を本番に足さない。cfg(unix) などを platform の外に置かない
    category: constraint
    applies_to: crates/
    trigger: ビルドとテストのとき
    logic: nfr_source.rs が通る。lib.rs の pub の行が変わらない。Cargo.toml の依存が変わらない
    violation: テストが落ちるか、レビューで差し戻す
    source: NFR2, NFR4, NFR5
  - id: BR6.2
    statement: コミットは functional-spec の 7 章の区分ごとにし、1 つのコミットに 1 つの区分だけを入れる
    category: policy
    applies_to: コミット
    trigger: コード生成のとき
    logic: 件名と差分が区分に合う
    violation: レビューで差し戻す
    source: NFR6
  - id: BR6.3
    statement: fmt、clippy -D warnings、テスト、行カバレッジ 80% 以上、CI のすべてのジョブが通る
    category: validation
    applies_to: 全体
    trigger: ビルドとテストのとき
    logic: Build and Test の確かめ
    violation: 差し戻す
    source: NFR3
```

## 規則のまとめ

| ID | 要約 | 出典 |
|---|---|---|
| BR1.1 | report の 3 つを子に移す | FR1.1 |
| BR1.2 | render は網羅の振り分け、9 つの節 | FR1.2 |
| BR1.3 | 言語と色の判定を後回しに保つ | FR1.3 |
| BR1.4 | 今の道筋を pub use で保つ | FR1.4 |
| BR1.5 | 文を変えない | NFR1 |
| BR2.1 | walk の 3 つを子に移す | FR2.1 |
| BR2.2 | キーの表は mod.rs に残す | FR2.2 |
| BR2.3 | env.rs・retry.rs を変えない | FR2.3 |
| BR2.4 | 誤りの順を保つ | NFR1 |
| BR3.1 | args を args/mod.rs にし、3 つを子に移す | FR3.1 |
| BR3.2 | build_command を 6 つの関数に | FR3.2 |
| BR3.3 | 誤りの順を保つ | FR3.2、NFR1 |
| BR3.4 | parse_args の道筋を保つ | FR3.3 |
| BR4.1 | 仕様書と文書コメント | FR4.1 |
| BR4.2 | team.md の 1 行 | FR4.2 |
| BR4.3 | 古い名前 | FR4.3 |
| BR4.4 | with_all_events | FR4.4 |
| BR5.1 | 最後に 0.10.4 | FR5.1 |
| BR6.1 | 公開・依存・ソースの検査 | NFR2、NFR4、NFR5 |
| BR6.2 | コミットの区分 | NFR6 |
| BR6.3 | 品質の関門 | NFR3 |
