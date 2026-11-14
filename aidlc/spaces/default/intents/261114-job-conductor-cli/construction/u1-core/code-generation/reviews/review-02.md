## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14
**Iteration:** 2
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | crates/u1-core/src/flow/resume.rs:17-35 and crates/u1-core/src/flow/next_step.rs (`Some(TransitionTarget::End)` branch) | 個別実行で `@end` に着くと、`NextStep.skipped_job_ids` に以降の全ジョブが入る(BR3.3)。呼び出し側がそれらを `byEnd` として記録すると、`resume_point_on_interrupt` は pending も error も見つけられず、次のジョブ(例: 4 ジョブ中の b)を呼んだ直後に `completed = true` を返す。BR3.6 は、個別実行では最後のジョブがスキップされた時点で初めて完了とすると定めているので、c と d がまだ呼ばれていないのに完了となる。ステートが初期化されると、c は新しい実行として run になり、AC4.2.3(3 つ目は実行されずスキップされる)が満たせない。`@end` の後に、ジョブ間で中断した場合の `resume_point_on_interrupt` も、同じく早く完了を返す。code-summary はこれを「設計の隙間」として u3-cli に送っているが、完了の判定そのものがこのユニットの関数にあり、誤った結果を返している。 | 個別実行の `@end` 後の完了判定を、このユニット内で成立させる。たとえば、完了は「記述順の最後のジョブが、実行またはスキップとして記録済み」のときだけとし、`byEnd` で先に記録された途中のジョブでは完了にしない。または、個別実行では `skipped_job_ids` を返さず、各ジョブが呼ばれた時点でスキップを記録する形に BR3.3 / BR3.6 の扱いを決める。どちらにするかを契約 C2 に明記し、`resume_point_on_interrupt` の説明と合わせる。 | Resolved (reviewer) |
| R-02 | Major | crates/u1-core/tests/flow_sequences.rs:176-202 (テスト `個別実行_end_のあとはスケジューラの呼び出しでスキップされ最後で完了する`)、traceability.json の AC4.2.2 / AC4.2.3 / BR3.6 | このテストは、`Sim::apply_step` が `skipped_job_ids` をすべて `byEnd` で記録するため、b を呼んだ時点で `completed` が true になる。しかしテストは `d` の後でしか `completed` を確かめない(b・c の後に `!completed` を確かめない)。そのため、上の不具合を検出できず、コメントで認めているだけになっている。traceability.json は AC4.2.2 / AC4.2.3 / BR3.6 を `OK` とするが、AC4.2.3 の期待(c が実行されない)を確かめるテストはない。 | b と c を呼んだ後に `assert!(!sim.completed)` を足し、実行を再現する流れで失敗するテストにしてから、上の不具合を直す。直すまでは、AC4.2.3 と BR3.6 の `OK` を部分達成として記録する。 | Resolved (reviewer) |
| R-03 | Major | Cargo.toml:6 (`rust-version = "1.85"`)、crates/u1-core/src/definition/walk.rs:391-393、crates/u1-core/tests/nfr_performance.rs:60-65 | `if let ... && let ...`(let チェーン)を使っているが、これは edition 2024 で Rust 1.88 から使える機能である。宣言した最低版 1.85 の道具では、コンパイルできない。code-summary も計画も、`rust-version` を「edition 2024 を使える最低」としていて、実際の最低版と合っていない。CI や新しい開発環境では気づかれず、版を固定した環境でだけ分かりにくい失敗になる。 | `rust-version` を 1.88 以上に上げる(toml 1.2.1 などの依存の最低版も確認する)。または、let チェーンをネストした `if let` に書き換える。選んだ最低版を CI の確認にも反映する。 | Resolved (reviewer) |
| R-04 | Minor | crates/u1-core/src/definition/intake.rs:61-79 | 「上限 + 1 バイトで読む」分岐(確認後にファイルが大きくなった場合)は、実装されているが、テストは `metadata` の大きさの検査だけを通る。NFR4.4 の要となる `take(MAX + 1)` の経路は、どのテストでも動かない。 | 読み取りの部分を、`Read` を受け取る小さな関数に切り出し、上限 + 1 を超えて読まないことを、模擬の読み取り元で確かめる。 | Resolved (reviewer) |
| R-05 | Minor | crates/u1-core/src/error.rs:150-164 (`escape_char`) | `char::is_control` は、Unicode の書式制御文字(U+202E の右から左への上書き、U+200B のゼロ幅など。種別 Cf)をエスケープしない。キー名やファイル名にこれらが入ると、端末の表示で、見えない文字や並びを変える文字がそのまま出る(NFR4.6 の制御文字エスケープの趣旨)。 | `is_control` に加えて、書式制御文字と行・段落の区切り(U+2028、U+2029)もエスケープし、テストを足す。 | Resolved (reviewer) |
| R-06 | Minor | crates/u1-core/src/flow/classify.rs:11-33、crates/u1-core/src/types.rs(`JobExecution.signal_number`) | `signal_number` が `None` の中断は 128 になり、負の値や大きな値は `128 + n` がそのまま返る(`i32` の溢れは `saturating_add` で防いでいる)。仕様に範囲がなく、終了コードの 0〜255 の範囲を外れる値が、そのままスケジューラへの終了コードになりうる。code-summary は `None` の場合だけを記録している。 | 受け付ける範囲(正の整数、または必須)を契約に書くか、範囲外は決めた値に丸める。範囲外の入力をテストに加える。 | Resolved (reviewer) |
| R-07 | Minor | crates/u1-core/src/flow/next_step.rs(`pending_ids`)、rules.md の BR3.2 / BR3.3 | `skipped_job_ids` は、記録済みの成功・警告・スキップのジョブを除いている。BR3.2 / BR3.3 は「間の全ジョブ」「以降の全ジョブ」と書いているため、規則の文面と異なる(コードの意図は妥当だが、code-summary の「設計との差」に載っていない)。 | 差として code-summary に載せ、規則の文面にも反映する(または文面どおりにする)。 | Resolved (reviewer) |
| R-08 | Minor | crates/u1-core/src/definition/mod.rs:42-58 (`parse_definition`)、tests/nfr_security.rs:155-160 | 入れ子の深さが 33 以上 1000 以下の入力を `parse_definition` に通すと `syntax` になることは、単体テスト(depth.rs)でだけ確かめている。`nfr4_7` は「受理されない」ことだけを確かめ、深さ 1000 で `syntax` が返る(security-design の判定基準)ことまでは確かめない。 | `parse_definition` に深さ 33 と 1000 を渡して、種類が `syntax` であることを確かめる検査を足す。 | New |
| R-09 | Minor | crates/u1-core/src/flow/resume.rs:35 と crates/u1-core/src/flow/mod.rs:37-41 | `resume_point_on_interrupt` の `end_reached` は、`skipped(byEnd)` の記録だけから決まる(契約 C2 の入力は定義とジョブの状態だけで、永続化された `end_reached` を受け取らない)。個別実行で `@end` に着いたあと、最初のスキップを記録する前に中断すると、`end_reached = false` を返す。Cli が戻り値でステートの `end_reached` を上書きすると、引き継ぎが失われ、後ろのジョブが `AfterEnd` ではなく通常の実行になる。テストの模型(flow_sequences.rs:113、120)は `\|=` で積み上げていて、この上書きの場合を検証していない。 | resume.rs の説明と code-summary に、「Cli は `end_reached` を上書きせず、一度 true になったら true を保つ(積み上げる)」と明記する。u3-cli の実装時にこの点を確かめる。 | New |

