## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no
**Date:** 2026-11-28T07:56:53Z

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261128-v0104-split-files/construction/functional-design/functional-spec.md > 5 分け終わりの判定(args/mod.rs 300 行以下、report/mod.rs 220 行以下、render の各枝は 1 行) | 数の目安が、今のコードから見積もると届かない可能性が高い。(1) args: 今の `args.rs` は 679 行で、先頭の `//!` と `use` が 71 行、`parse_args`(363-430)が 68 行、`build_command`(450-583)が 134 行ある。3.1 でこの 3 つは `mod.rs` に残る。`build_command` を 6 つの関数と `convert_operand`・`init_operand` に分けると、関数の頭と閉じ、文書コメントで 30〜50 行増える。見積もりは 310〜325 行で、300 行を超える。(2) report: 先頭と `use` の約 60 行、`Reporter` の `new`〜`show`(326-376)の約 51 行、`detect_language` の 17 行に、35 の枝の振り分けを足す。`RetryScheduled`(5 項目)、`JobResult`(4 項目)、`ConsistencyMismatch`(3 項目)などは、`cargo fmt` で 1 行に収まらず 3〜7 行になる。振り分けは 45 行ではなく 80 行前後で、合計は 220 行をわずかに超える見込み。「各枝は 1 行」も、`cargo fmt` を通す以上、守れない枝が出る。(walk の 420 行以下は、移す約 297 行を引いて約 400 行で、届く) | 基準を、実測に合う形にする。たとえば `args/mod.rs` を 340 行以下、`report/mod.rs` を 250 行以下にする。または、行数でなく「3.1・4.1 の移す項目が `mod.rs` に無い」を主な基準にし、行数は参考値にする。「各枝は 1 行」は「各枝はメソッドの呼び出しだけ(`cargo fmt` で折れてよい)」と書き直す。コード生成の途中で基準を満たせず、作業が止まるのを防ぐ | New |
| R-02 | Minor | aidlc/spaces/default/intents/261128-v0104-split-files/construction/functional-design/functional-spec.md > 3.2 build_command の分け方 | 6 つの関数の受け渡しに、実装者が決める隙間がある。今の `build_command` は、`positionals` のイテレータを `first`(469-476)で 1 つ進め、517-538 の手前で `given.take(Opt::RunId)` をして、`--all` の検査(478-515)と `fill_options` の両方で `run_id` を使う。表では `check_options` が `()` を返し、`command_kind` は `CommandKind` だけを返すので、イテレータの引き継ぎと `run_id` の取り出しの場所が決まらない。取り出しを `check_options` に入れると、`fill_options` が受け取れない。取り出しの順を間違えると、誤りの順(`--all` と `--run-id` → `graph` の `--run-id` → `--assume` の値)が変わる | 3.2 に 2 点を足す。(1) `collect_tokens` は位置引数を `Vec` で返し、`command_kind` が先頭を取って、残りを `check_operands` に渡す。(2) `check_options` は `given.count(Opt::RunId)` で見るだけで取り出さず、取り出しは `fill_options` の最初に置く(`count > 0` と `take(...).next().is_some()` は同じ。重複は `CHECKS` が先に誤りにする)。あわせて `args_grammar.rs` の誤りの順のテストを、分けるコミットの確かめに名指しする | New |
