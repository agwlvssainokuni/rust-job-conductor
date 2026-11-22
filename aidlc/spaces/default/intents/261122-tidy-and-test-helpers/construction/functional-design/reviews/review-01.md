## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/inventory.md > 4e(`classify_exit_code`) | `classify_exit_code(job, code) -> Classification` は分類だけを返す。しかし `classify` が使う返す終了コード(成功・警告は 0、エラーはそのコード、成功コードに無い 0 は 1)と `conductor_error` は、現状 `classify.rs` の手順 5 の同じ if 連鎖で決まる(`code == 0` の枝)。「`classify` はそれを使って返す値と `conductor_error` を決める」だけでは、`code == 0` の判定が `classify` 側に二重に残るのか、戻り値に含めるのかが決まらない。実装者が推測することになり、「規則は 1 か所」の目的を損なう。 | 戻り値の形を決めて書く。たとえば `(Classification, reported_exit_code, conductor_error)` を返すか、`Classification::Error` かつ `code == 0` の扱いを `classify_exit_code` の内側に閉じる。`simulate.rs` が使う側の形も 1 行で示す。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/inventory.md > 5c(`types.rs:43` で `crate::args::Opt::RunId.subject()`) | 今の `args.rs` は `types.rs` を使う(`types.rs` は `args` を use しない。`types.rs` の import は `std`、converter、core だけ)。5c は `types.rs` の `RunIdArg::parse` から `args::Opt` を呼ぶので、モジュール間に `types` と `args` の相互参照が新しくできる。Rust では通るが、型の置き場である葉のモジュールが、解析の側に依存する。`Opt`・`subject` を `pub(crate)` にする公開範囲の広がりも伴う。 | 2 つの案を比べて選び、設計に書く。(a) `types.rs:43` は定数(たとえば `pub(crate) const RUN_ID_SUBJECT`)にして、`Opt::subject` もそれを使う。(b) 今のまま相互参照を受け入れ、理由を記録する。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/functional-spec.md > §4.1 T1〜T6、inventory.md 7a | 5 つの docs テストには、7 語の配列だけでなく、それを回す `for` と `assert!` も写しで残る。しかも失敗の文が違う(`docs_retry.rs` は「README に {word} がない」、ほかの 4 つは「docs-check の語句 {word} がない」)。設計は T5 だけ文を保つとし、配列だけを共通にする。これは 7a の目的(写しを 1 か所に)を半分しか満たさず、T6 で `expect` の文だけが共通の文に変わる扱いとも一貫しない。 | 配列だけの共通化でよいことを §4 に 1 行明記する。または検査の関数(`assert_docs_check_words(text)`)まで共通にして、失敗の文の差を「既定で進めるもの」に足す。どちらかに決める。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/functional-spec.md > §4.1 T14(`args_grammar.rs` に `mod common;`) | `args_grammar.rs` は 60 件の文法のテストで、偽の `Runtime`・`Sink`・`Signals` や `Harness` などを持つ大きな `common/mod.rs` をまるごと取り込むことになる。`#![allow(dead_code)]` で警告は出ないが、`FakeEnv::with` 1 つのためにコンパイルの範囲と結合が大きく広がる。設計の根拠(「写しを消す」)に対し、この副作用が書かれていない。 | 副作用を T14 の理由に 1 行書いて承認の材料にする。または `FakeEnv` を小さな補助のモジュールに分ける案と比べて、選んだ理由を残す。 | New |
| R-05 | info | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/functional-spec.md > §4.1 T19(★)、inventory.md 7e | T19 の根拠を実物で確かめた。`MemRuntime` を使う 4 ファイルのうち、`FakeSignals` は `run_mem` と `nfr_robust.rs:112` で `Noop`、`RaiseOnStart` は `ScriptedRuntime` を使うテスト(`flow_run.rs`、`flow_pipeline.rs`)だけ。`MemRuntime` で合図が立つのは `fake_wait` が `interrupt_wait` で `raise` する `flow_retry.rs:365` だけで、設計の記述と合う。台本の消費の順を変えても観測できる差はなく、この変更は実質、`ScriptedRuntime` との一貫性のためだけである。 | 対応は不要。★ のまま、承認のときに「結果は変わらず、一貫性のための変更」と示すだけでよい。 | New |
| R-06 | info | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/functional-spec.md > §6 件数の見込み | 件数の見込みを実物で確かめた。基準は `cargo test -- --list` で 1,143 件(1 件は無視)、これは 1,142 通過と 1 無視に合う。core の単体 112、runtime の単体 71、converter の単体 53、`args_grammar.rs` 60、`flow_run.rs` 52、`flow_retry.rs` 18 も一致する。計算は、core +3(−1 +4)、runtime −3(−4 +1。`writer.rs` の 4 件は移すので差し引きゼロ)、converter +1、cli +3 で、合計 +4 = 1,146 となり、整合する。 | 対応は不要。 | New |
| R-07 | info | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/inventory.md > §8 の検索で拾った範囲 | §8 の検索を再実行して一覧との漏れを探した。`fn readme`、`status_of`、`send`、`MapEnv`、`median`、`SECRET`、`kinds`、`run`、`program_name`、`.writes`、`dir_or_current`、`create_temp`、`temp_name`、`JobExecution {` のどれも、T 表と 7h の表にある場所とすべて一致し、拾い漏れはなかった(`JobExecution {` の手組みは `flow_sequences.rs` 2、`retry_flow.rs` 6、`graph_edges.rs` 1、`classify.rs`、`next_step.rs` 5、`record.rs`、`nfr_content.rs` で、7h の表と同じ)。同名の補助を 2 つ以上持つ関数(`definition`、`put`、`paths`、`ctx`、`content`)も確かめたが、引数や中身が違うので、範囲に入れない判断は妥当。`T37` の `Skipped` の根拠(`plan_pipeline.rs:182-191`、`:203-213` の直後に理由を書き直す)、`T33` の `kinds`(受理された入力では失敗する今の動きに頼るテストが無い)も実物と合う。 | 対応は不要。 | New |
| R-08 | info | aidlc/spaces/default/intents/261122-tidy-and-test-helpers/construction/functional-design/inventory.md > 4a、4b、4d、2a | 利用者から見える動きを変えない主張を実物で確かめた。4a: 規則 1〜4 の後は `status` が必ず `Pending`(`record_of` と `status_at` は同じ記録を引く)なので、`index == 0` では規則 6 が同じ `Run` を返し、規則 5 は結果に効かない。4b: `Err(_)` の今の動きは `InvalidValue` を記録して `break` で、設計の守りの枝と同じ。4d: `retry_decision` が `Retry` を返す条件と let-else の条件が一致する。2a: `skip_reason` と `NOTE_BY_*`、`plan_disposition` と `skip_reason` の `AlreadyExecuted`、`GRAPH_*` と `PLAN_*` の各対は、英語・日本語とも同じ文字列。4e: `JobExecution::exited` は `Exited{Some, None, None}` なので `is_retry_target` の結果と一致する。3a: 移動する 4 件の単体テストは `SEQUENCE` に触れないので、コミット 13 で先に移してもビルドが通る。 | 対応は不要。実装では、4d と 4b の守りの枝(到達しない else)が行カバレッジに与える影響を、段の終わりの件数の記録と一緒に確認する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| `cargo test --locked --workspace -- --list`(件数) | 1,143 件(1,142 通過 + 1 無視)。各ファイルの件数も設計の基準と一致 | 基準の件数と、1,146 の見込みの計算は整合する |
| §8 の検索(`fn readme`、`status_of`、`send`、`MapEnv`、`median`、`SECRET`、`kinds`、`JobExecution {`、`program_name`、`.writes`、`dir_or_current`、`create_temp`) | T 表・7h の表と一致。拾い漏れなし | テストの変更の一覧は網羅的 |
| `grep` による制約の確認(`nfr_source.rs` の検査の範囲、新しい依存、`cfg`、`unsafe`) | 設計は依存・`unsafe`・`cfg` を足さず、`docs_support` と `signal_support` は `cfg` を含まない。`signal_support` は `#![cfg(unix)]` のファイルからだけ使う | 制約は守られている |

### Summary

設計は実物とよく一致している。テストの変更の一覧(T1〜T46)は、§8 の検索を再実行しても拾い漏れがなく、件数の見込み(1,142 − 1 + 5 = 1,146、クレートごとの内訳も)と、★ の 4 件(T9、T19、T33、T37)の根拠も実物で裏づけられた。コミットの並びも各コミットでビルドが通る形になっている。残るのは `classify_exit_code` の戻り値の形、`types` から `args` への依存の向き、docs テストの共通化の範囲、`args_grammar.rs` への `common` の取り込みの 4 件の軽微な点だけで、承認前に直すか承認の材料にするかを決めればよい。
