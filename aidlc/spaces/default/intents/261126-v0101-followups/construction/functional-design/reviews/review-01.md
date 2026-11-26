## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-26T22:19:25Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 2〜3(BR3.4) | 「plan の中だけを直す」は、書かれた手順では実現できない。遅さの元になりうる場所が、変えないと決めた共有の関数の中にある。`crates/core/src/flow/decide.rs` の `decide_before_run` は、53 行で `definition.job_index(job_id)`、56 行で `record_of(job_states, job_id)`(どちらも先頭からの線形の探索)を呼び、さらに規則 6 で `definition.job[..index]` の全員を毎回見る(済んだジョブが続くと、ジョブごとに位置の分だけ見るので二乗になる)。`crates/core/src/flow/next_step.rs` の 128 行も `definition.job_index(&job.id)` を呼ぶ。`plan/simulate.rs` は `decide_from_state` と `classify_and_decide` をジョブ ID だけで呼ぶので、位置を渡せない。手順 2 の「繰り返しの位置から渡す」は、この呼び出しの形では成り立たない。手順 3 の逃げ道(plan の中に位置つきの引き方を置いて呼ぶ側を差し替える)は、`decide_before_run` が `decide.rs` の非公開の関数で、規則 1〜7 を持つため、plan に判断の規則を丸ごと複製することになる。`simulate.rs` の冒頭にある「実行と同じ判断を使う(BR1.2)」と衝突し、本番と模擬の判断が別々に変わりうる。 | W3 と BR3.4 を直す。(a) 遅さの元が `flow/` の中にあるときの扱いを決める。たとえば、位置を引数に取る内部の入口を `flow` に足して、本番と模擬の両方が同じ判断の本体を呼ぶ形にする(本番の動きは変えず、`decide.rs` と `next_step.rs` の共有の変更を認める)。または、「plan の中だけ」を守れず直さないと決めて BR3.6 の記録にする。(b) 判断の規則を plan に複製しない、と書く。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W4 のテスト(BR5.5) | 「子の標準入力に改行を与える」は、今のテストの補助と合わない。`crates/cli/tests/bin_support/mod.rs` の `run_limited`(86 行)は、呼び出し側の指定にかかわらず `command.stdin(Stdio::null())` で上書きする。`unix_demo.rs` の `run_demo` もこれを通る。また `run-demo.sh` は `set -e` を使わない(`set -u` のみ)ので、標準入力が空でも `read -r _` は EOF ですぐ戻り、止まらない。つまり改行は要らず、書かれた方法はそのままでは実現できない。 | 「改行を与える」を、「`DEMO_PAUSE=1` を環境に足し、標準入力は `run_limited` の既定(空)のままにする。`read` は EOF ですぐ戻る」に直す。改行が要ると考えるなら、`run_limited` を変えずに別の起動の補助を使う、と書く。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の手順 1 と図 | 測定の表の置き場所が食い違う。W3 の手順 1 は「`code-summary.md`」に残すとするが、FR3.3 と `rules.md` の BR3.6 は、直さないときの測定と理由を `build-and-test-summary.md` に書くとする。図の「いいえ」の枝は置き場所を言わない。 | 置き場所を 1 つに決める(測定は `code-summary.md`、直さないときの判定の理由は `build-and-test-summary.md` に、など)。BR3.6 と W3 の文を揃える。 | New |
| R-04 | Minor | aidlc/spaces/default/intents/261126-v0101-followups/construction/functional-design/functional-spec.md > W3 の図(F から V への繰り返し) | 「直す → 確かめる(不合格なら直す)」の繰り返しに、抜ける条件が無い。75ms と 2.5 倍に届かないとき、無限に直し続けることになる。 | 抜ける条件を足す(たとえば、直しても届かないときは、直しを戻して BR3.6 の記録にする。または人に確かめる)。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| `grep -rn '33 種' docs/spec/rust/ crates/` | 12 件(00:23、02:103・297、03:523、10:120、11:213、12:22・24・105・321、`env_types.rs`:56、`report_messages.rs`:1225) | W6 の一覧と完全に一致する。105 行だけが残る、という確かめも成り立つ。`DefinitionErrorKind` は 34 変種(`error.rs` を数えた)で、「34 種」も正しい。 |
| W1 の手追い(`safe_log_stem`、`log.rs` 80〜105 行) | 要件の FR2.1 の表の 7 行すべてと一致 | `con ` は `_con_`、` con` は `_ con`、` con ` は `_ con_`、`con .x` は `_con .x`、`nul .tar.gz` は `_nul .tar.gz`。手順 4 の「比べるときだけ空白を除く」は、現在の 88〜94 行への 1 行の変更で書ける。 |
| W2 の実現性(`runner.rs` の `copy_stream` と `mod tests`) | 実現できる | `copy_stream` はモジュールの中の非公開の関数で、`mod tests` が `Shared::new`、`FailAfter`、`Chunks` を既に使っている。読み取り回数を数える読み手はテストの中に足せ、本番に引数や `pub` は要らない。 |
| W4 の待ちの位置(`run-demo.sh` 94〜96 行) | 妥当 | 説明の 2 行(94〜95 行)と `watch --follow`(96 行)の間に `pause` を足せる。`pause` は既存の関数。UNIX 版のスクリプトに `set -e` は無い。 |

### Summary

W1・W2・W4 の位置・W6 の一覧は、ソースに照らして成り立つ。主な懸念は W3 で、「plan の中だけを直す」と決めた制約が、遅さの元になりうる `flow/decide.rs` と `flow/next_step.rs` の線形の探索を直せず、手順 3 の逃げ道は判断の規則の複製になる。残りの指摘(W4 のテストの標準入力、測定の置き場所、繰り返しの抜け方)は小さい。Major は 1 件で、READY の基準の範囲内だが、W3 は Code Generation の前に直すことを勧める。
