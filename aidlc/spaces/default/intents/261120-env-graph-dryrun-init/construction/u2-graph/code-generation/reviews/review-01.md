## Review

**Verdict:** NOT-READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-21T21:32:09Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | demo/windows/run-demo.cmd (whole file; commits 0e8de61..54a2359) | 既存の Windows 用バッチの改行が、CRLF から LF に変わった。変更前(c6328f5)は 293 行に CR があり、いまは 0 行(`git ls-files --eol` は `w/lf`)。差分は 607 行に膨らんだが、空白と改行を無視すると、本来の変更は場面 12 の 21 行の追加だけ。`.gitattributes` は無く、改行を保つ仕組みも無い。LF だけのバッチは、`call :scene` / `call :runcmd` のラベル解決が、cmd.exe で失敗することが知られている(日本語を含むファイルで、とくに起きやすい)。`windows_demo.rs` は `cfg(windows)` で、この環境では実行もコンパイルもされていないので、確認の手段がない。code-summary.md 計画からの逸脱の 5(シェルの `printf`/`python3`/`sed -i` での編集)が、原因と見られる。 | `demo/windows/run-demo.cmd` を、変更前の CRLF に戻したうえで、場面 12 の 21 行だけを CRLF で足す(`git diff -w --ignore-cr-at-eol` が 21 行の追加だけになること、`git ls-files --eol` が `w/crlf` になることで確かめる)。Windows の CI(`test`)で、場面 12 の `windows_demo.rs` が通ることを、承認の前に確かめる。 | New |
| R-02 | Minor | crates/core/tests/graph_perf.rs; crates/cli/tests/ | BR2.7 は「`graph` を text と Mermaid で実行して 1 秒以内」と定めるが、性能のテストは core の `build_graph` と `layout_graph` だけを測る。text の整形(`render_lanes`、`render_edge_list`)と `render_mermaid` は、1,000 ジョブ(入れ子の最大と、交差する形)で測られていない。`-v` の詳細の行(ジョブごとに全辺を走査)も同じ。整形は O(行 × 列) の単純な処理で、いま問題が起きる様子はないが、規則の文言どおりのテストが無い。 | 1,000 ジョブの 2 つの形で、text、`-v` つきの text、Mermaid の整形まで含めて、1 秒以内を確かめるテストを足す(終了を確かめるだけでもよい)。 | New |
| R-03 | Minor | crates/cli/tests/graph_report.rs; crates/cli/src/report/graph_text.rs (`render_lanes`) | 敵対的な定義(制御文字、非常に長いジョブ ID、Mermaid の特殊文字、1,000 ジョブの入れ子)を text で出すテストが、薄い。Mermaid のエスケープは `escape_label` の単体テストで確かめられているが、text でジョブ ID の制御文字が `escape_path` で逃がされること、長い ID でも落ちないことは、テストされていない(コードを読むかぎり落ちない)。Mermaid の `escape_label` は `char::is_control`(Cc だけ)で、U+2028 や双方向の制御文字は逃がさない(text 側の `escape_path` は逃がす)。 | text のジョブ ID とコマンドに、制御文字と長い文字列を入れたテストを足す。Mermaid も、text と同じ範囲の制御文字を逃がすか、逃がさない理由を書く。 | New |
| R-04 | Minor | crates/core/tests/graph_layout_oracle.rs | 正解データの例は、成功の遷移だけで作られる(`TransitionOn::Success`)。警告だけの飛び越しが、同じレーンの図になることは、`graph_edges.rs` の辺のテストで間接的に確かめられるが、レーンの配置を、警告の遷移で確かめる例は無い。537 個の例の一致、ファイルの複製が元と同一(`cmp` で一致)、例の数の検査(537)、一致しない例が 1 つでもあれば失敗することは、確認できた。 | 警告だけの飛び越しの定義から、成功のときと同じレーンの図になるテストを、1 件足す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `cmp` (functional-design/layout-oracle.md と crates/core/tests/fixtures/graph_layout_oracle.md) | 一致 | 複製は、元と同一。 |
| `cargo test -p conductor-core --test graph_layout_oracle --test graph_perf --test graph_edges --test graph_drawable` | すべて通過(oracle 4 件、537 例すべて一致。perf 2 件) | BR2.3、BR2.4、BR1.x が、実装どおりに働く。 |
| `cargo test --locked --workspace` | 失敗 0 件(macOS) | 既存のテストは、通る。windows_demo.rs、ubuntu、MSRV は、この環境では未実行。 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし | |
| `git diff c6328f5 HEAD -- Cargo.lock '**/Cargo.toml'` | 差分なし | 新しい依存は無い。 |
| `grep` の GPL の識別子(`graph_update_state`、`GRAPH_PADDING`、`column_state`、`GRAPH_PRE_COMMIT`)を `crates/core/src/graph/` で検索 | 該当なし | `layout.rs` は、スタックによる独自の走査で、git の `graph.c` の構造(状態機械)と違う。複製の兆候は無い(BR2.6)。 |
| `grep` の `unsafe`、`cfg(unix)`、`cfg(windows)` を、graph の新しいファイルで検索 | 該当なし | 規則どおり。 |

### Summary

実装そのものは健全だった。BR1.5 の `resolve_transition` の切り出しは `classify_and_decide` の振る舞いを変えず(`graph/edges.rs` が同じ関数を呼ぶ)、`is_lane_drawable` は BR2.4 の 3 条件(jobs=1 を含む)を実装し、537 例のすべてに一致し、`--state` は `read_state_raw` だけを使い、環境変数の値は、どの出力にも出ない(バイナリのテストが秘密の値で確認)。既存のテストへの変更は、すべて追加か範囲の拡張で、表明の弱めや削除は無い(args_grammar、report_messages、unix_demo、windows_demo を確認)。source-manifest.json は、差分の 37 の経路をすべて含む。ただし、既存の Windows 用デモのバッチの改行が、CRLF から LF に全面で変わっており、Windows の CI で場面が壊れる恐れが、この環境では確かめられない。これを直してから通すこと。
