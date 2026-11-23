## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | functional-spec.md:248(§7 の完了判定、3a の検索) | 3a の完了判定は `grep -rn "RunIdError::" crates docs/spec/rust` が 0 件であること。しかし仕様書 rust 02:99 の表と 10:125 の行は、変種名(`Empty`・`TooLong`・`InvalidChar`・`LeadingDot`)を `RunIdError::` の形で書いていない。直し漏れがあっても、この検索では見つからない。`run_id.rs` の `RunId::parse` の説明に変種名が残っても同じ。 | 検索に `grep -rn "TooLong\|LeadingDot\|InvalidChar" crates/core docs/spec/rust` を足す。残ってよいのは、05 の 6.1 の条件の説明だけ(language-agnostic は D4 で対象外)。 | New |
| R-02 | Minor | functional-spec.md:152(T8)、:204-207(§4.6) | T8 で `extract.effective_success_codes() == [0]` を消すと、定義ファイルから読んだ定義の既定の成功コードを、値で確かめる行がなくなる。残るのは `:51` の `success_exit_codes == None`、`types.rs:524` の単体テスト(組み立てた定義で `[0]`)、`classify.rs` の分類。「None を [0] とする」ことは確かめられるので、動きの確かめとして足りる。ただし、承認の表に「ファイルから読んだ定義で値を比べる確かめが 1 行減る」とは書いていない。 | T8 の「変える内容」に、減るのは「ファイルから読む経路と `[0]` を結ぶ確かめ」であり、2 つの確かめの合わせで保つことを 1 文足す(承認のユーザーが判断できるように)。 | New |
| R-03 | Minor | functional-spec.md:263-297(§8 のコミットの並び) | T20、T23、T25 は 2 つのコミットに分かれる(1a のコミット 1 と、3b のコミット 20。T25 は 2a のコミット 10 と 20)。同じファイルの `use` の塊を 2 回直すので、後のコミットで、前のコミットが書いた import を取り違えると、そのコミットだけビルドが止まる。実物で見ると、`flow_with_state.rs:33-39` の import は 10 と 20 の両方が触る。 | コミットの計画に、同じファイルを複数のコミットが触る箇所(T20、T23、T25)で、各コミットの終わりの import がどうなるかを 1 行ずつ書く。または、実装でコミットごとに `cargo test --workspace --no-run` を通す手順を §7 に足す。 | New |
| R-04 | info | functional-spec.md:114-137(§4.0) | 検索を実際に走らせて、T1〜T32 と照らした。`From`(`.into()` と `impl Into<Command>` を含む。core の `common/mod.rs:62`、`graph_common/mod.rs:37`、`test_support.rs:24`、runtime の `runner_basic.rs:23`・`:173`、`common/mod.rs:36`・`:68`)、`RunIdArg::as_str`(`args_grammar.rs:495・507・514・702・795`、`run_id_arg.rs:25・32`)、`is_raised`(`runtime/types.rs`、`signals_real.rs:46`、`signals_order.rs:38`、`docs/spec/rust` 02:125・07:277)、`Default` 4 つ、`sanitize`(本番 2 か所と単体 6 か所)、`EnvFileError::reason`(`env_file_parse.rs:179`、`env_types.rs:270`、`envfile/mod.rs:169`)、`decide_before_run`、`classify`、`MAX_RUN_ID_LEN`・`is_valid_env_name`・`is_reserved_env_name`・`effective_success_codes`、cli のモジュールと再公開、`RunIdError`、runtime の再公開 7 つの使用(cli 本番 5 ファイル、runtime のテスト 4、cli のテスト 1)、種類の `Display`・`Error`・`Hash` の使用(`HashSet` は T10 と T26 の 2 か所だけ)は、すべて T 表か「変えずに通る」の一覧にある。漏れは見つからなかった。Windows でだけコンパイルされるテスト(`runner_basic.rs:292・332`)も T21 にある。 | 対応なし。 | New |
| R-05 | info | functional-spec.md:200-209(§4.6) | 消す 4 件を確かめた。(1) `文字列から_line_を作れる` は `From` だけを試す。残る `definition_valid.rs:76` と `nfr_security.rs:28` が、文字列の `command` が `Command::Line` で読まれることを確かめる。(2) `状態の重ね合わせの既定は空…` の前半は `Default` だけ、後半は公開の項目を読むだけ(型が保つ)。空の重ね合わせは `graph_overlay.rs` の 5 件が残る。(3) `耐久性の既定は_durable` は `Default` だけを試す。`state_write.rs` などが `Durable`・`Fast` を明示して確かめる。(4) `u1_core_の_sanitize_と同じ結果` は、`escape_display` の再公開と包みが同じ関数になるので、比べる 2 つが 1 つになる。表の確かめは残る。いずれも、もともと何も確かめていない(または型が保つ)ものを正直に書いている。 | 対応なし。 | New |
| R-06 | info | functional-spec.md:67-88、:229-259 | 利用者から見える動きは変わらない。(1) `RunIdError`: 本番の呼ぶ所(cli 2 か所の `map_err(\|_\| ..)`、runtime の `.ok()`)は種類を見ない。4 つの条件を 1 つの式にしても、結果は同じ(空、64 超、不正な文字、先頭の `.` のどれかなら `Err`)。(2) `InterruptSignal` の `Debug`: `raised` を `signal_number().is_some()` にしても出力は同じ。`PartialEq` を使う本番の型は無い(`ExecutionResult::NotStarted` は `i32`)。(3) 種類の `Display`: 本番の表示は `kind.label()` を `render_error` に渡す。converter の `ConversionErrorKind` は `Display` を残すので、`error.rs:107` の `{}` は変わらない。(4) `Hash`: 種類をキーにする `HashMap`・`HashSet` は、本番に無い。(5) `Default`: `derive(Default)` の構造体で `JobStatus`・`Durability`・`StateOverlay`・`Language` を持つものは、本番にもテストの補助にも無い。 | 対応なし。 | New |
| R-07 | info | functional-spec.md:55-65(§2.4)、:213-225、:268-299 | 本番にテスト用の要素を足していない。D5 の補助(`impl Into<String>` と `Command` を取る `run`)は `tests/` の中だけ。D6 の狭め方も、使う所を確かめた。`classify` は `next_step.rs` が `super::classify::classify` で使い、`decide_before_run` は `decide_from_state` だけが呼び、`is_valid_env_name` と `is_reserved_env_name` は core の `definition/walk` と `envfile/parse` だけが使う。件数も合う(core 384−2、runtime 198−1、cli 459−1、converter 変わらず。合計 1,157−4=1,153、1 無視)。新しい依存、`unsafe`、src の `cfg`、版の変更は、計画に無い。 | 対応なし。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| §4.0 の検索の再実行(grep) | 参照はすべて T1〜T32 か「変えずに通る」の一覧にある | 網羅は取れている。残るのは完了判定の検索の弱さ(上の minor 1 件)だけ |
| 件数の計算 | core 382、runtime 197、converter 116(+1 無視)、cli 458 で合計 1,153 | 基準 1,157 から §4.6 の 4 件を引いた値と一致 |
| cargo の実行(コンパイル、テスト) | 実行していない(設計の段で、ソースを変えない制約のため) | 設計は、写しの木でのコンパイル、clippy、テストの結果(`inventory.md` §0.2)を根拠にしている。実装後に §7 のとおり実行して確かめる |

### Summary

設計は、消す・狭める名前の参照を T1〜T32 で漏れなく押さえ、消す 4 件のテストは代わりが残ることか、もともと何も確かめていないことを正直に示している。利用者から見える動き、本番へのテスト用要素の追加、件数(1,153)、コミットの並びのどれにも、重大な問題は見つからなかった。残るのは、完了判定の検索の穴、T8 の承認の表の説明、複数のコミットが同じファイルを触る箇所の手順の 3 点で、いずれも小さな直しで済む。
