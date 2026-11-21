## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T21:35:07Z
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | demo/windows/run-demo.cmd (whole file; commits 0e8de61..54a2359) | 既存の Windows 用バッチの改行が、CRLF から LF に変わった。変更前(c6328f5)は 293 行に CR があり、いまは 0 行(`git ls-files --eol` は `w/lf`)。差分は 607 行に膨らんだが、空白と改行を無視すると、本来の変更は場面 12 の 21 行の追加だけ。`.gitattributes` は無く、改行を保つ仕組みも無い。LF だけのバッチは、`call :scene` / `call :runcmd` のラベル解決が、cmd.exe で失敗することが知られている(日本語を含むファイルで、とくに起きやすい)。`windows_demo.rs` は `cfg(windows)` で、この環境では実行もコンパイルもされていないので、確認の手段がない。code-summary.md 計画からの逸脱の 5(シェルの `printf`/`python3`/`sed -i` での編集)が、原因と見られる。 | `demo/windows/run-demo.cmd` を、変更前の CRLF に戻したうえで、場面 12 の 21 行だけを CRLF で足す(`git diff -w --ignore-cr-at-eol` が 21 行の追加だけになること、`git ls-files --eol` が `w/crlf` になることで確かめる)。Windows の CI(`test`)で、場面 12 の `windows_demo.rs` が通ることを、承認の前に確かめる。 | Resolved (reviewer) |
| R-02 | Minor | crates/core/tests/graph_perf.rs; crates/cli/tests/ | BR2.7 は「`graph` を text と Mermaid で実行して 1 秒以内」と定めるが、性能のテストは core の `build_graph` と `layout_graph` だけを測る。text の整形(`render_lanes`、`render_edge_list`)と `render_mermaid` は、1,000 ジョブ(入れ子の最大と、交差する形)で測られていない。`-v` の詳細の行(ジョブごとに全辺を走査)も同じ。整形は O(行 × 列) の単純な処理で、いま問題が起きる様子はないが、規則の文言どおりのテストが無い。 | 1,000 ジョブの 2 つの形で、text、`-v` つきの text、Mermaid の整形まで含めて、1 秒以内を確かめるテストを足す(終了を確かめるだけでもよい)。 | Unresolved |
| R-03 | Minor | crates/cli/tests/graph_report.rs; crates/cli/src/report/graph_text.rs (`render_lanes`) | 敵対的な定義(制御文字、非常に長いジョブ ID、Mermaid の特殊文字、1,000 ジョブの入れ子)を text で出すテストが、薄い。Mermaid のエスケープは `escape_label` の単体テストで確かめられているが、text でジョブ ID の制御文字が `escape_path` で逃がされること、長い ID でも落ちないことは、テストされていない(コードを読むかぎり落ちない)。Mermaid の `escape_label` は `char::is_control`(Cc だけ)で、U+2028 や双方向の制御文字は逃がさない(text 側の `escape_path` は逃がす)。 | text のジョブ ID とコマンドに、制御文字と長い文字列を入れたテストを足す。Mermaid も、text と同じ範囲の制御文字を逃がすか、逃がさない理由を書く。 | Unresolved |
| R-04 | Minor | crates/core/tests/graph_layout_oracle.rs | 正解データの例は、成功の遷移だけで作られる(`TransitionOn::Success`)。警告だけの飛び越しが、同じレーンの図になることは、`graph_edges.rs` の辺のテストで間接的に確かめられるが、レーンの配置を、警告の遷移で確かめる例は無い。537 個の例の一致、ファイルの複製が元と同一(`cmp` で一致)、例の数の検査(537)、一致しない例が 1 つでもあれば失敗することは、確認できた。 | 警告だけの飛び越しの定義から、成功のときと同じレーンの図になるテストを、1 件足す。 | Unresolved |
| R-05 | Minor | aidlc/spaces/default/intents/261120-env-graph-dryrun-init/construction/u2-graph/code-generation/code-summary.md > レビューの指摘への対応 | Windows の CI で windows_demo.rs の場面 12 が通ることは、この環境では確かめられていない(code-summary.md も明記している)。 | マージ前に GitHub Actions の test(windows)ジョブで確認する。 | New |

> R-01 Reviewer note: demo/windows/run-demo.cmd は CRLF に戻った(i/crlf w/crlf、HEAD の 314 行すべてに CR)。c6328f5 からの差分は場面 12 の 21 行の追加だけ。c6328f5 時点で CR を持っていた demo/、README.md、crates/ のファイルで、CR の数が減ったものはない。

> R-02 Reviewer note: 性能テストが測るのは辺とレーンの配置だけで、text と Mermaid の整形は測っていない。code-summary.md の「既知の限界」に記録して受け入れ済み。ブロックしない。

> R-03 Reviewer note: 制御文字、長い ID のテストが薄い。Mermaid のエスケープが U+2028 と双方向制御文字に及ばない。「既知の限界」に記録して受け入れ済み。ブロックしない。

> R-04 Reviewer note: レーン配置の正解データが成功の遷移だけで作られている。「既知の限界」に記録して受け入れ済み。ブロックしない。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| git ls-files --eol demo/windows/run-demo.cmd | i/crlf w/crlf | CRLF が戻っている。 |
| HEAD の run-demo.cmd の CR 行数と全行数 | 314 と 314 | すべての行が CRLF で、混在はない。 |
| git diff --stat c6328f5 HEAD -- demo/windows/run-demo.cmd | 21 insertions | 場面 12 の追加だけ。 |
| c6328f5 から HEAD の CR 数の減少を調べる走査(demo/、README.md、crates/) | 減ったファイルなし | 他のファイルの改行は変わっていない。 |
| cargo test --locked -p conductor-cli --test unix_demo | 3 passed | 合格。 |
| cargo fmt --all -- --check | OK | 合格。 |

### Summary

1 回目の Major だった CRLF の消失は直り、差分は場面 12 の 21 行だけになった。3 件の Minor は code-summary.md の「既知の限界」に正直に記録されていて、1 回目の指摘と食い違わない。ブロックする指摘はない。
