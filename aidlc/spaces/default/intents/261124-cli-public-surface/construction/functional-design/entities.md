# エンティティ(Entities): cli のライブラリの公開範囲を狭める

永続のデータ(ステートファイル)は扱わない。利用者に見える出力も変えない。変わるのは、cli のライブラリとしての公開範囲だけ(13 項目を残し、15 項目を crate の中に狭める)。型の形、項目、変種、メソッドは変えない。単体テストの置き場所は `src/tests/`(D1。決定: 推奨のとおり。ユーザー、2026-11-24)で、本番の成果物には入らない。詳細は `inventory.md` §1。

```yaml
entities:
  # ---- 公開に残す(変えない) ----
  - name: run_cli / run_cli_with
    description: 入口の関数(公開)。main が run_cli を、流れのテストが run_cli_with を使う。シグネチャと本体は変えない。
  - name: Deps
    description: 差し替え口の束(公開。pub の項目 runtime・sink・env・signals)。変えない。
  - name: Runtime / Sink / Env / Signals
    description: 依存の注入の継ぎ目の trait(公開)。メソッドと引数は変えない。
  - name: SystemRuntime / SystemSignals / StdSink / SystemEnv
    description: 本番の実装(公開)。StdSink・SystemEnv は main が使い、SystemRuntime・SystemSignals は run_cli と統合テストが使う。変えない。
  - name: SignalError
    description: Signals::start の失敗(公開。変種 Failed)。継ぎ目に出る唯一の cli の型(偽物が返す)。変えない。
  - name: install_panic_hook
    description: パニックのフック(公開)。main が使う。変えない。

  # ---- crate の中に狭める(形は変えない) ----
  - name: parse_args
    description: 引数の解釈(args)。公開の再公開を消し、lib.rs の非公開の use で run_cli_with が使う。
  - name: CliCommand / CommandKind / GraphFormat / InitOptions / InitTarget / Language / Level / ParsedArgs / RunIdArg / UsageError / UsageErrorKind
    description: 解析の結果・表示の言語と水準・使い方の誤りの型(types)。公開の再公開を消す。CommandKind と ParsedArgs は lib.rs の非公開の use で run_cli_with が使う。
  - name: exit
    description: 終了コードの定数(EXIT_OK〜EXIT_INTERNAL)と exit_code_for・outcome_exit_code・signal_exit_code。pub mod を mod にする。値は変えない。
  - name: report
    description: 表示(DisplayEvent、Reporter、ResultDetail、StateContext、push_line と、下位の escape・graph_mermaid・graph_text・init_template・plan_text・messages・watch)。pub mod を mod にする。下位の中の pub は pub のまま(外から見えなくなる)。
  - name: run_id
    description: run ID の解決(resolve_run_id、aux_run_id)。pub mod を mod にする。

  # ---- テストのときだけのもの(本番の成果物に入らない) ----
  - name: tests(src/tests/)
    description: クレートの内部を直接確かめる単体テスト。lib.rs の最後の #[cfg(test)] mod tests; から読む。元の統合テストと同じ名前の 13 ファイルと、補助 common/(FakeEnv・FakeSink・state_error の写し、誤りの種類の一覧 kinds)、docs_support.rs(readme の写し)。
  - name: Language(tests/common/mod.rs)
    description: 統合テストの偽の環境のロケールを選ぶ、テストの言語(写しの enum。変種 En / Ja)。FakeEnv::for_language の引数(D5。決定: 推奨のとおり。ユーザー、2026-11-24)。cli の types::Language とは別の型。
```

## 利用者に見える変わること

なし。`conductor` の出力、誤りの表示、終了コード、ステートファイル、`--dry-run` の出力、`conductor --version`(`conductor 0.9.8`)は変わらない。cli のライブラリ(`publish = false`。crates.io には出さない)から、`parse_args`・解析の結果の型・`exit`・`report`・`run_id` が外から見えなくなる(使うのは同じクレートの `main` と、テストだけだった)。
