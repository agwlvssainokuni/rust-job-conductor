## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | crates/runtime/src/state/writer.rs:172-185 (serialize) | definition_key を型から消したため、書き込みは常に definition_path と同じ文字列になった。以前は読み込んだ definition_key をそのまま書き戻していた。definition_key と definition_path が違う手書きのステートを読んで書き直すと、書き直したあとのバイト列が変わる。本番の書き込みは initial_state が両方に同じ文字列を入れるので、通常の経路では差が出ない。仕様書(06 章)も「同じ文字列を書く」と定めている。 | 対応は必須ではない。code-summary.md か unit-test-instructions.md に、この端の差(手で違う値にしたステートを書き直すと揃う)を 1 行、既知の差として書いておくとよい。 | New |
| R-02 | info | crates/runtime/src/state/reader.rs:237 | definition_key は存在と文字列であることだけを検査し、値は捨てる。job_order は job_states との並びの一致と、再開位置が含まれることを引き続き検査している。照合は job_states のジョブID の並びで行うので、consistency.rs の比較は以前と同じ意味になる。読み込みの検査は落ちていない。 | なし。 | New |
| R-03 | info | crates/cli/src/platform/mod.rs:33-36, crates/cli/src/platform/windows.rs:27-29 | 登録済みの印(REGISTERED)と AlreadyRegistered を外した。本番の呼び出しは Ctx の prepare_session の 1 か所だけで、1 コマンドにつき 1 回なので、本番の動きは変わらない。Windows で ctrlc の MultipleHandlers は Failed(70)になるが、2 回目の登録は契約の外と、コードと仕様書の両方に書いてある。 | なし。 | New |
| R-04 | info | crates/cli/src/panic_hook.rs:31-48 | フックの本体は write_internal_error(非公開)で、固定の英語の文を書いて 70 を返す。install_panic_hook は std::process::exit(本体の戻り値)を呼ぶだけで、本番の動きは変わらない。単体テストは src の #[cfg(test)] の中にあり、nfr_source.rs の検査(forbid(unsafe_code)、依存、cfg)と矛盾しない。cargo test --workspace --locked は全て ok。 | なし。 | New |
| R-05 | info | crates/cli/tests/common/kinds.rs, crates/core/tests/common/kinds.rs | 一覧の定数、_ を使わない網羅の match、i 番目の位置が i であることのコンパイル時の assert が揃っている。種類が増えると match が止まり、並びの誤りや重複もコンパイルで止まる。本番の ALL と code() は core・converter・runtime・cli のすべてから消えている。 | なし。 | New |
| R-06 | info | crates/*/src(#[cfg(test)] より前) | 新しい #[doc(hidden)]、テスト専用の pub、テストのための引数は足していない。差分の追加行に unsafe、cfg、unwrap、expect は無い(unwrap 等は tests と #[cfg(test)] の中だけ)。Reporter の language_resolved と color_resolved は消え、FakeEnv の color_read(tests/common/mod.rs)に置き換わった。decide_from_state は本番の run・pipeline・plan が使う pub で、テスト専用ではない。 | なし。 | New |
| R-07 | info | crates/cli/src/step.rs, crates/core/src/flow/decide.rs | step::decide の本体は、ステートが無いときは空の状態の一覧、再開位置なし、end_reached 偽、という扱いも含めて decide_from_state に同じ形で移った。pipeline、run、plan_run、plan_pipeline の呼び出し側もこの形になっている。plan_run の simulate_job に渡す状態の一覧は、以前と同じ値になっている。 | なし。 | New |
| R-08 | info | crates/core/src/flow/classify.rs:79-100, crates/runtime/src/error.rs:32-48 | ExitCodeClass を消して classify_exit_code が JobOutcome を返す。分類、終了コード、conductor_error の組み合わせは以前と同じ。CommandErrorKind::reason() は以前の文と同じ 2 つの固定の文で、CommandError の表示も同じ。 | なし。 | New |
| R-09 | info | crates/runtime/src/state/reader.rs:109-111 | inspect_run_state の分岐が Ok(None)=NotOurs、Ok(Some)=Ours になった。parse_state_raw_for が Ok(None) を返すのは bytes が空のときだけなので、以前の分岐と同じ結果になる。 | なし。 | New |
| R-10 | info | crates/*/tests | 既存のテストの変更は、型から消した 2 つのフィールドの削除、ALL の置き換え、削除 5 件(使い方の誤りの全種類、すでに登録済みでも続ける、設定したフック、core の src の全種類と識別子、converter の src の表示)、panic_hook の 2 件の src への移動、名前の変更 3 件(最初のシグナルの番号だけが残る、定義の誤りの種類は 33 種で表示が空でない、env_file の誤りの種類は 13 種で理由が重複しない)、flow_with_state.rs:140 のコメントに収まっている。期待の文字列の変更は見つからない。 | なし。 | New |
| R-11 | info | docs/spec/rust, docs/spec/language-agnostic | rust と language-agnostic の本文に「すでに登録済み」は残っていない(crates と README も同様)。AC-299 は ID を残して「該当なし」にした。AC-285、AC-348、AC-368 は実装と合っている。ただし docs/spec/go, csharp, powershell, python, typescript は凍結で差分が無く、go の仕様書(02、07、10 章)には AlreadyRegistered の記述が残る。language-agnostic と食い違うが、凍結の扱いなので意図どおり。 | なし。凍結した仕様書を次に直すときの申し送りでよい。 | New |
| R-12 | info | Cargo.toml, Cargo.lock, aidlc/.../code-generation/traceability.json | Cargo.toml と Cargo.lock の差分は無い。版は 0.9.8 のまま。converter は core に依存しない。cargo clippy --workspace --all-targets --locked -D warnings と cargo fmt --check は通る。traceability.json の 28 件の ID はすべて functional-design の中にあり、target のファイルはすべて実在する。 | なし。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| cargo test --workspace --locked | PASS(全クレート、失敗 0) | 本番の動きの変更が無いことを裏づける |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS | 警告なし |
| cargo fmt --all -- --check | PASS | 差分なし |
| traceability.json の確認(ID と target) | PASS | 欠けた target なし、設計に無い ID なし |

### Summary

利用者から見える動き(ステートのバイト列、書き込みと読み込みの検査、終了コード、表示の文)は、変わっていない。例外は、definition_key と definition_path が違う手書きのステートを書き直すと揃うという端の差だけで、minor にとどめた。本番のコードからテストのためだけの要素が消え、新しく足されたものも無く、変更の範囲は T1〜T31 と削除 5 件に収まっている。
