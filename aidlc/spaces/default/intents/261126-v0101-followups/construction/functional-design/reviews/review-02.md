## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-26T22:25:16Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 2〜3(BR3.4) | 「plan の中だけを直す」は、書かれた手順では実現できない。遅さの元になりうる場所が、変えないと決めた共有の関数の中にある。`crates/core/src/flow/decide.rs` の `decide_before_run` は、53 行で `definition.job_index(job_id)`、56 行で `record_of(job_states, job_id)`(どちらも先頭からの線形の探索)を呼び、さらに規則 6 で `definition.job[..index]` の全員を毎回見る(済んだジョブが続くと、ジョブごとに位置の分だけ見るので二乗になる)。`crates/core/src/flow/next_step.rs` の 128 行も `definition.job_index(&job.id)` を呼ぶ。`plan/simulate.rs` は `decide_from_state` と `classify_and_decide` をジョブ ID だけで呼ぶので、位置を渡せない。手順 2 の「繰り返しの位置から渡す」は、この呼び出しの形では成り立たない。手順 3 の逃げ道(plan の中に位置つきの引き方を置いて呼ぶ側を差し替える)は、`decide_before_run` が `decide.rs` の非公開の関数で、規則 1〜7 を持つため、plan に判断の規則を丸ごと複製することになる。`simulate.rs` の冒頭にある「実行と同じ判断を使う(BR1.2)」と衝突し、本番と模擬の判断が別々に変わりうる。 | W3 と BR3.4 を直す。(a) 遅さの元が `flow/` の中にあるときの扱いを決める。たとえば、位置を引数に取る内部の入口を `flow` に足して、本番と模擬の両方が同じ判断の本体を呼ぶ形にする(本番の動きは変えず、`decide.rs` と `next_step.rs` の共有の変更を認める)。または、「plan の中だけ」を守れず直さないと決めて BR3.6 の記録にする。(b) 判断の規則を plan に複製しない、と書く。 | Resolved (reviewer) |
| R-02 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W4 のテスト(BR5.5) | 「子の標準入力に改行を与える」は、今のテストの補助と合わない。`crates/cli/tests/bin_support/mod.rs` の `run_limited`(86 行)は、呼び出し側の指定にかかわらず `command.stdin(Stdio::null())` で上書きする。`unix_demo.rs` の `run_demo` もこれを通る。また `run-demo.sh` は `set -e` を使わない(`set -u` のみ)ので、標準入力が空でも `read -r _` は EOF ですぐ戻り、止まらない。つまり改行は要らず、書かれた方法はそのままでは実現できない。 | 「改行を与える」を、「`DEMO_PAUSE=1` を環境に足し、標準入力は `run_limited` の既定(空)のままにする。`read` は EOF ですぐ戻る」に直す。改行が要ると考えるなら、`run_limited` を変えずに別の起動の補助を使う、と書く。 | Resolved (reviewer) |
| R-03 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 1 と図 | 測定の表の置き場所が食い違う。W3 の手順 1 は「`code-summary.md`」に残すとするが、FR3.3 と `rules.md` の BR3.6 は、直さないときの測定と理由を `build-and-test-summary.md` に書くとする。図の「いいえ」の枝は置き場所を言わない。 | 置き場所を 1 つに決める(測定は `code-summary.md`、直さないときの判定の理由は `build-and-test-summary.md` に、など)。BR3.6 と W3 の文を揃える。 | Resolved (reviewer) |
| R-04 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の図(F から V への繰り返し) | 「直す → 確かめる(不合格なら直す)」の繰り返しに、抜ける条件が無い。75ms と 2.5 倍に届かないとき、無限に直し続けることになる。 | 抜ける条件を足す(たとえば、直しても届かないときは、直しを戻して BR3.6 の記録にする。または人に確かめる)。 | Resolved (reviewer) |
| R-05 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 2〜3(BR3.4) | 遅さの元は規則 6 の走査と `job_index`・`record_of` だけではない。`apply_skip_in`(`record.rs` 197〜212 行)は、スキップのたびに `any(Error)` と `first_pending` の探索で全ジョブを見るので、スキップが続くと二乗になる。`next_step.rs` の `pending_ids`(138〜145 行)も `@end` のたびに後ろ全部を走査する。手順 3 の「最初の済んでいないジョブの位置」の目印は規則 6 向けの定義で、`first_pending`(状態が pending の最初)とは条件が違う。どの走査を目印で直し、どれを測定の結果しだいで残すかが書かれていないので、実装者が推測することになる。 | 手順 1 の測定で特定した走査ごとに、使う目印を決める、と書く。たとえば、目印が「最初の済んでいない位置」だけで足りない走査(`apply_skip_in` の `first_pending`・`any(Error)`)は、別の値を持ち回るか、測定で効かなければ直さない、と書く。 | New |
| R-06 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 4(本番の動きが変わらないことの確かめ) | 確かめは既存のテストが変えずに通ることだけである。公開の入口は、内部の入口に渡す位置と目印を自分で求めるように変わる(本番の経路に新しい計算が入る)。目印が「済んだ」から「済んでいない」に戻る(ステートの記録がジョブ ID の順に並ばない、重複、エラーの再開、`end_reached` など)入力は、既存のテストが網羅するとは限らない。目印つきの経路と、位置を自分で求める経路の判断が一致することの確かめが要らないのかが、書かれていない。 | 内部の入口(位置と目印を渡す)と、目印を使わない求め方で、同じ入力に同じ判断になることを確かめるテストを 1 つ足す、と書く。入力は、記録が定義の順でないステート、記録の欠け、再開位置、`end_reached` を含める。 | New |

