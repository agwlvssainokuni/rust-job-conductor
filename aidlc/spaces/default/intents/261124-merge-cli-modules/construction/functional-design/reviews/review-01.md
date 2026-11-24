## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-25
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | info | aidlc/spaces/default/intents/261124-merge-cli-modules/construction/functional-design/functional-spec.md:§2 項目 2、§10 D2 | D2 の「pub(crate) は今の実効の範囲と同じ」は、lib.rs:101 の `mod report;` が非公開であることに依る(HEAD で確認。`pub mod escape` の中の `pub fn` は crate の外に出ていない)。この前提が文書に明記されていない。前提が成り立つので、広げることにはならず、設計は正しい。 | 実装の記録か D2 の欄に「`report` は lib.rs で非公開のため、`pub` の実効の範囲は crate の中」と 1 文を足す(任意)。 | New |
| R-02 | info | aidlc/spaces/default/intents/261124-merge-cli-modules/construction/functional-design/functional-spec.md:§1 と §9(言語に依らない仕様書) | 言語に依らない仕様書に直す所が無いという判断は正しい。`docs/spec/language-agnostic/05` の `run_job_with_retry` と `11` の `escape_name` は、ファイルでもモジュールのパスでもなく、仕様上の関数名で、名前は変わらない。cli の 4 ファイル名・`report::escape`・`crate::retry` などは、language-agnostic に 0 件(HEAD で検索)。 | 対応は不要。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261124-merge-cli-modules/construction/functional-design/functional-spec.md:§7 の仕様書の検索コマンド | 仕様書の検索は最後に `grep -v "core/src\|definition/walk\|flow/retry\|definition_retry"` で行を除く。除外の語を含む行に、直す所が同居していても見逃す(今の S1〜S24 では該当なし。HEAD で確認)。 | 実装の最後の確かめで、除外の前の出力も一度目で見て、除外された行に cli の 4 ファイルの参照が無いことを確かめる。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261124-merge-cli-modules/construction/functional-design/functional-spec.md:§2.2 の `report/mod.rs` のモジュールの説明 | 今の説明(report/mod.rs:23)は「`escape` を通す」と、消えるモジュール名の `escape` を指す。§2.2 はこの箇条を書き換える方針だが、`escape_name`・`escape_full` の関数名に直すことを検索の完了条件には入れていない。 | コミット 2 の確かめに、`report/mod.rs` の説明に消えたモジュールを指す「`escape` を通す」が残らないことの目視を足す。 | New |

### 確かめた点(指摘なし)

- 網羅(観点 1): HEAD で `crate::retry`・`retry::`・`report::escape`・`super::escape`・`escape::`・`crate::graph`・`graph::run_graph`・`crate::init`・`init::run_init`・`mod retry|graph|init|escape` を検索した。本番の参照は、step.rs:43、run.rs:18(説明)、lib.rs:187-188、report/mod.rs:28・50、plan_text.rs:26、graph_text.rs:29、watch.rs:22 の 9 か所、テストは escape_table.rs:20 の 1 か所で、設計の一覧と一致する。`run_job_with_retry` の呼び出しは step.rs:143 だけ(pipeline.rs:105 はコメント)。`run_graph`・`run_init`・`read_overlay` の呼び出しは lib.rs の 2 行だけ。`tests/`(統合テスト)に消すモジュールのパスは無い。
- 名前の衝突(観点 2): aux_commands.rs の既存の `use`(Path、Ctx、definition_path、DisplayEvent、StateContext、CliCommand など)と graph.rs・init.rs の `use` は、和集合で重なりは同じ名前の同じ項目だけ。関数名は `run_graph`・`read_overlay`・`run_init` で、既存と重ならない。step.rs に足す名前(EnvContext、RetryDecision、merge_env、retry_decision、InterruptSignal、WaitOutcome)も、既存の名前と重ならない。`MAX_DISPLAY_CHARS` と `escape_display` は report/mod.rs に既存が無い。
- 可視性(観点 2): `run_job_with_retry` の呼び出し元は同じモジュールだけなので非公開で足りる。`escape_name`・`escape_full` は `pub(crate)` で、本番に新しい公開を足さず、単体テスト escape_table.rs が読める最小の広さ。
- platform(観点 3): 今の mod.rs の中継は `unix::spawn_watcher` を呼ぶだけで、再公開への変更で処理は変わらない。`cfg` は platform の中だけにとどまり、nfr_source.rs の「os ごとの分岐は platform の中だけ」(platform を含むパスを除外)に当たらない。`unsafe`・`low_level` は足さない。unix.rs・windows.rs の可視性だけの変更で、シグナルの扱いは変わらない。deps.rs:288 の `platform::start_signal_watcher` は名前が変わらない。runtime の platform/mod.rs と同じ再公開の形。Windows の対象のコンパイルは、設計が写しで確かめた記録(clippy)を挙げており、実装の最後の確かめにも入っている。
- 利用者から見える動き(観点 4): 設計は、置き場所・可視性・`use`・説明だけを変え、関数の本体を取り出して比べる確かめ(§7)を持つ。出力・終了コード・ステート・`--dry-run`・シグナルへの影響は無い。
- テストと並び(観点 5): テストの変更は escape_table.rs の `use` 1 行だけで、件数(1,149 通過、1 無視)とテスト名は変わらない。4 コミットは、扱うファイルが互いに独立で、重なるのは lib.rs の表の別の行と仕様書 02 章の別の行だけなので、各コミットでビルドとテストが通る。
- 仕様書 rust(観点 6): HEAD で検索した直す所は、02(164、216、373、375、377、387 ほか)、04(3、203、218)、05(5、634)、07(4、25)、08(3、564、566、605)、12(85、107、117、137)で、S1〜S24 と一致する。02:376 の「同」(dry_run.rs)が、前の行が消えると意味を失う点(S19)も押さえている。
- 制約(観点 7): 新しい依存、`unsafe`、凍結した仕様書、版の更新は、どの項目にも無い。

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| git grep(消すモジュールのパスの参照。crates と docs/spec) | 設計の一覧と一致 | 漏れは無い |
| nfr_source.rs の読み取り(cfg、unsafe、ソースのファイル数) | 設計の変更は当たらない | ファイル数は 40 から 36 で、下限 10 を満たす |

### Summary

設計は、置き場所・可視性・`use` だけを変え、HEAD の参照の一覧と仕様書の直す所に漏れも名前の衝突も無く、platform の再公開も nfr_source.rs に当たらない。テストの変更は T1 の 1 行だけで、実装できる。指摘は info 2 件と minor 2 件で、いずれも確かめの精度を上げるためのもの。
