## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup/construction/functional-design/inventory.md > 4a 直す文書 | 直す文書として AC-476a と AC-476b だけを挙げているが、`docs/spec/language-agnostic/11-quality-and-build.md` の AC-476c(933 行から続く表の 934)、AC-476d(936)、AC-476e(935)も、`graph_crossing_lanes`、`graph_mixed_shapes`、`graph_layout_unify`、`graph_perf` というテストファイル名を参照している。形ごとのファイルに組み直すと、これらの参照も古くなる。AC-475(`graph_drawable`)の行も、件数や移した件の確認が要る。 | 直す文書の一覧に AC-476a〜e と AC-475 を入れる。組み直した後のファイル名と件数で、`docs/spec/rust/11:201`、`05:804`、`language-agnostic/11` の 932〜938 行を全部更新する。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup/construction/functional-design/inventory.md > 4a と 冒頭の graph のテストの件数 | graph のテスト(`graph_lane_polish` など)は `crates/core/tests/` にある。`crates/cli/tests/` にはない。しかし 4a はクレートを書かず、「cli の前提」の段落のすぐ下に件数を置いているため、新しい `graph_shape_*.rs` の置き場所が読み手に曖昧になる。 | 4a に「`crates/core/tests/`(`graph_common/mod.rs` も同じ場所)」と明記する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup/construction/functional-design/inventory.md > 2c | runtime の `consistency.rs` の `mod tests` は `use super::*;` で上の名前を使う。1 文字も変えずに残すには、薄くしたファイルが、テストの使うすべての名前(`PipelineState`、`PipelineDefinition`、`ConsistencyResult`、`MismatchKind`、`check_consistency` と補助)を、親の側で import している必要がある。「`#[cfg(test)] use` だけ」と書くだけでは、ビルドが通る保証にならない。 | テストが使う名前を洗い出し、薄いファイルで足りる import を一覧に書く。足りない名前があれば、そのときは変更の承認を取る。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup/construction/functional-design/inventory.md > 3d | converter の `error.rs` の文は `(上限 32)` で、core の `depth.rs` の文は `(上限は 32 段)` と、書き方が違う。2 つの文は別のままにすること、`format!` が core の文を 1 文字も変えないことが、書き分けられていない。`crate::raw::MAX_DEPTH` が実在し、const 文脈で参照できることも、この一覧では確かめていない。 | 実装の前に `crate::raw::MAX_DEPTH` の存在と型を確かめて書く。`depth.rs` の文の現行の文字列を、`format!` の期待として明記する。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup/construction/functional-design/functional-spec.md > 4 確かめ方 | 4d は README の文章を変える。動きは変えないが、README は `docs-check` と `docs_*.rs` の必須語句の検査の対象で、68 の行は 4 か所(254、493、515 と 664 など)に関連する。「利用者から見える動き」の方針と、README の変更の関係が、確かめ方に書かれていない。 | 確かめ方に「4d は文書だけの変更で、`docs_*.rs` と CI の `docs-check` が通ることを確かめる」と 1 行足す。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| コードとの突き合わせ(手作業) | 1a、1b、1d: `ctx.rs:130-200`(prepare は「定義 → ジョブ → `--assume` → run ID → `.env` → ステート」)、`graph.rs:80-109`(run ID → `read_state_raw` → 照合。`.env` は別の場所)、`aux_commands.rs:35-50` の記述は、コードと一致した | 順序の指定は正しい。run ID とステートを別の関数にする案は、順序を守れる |
| 同上 | 2a: `DisplayEvent` を組み立てるテストは `cli/tests/report_messages.rs` だけ(238、246、251、528、537、547 行)。`ctx.rs:306-350` と `next_step.rs:108` の写しも確認した | 「変えるテストは 1 か所」は正しい。`report` は `pub mod` なので、公開の型の変更である点だけ承認が要る(spec の 3 節に書いてある) |
| 同上 | 2c: 使用箇所は runtime の `lib.rs`、`types.rs`、`tests/flow_with_state.rs`、`tests/nfr_performance.rs`、cli、core の `tests/plan_pipeline.rs` で、runtime からの再エクスポートで道は保てる | 再エクスポート案は妥当 |
| 同上 | 2d: `reader.rs:76-91` は open と metadata の失敗を Unreadable、上限超えを TooLarge、`inspect_run_state` は `NotFound` も含めすべて Broken。一覧の写像の説明と一致した | 問題なし。`Ok(None)` を Broken にする写像を忘れないこと |
| 同上 | 3d/4b: `nfr_security.rs:191-209` の `env_probe_child` と `--ignored --exact env_probe_child` の文字列は一覧のとおり。`depth.rs:79` の文も一致 | 改名時は 2 か所を一緒に変える指定で足りる |
| 同上 | 4a: 6 組のうち、組 1、組 2(混ざる形。入力 5 遷移と期待の全文)、組 3(d1。入力 `both("a", c)` 4 ジョブ、期待は同じ文字列)、組 4(n1 と同じ遷移 6 ジョブで期待も同じ)、組 5(b1)は、入力と期待の全文が同じだった。組 6 の性能は、`graph_perf` のほうが同じ構成で、さらに `rows.len() > N` を確かめる | まとめる対象の選び方は正しい。graph のテストは cli ではなく core にある(上の Minor) |
| 同上 | `nfr_source.rs`(runtime は再帰して走査、cli は `src/` を読む)と `msrv_consistency.rs` | 3a の `walk/` への分割と 2c の移動は、再帰の走査に影響しない。`platform/unix.rs` の例外にも触れない |

### Summary

一覧の行番号、順序、「テストへの影響なし」の主張は、抽出した範囲でコードと一致した。段の順序(cli、型、core、テストと文書)とコミットの細かさも妥当で、動きが変わる危険は見つからなかった。残るのは、更新が必要な仕様書の一覧の不足(AC-476c〜e、AC-475)など、実装の前に直せる軽い指摘である。
