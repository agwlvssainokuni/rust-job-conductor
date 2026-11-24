## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Minor | aidlc/spaces/default/intents/261124-converter-testing-module/construction/functional-design/inventory.md:5 の仕様書の表(S1〜S12)。対象の仕様書は docs/spec/rust/11-quality-and-build.md の 150 行目(3 章「時間の上限」の行) | 3 章の「時間の上限」の行は、場所の列に「各 `common/mod.rs`」と書く。converter の `with_timeout` は、コミット 1 で `tests/common/mod.rs` から `src/tests/mod.rs` へ移る。この行は S の一覧に無く、直す対象か、直さない対象かの判断も書かれていない。行の本文は「プロセスを起動するテスト」の上限を指すので、converter の `with_timeout`(スレッドの時間の上限)は元から含まれていないと読める。ただし、読み手が確かめる手がかりが無い | 一覧に「11:150 は変えない。理由: 行はプロセスを起動するテストの上限で、converter の `with_timeout` はそれに当たらない」と 1 行足す。当たると判断するなら、S12 の `mod.rs` の行に合わせて直す | New |
| R-02 | info | aidlc/spaces/default/intents/261124-converter-testing-module/construction/functional-design/functional-spec.md:15(BR3.2)、:24(項目 3) | `impl PartialEq for Value` に `#[cfg(test)]` を付けると、`tests/nfr_security.rs` の 2 件のソースの検査(最初の `#[cfg(test)]` より前を製品のコードとみなす)が見る `types.rs` の範囲は `eq` の本体の前で終わる。`eq` は `.all(...)` などだけで、`unwrap`・書き込みの API は無く、テストのときだけのコードなので、実害は無い。ただし、この範囲の縮みは設計の文書に書かれていない。検査は文字列の分割に頼るので、今後 `eq` の中に `unwrap` を足しても検査は止まらない | 対応は必須ではない。BR3.2 に「`eq` は検査の範囲の外になる。テストのときだけのコードなので許す」と 1 文足すと、後で読む人が迷わない | New |
| R-03 | info | aidlc/spaces/default/intents/261124-converter-testing-module/construction/functional-design/functional-spec.md:96、:15(BR3.2) | `src/tests/` は、統合テストのソースの検査(`src_files()` が `src/` の直下の `*.rs` だけを読む)の対象の外になる。そのため、ここに置くテストのコードには、`std::env` や書き込みの API の禁止が働かない。移す 45 件のうち、禁止の語を使うのは `fixture()` の `env!("CARGO_MANIFEST_DIR")`(`std::env` ではない)だけで、今は問題が無い。これは決定した設計(D2)の結果で、設計の方針には合う | 対応は不要。実装のコード生成の記録に、「`src/tests/` は検査の外。禁止の語は使っていない」と書いておくとよい | New |

### Validation Tool Results

| Tool | Result | Interpretation |
| --- | --- | --- |
| `git grep` で `read_tree`・`testing::`・`opposite`・`doc(hidden)` を `crates` と `docs/spec/rust`・`docs/spec/language-agnostic` へ | crates は converter の `lib.rs`(42、171、177 行)、`types.rs`(32、139〜143 行)、`tests/common/mod.rs`(24、28、32 行)だけ。cli・core・runtime は 0 件。仕様書は rust 02:129、:132、:331 だけ。language-agnostic は 0 件 | 設計の一覧(S1〜S3、項目 1・2)と一致する。漏れなし |
| `tests/` の `common::` の使用箇所の確認 | `yaml`・`toml`・`with_timeout` は `nfr_security.rs` の 479〜553 行(移す範囲)でだけ使う。`convert_ac.rs` は 43〜55 行の 2 件だけ。残るテストは `common::kinds` だけを使う | T4・T3・T6 の分け方が正しい。残る統合テストは `common` の補助に頼らない |
| 補助 `a`・`i`・`m`・`s`・`toml`・`yaml`・`with_timeout` の `read_resolve.rs` での使用 | すべて 1 回以上(a 2、i 17、m 18、s 9、toml 7、yaml 41、with_timeout 6) | コミット 1 で、`src/tests/mod.rs` の補助が使われずに警告で止まることは無い |
| 補助 `yaml()`・`toml()` の向き(読み比べ) | 元の `read_tree(text, Yaml, "t.yaml")` は `read_value(text, Toml, "t.yaml")`。元の `read_tree(text, Toml, "t.toml")` は `read_value(text, Yaml, "t.toml")`。設計の `yaml`・`toml` と一致。ファイル名の `file` も同じ | 移したテストが同じことを確かめ続ける |
| `include_str!` のパス | `src/tests/X.rs` から `../../tests/fixtures/…` は `crates/converter/tests/fixtures/…` を指す | 正しい。`fixture()` の `env!("CARGO_MANIFEST_DIR")` は変わらない |
| 本番での `Value` の `==` と `PartialEq` の導出 | `raw.rs` に `Value` は無い。`scalar.rs` の `Resolved` も `Value` を持たない。`types.rs:89` の `*other == v` は `eq` の本体の中。`HashMap` は `types.rs:20` と `:84` だけ | `#[cfg(test)]` にしても本番で使う所は無い。`HashMap` を `eq` の本体へ移す設計も正しい(`types.rs` の単体テストは `HashMap` を使わない) |
| `#[cfg(test)]` の位置の確認 | `lib.rs` に `#[cfg(test)]` は無い(最後に 1 件足す設計)。`types.rs:96` が最初(`impl PartialEq` を付けると前に出る)。`tests/nfr_security.rs:75、:126` が文字列で分割 | BR3.2 の 3 つの条件(直下に置かない、`lib.rs` の最後、説明に書かない)の根拠が実物にある |
| Windows でだけコンパイルされるテスト | converter の `cfg(windows)` は 0 件。`cfg(unix)` は `intake.rs`(単体)と `tests/convert_ac.rs:186`(残る統合テスト)だけ | 移す 45 件に OS の分岐が無い。設計の主張どおり |
| `Cargo.toml` と CI の参照 | converter の `Cargo.toml` に `[[test]]` の指定は無い。`.github`・README に `read_resolve` などの参照は無い | ファイルの移動で壊れる設定は無い |
| 件数の算術 | 単体 52 − 1 + 45 = 96。統合 64 − 45 = 19(+ 無視 1)。全体 1,153 − 1 = 1,152。コミットごと(80/36、93/23、95/21、97/19、96/19)も、28、13、2、2 の移動と整合 | 見込みは矛盾しない |
| 今回は実行しなかったもの | `cargo test`・`clippy`・`cargo doc`・カバレッジは、この審査の範囲(設計の読み)では走らせていない(ソースを書き換えた試しは禁止のため)。設計が引く「写しの木での検証」(`inventory.md` §0.2)の結果は、そのまま信頼した | 実装のコード生成の段で、§7 の確かめで確認される。審査としての欠けの指摘ではなく、見えている範囲の注記 |

### Summary

設計は、網羅(参照 12 件、補助の使用、`include_str!`、仕様書の 02・09・11 章)、移すテストの同一性(`yaml`・`toml` の向きとファイル名)、本番に足さないこと(`pub`・`doc(hidden)`・テスト用の引数なし)、BR3.2 のソースの検査との関係、件数とコミットの並びが、実物と矛盾しない。見つけたのは、仕様書 11 章 150 行目の扱いが一覧に無いこと(minor)と、`eq` の本体とテストのディレクトリが検査の外になる点の注記(info 2 件)だけで、どれも実装の前に直す必要は無い。READY。