> R-01 Reviewer note: W3 の手順 2〜3 と BR3.4 は、`flow` に `pub(crate)` の内部の入口を足し、公開の入口がそれを呼ぶ形に直った。判断の本体(規則 1〜7、`classify_and_decide`、記録の規則)を `flow` に 1 か所で残し、plan は位置と目印を持ち回るだけで、規則を複製しないと明記された。`decide.rs` の `job_index`(53 行)と `record_of`(56 行)、規則 6 の `job[..index]` の走査、`next_step.rs` 128 行の `job_index` は、位置と「最初の済んでいないジョブの位置」を受け取れば本体の内側で直せる。`apply_skip_in` は既に `hint` を受け取る形(`record.rs` 170 行)なので、同じ作り方に合う。

> R-02 Reviewer note: W4 のテストは、`DEMO_PAUSE=1` を足し、標準入力は `run_limited` の既定(`Stdio::null()`、`bin_support/mod.rs` 86 行)のままにする、と直った。`run-demo.sh` は `set -u` のみで(20 行)、`pause` の `read -r _`(57 行)は EOF で戻るので、書かれた方法で実現できる。

> R-03 Reviewer note: 測定の表は、どの枝でも `code-summary.md` に残し、直さない枝では `build-and-test-summary.md` にも書く、と W3 の手順 1 と BR3.6 で揃った。

> R-04 Reviewer note: W3 の手順 6 と BR3.5 に、届かなければ直しを戻して BR3.6 の記録にする、繰り返さない、という抜ける条件が入った。図の「いいえ」の枝も戻す枝になった。

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `grep` で `decide.rs`・`next_step.rs`・`record.rs`・`flow/mod.rs` の走査を確認 | 規則 6 の走査と `job_index`・`record_of` が `decide.rs` 53・56・78〜81 行にある。`apply_skip_in` は `hint` を既に受け取る | 手順 2〜3 の内部の入口は、`apply_*_in` の作り方と同じで、実現できる |
| `demo/run-demo.sh` と `bin_support/mod.rs` | `set -u` のみ、`read -r _` は EOF で戻る、`run_limited` は標準入力を空にする | R-02 の直しは実現できる |

### Summary

R-01 から R-04 は直った。内部の入口を `flow` に置き、plan が規則を複製しない形で、W3 は実現できる。残る懸念は、`apply_skip_in` などの別の走査の扱いが決まっていないことと、本番の経路の一致を確かめるテストが無いことの 2 つで、どちらも Minor であり、READY を妨げない。
