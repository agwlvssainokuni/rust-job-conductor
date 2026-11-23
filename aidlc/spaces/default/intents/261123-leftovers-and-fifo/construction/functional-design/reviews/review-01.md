## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261123-leftovers-and-fifo/construction/functional-design/functional-spec.md:143(§4.5)、:10(§1)、:219(§7)、inventory.md 2b | 既存のテスト `crates/runtime/tests/state_read.rs:114-121` の `終わりのない入力でも上限で止まる` は、ステートのパスを `/dev/zero` へのシンボリックリンクにして `StateErrorKind::TooLarge` を期待する。2b の形 B は、パスのメタデータで「通常のファイルでない」ものを開かずに `unreadable` にするため、`/dev/zero`(文字デバイス)は `TooLarge` ではなく `Unreadable` になり、このテストは落ちる。§4.5 と 2b の「確かめ方」は、このテストを「そのまま通る」ものに数えており、T 表にも ★ にも無い。§4.0 の検索に特殊ファイル(`/dev/`)を探すものが無く、見落としが起きた | ★ T23 として、このテストの変更を T 表に足す(期待を `Unreadable` に変える、または「終わりのない読み取りが上限 + 1 バイトで止まる」ことを `bounded_read` の単体テスト(`Endless` のような読み手)に移し、このテストは名前付きパイプ側の N5 と重ならない形に直す)。§1 の「例外は ★ だけ」の列挙、§4.5 の「期待の変わるテストが無い」、2b の「変わらないこと(1 MiB 超は `tooLarge`)」の書き方(通常のファイルに限る)、仕様書 06 の §5.1 を合わせる。ユーザーには、設計の承認で ★ として決めてもらう | New |
| R-02 | Minor | functional-spec.md:25(1f)、:239・:245(§8 コミット 13・19)、inventory.md 1c-2・1f | 1f は `read_limited` を非公開にするが、ステートの読み込み(`reader.rs` の `read_bounded` が `read_limited` を呼ぶ)が `bounded_read::read_regular_file` に移るのは段 2 のコミット 40。コミット 13(`read_limited` を `bounded_read.rs` に移す)とコミット 19(非公開に)の間と、そのあとコミット 40 まで、`read_bounded` が `read_limited` を使い続けるので、コミット 19 で非公開にするとビルドが通らない | `read_limited` の非公開化(1f)は、コミット 40(2b)に移す。それまでは `pub(crate)` のままとし、§8 と 1f の表の行を直す。1c-2 の完了の判定(`read_to_end` が 1 件)は、そのままでよい | New |
| R-03 | Minor | functional-spec.md:152-155(§4.6 の 3 行目) | `yaml_read.rs` の `とても深い入力は_異常終了せずに失敗する` を「同じ YAML の 5 つの形」を持つ統合テストで置き換えられる、とあるが、入力は同じではない。単体テストは閉じない流れのマップ(`a: {a: {a: …`)と、末尾の無い `- - - …` を渡し、統合テストは閉じたマップ(`{a: … 1 }`)と `x` つきの配列を渡す。実際には深さの誤りが入力の終わりより前に出るので同じ結果になるはずだが、「同じ」とは言い切れない。単体テストは `is_err()` だけ、統合テストは `syntax` まで確かめるので、強さは統合テストのほうが上 | 表の文を「ほぼ同じ形(閉じない入力は統合テストに無いが、深さの誤りが先に出るので同じ結果)」に直す。承認する人が、閉じない入力を残したいかを選べるようにする。1 行目の「行の番号」が、境界の 33 では確かめられなくなる(深さ 40 の配列だけ)ことも、同じ欄に書く | New |
| R-04 | Minor | functional-spec.md:137-138(T20・T21)、:132(T15)、inventory.md 1i-2b | §4.0 の検索(`run_cli_with` と `Deps {`)は、`crates/cli/tests/nfr_robust.rs:136-142` にも `Deps` を組んで `run_cli_with` を呼ぶ形を見つけるが、T 表にも「残す」の理由にも無い。3 か所を `run_with_fakes` に寄せるのに、同じ形の 4 か所目だけが黙って残る | `nfr_robust.rs` の呼び出しを T として足して `run_with_fakes` に寄せるか、「ランダムな引数の `ScriptedRuntime` を使わない形なので意図して残す」と §3 の範囲から外したものに書く | New |
| R-05 | Minor | functional-spec.md:184-189(§5.2)、:168-169(N1・N2) | P1 は見積もり 72 ms に対して上限 350 ms(4.9 倍)、P2 は 324 ms に対して 1,600 ms(4.9 倍)。前回の `env_parse_perf.rs` と同じ 5 倍の考え方で、手元の M2 が基準だが、CI の `coverage`(計測つき、debug、3 OS、ほかのテストが並行)は、手元の数倍遅くなりうる。P1・P2 の余裕は、前回の実績(CI で落ちなかった)に頼っており、この設計では確かめていない。N1 の「末尾に `k0 = "x"` を足す」は、定義の末尾が `[[job]]` の表なら `[pipeline.env]` に入らない(`unknownKey` になる)ので、足す場所の指定が足りない。また、時間を測るのが最初の 1 回の読みだけか、足したあとの読みも含むかが書かれていない | 実装のときに、CI の `coverage` ジョブでの実測値(ubuntu・macos・windows)を code-summary に記録し、余裕が 3 倍を切るようなら上限を上げる、と §5.2 に一文足す。N1 は「`[pipeline.env]` の最後に足す(`[pipeline.env]` を定義の最後の表にする)」と、「時間を測るのは最初の読みだけ」を書く | New |
| R-06 | Minor | functional-spec.md:172(N5)、inventory.md 2b | 2b は、権限のないファイルが「大きさにかかわらず `unreadable`」であることを、パスのメタデータ → 開く → 開いたハンドルの大きさ、の順で保つ。converter には 1 MiB 超の権限なしの単体テストがあるが、ステートには小さいファイルの `権限のないファイルは_unreadable`(`state_read.rs:226`)しかない。順が崩れる(大きさを先に見る)と、前回の `.env` と同じく `tooLarge` に変わる回帰が起きうるのに、検知するテストが無い | N5 に、権限 000 で 1 MiB を超えるステートが `unreadable` になる場合を足す(`#[cfg(unix)]`、権限を戻してから終わる)。または 2b の「確かめ方」に、これが回帰の守りになっていないことを書く | New |
| R-07 | info | functional-spec.md:20、inventory.md 1a-1・§7 の版の検査 | 版の検査 `grep -rn "0.9.7" crates Cargo.lock` が 0 件、という合格条件がある。1a-1 の目標文の括弧「v0.9.7 で `job` を外した」を、そのまま `lib.rs` のコメントに書くと、この検査に 1 件当たる(今の `crates` の `0.9.7` は `Cargo.toml` と `binary_basic.rs` だけ) | 1a-1 のコメントには版の数字を書かず「`job` は外れた」とだけ書く。または検査を「コメントの注記を除く」に直す | New |
| R-08 | info | functional-spec.md:109(T6) | T6 の行番号 `consistency.rs:57`、`resume.rs:44`、`decide.rs:79` は、定義を手組みしている箇所(`consistency.rs:83`、`resume.rs:69`、`decide.rs:104`)と合わない。`mod tests` の先頭を指しているように見える。箇所の数(8)と対象のファイル(7)は、`grep "warning_exit_codes:"` の結果と合っている | 実装のときに、行を手組みの箇所に直す(承認に影響しない) | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| §4.0 の検索(`EffectiveEnv`・`.names()`・`e3.job_id`) | 参照は `crates/core/tests/env_merge.rs` の 85・150・167・169・170 だけ | T1a・T1b と一致。漏れなし |
| 検索(`apply_interrupt_in`、`escape_path`、`classify_exit_code`、`interrupted_signal`、`command_line`) | 製品のコードの参照は spec と一致。`apply_interrupt_in` の仕様書は rust 05:172 だけ。`escape_path` の仕様書は rust 02・08・11(python は凍結) | 漏れなし。段 1 の挙動は変わらない(`interrupted_signal` は `result_detail` と同じ条件、模擬の分類は `classify` の手順 5 と同じ値) |
| 検索(`with_run_id`・`for_read`・`reset_hint`・`ConsistencyMismatch`) | 作る所は `ctx.rs`・`aux_commands.rs`、テストは `report_messages.rs:178` だけ | T18 と一致。`for_read` と `reset_hint` の置き換えは 1 対 1 で表示は同じ |
| 検索(`ConversionError {`・`reason: "…".into()`) | cli のテスト 4 か所(`flow_aux.rs:364`、`report_messages.rs:206`・`:213`・`:1080`) | T19 と一致。converter の `ConversionError::new` はすべて定数か `&'static str` を渡している |
| 検索(core の公開モジュールのパスの参照) | 外からの参照は `graph_types.rs:92` の 1 件 | T5 と一致 |
| 検索(`read_limited`・`read_state_file_raw`・`parse_state_for`・`parse_state`) | 参照は `reader.rs` の中だけ。`parse_state` の単体テストの呼び出しは 9 か所で、T4 のとおり補助だけを直せばよい | T4 と一致。ただしコミット 19 の `read_limited` の非公開化は、ステートが段 2 まで使うので、順が合わない(上の minor) |
| 検索(`/dev/`) | `state_read.rs:118` の `/dev/zero` のシンボリックリンクだけ | §4.0 に無かった検索。形 B で期待が変わるテストを見つけた(上の major) |
| 件数の計算 | core 115+268 → 113+271 = 384、runtime 68+127 → 69+128 = 197、converter 54+64 → 53+64 = 117(+1 無視)、cli 461 → 462、全体 1,157 → 1,160。`#[test]` の数は `read_resolve.rs` 29(→ 28)、`convert_ac.rs` 9(→ 10)、`state_read.rs` 14(→ 15)、`flow_validate.rs` 12(→ 13)、`definition_errors.rs` 28 | spec §6 と一致。★ T23 で期待を変えても件数は変わらない |
| 1h の挙動の同値(読んで確認) | env の表: `seen` に入れる条件と順は同じで、誤りの順序と内容は変わらない。遷移の重複: 今も 2 種類だけの比較で、真偽値 2 つと同値。`env_file_refs`: 書いた順を保つ | 誤りの順序と内容は変わらない |
| 1c-1 の挙動の同値(読んで確認) | `writer.rs` と `file_create.rs` の手順は同じ。誤りの種類の写し方は呼ぶ側に残る | 変わらない |
| 実行(`cargo test`・`clippy`・`llvm-cov`) | 実行していない(設計の段で、コードはまだ変わっていないため) | Code Generation の段で実行する |

### Summary

設計は、段 1 の挙動が変わらないこと、件数の計算、T 表の網羅(`EffectiveEnv`、`escape_path`、`StateContext`、`ConversionError`、公開範囲、テストの補助)が実物と合っており、承認に進める。ただし、2b の形 B は、既存のテスト `state_read.rs` の `終わりのない入力でも上限で止まる`(`/dev/zero`)の期待を `TooLarge` から `Unreadable` に変えるのに、T 表にも ★ にも無く、「期待の変わるテストは無い」という説明が誤っている。★ T23 として足し、ユーザーに承認してもらうこと。ほかに、`read_limited` の非公開化のコミットの順(19 と 40)、§4.6 の 3 行目の書き方、`nfr_robust.rs` の `Deps` の写し、P1・P2 の CI の余裕、状態の権限なし大のテストなどの小さな点がある。
