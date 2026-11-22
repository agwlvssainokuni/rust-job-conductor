# ビジネスルール(Rules): args.rs のオプションの表化

詳細は `functional-spec.md`。

```yaml
rules:
  - id: BR1.1
    statement: オプションの性質(名前、値を取るか、使える場所)を 1 つの表 OPTIONS に置く。
    category: policy
    applies_to: crates/cli/src/args.rs
    trigger: 常に
    logic: functional-spec.md 3.1
    violation: オプションごとの性質が表の外に残る
    source: intent-statement.md(Success Metrics 1 つ目)
  - id: BR1.2
    statement: 字句の段階は OPTIONS を名前で引いて、値を取るかで分ける。map_lex_error の値を取らないオプションの集合も OPTIONS から作る。
    category: policy
    applies_to: crates/cli/src/args.rs
    trigger: 引数の字句の段階
    logic: functional-spec.md 3.3
    violation: オプションごとの match の腕が残る
    source: intent-statement.md(Success Metrics 1 つ目)
  - id: BR1.3
    statement: 「使えない」と「重複」の検査は、仕様書の 04 章 4.2 の 6・7 の順を写した CHECKS の並びで行う。
    category: policy
    applies_to: crates/cli/src/args.rs
    trigger: 引数の組み立ての段階
    logic: functional-spec.md 3.2
    violation: オプションごとの if の並びが残る
    source: intent-statement.md(Success Metrics 1 つ目)
  - id: BR2.1
    statement: 誤りの種類、対象、報告の順は、--to の件を除き今と同じ。既存のテストを 1 行も変えずに通す。入り組んだ順は、表化の前にテストで固定する。
    category: constraint
    applies_to: crates/cli/src/args.rs、crates/cli/tests/args_grammar.rs
    trigger: 常に
    logic: functional-spec.md 5
    violation: 既存のテストを変える、または順が変わる
    source: intent-statement.md(Success Metrics 2 つ目)
  - id: BR3.1
    statement: convert の --to の値が yaml/toml 以外(UTF-8 でない値を含む)のとき、InvalidValue の対象は --to で、入力の値を含めない。
    category: validation
    applies_to: crates/cli/src/args.rs
    trigger: convert の --to の値の検査
    logic: functional-spec.md 4
    violation: 対象に値が出る
    source: intent-statement.md(Success Metrics 4 つ目、Q1)
  - id: BR3.2
    statement: rust と language-agnostic の仕様書の --to の誤りの記述と例を新しい動きに合わせ、版の更新の候補として記録する。--to 以外の対象と、凍結した 5 つの仕様書は変えない。
    category: policy
    applies_to: docs/spec/rust/04-cli.md、docs/spec/rust/10-errors-and-exit-codes.md、docs/spec/language-agnostic/04-cli.md、docs/spec/language-agnostic/10-errors-and-exit-codes.md、docs/spec/language-agnostic/11-quality-and-build.md
    trigger: 常に
    logic: functional-spec.md 4
    violation: 仕様書と実装が食い違う
    source: intent-statement.md(Success Metrics 4・5 つ目、Q1)
  - id: BR4.1
    statement: enter_flow の位置引数なしの分岐は残し、コメントを「公開の型 CliCommand から来たときの守り」にする。
    category: policy
    applies_to: crates/cli/src/dry_run.rs
    trigger: 常に
    logic: functional-spec.md 2
    violation: 分岐を消す、またはコメントが「来ない」のまま
    source: intent-statement.md(Success Metrics、Q3)
  - id: BR5.1
    statement: fmt、clippy -D warnings、全テスト、行カバレッジ 80% 以上、MSRV 1.100 を通す。新しい依存、unsafe を足さず、src/ の platform の外に cfg を足さない(テストの #[cfg(unix)] は既存の置き方に倣ってよい)。
    category: constraint
    applies_to: リポジトリ全体
    trigger: 常に
    logic: functional-spec.md 5
    violation: どれかが通らない
    source: intent-statement.md(合格の条件)
```
