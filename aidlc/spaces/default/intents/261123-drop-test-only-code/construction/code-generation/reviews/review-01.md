## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | info | crates/core/src/run_id.rs:parse | RunIdError を単位構造体にしても、成功と失敗の判定条件は元の 4 つの早期 return と同じ(空、64 バイト超、許可外の文字、先頭が `.`)。cli は 値を捨てる `map_err` で InvalidValue で値を使わないため、`invalid value`(68)の表示と終了コードは変わらない。 | 対応は不要。 | New |
| R-02 | info | crates/runtime/src/types.rs:InterruptSignal の Debug | `raised` を `signal_number().is_some()` で出す。元の `is_raised` は state が NOT_RAISED でないことを調べていた。`signal_number()` も同じ値を見るので、Debug の出力は同じ。 | 対応は不要。 | New |
| R-03 | info | crates/core/src/error.rs:render_error、quote_key | `sanitize(x, n)` を `escape_display(x, Some(n))` に置き換えた。長さは MAX_FILE_DISPLAY_CHARS(256)と MAX_KEY_DISPLAY_CHARS(64)のままで、切り詰めの長さは同じ。 | 対応は不要。 | New |
| R-04 | info | crates/cli/src/report/escape.rs と crates/core、crates/converter(Display・Hash・Error を消した種類) | DefinitionErrorKind、EnvFileErrorKind、ConversionErrorKind から Display、Hash、Error を消し、TargetFormat から Hash を消した。本番で使う所は無く、`cargo test --workspace --locked` は全て通る(1153 件が成功、失敗 0)。`cargo clippy --workspace --all-targets --locked -- -D warnings` も通る。 | 対応は不要。 | New |
| R-05 | info | git diff 23b8975..HEAD(crates の src) | 追加行に `unsafe`、`cfg(`、`#[doc(hidden)]`、`#[allow(`、`unwrap()` は無い。テストだけが使う pub やテストのための引数も足していない。Cargo.toml、Cargo.lock、docs/spec/language-agnostic に差分は無く、版は 0.9.8 のまま。 | 対応は不要。 | New |
| R-06 | info | crates/core/src/flow/mod.rs、crates/cli/src/lib.rs の狭め方(D6) | `decide_before_run` は decide.rs の中だけの非公開関数、`classify` は `pub(super)` にした。is_valid_env_name、is_reserved_env_name、effective_success_codes は `pub(crate)`、MAX_RUN_ID_LEN は run_id.rs の中だけにした。いずれも crate の中の使い方と合い、ビルドが通る。cli の `pub mod report` と `pub mod run_id` は、テストと main が使うため残してある。これは範囲の外で、方針に反しない。 | 対応は不要。 | New |
| R-07 | info | crates/cli/tests、crates/core/tests、crates/runtime/tests | 既存のテストの変更は、`.into()` を `Command::Line(..)` に、`as_str` を値の比較に、`decide_before_run` を `decide_from_state` にする置き換えが中心。期待の文字列とテスト名は変えていない。消したのは、StateOverlay の Default のテスト、`u1_core_の_sanitize_と同じ結果`、Command の From のテスト、Durability の Default のテスト、`is_raised`・`PartialEq`・`reason()` の 1〜2 行。いずれも §4.6 の 4 件と T 表の範囲に収まる。置き換えの `Sim::decide` と `signal_number()` による確かめが、同じ動きを残している。 | 対応は不要。 | New |
| R-08 | Minor | docs/spec/rust/02-architecture.md:42 | 再公開の説明に「v0.9.6 までは」「v0.9.7 までは」という版の注記が残る。方針は「版を上げないので、版の注記は足さない(既存の注記は残す)」なので、既存の注記を残すのは方針どおり。ただし、今回の変更の分(例: `decide_before_run`、`sanitize`)と、それ以前の変更の注記が同じ行に混ざり、読み分けにくい。 | 任意。次の仕様書の整理のときに、今の形だけを書く文に整える。 | New |
| R-09 | info | docs/spec/rust(02、05、10、12) | `RunIdError` は単位構造体、`decide_before_run` は `flow/decide.rs` の中だけ、`decide_from_state` が公開、という記述が実装と合う。02 の 3.2 のシーケンスは `core::decide_from_state(中で decide_before_run)` と書かれ、実装と合う。`TooLong`、`LeadingDot`、`is_raised` の古い記述は残っていない。 | 対応は不要。 | New |
| R-10 | info | aidlc/spaces/default/intents/261123-drop-test-only-code/construction/code-generation/traceability.json | coverage の ID(SM1〜SM9、BR1.1〜BR1.6、BR2.1〜BR2.6、BR3.1、BR4.1、BR5.1〜BR5.3、BR8.1、BR9.1〜BR9.6)は、functional-spec.md と rules.md に実在する。target のパスはすべて実在し、MISSING は無い。 | 対応は不要。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| cargo test --workspace --locked | PASS(1153 件が成功、失敗 0) | 利用者から見える動きを守るテストが全て通る。 |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | 警告なし。 |
| git diff 23b8975..HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml docs/spec/language-agnostic | 差分なし | 依存と凍結した仕様書は変わらない。 |

### Summary

方針の「テストのコードの都合で本番のコードを曲げない」に沿った変更です。本番のコードから消す・狭める名前は実際に消え、狭まり、テストのための要素は新しく足されていません。利用者から見える動き(誤りの表示、終了コード、InterruptSignal の Debug、切り詰めの長さ)は、コードの読みとテストの全通過の両方で変わっていないと確認できました。テストの変更は T 表と §4.6 の範囲に収まり、仕様書と traceability.json も実装と合っています。指摘は仕様書の版の注記に関する minor 1 件だけで、ブロックしません。
