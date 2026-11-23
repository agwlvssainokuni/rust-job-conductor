## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §8 コミットの並び(コミット 8 と 9)、§4.1 T17 | T17 は 2 つの変更を 1 行にまとめている。1 つは `classify.rs:177` の `o.job_id` の確認を消すこと、もう 1 つは補助 `exec` の本体を `JobExecution::new(result)` にすること。コミット 8 は `JobOutcome` の `job_id` を消すだけで、`JobExecution` の組み立ての関数はまだ `job` を受け取る(外れるのはコミット 9)。そのため T17 を丸ごとコミット 8 に置くと、`exec` の変更はコミット 8 でビルドできない。T19 は `outcome`・`step`(コミット 8)と `exec`(コミット 9)に分けて書いてあり、T17 だけ分けていない。 | T17 を 2 つに分ける。`:177` の 1 行の削除はコミット 8、`exec` の本体の変更はコミット 9 に置く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §5.1 N7 | N7 の期待が「辺の数が 1,996 + 末尾の 2 ジョブの分」と書かれていて、数が決まらない。`flow_edges` を表で引く変更(2c)が辺を落とさないことを確かめる核心の検査なのに、実装者が数を選べてしまう。 | N7 の期待の辺の数を、入力(1,000 ジョブ、各ジョブが成功と警告で後ろへ遷移を書く)から導いた 1 つの数で書く。`@end` 行きの辺が入るかどうかも書く。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §5.1 N2(`label() == to_string()`) | D10 で `Display` が `f.write_str(self.label())` になるため、`label() == to_string()` は構成上いつも真で、何も検出しない。cli が日本語の文を別に持たないことの確かめは N3 が担う。 | N2 の `label()` の確認を、全種類で空でなく互いに異なること(`code()` の一意の確認と同じ形)に替える。または件数を増やさず `reason()` の確認に統合する。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §1 方針の制約の列、§8 コミット 34(3b) | runtime の `nfr_source.rs` は、製品のコードに `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(` が無いことを検査する。cli の `nfr_source.rs` は src に `unsafe` の語と `cfg(unix)` などが無いことを検査する。制約の列にはこの panic 系の禁止が無い。3b の形 B(`File::open`、`metadata`、`take(上限+1)` の連鎖)は、`?` で書けず `expect` を使いたくなりやすい箇所である。 | 制約の列に「runtime の製品のコードに `unwrap`・`expect`・`unreachable!` を書かない(`nfr_source.rs`)」を足す。コミット 34 の合格の条件にも含める。 | New |
| R-05 | info | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §5.2 性能の基準 | 確認できた事実。D6 の決定(上限は直した後の見積もりの約 5 倍)のため、P1・P2・P5 は直す前の木でも通り、P3・P4 も手元ではほぼ同じ。段 2 の改善はテストでは守れない。仕様書もこの点を明記している。上限の幅(4.5〜5.1 倍)は debug・3 OS の CI で落ちにくい。P5 の直す前(98.1 ms)は上限 100 ms にほぼ並ぶが、直す前は検知の対象外なので問題ない。 | 対応は不要。直す前と後の時間を code-summary と test-results に記録する約束を、コミット 23・26・28 の完了の条件に入れておく。 | New |
| R-06 | info | aidlc/spaces/default/intents/261122-types-perf-visible/construction/functional-design/functional-spec.md > §4 T 表、§6 件数、§8 コミットの並び | §4.0 と inventory §8 の検索を実行して照合した。型・項目・関数の参照(`ResumeInfo`、`PlanDisposition`、`SimulatedResult`、`GraphNode`、`RetryPolicy` の項目、`RetryDecision`、`JobExecution` の組み立て、`run_job`、`EnvFileError.reason`、`reason::SYNTAX`、`0.9.6`)に、T1〜T26 から漏れたテストのファイルは見つからなかった。`flow_sequences.rs` の `sim.resume` は別の構造体で対象外。件数も整合する。core 7、runtime 2、converter 1、cli 1 で全体 1,157。ファイルごとの件数(`retry_flow` 17、`env_types` 6、`env_loader` 9、`convert_ac` 9、`report_messages` 36)も合う。変えるテストのファイルは 30 で、内訳は core 19、runtime 7、converter 1、cli 3。`NextStepNote` の写し方(エラー → `Stopped`、次のジョブ → `NextJob`、`end_reached` かつ未完了 → `EndReached`、他 → `Completed`)は `next_step.rs` の分岐と今の表示の順に一致する。`.env` の重複の位置の表は、今の「前を除いて末尾に足す」と同じ並びになる。P5 の期待(`K4999`…`K0`、警告 5,000 件、項目数 10,000 が `MAX_ENV_ENTRIES` 以内)も正しい。3b の形 B は、既存の「1 MiB + 1 で UTF-8 でない → `TooLarge`」のテストを変えない。`1k` の `types::*`・`error::*` の名前の一覧は、実物の `pub` の項目と過不足なく一致した。 | 対応は不要。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| §4.0 と inventory §8 の grep(型・項目・関数の参照、`reason`、`SYNTAX`、`0.9.6`) | 一覧に無いテストのファイルの参照は 0 件。`init_template`・`flow_validate`・`graph_report`・`flow_retry`・`nfr_security` の `multiplier` は TOML の文字列で型の参照ではない。converter の `.reason` は構造体の項目のまま | T 表は網羅的。 |
| `docs/spec` の `文法の誤りです` の行と `0.9.6` の出現 | rust・language-agnostic の行は inventory 3a の一覧と一致。`0.9.6` は 16 ファイルで `ee9f409` の 22 ファイルの範囲に一致。`Cargo.lock` の版は 4 クレート | 版の更新の範囲は妥当。 |
| `nfr_source.rs`(runtime、cli)の読み取り | 新しい依存、`unsafe`、src の `cfg` を足さない設計と矛盾しない。panic 系の禁止は制約の列に無い(上の minor) | 制約はほぼ満たす。 |

### Summary

設計は実装できる形で、T 表の網羅性、件数(1,157)、段 1・2 の「動きを変えない」根拠(`NextStepNote` の写し方、重ね合わせ・遷移先・`.env` の表)、段 3 の範囲はコードと検索の結果で裏づけられた。重大な指摘はなく、コミット 8 と 9 での T17 の分け方、N7 の期待の数の具体化、N2 の確認が無意味になる点、runtime の panic 系の禁止の追記の 4 つの minor だけが残る。
