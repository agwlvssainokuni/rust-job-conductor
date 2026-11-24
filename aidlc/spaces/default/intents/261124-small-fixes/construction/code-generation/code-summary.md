# コード生成のまとめ(Code Summary): 小さな直し

計画 `code-generation-plan.md` の Step 1〜9 を実装した。基準は HEAD c3a2a5b(コードの計画のコミット。コードと仕様書は 6ce7d45 と同じ)。コードと仕様書のコミットは 10 件で、`functional-spec.md` §8 の並び 1〜10 と 1 対 1 に対応する(11 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。本番のコードで変えたのは説明の文(C1〜C4)だけで、関数の本体、型、可視性は変えていない。利用者から見える動きは変えていない。

## コミットの一覧

| # | §8 | ハッシュ | 内容 |
|---|----|----------|------|
| 1 | 1 | `4ecf31e` | core: 計画と次の動きの説明の、起動前の判断の関数の名前を直す。C1(`crates/core/src/plan/mod.rs`)、C2(`crates/core/src/types.rs`)。仕様書 rust `05` の 341・850 行目(D5) |
| 2 | 2 | `80c827d` | core: ステートの更新の関数を runtime が再公開するという説明を直す。C3(`crates/core/src/lib.rs`。D1) |
| 3 | 3 | `185e93b` | docs: 仕様書 rust 02 の `resolve_run_id` の引数の型を実装に合わせる(S1) |
| 4 | 4 | `459d273` | docs: 仕様書 rust 02 の core の公開の一覧に `consistency` を足す(S2、S3) |
| 5 | 5 | `a85e30e` | docs: 仕様書 rust 02 の公開の段落の版の注記を、版の履歴に分ける(S4〜S6、S7。D2) |
| 6 | 6 | `46907fa` | docs: 仕様書 rust 06 の実装の所在を直す(X2。D6) |
| 7 | 7 | `2fff706` | core: 計画の模擬の性能のテストと同じ場面の、終点の後のスキップの性能のテストを消す(T1。使われなくなる import も消した) |
| 8 | 8 | `54818fd` | core: env と env_file を持つ計画の性能のテストを `plan_simulate_perf.rs` へ移し、`plan_perf.rs` を消す(T2。D8)。仕様書 S8、S9、S10、S12(D3) |
| 9 | 9 | `3ac9df7` | cli: 引数の文法のテストと重なる init の型のテスト 2 件を消す(T3)。C4(`crates/cli/src/types.rs`)。仕様書 S11 |
| 10 | 10 | `5bd641f` | cli: ソースの検査で、製品のコードの include の 3 つのマクロも止める(T4。D4)。仕様書 S13、S14 |
| — | 11 | (コミットなし) | §7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -euo pipefail`。`target/small-fixes/` に置き、終わってから消した)で、`cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、変えたクレートのテスト(`cargo test -p <クレート> --locked`)を通し、件数をコミットごとの見込みと照らした。仕様書だけのコミット(3〜6)は §7 の該当の検索で確かめた。最後に全テストを実行した。

## 件数の移り変わり(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | core 単体 / 統合 | cli 単体 / 統合 | 全体 | 見込み(§8) |
|------|------------------|-----------------|------|--------------|
| 基準(c3a2a5b) | 112 / 270 | 164 / 294 | 1,152 通過、1 無視 | — |
| 1(`4ecf31e`) | 112 / 270 | (変えていない) | 1,152 | 1,152 |
| 2(`80c827d`) | 112 / 270 | (変えていない) | 1,152 | 1,152 |
| 3〜6 | (仕様書だけ) | (仕様書だけ) | 1,152 | 1,152 |
| 7(`2fff706`) | 112 / **269** | (変えていない) | 1,151 | 1,151(core 統合 269) |
| 8(`54818fd`) | 112 / 269(`plan_perf` 無し、`plan_simulate_perf` 2 → 3) | (変えていない) | 1,151 | 1,151 |
| 9(`3ac9df7`) | (変えていない) | **162** / 294 | 1,149 | 1,149(cli 単体 162) |
| 10(`5bd641f`) | (変えていない) | 162 / 294(`nfr_source` 6) | 1,149 | 1,149 |
| 最後(全テスト) | 112 / 269 | 162 / 294 | **1,149 通過、1 無視、0 失敗**(core 381、runtime 197、converter 115 + 1 無視、cli 456) | 1,149 |

runtime(69 / 128)と converter(96 / 19 + 1 無視)は変えておらず、件数も変わらない。

## 消したテストと移したテスト

| 扱い | テスト | 元 → 先 | 同じことを確かめて残るテスト(`functional-spec.md` §4.6) |
|------|--------|---------|------------------------------------------------------------|
| 消した(T1) | `終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる` | `crates/core/tests/plan_perf.rs` | `plan_simulate_perf::p2_終点のあとの_999_個のスキップの計画は_150ms_以内` |
| 消した(T3a) | `init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml` | `crates/cli/src/tests/init_types.rs` | `tests::args_grammar::init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` |
| 消した(T3b) | `コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ` | `crates/cli/src/tests/init_types.rs` | 同上 |
| 移した(T2) | `千個のジョブがすべて実行する計画は_1秒以内に作れる` | `crates/core/tests/plan_perf.rs` → `crates/core/tests/plan_simulate_perf.rs` の最後 | (移しただけ。名前と本体は変えていない) |
| 確かめを広げた(T4) | `テスト専用の経路を実行ファイルに持たない` | `crates/cli/tests/nfr_source.rs`(名前と数は変えていない) | — |

- 移したテストの本体: 実装の前の `plan_perf.rs`(`git show 6ce7d45:…`)と後の `plan_simulate_perf.rs` で、`sed -n '/^fn 千個のジョブがすべて実行する/,/^}/p'` の出力が一致した(`diff` の差なし)。
- `signals_order.rs`・`signals_real.rs` は変えていない。T1〜T4 の外の既存のテストは変えていない。足したテストは 0 件、名前を変えたテストは 0 件。
- T4 の確かめ: 本番のファイル `crates/cli/src/exit.rs` の最後に `const _: &str = include_str! ["../Cargo.toml"];`(空白と角かっこの形)を一時的に足すと、`nfr_source` の 6 件のうち `テスト専用の経路を実行ファイルに持たない` だけが失敗し、その行を指した。足した行はすぐ消し(`git diff` が空であることを確かめた)、6 件が通ることを確かめ直した。コミットには入れていない。

## テスト名の比較

`cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sed 's/: test$//; s/.*:://' | sort` を実装の前(1,153 行。無視の 1 件を含む)と後(1,150 行)で比べた。差は、前にだけある次の 3 行だけ(後にだけある名前は 0 件)。

```
< init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml
< コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ
< 終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる
```

移した 1 件は同じ名前のまま残り、実行ファイルごとの件数(`plan_perf` 2 → 無し、`plan_simulate_perf` 2 → 3)で移ったことを確かめた。

## 完了の判定の検索(`functional-spec.md` §7。最後のコミット 5bd641f の後)

| 検索 | 見込み | 結果 |
|------|--------|------|
| `git grep -n "decide_before_run" -- crates ':!crates/core/src/flow/decide.rs'` | 0 件 | 0 件 |
| rust 02 の `resolve_run_id` の行で `Option<&RunIdArg>` | 1 | 1 |
| rust 02 の再公開の段落で `consistency::{ConsistencyResult, MismatchKind, check_consistency}` | 1 | 1 |
| `公開モジュール: \`consistency\`、\`definition\`` | 1 | 1 |
| `^再公開(\`lib.rs\`)` の段落 | 3 | 3 |
| そのうち `v0.` を含むもの | 0 | 0 |
| `^版の履歴(古い順):$` | 3 | 3 |
| 版の履歴の箇条書き(awk) | 7 行。各節の中で古い順 | 7 行(46〜48: core の v0.9.0 → v0.9.6 → v0.9.7、115〜117: runtime の v0.9.0 → v0.9.6 → v0.9.7 以降、153: cli の v0.9.6) |
| `test ! -e crates/core/tests/plan_perf.rs` | 無い | 無い |
| `#[test]` の数(`plan_simulate_perf.rs`、`init_types.rs`) | 3、1 | 3、1 |
| `git grep -n "plan_perf" -- ':!aidlc'` | 0 件 | 0 件 |
| rust 11 の `init_types.rs` | S11 の文 | 302 行目「v0.9.0 以降。`InitTarget::from_path`(`-` だけが標準出力)。`init` のオプションの既定は `args_grammar.rs` が確かめる」 |
| `nfr_source.rs` の `include!` など | 確かめの 1 行と説明 | 20 行目(モジュールの説明)、162 行目(コメントの既知の限界)、168 行目(止める語の配列) |
| `grep -rn "include\|#\[path" crates/cli/src \| grep -v "^crates/cli/src/tests/"` | 0 件 | 0 件 |
| rust 11 の `include` | S13、S14 | 166 行目(3.3 の表)、178 行目(3.5) |
| `git diff --stat 6ce7d45 -- Cargo.lock '*Cargo.toml' .github README.md crates/runtime crates/converter crates/core/src/flow` | 空 | 空 |
| `git diff --stat 6ce7d45 -- docs/spec/{go,python,typescript,csharp,powershell}` | 空 | 空 |
| `git diff --stat 6ce7d45 -- crates/cli/tests/signals_order.rs crates/cli/tests/signals_real.rs` | 空 | 空 |
| `git diff -U0 6ce7d45 -- 'crates/*/src' ':!crates/cli/src/tests'` のコメント以外の行 | なし | なし(変わったのは説明の 4 行だけ) |
| 足した行に `unsafe`・`cfg(unix\|windows)` | なし | なし |

変えたファイル(`git diff --stat 6ce7d45 -- ':!aidlc'`): 13 ファイル(`crates/cli/src/tests/init_types.rs`、`crates/cli/src/types.rs`、`crates/cli/tests/nfr_source.rs`、`crates/core/src/lib.rs`、`crates/core/src/plan/mod.rs`、`crates/core/src/types.rs`、`crates/core/tests/plan_perf.rs`(消した)、`crates/core/tests/plan_simulate_perf.rs`、`docs/spec/language-agnostic/11-quality-and-build.md`、`docs/spec/rust/02-architecture.md`、`docs/spec/rust/05-execution-flow.md`、`docs/spec/rust/06-state-file.md`、`docs/spec/rust/11-quality-and-build.md`)。

## 合格の条件

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 通った |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 通った |
| `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings`(本番の形) | 通った |
| `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings`(macOS から) | 通った(実行は CI の `test`(windows)で確かめる) |
| `cargo test --workspace --locked` | 1,149 通過、1 無視、0 失敗(見込みどおり) |
| `cargo doc --workspace --no-deps --locked` の警告 | 5 件(core 1、runtime 4)。基準のまま |
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | 終了コード 0。行 98.06%(80% 以上) |
| MSRV | ローカルの rustc・cargo が 1.100.0 で、上のすべてをこの版で走らせた |
| 依存 | `Cargo.toml`・`Cargo.lock` の差分なし |
| 版 | 0.9.8 のまま |

## 計画からのずれ

指示や計画に反した点は無い。T1〜T4 の外の既存のテストを変える必要は出なかった。件数はすべてのコミットで見込みどおり。細かな点を記録する。

- **T4 の既知の限界の説明**: 計画の「守ること」の「既知の限界を説明に書く」に従い、`nfr_source.rs` の確かめの前のコメントに、既知の限界(空白を除いた部分一致なので `my_include!` のような別のマクロや、文字列・`/* */` のコメントの中の語にも当たる。`#[cfg_attr(…, path = "…")]` の形は見逃す)を 2 行足した。`functional-spec.md` §2.4 のコードの形には、この 2 行は書かれていなかった。テストの確かめの中身は §2.4 のとおり。
- **T4 の `#[path` の比べ方**: §2.4 の形のとおり、`#[path` も空白を除いた行で比べるようになった(前は空白を除かない行で比べていた)。止める範囲は広がる側だけで、今の製品のコードでは通る。
- **S4 の版の履歴の 3 つ目の文**: 元の文の「これらの定数も公開だった」は、段落から抜くと「これら」の指す先が離れるので、`inventory.md` §2.3 の表の文のとおり「上の crate の中の定数も公開だった」にした。S4 の 1 つ目の箇条書きは、一覧の中身が「今の形」の再公開の一覧に入ったので、`inventory.md` の表の文のとおり `envfile::{…}` と `plan::{…}` と略した。
- **S9 の括弧**: 「、1,000 ジョブの性能」を消し、「(05 の 16 章)」を残したので、「`--assume`(使われなかった仮定、既定の 0)(05 の 16 章)」と括弧が 2 つ続く形になった(`inventory.md` §3.4 のとおり)。
- **T4 の一時的な本番の変更**: 確かめのために `crates/cli/src/exit.rs` に 1 行を一時的に足して消した(上の「消したテストと移したテスト」)。コミットには入っていない。
