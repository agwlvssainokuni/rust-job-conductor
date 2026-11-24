## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | inventory.md:206 (S7)、functional-spec.md:281-284(§8 のコミット 5・12) | S7(02-architecture の「テストの偽の実装」の行)の割り当てが食い違う。inventory は S7 をコミット 5 に置き、直す文に「`FakeEnv`・`FakeSink` の写しを持つ」と書く。だがコミット 5 の時点で `src/tests/common/mod.rs` にあるのは `FakeEnv` の一部だけで、`FakeSink` はコミット 9 で入る。一方 functional-spec §8 の表は、コミット 5 の仕様書の欄に S7 を書かず、S1〜S7 をコミット 12 に置く。 | S7 を 1 つのコミットに決め、inventory と §8 を揃える。コミット 5 に置くなら文を「`FakeEnv` の写し」にし、コミット 9 で `FakeSink` を足す。コミット 9 かコミット 12 に置くなら inventory の「5」を直す。 | New |
| R-02 | Minor | traceability.json の SM6 と functional-spec.md:328(§11 の SM6 の行) | SM6 の対応先が食い違う。traceability.json は `BR1.3, BR9.4`、§11 の表は `BR9.4` だけ。 | どちらかに揃える。`dead_code` と doc の警告を SM6 の合格の条件に含めるなら §11 に BR1.3 を足す。 | New |
| R-03 | Minor | functional-spec.md:93-112(§2.3)の `nfr_source.rs` の検査 | D2 の検査の範囲を `src/tests/` のディレクトリ全体にしている一方、「`src/tests/` が本番に入らない」ことの確かめは、`lib.rs` の末尾が `#[cfg(test)]` と `mod tests;` であることだけ。製品のファイルから `#[path = "tests/…"] mod …;` で `src/tests/` のファイルを読む道は、この確かめで塞がらない。実際に今は無く、`src/tests/` を読むのは末尾の宣言だけだが、将来入ったとき OS 分岐と `panic!(` が検査されないまま本番に入りうる。 | 同じテストに、`src/tests/` の外のファイルに `#[path` が無いこと(または `tests/` を指す `#[path` が無いこと)の確かめを 1 行足すか、足さない理由(今は無い、変更が要る)を §2.3 に 1 文書く。 | New |
| R-04 | Minor | functional-spec.md:269、291(§8 の各コミットの確かめ) | コミット 5〜8 は、`src/tests/common/mod.rs` に使う項目だけを置く(`#![allow(dead_code)]` を付けない)。ところが、写しの木で確かめたのは最後の形だけで、途中の形は未確認(§8 自身も認める)。項目や `FakeEnv` のメソッドの置き分けを誤ると、`clippy -D warnings` の `dead_code` でそのコミットが止まる。設計の誤りではなく、実装時の確認事項。 | 実装でコミット 5〜8 を作るとき、各コミットで `cargo clippy --workspace --all-targets --locked -- -D warnings` を通す(§8 のとおり)。置き分けが合わなければコミットの切り方を直し、記録に残す。 | New |
| R-05 | info | functional-spec.md:306(D2)と project.md の Forbidden(「NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない」) | ユーザーは、この規則を製品のコードについてのものと読むと認めた。だが規則の文は変わらないので、次に読む人は `src/tests/args_grammar.rs` の `#[cfg(unix)]` を違反と読むかもしれない。仕様書 S10・S11・S16 は直す設計になっているが、プロジェクトの規則の文は直さない。 | 範囲外でよい。気になるなら、実装の後で、学びの記録の手順で規則の文に「製品のコード(単体テスト `src/tests/` を除く)」と足すかを、ユーザーに聞く。 | New |
| R-06 | info | 観点 1・2・3・5・6・7 の確認(指摘なし) | 次を実物で確かめた。(1) `lib.rs` の公開は、`pub mod` 3 + `parse_args` 1 + `deps` 10 + `install_panic_hook` 1 + `types` 11 + `run_cli`・`run_cli_with` の 28 項目で、13 と 15 に過不足なく分かれる。`Runtime`・`Sink`・`Env`・`Signals` の引数と戻り値の cli の型は `SignalError` だけ(ほかは core・runtime・converter の型)。`main.rs` が使うのは `StdSink`・`SystemEnv`・`install_panic_hook`・`run_cli` だけ。狭める `exit` の定数・関数、`resolve_run_id`、`render_*`、`Reporter` のメソッドは、すべて本番の流れから使われ、`dead_code` は出ない。(2) `conductor_cli` を参照する `tests/` は 19 ファイル。狭めた名前を参照する 14 ファイルと `common/mod.rs`・`common/kinds.rs` は、すべて T 表(T1〜T9)に入っている。残る `signals_order.rs`・`signals_real.rs` は `Signals`・`SystemSignals`(残す項目)だけ。`bin_support`・`docs_support`・`signal_support`、`src/bin` は参照が 0 件。`#[test]` の数は、移す 10 ファイルで 154、`docs_*` の 8 件で 162 になり、`cargo test` の実測(`docs_graph` 8、`docs_dry_run` 2、`docs_init` 2)と合う。移す先の `tempfile`(dev 依存)、`include_str!`、`../` のパスの使用は 0 件で、禁止語の検索(unsafe、ネットワーク)も 0 件。(3) `init_template.rs` の原本では `my-pipeline` が `name = "…"` の 1 行にだけ現れるので、D4 の「1 回だけ置き換える」は元のバイト列と同じになる。(5) 本番の処理のコードは `lib.rs` の可視性・説明・末尾の宣言と `escape.rs` の説明だけで、利用者から見える動きは変わらない。(6) コミットごとの cli の件数(9/449、12/446、13/445、16/442、76/382、106/352、113/345、120/338、156/302、164/294)は、基準(2 と 456)から計算して合う。全体は 382 + 197 + 115 + 458 = 1,152。(7) 新しい依存、`unsafe`、本番の `cfg(unix\|windows)` の追加は設計に無い。 | 対応は要らない。 | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| cargo test -p conductor-cli --locked | PASS(HEAD。単体 2、統合 456 の構成。ファイルごとの件数は設計の表と一致) | 基準の件数が正しい |
| grep(`conductor_cli` の参照、`#[test]` の数、禁止語、`include_str!`) | 設計の T 表と一致。漏れ 0 | 網羅は成り立つ |
| grep(`src/` での各項目の使用) | 狭める項目はすべて本番から使われる | `dead_code` の恐れなし(写しの木での検証と合う) |

### Summary

設計は、公開範囲 13 と 15 の分け方、継ぎ目の型(`SignalError` だけ)、移す 162 件の網羅、D4 のひな形の写し、D2 の検査の範囲の直しのどれも、実物で確かめて成り立つ。残る指摘は、仕様書 S7 の割り当ての食い違い、traceability の SM6 の対応先の食い違い、`src/tests/` を本番から読む道(`#[path]`)を塞ぐ確かめの足りなさ、途中のコミットの `dead_code` の確認という軽いものだけで、承認の妨げにならない。
