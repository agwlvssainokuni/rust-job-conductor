## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-cli-step-unify/construction/functional-design/functional-spec.md > §3 record_skip | `record_skip` は土台が `None` のとき内部で `initial_state` を作るため、`run` 側は「土台と比べて変わったか」を知れない。`run.rs` 69-90 では、変わったときだけ(書いたあとに)reset し、変わらなければ書かず消さない。`pipeline` は変わったかを要しない(`recorded.completed` なら常に `finish`)。仕様の「呼び出し側は、土台と比べて知る」は、`None` のとき実装できない。 | `record_skip` の戻り値に「書いたか(`changed`)」を含める(例: `(PipelineState, bool)` または小さな構造体)と仕様と entities.md に明記する。`run` は `changed && completed && remove_on_complete` のときだけ reset する。 | New |
| R-02 | Major | aidlc/spaces/default/intents/261122-cli-step-unify/construction/functional-design/functional-spec.md > §5 テストの方針 | 新規の単体テスト(`decide`、`record_skip`、`remove_if_requested`)は `pub(crate)` の関数が対象だが、cli の `src/` には `#[cfg(test)]` が 1 つもなく、`tests/` の統合テストからは `pub(crate)` に触れない。`record_skip` と `remove_if_requested` は `Ctx`(`Runtime`、`Signals`、`Sink`、`Env`、`Reporter` の偽)を要し、置き場所と偽の作り方が決まっていない。 | テストの置き場所を決める(`step.rs` 内の `#[cfg(test)]` で `Ctx` を偽で組む、または既存の `flow_*` の経路で `run` と `pipeline` から確かめる)と明記する。80% の下限は既存のフローテストで満たせるかも確かめる。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-cli-step-unify/construction/functional-design/functional-spec.md > §4 仕様書の書き直し | 書き直しの対象が 05 の §5.2、§5.3 と 02 のモジュールの表だけ。`docs/spec/rust/02-architecture.md` 180-203 行の呼び出しの流れ(`finish` を名指し)と 341 行の `pipeline.rs` の欄(`finish`)も、`finish` を置き換えると古くなる。`04-cli.md` 234 行の `run_individual`・`run_pipeline` は、関数が残るので変わらない。 | 02 の 180-203 行の流れの図と 341 行の表も更新の対象に加える。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261122-cli-step-unify/construction/functional-design/functional-spec.md > §3 run_step と §2 の 4 | `run` の `job_index` の到達不能の分岐(`run.rs` 111-114)は、今は初期ステートの書き込みの**後**。`run_step` は `job` を受け取るので、検索は書き込みの前に移る。到達しないため動きは変わらないが、仕様に書かれていない。 | 「到達不能で、順序が変わっても動きは変わらない」と §2 の 4 に 1 行足す。 | New |
| R-05 | Minor | aidlc/spaces/default/intents/261122-cli-step-unify/construction/functional-design/entities.md > StepResult | `base` の所有: `run` は `session.state.clone()`、`pipeline` は `current.take()` を渡す。`run_step` が `recorded` を返すので `pipeline` は `current = Some(recorded)` で戻せ、早期 return の経路でも `current` は読まれず、実害はない。借用は、`Session` が `Ctx` から独立なので `&mut Ctx` と `&Session` を同時に取れ、成り立つ。ただし仕様に書かれていない。 | `base` は所有で渡し、`take()` と `clone()` は呼び出し側に残すと 1 行で明記する。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| コードとの突き合わせ(run.rs、pipeline.rs、ctx.rs、retry.rs) | 重複の表(§1)と残す違い(§2)は、コードと一致 | `run` の Skip は書いたあとに表示の前に消し、`pipeline` は表示の後に `recorded.completed` なら常に消す。Run のあとは、`run` は完了なら消してから常に `exit_code_for`、`pipeline` はエラーか分類なしなら先に return、次に完了、次に中断の合図、の順。いずれも設計どおり。`after` の文脈と Durability は同じで、見落としはなかった。 |

### Summary

設計は、削除の時機の違いを呼び出し側に残す判断を含め、コードと一致し、借用の面でも実装できる。実装前に、`record_skip` の「変わったか」の返し方と、`pub(crate)` の関数の単体テストの置き場所を決める必要がある(Major 2 件、READY の範囲内)。
