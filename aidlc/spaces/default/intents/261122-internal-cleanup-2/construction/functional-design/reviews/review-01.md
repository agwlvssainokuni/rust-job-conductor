## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-22
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261122-internal-cleanup-2/construction/functional-design/inventory.md > 2d(目標の段落「graph の text は `EdgeTarget` の `label()` をエスケープ」) | 実物では、遷移の表示(`report/mod.rs:417-423`)は `escape_name`(64 文字で切る)を使い、graph の text(`graph_text.rs:86-89` の `target_text`)は `escape_path`(切らない)を使う。目標の文は「エスケープ」とだけ書き、どちらか決めていない。graph の text を `escape_name` にすると、65 文字以上のジョブ ID で graph の出力が変わる(定義側にジョブ ID の長さの上限は見当たらない)。「表示は 1 文字も変わらない」に反しうる。 | 2d に「遷移の表示は `escape_name(target.label())`、graph の text は `escape_path(target.label())` のまま」と書き分ける。可能なら、65 文字以上のジョブ ID の graph の出力を確かめる既存または新しいテストを段 2 のコミット 18 に含めるか、確かめ方(§7)に手作業の確認を足す。 | New |
| R-02 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup-2/construction/functional-design/functional-spec.md > §5.2・§5.3(A3a の「直した後の見積もり」) | A3a の直した後の時間は測っておらず、A3b(項目は 2 倍)の見積もり 8.7 ms 以下という推定だけで、余裕「5.7 倍以上」を出している。上限 50 ms 自体は直す前の 353 ms を確実に落とし、直した後の推定との開きも大きいので、基準としては妥当。ただし表の数字が測定に見える。 | 表の A3a の欄に「推定(未測定)」と明記するか、実装の段階で直した後の A3a を測って記録する旨を §5.3 の「前後の記録」に足す。 | New |
| R-03 | Minor | aidlc/spaces/default/intents/261122-internal-cleanup-2/construction/functional-design/functional-spec.md > §4 T8 と inventory.md 1c・2i | 2i のあと、cli の `escape_display` は core の関数そのものになる。`escape_table.rs` は core と同じ関数を 2 回見る形になり、「3 つの実装を同じ表で揃える」役目は core と converter の表の揃え(converter の単体テスト側)に移る。T8 はコメントの直しだけで、`:17` の「core の `sanitize` は公開されているため、同じ入力に同じ結果を返すことを直接確かめる」という文(本文の説明)が 2i のあとも正しいかを書いていない。 | T8 の変更の中身に、`:17`・`:18` の文が 2i のあとにどうなるか(例: core の `escape_display` を直接確かめ、converter は単体テストの同じ表で揃える、と直す)を足す。テストの本体は変えなくてよい。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| grep(`DecisionReason` を crates と docs/spec/rust で) | 使うテストは `report_messages.rs`、`flow_sequences.rs`、`plan_run.rs` と、単体テストの `decide.rs`、`record.rs` | T1〜T6 の一覧と一致し、漏れなし。`plan_pipeline.rs` などの `PlanDisposition` は別の型で影響なし |
| grep(`PlanMode`・`EdgeKind`・`EdgeTarget` を tests と src で) | 派生(`Debug, Clone, Copy, PartialEq, Eq` と `Debug, Clone, PartialEq, Eq`)が元の型と同じで、`PartialOrd` や `Hash` を持たない。`pub use … as` で既存のテストはそのまま通る | 4c の別名の方針は妥当。変種の順番が違う(`PlanMode`)が、順序の派生が無いので影響なし |
| grep(`run_id` を `CliCommand` を使うテストで) | `args_grammar.rs:483, 489, 493, 700, 793, 621-624`、`resolve_run_id` の呼び出しはここだけ。`flow_run_id.rs:41-42`、`binary_run_id.rs:86` は `PipelineState.run_id`(別の型) | T7 の範囲と一致。`CliCommand::new` を使う `init_types.rs` は影響なし |
| grep(`DisplayEvent::Transition` を tests で) | `report_messages.rs:252, 257, 553` のみ(`:351` は `..` で `target` に触れない) | T1 と一致 |
| 実物の抜き取り(1b、1e、1h、2a、2c、2d、2g、3c の行) | `report/mod.rs:66, 123`、`exit.rs:23`、`types.rs:143`、`deps.rs:108`、`core/types.rs:313`、`intake.rs:128`、`runtime/error.rs:103`、`converter/error.rs:131-133`、`ctx.rs:136-345` が一覧の記述どおり。3c は `require_known` に空の並びで同じ順序と表示 | 「今の場所」は実物と合っている。目標の形で出力の文、終了コード、誤りの順が変わる項目は、上の Major(2d の escape)以外には見つからなかった |
| `nfr_source.rs`(cli、runtime)、`nfr_security.rs`(converter) | cli は `src` に語 `unsafe` を禁じる(`forbid(unsafe_code)` の行だけ除外)、runtime は `.unwrap()`・`panic!(`・`unreachable!(` などを製品のコードで禁じる、converter も同様 | 足す `const _: () = assert!(…)` は対象の語を含まない。1d の説明文に語 `unsafe` を書かなければ通る。`forbid(unsafe_code)` は core・converter の lib にだけ効き、統合テスト(別のクレート)の `#[cfg(unix)]` には影響しない。core・converter の `src` に `unsafe` は 0 件で、`set_var` なども無い |
| `cargo test` | 実行していない | 今回はファイルの変更が無い設計の確認で、基準の 1,128 件は intent の段階の記録を信頼した |

### Summary

設計は実装できる水準で、承認が要るテストの変更 T1〜T9 は、grep で確かめた限り漏れが無く、組み立ての値と import だけに収まる(T6 の 1 行の削除は到達しない分岐の確認で、明記済み)。制約(依存、unsafe、cfg、入力の値を含めない、converter は core に依存しない)と範囲の外には踏み込んでいない。性能の基準(50 ms、50 ms、100 ms)は、直す前の 5.6〜21.7 倍で確実に落ち、直した後の 5.7 倍以上の余裕があり妥当。主な懸念は 2d の graph の text で `escape_name` と `escape_path` を取り違えると、長いジョブ ID の出力が変わりうること(Major 1 件)で、書き分けを足せば READY のまま進められる。