> R-01 Reviewer note: next_step.rs:110-112 は、個別実行の `@end` で `skipped_job_ids` を空、`completed = pending.is_empty()` にする。後ろに未処理のジョブがあれば完了しない。呼び出し時は decide.rs:45-47 が `Skip(AfterEnd)` を返し、resume.rs:22-36 は先頭の pending/error を再開位置にして、最後のジョブが記録されたときだけ `completed = true` にする。一連の実行は BR3.3 のまま一括スキップと完了。BR3.6 の個別実行の規則と一致する。

> R-02 Reviewer note: flow_sequences.rs:176-208 は、`@end` 直後に `end_reached && !completed` と後ろのジョブが pending のままであることを検証する。続けて b、c を `Skip(AfterEnd)` で呼び、その都度まだ完了しないこと、最後の d で `completed` になり resume が空になることを確かめる。next_step.rs:253-267 は両方の方式を比べる。以前の誤りを検出できるテストになった。traceability の AC3.1.3、AC4.2.3、BR3.6 の OK の対象は、この流れで裏付けられる。

> R-03 Reviewer note: `&& let` や `\|\| let` の let チェーンは crates 配下に残っていない。`rust-version` 1.85 の宣言と書き方が合っている。1.85 の実機コンパイルは未実施(rustc 1.97 で確認)なので、CI の MSRV ジョブで確かめるとよい。

> R-04 Reviewer note: intake.rs:98-107 の `read_bounded` に分けられ、intake.rs:144-164 が上限ちょうど、上限 + 1、終わりのない入力、空の入力をテストしている。

> R-05 Reviewer note: error.rs:165-170 の `escape_char` が `is_control()` に加えて書式・双方向制御文字も扱い、対応するテスト(書式文字と双方向制御文字)が通る。

> R-06 Reviewer note: `signal_number = None` は 128 とし、挙動を code-summary に記録した。classify.rs:251 が `i32::MAX` の範囲のテストを持つ。

> R-07 Reviewer note: code-summary.md:31 に、`skipped_job_ids` が記録済みのジョブを除く差を記録した。深さ 32/33/1000 の `syntax` は `parse_definition` でも確かめている。

> R-08 Not re-checked this round

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| cargo test -p conductor-core | PASS(71、18、8、9、3、1、5 の各件が全部成功、失敗 0) | 個別実行の `@end` の流れのテストも通る。 |
| cargo clippy -p conductor-core --all-targets -- -D warnings | PASS | 警告なし。 |
| cargo fmt --check | PASS | 整形の差なし。 |

### Summary

前回の 7 件はすべて直っている。個別実行の `@end` は、`decide_before_run` の `Skip(AfterEnd)` と `resume_point_on_interrupt` の完了判定を組み合わせると、最後のジョブが記録された時点でだけ完了し、BR3.3、BR3.6、AC3.1.3、AC4.2.3 と整合する。残る指摘は、中断時の `end_reached` の引き継ぎの契約に関する Minor 1 件だけで、読み取った範囲では阻害要因はない。
