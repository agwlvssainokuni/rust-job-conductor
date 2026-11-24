# コード生成のまとめ(Code Summary): テストのファイルの統合

計画 `code-generation-plan.md` の Step 1〜8 を実装した。基準は HEAD ba82c30(コードの計画のコミット。コードと仕様書は 16ac9d6 と同じ)。コードと仕様書のコミットは 12 件で、`functional-spec.md` §8 の並び 1〜12 と 1 対 1 に対応する(13 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。本番のコード(各クレートの `src/` のうち `src/tests/` の外)、`Cargo.toml`、`Cargo.lock`、CI の設定、README は 1 行も変えていない。テストの名前・確かめの行・期待の値は変えていない。消したテスト・足したテスト・名前を変えたテストは無い。

## コミットの一覧

| # | §8 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `ede93e2` | converter: 出力が毎回同じことのテストを、受け入れ基準の統合テストのファイルへ移す(T1)。元: `crates/converter/tests/convert_roundtrip.rs`。仕様書 S7、L3 |
| 2 | 2 | `6b1f458` | converter: 変換の結果の木を比べる受け入れ基準の単体テストを、往復の単体テストのファイルへ移す(T2)。元: `crates/converter/src/tests/convert_ac.rs`。`src/tests/mod.rs` の説明。仕様書 S2、S7、S9、L1、L2 |
| 3 | 3 | `489423f` | converter: 変換の失敗の種類の一覧を、表示の検査のファイルへ移す(T3。別名 `C` → `K`、`pub` を外す)。元: `crates/converter/tests/common/{mod,kinds}.rs`。仕様書 S8 |
| 4 | 4 | `ea11500` | runtime: 子の環境変数のテストを、実行の統合テストのファイルへ移す(T4。Windows だけの 3 件の前に節「環境変数の引数」)。元: `crates/runtime/tests/runner_env.rs`。仕様書 S4、S5 |
| 5 | 5 | `71baf76` | runtime: 完了のステートも返す読み取りのテストを、`state_read.rs` の子のモジュール `mod raw` へ移す(T5。D3)。元: `crates/runtime/tests/state_read_raw.rs`。仕様書 S6、L4、L5 |
| 6 | 6 | `85293bd` | cli: init の出力先の型のテストを、`args_grammar.rs` の init の節へ移す(T6)。元: `crates/cli/src/tests/init_types.rs`。仕様書 S13、S14 |
| 7 | 7 | `d642724` | cli: run ID の指定の型のテストを `--run-id` の節へ移し、`fr2_5_` の 2 件を新しい `--remove-on-complete` の節へ並べ直す(T7、T8。D4)。元: `crates/cli/src/tests/run_id_arg.rs`。仕様書 S13、S14 |
| 8 | 8 | `f9cb02f` | cli: 長いジョブ ID の図のテストを、`graph_report.rs` の最後へ移す(T9。補助を `long_id_definition` に。D2)。元: `crates/cli/src/tests/graph_long_id.rs`。仕様書 S13、S15 |
| 9 | 9 | `5051108` | cli: README の例を比べる単体テストを 1 つのファイルにまとめる(T10。`git mv docs_graph.rs docs.rs` のあと、補助 `readme` と dry-run・init の 1 件ずつを足す)。元: `crates/cli/src/tests/docs_{graph,support,dry_run,init}.rs`。仕様書 S3、S13、11 章の `docs_env.rs`… の行の末尾、L8a |
| 10 | 10 | `0239bde` | cli: README の語句と例の読み取りの統合テストを `src/tests/docs.rs` へ移す(T11。定数を `RETRY_HEADING`・`ENV_HEADING` に。D1)。元: `crates/cli/tests/docs_{retry,env,graph,dry_run,init}.rs`、`crates/cli/tests/docs_support/mod.rs`。仕様書 S10、S13、L6、L7、L8b、L9、L10 |
| 11 | 11 | `3ee1a52` | cli: 偽の `Env`・`Sink` と `state_error` の写しを、共有する `crates/cli/tests/common/fakes.rs` の 1 つにする(T12。統合テストは `mod fakes;`、単体テストは `#[path]`。`Language` の注意書き。統合テストの `use std::io;` を消す)。仕様書 S1、S11、S12 |
| 12 | 12 | `a008bc8` | cli: init のひな形の原本をデータのファイル `crates/cli/tests/fixtures/init_{ja,en}.toml` にし、両方のテストから `include_str!` で読む(T13。D8)。リポジトリの根に `.gitattributes`(D5)。仕様書 S11、S16 |
| — | 13 | (コミットなし) | §7 の検索、件数、テスト名の差、本体の比べ、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -euo pipefail`。`target/mtf/` に置き、終わってから消した)で、`cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、変えたクレートのテスト(`cargo test -p <クレート> --locked`。実行ファイルごとの件数)を通し、件数をコミットごとの見込みと照らした。既にあるファイルへまとめたコミットは、本文に「元: <元のパス>」を書いた。`git log --follow crates/cli/src/tests/docs.rs` は `docs_graph.rs` の履歴まで追える。

## 件数の移り変わり(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | 変わる実行ファイル | 全体 | 見込み(§8) |
|------|--------------------|------|--------------|
| 基準(ba82c30) | converter: lib 96、`convert_ac` 8、`convert_roundtrip` 1、`nfr_security` 10 + 1 無視。runtime: `runner_basic` 12、`runner_env` 5、`state_read` 15、`state_read_raw` 4。cli: lib 162、`docs_*` 5 つ(計 10)、`flow_init` 9 | 1,149 通過、1 無視 | — |
| 1(`ede93e2`) | `convert_ac` **9**、`convert_roundtrip`(統合)無し | 1,149 | `convert_ac` 9 |
| 2(`6b1f458`) | converter lib 96(`tests::convert_roundtrip` 15) | 1,149 | converter lib 96 |
| 3(`489423f`) | `nfr_security` 10 + 1 無視 | 1,149 | 10 + 1 無視 |
| 4(`ea11500`) | `runner_basic` **17**、`runner_env` 無し | 1,149 | 17 |
| 5(`71baf76`) | `state_read` **19**、`state_read_raw` 無し | 1,149 | 19 |
| 6〜9 | cli lib 162(統合は変わらない) | 1,149 | cli lib 162 |
| 10(`0239bde`) | cli lib **172**、`docs_*` の実行ファイル無し(統合 **284**) | 1,149 | lib 172、統合 284 |
| 11(`3ee1a52`) | cli lib 172、統合 284(cli 計 456) | 1,149 | 1,149 |
| 12(`a008bc8`) | cli lib 172、`flow_init` 9 | 1,149 | 1,149 |
| 最後(全テスト) | core 112 / 269、runtime 69 / 128、converter 96 / 19 + 1 無視、cli 172 / 284 | **1,149 通過、1 無視、0 失敗** | 1,149 = 基準 1,149 − 消したテスト 0 + 足したテスト 0 |

## まとめたファイルと移したテスト

無くなったファイル 19(cli 13、converter 4、runtime 2)。新しく作ったファイル 4(`crates/cli/src/tests/docs.rs`、`crates/cli/tests/common/fakes.rs`、`crates/cli/tests/fixtures/init_ja.toml`、`init_en.toml`)と `.gitattributes`。

| ID | 元 → 先 | 件数 | パスの変わり方 | 本体で変わった行 |
|----|---------|------|----------------|------------------|
| T1 | converter `tests/convert_roundtrip.rs` → `tests/convert_ac.rs` | 1 | 変わらない(実行ファイルが `convert_ac` に) | なし |
| T2 | converter `src/tests/convert_ac.rs` → `src/tests/convert_roundtrip.rs` | 2 | `tests::convert_ac::` → `tests::convert_roundtrip::` | なし |
| T3 | converter `tests/common/{mod,kinds}.rs` → `tests/nfr_security.rs` | 0(一覧) | — | 一覧の別名 `C` → `K` |
| T4 | runtime `tests/runner_env.rs` → `tests/runner_basic.rs` | 5 | 変わらない(実行ファイルが `runner_basic` に) | なし |
| T5 | runtime `tests/state_read_raw.rs` → `tests/state_read.rs` の `mod raw` | 4 | `<名前>` → `raw::<名前>` | 字下げだけ(`cargo fmt` が `fr3_4_…` の 1 行を折り返した) |
| T6 | cli `src/tests/init_types.rs` → `args_grammar.rs` の init の節 | 1 | `tests::init_types::` → `tests::args_grammar::` | なし |
| T7 | cli `src/tests/run_id_arg.rs` → `args_grammar.rs` の `--run-id` の節 | 1 | `tests::run_id_arg::` → `tests::args_grammar::` | なし |
| T8 | `args_grammar.rs` の `resolve_run_id` の節 → 新しい `--remove-on-complete(FR2.5)` の節 | 2 | 変わらない | なし(並べ替え) |
| T9 | cli `src/tests/graph_long_id.rs` → `graph_report.rs` の最後 | 2 | `tests::graph_long_id::` → `tests::graph_report::` | 2 行(`definition(&long)` → `long_id_definition(&long)`) |
| T10 | cli `src/tests/docs_{graph,dry_run,init}.rs`、`docs_support.rs` → `src/tests/docs.rs` | 8 | `tests::docs_<話題>::` → `tests::docs::` | なし |
| T11 | cli `tests/docs_{retry,env,graph,dry_run,init}.rs`、`docs_support/mod.rs` → `src/tests/docs.rs` | 10 | `<名前>`(統合)→ `tests::docs::<名前>`(単体) | 3 行(`HEADING` → `RETRY_HEADING` 1 行、`ENV_HEADING` 2 行) |
| T12 | cli `tests/common/mod.rs` と `src/tests/common/mod.rs` の写し → `tests/common/fakes.rs` | 0(補助) | — | —(使う側は変えていない) |
| T13 | cli `tests/flow_init.rs` と `src/tests/init_template.rs` の定数 → `tests/fixtures/init_{ja,en}.toml` | 0(定数) | — | —(定数の名前と型は同じ) |

`src/tests/docs.rs` の並び: 補助(`DOCS_CHECK_WORDS`、`readme`、`assert_readme_has`、`assert_docs_check_words`、`block_after_heading`)→ リトライ → 環境変数 → `graph` → `--dry-run` → `init`。各節は、統合テストだった語句の検査、単体テストだった例の比べ方の順。`#![allow(dead_code)]` は要らなくなった。

## 確かめ(§7。最後のコミット a008bc8 の後)

### テスト名の比較

- `cargo test --workspace --locked -- --list` の `: test` の行は、前後とも 1,150 行(無視の 1 件を含む)。
- 名前の最後の部分で比べた差: **0**。
- パスつきの差: **56 行**(移った 28 件の前と後)。内訳は上の表の「パスの変わり方」のとおり(`raw::` 4、`tests::args_grammar::` 2、`tests::convert_roundtrip::` 2、`tests::docs::` 18、`tests::graph_report::` 2)。T1・T4 の 6 件は実行ファイルごとの件数(`convert_ac` 9、`runner_basic` 17)で確かめた。

### 移したテストの本体の比べ

元(ba82c30)の各テストの関数を取り出し、移す先の同じ名前の関数と、空白を除いて比べた。

- 移したテスト 34 件: 同じ 30 件、違う 4 件。違うのは T9 の 2 行(`long_id_definition`)と T11 の 3 行(`RETRY_HEADING` 1 行、`ENV_HEADING` 2 行)の計 5 行だけ。確かめの行(`assert…`)と期待の値は 1 行も違わない。
- 移す先に元からあったテストと、定数の宣言だけが変わるファイルのテスト(`args_grammar.rs`、`graph_report.rs`、`init_template.rs`、`flow_init.rs`、`convert_ac.rs`、`convert_roundtrip.rs`、`nfr_security.rs`、`runner_basic.rs`、`state_read.rs`。並べ替えた `fr2_5_` の 2 件を含む)165 件: すべて同じ。

### 完了の判定の検索(§7 のコマンドをそのまま走らせた結果)

| 検索 | 期待 | 結果 |
|------|------|------|
| 無くなるファイル 19 の存在 | 出力なし | 出力なし |
| 新しいファイル 4 と `.gitattributes` | すべてある | すべてある |
| `grep -c "#\[test\]" crates/cli/src/tests/docs.rs` | 18 | 18 |
| `fn readme()` の数 | 1 | 1 |
| `struct FakeEnv` などのあるファイル | `crates/cli/tests/common/fakes.rs` だけ | `crates/cli/tests/common/fakes.rs` だけ |
| ひな形の生の文字列(`r##"# conductor の定義ファイルのひな形` など) | 0 | **2**(下の注。本番の `crates/cli/src/report/init_template.rs` の `HEAD_JA`・`HEAD_EN`。基準から同じ 2 か所にあり、変えていない。本番を除くと 0) |
| ひな形の `include_str!` | 4 | 4 |
| `args_grammar.rs` の節の並び | init → `--run-id` と `--all` → `resolve_run_id` → `--remove-on-complete` → graph | init(391)→ `--run-id` と `--all`(504)→ `resolve_run_id`(656)→ `--remove-on-complete(FR2.5)`(697)→ graph(726)の順 |
| `^mod raw`(`state_read.rs`) | 1 行 | 1 行(315) |
| `src/` の `src/tests/` の外の `include`・`#[path` | 0 件 | 0 件 |
| 仕様書の無くなったファイルの名前 | 0 件 | 0 件 |
| rust 11 の `convert_roundtrip.rs` の行 | 1 行(単体) | 1 行 |
| rust 11 の `convert_ac.rs` の行 | 1 行(統合) | 1 行 |
| rust 11 の `common/kinds.rs` の行 | 2 行(core 統合、cli 単体) | 2 行(221、286) |
| 言語に依らない 11 の `tests/common/kinds.rs` | 1 行(772。D7 で直さない) | 1 行(772) |
| 言語に依らない 11 の `convert_ac::`・`convert_roundtrip::` | `ac_7_1_1`・`ac_7_1_2` は `convert_roundtrip`、`出力は毎回同じ` は `convert_ac` | そのとおり(588、589 が `convert_roundtrip`、605 が `convert_ac`) |
| 新しい名前の記述 | `state_read::raw::fr3_4`、`fixtures/init_`、`common/fakes.rs`、`docs.rs` の行がある | ある(言語に依らない 11 の 718、rust 02 の 273、rust 11 の 272〜274、285、288、294) |
| `git diff --stat 16ac9d6 -- Cargo.lock '*Cargo.toml' .github README.md` | 空 | 空 |
| 本番のコード(`src/tests/` の外)の差 | 空 | 空(16ac9d6 からも ba82c30 からも) |
| `crates/core`、`nfr_perf.rs`、`msrv_consistency.rs`、cli・runtime の `nfr_source.rs` の差 | 空 | 空 |
| 凍結した仕様書(go、python、typescript、csharp、powershell)の差 | 空 | 空 |
| 足した行の `unsafe`・`cfg(unix)`・`cfg(windows)` | なし | なし |

注: ひな形の生の文字列の検索は、テストの写しが残らないことを確かめる意図だが、検索の語が本番のひな形の定数にも当たる(設計の §7 の見込みの 0 は、本番の 2 か所を数えていなかった)。`grep … | grep -v "^crates/cli/src/report/"` で 0 件。本番のコードは変えていない。

### そのほかの確かめ

| 項目 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 通る |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通る |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(本番の形) | 通る |
| `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings`(macOS から) | 通る(コミット 11 の後と最後の 2 回) |
| `cargo test --workspace --locked` | **1,149 通過、1 無視、0 失敗** |
| 製品のコードの検査 | cli `nfr_source` 6 通過、runtime `nfr_source` 5 通過、converter `nfr_security` 10 通過 + 1 無視(数と中身は変えていない) |
| `cargo doc --workspace --no-deps --locked` の警告 | 5 件(core 1、runtime 4)。基準と同じ |
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | 通る。行 **98.06%**(領域 97.71%、関数 98.20%) |
| MSRV | ローカルの rustc が 1.100.0(MSRV と同じ)で、上のビルドとテストが通る |
| ひな形のデータのファイル | ba82c30 の `src/tests/init_template.rs` と `tests/flow_init.rs` の原本の文字列と、`cmp` でバイト単位で一致(4 通りとも)。`git check-attr text eol` が `text: set`、`eol: lf` |
| `Cargo.toml`・`Cargo.lock` | 差分なし(新しい依存なし) |

### D5 の最終確認(Windows の CI)

**未確認(push の後の CI で確かめる)**。ローカルの macOS では、`.gitattributes` が Windows の取り出しで効くかを確かめられない。

1. コミット 12(`a008bc8`)の後の CI の `test`(windows)の結果: 未確認。確かめるのは、`flow_init` の 9 件と `init_template` の 4 件(単体テストの lib に含まれる)の成否。
2. 失敗したときの戻し先: テストの側で `\r\n` を `\n` に直す案(D5 の代案)。`include_str!` の定数を、改行を直した `String` を返す関数か `LazyLock` にし、使う側の数行を合わせる。テストの名前・確かめの行・期待の値は変えない。変える行は記録し、承認を取る。

## 計画からのずれ

- **コミット 9 で、統合テストの `tests/docs_graph.rs`・`docs_dry_run.rs`・`docs_init.rs` の説明の 1 行ずつ(単体テストのファイル名)を `src/tests/docs.rs` に直した**。計画の §8 の 9 には無い。コミット 9 の時点で、説明が無くなったファイルを指さないようにするため。3 つのファイルはコミット 10 で消えるので、最後の形には残らない。テストの本体は変えていない。
- 途中のコミットで、説明と仕様書の行に、その時点で正しい中間の文を書いた(コミット 1 の `tests/convert_ac.rs` のモジュールの説明と rust 11 の `convert_ac.rs` の行、コミット 6 の `args_grammar.rs` のモジュールの説明と S14、コミット 9 の `docs.rs` のモジュールの説明と S13 の `docs.rs` の行)。最後のコミットの後の形は、設計の §2 と inventory の §7 の形。
- 説明の文の小さな違い: converter の `src/tests/convert_roundtrip.rs` の説明は、設計の文に元のファイルの説明の番号(AC7.1.1、AC7.1.2)を残した。runtime の `runner_basic.rs` の説明は、元の `runner_env.rs` の説明 2 行を、番号(U1。BR4.3、BR4.4、FR1.3)つきの 1 段落として足した。
- §7 の検索のうち、ひな形の生の文字列の見込み 0 は 2 になった(上の注。本番のひな形の定数で、変えていない)。実装のずれではない。
- 見つけた、範囲の外の古い記述(直していない): `docs/spec/rust/12-rebuild-guide.md` の 142 行目「`docs_*.rs` が、必須の語句と例を検査する」。§7 の検索の語(`docs_(dry_run|env|…)`)に当たらず、S1〜S16 にも無い。README の検査は今は `crates/cli/src/tests/docs.rs` にある。直すかは、次の機会に決めてほしい。
- そのほかは計画どおり。T1〜T13 の外の既存のテストを変える必要は出なかった。消したテスト・足したテスト・名前を変えたテストは無い。
- **レビューの minor(ユーザーの指示)**: コード生成の承認のときに、ユーザーが仕様書 rust 12 の 142 行目の直しを求めた。「`docs_*.rs` が」を「cli の単体テスト `src/tests/docs.rs` が」にした(仕様書だけ)。

**追記(2026-11-25、Build and Test の段)**: 確認済み。`f4fafde` の CI(run 45503331915)の `test (windows-2022)` は失敗 0 で通り、ひな形を読むテストも成功した。戻し先の代案は使わなかった。
