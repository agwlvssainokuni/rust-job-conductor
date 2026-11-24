# 機能の仕様(Functional Spec): 小さな直し

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 B、Q3 A)。**変える場所の正確な一覧(説明の C1〜C4、仕様書の S1〜S14、ほかの章の食い違い X1〜X5、テストの重複の比べ方、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD 6ce7d45 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表。消すテストと、同じことを確かめて残るテスト)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

決めてほしい点(§10 の D1〜D8)は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。この文書の D の箇所は、その決定のとおりの形を書いている。§4 の T 表(T1〜T4)と §4.6 の消すテスト 3 件は、設計の承認で認めてもらう(承認待ち)。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない**(ユーザーの判断)。本番のコードで変えるのは、説明(ドキュメントコメント)の行だけ(C1、C2、C3、C4。C3 は D1 の決定)。関数の本体、型、可視性は変えない。
- 利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` の出力)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- 消すテストは 3 件(core 1、cli 2)。どれも、同じ場面か同じことを確かめて残るテストがある(§4.6)。移すテストは 1 件(名前と本体を変えない)。足すテストは 0 件。名前を変えるテストは 0 件(★ なし)。`signals_order.rs` は残す [Q2]。
- ソースの検査は、製品のコードの `#[path` に加えて、`include!`・`include_str!`・`include_bytes!` をすべて止める。テストの数と名前は変えない [Q3]。
- 仕様書は `docs/spec/rust` を直す。言語に依らない仕様書は、テストのファイルの名前が変わる 1 か所だけ(D3。決定: 直す)。凍結した仕様書は触らない。仕様書の版の注記は、rust 02 の 3 段落で「今の形」と「版の履歴」に分ける(中身は移すだけ)。ほかの所に版の注記を足さない。
- 小さなコミットの並び(1 項目 1 コミット)で、どのコミットでもビルド、`clippy -D warnings`、テストが通る [memory:M1] [memory:M4]。承認は、この設計とコードの計画の 1 回。結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない [memory:M5]。`unsafe` と製品のコードの `cfg(unix)`/`cfg(windows)` を足さない。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 変えるもの | 本番の形の変わり方 | テスト |
|------|------------|--------------------|--------|
| 1 古い説明 | `core/src/plan/mod.rs:18`(C1)、`core/src/types.rs:440`(C2)の `decide_before_run` を `decide_from_state` に。`core/src/lib.rs:38`(C3。D1 の決定)。仕様書 rust `05:341`・`05:850`(D5 の決定) | 説明だけ | 変わらない |
| 2 仕様書 rust 02 | S1(`resolve_run_id` の型)、S2・S3(`consistency`)、S4〜S6(版の注記を分ける)、S7(D2 の決定)。仕様書 rust `06:5`(X2。D6 の決定) | — | — |
| 3 テストの重複 | core の `plan_perf.rs` の 1 件を消し(T1)、残る 1 件を `plan_simulate_perf.rs` へ移して `plan_perf.rs` を消す(T2)。cli の `src/tests/init_types.rs` の 2 件を消す(T3)。`cli/src/types.rs:221` の説明(C4)。仕様書 S8〜S11、S12(D3 の決定) | 説明だけ(C4) | 3 件減る、1 件移る |
| 4 ソースの検査 | `cli/tests/nfr_source.rs` の `テスト専用の経路を実行ファイルに持たない` の最後の確かめ(T4)。仕様書 S13、S14 | — | 数と名前は変わらない |

### 2.1 項目 1 の形

```rust
// crates/core/src/plan/mod.rs:18-19(直した後)
//! ファイルにも、プロセスにも、環境変数にも触れない。実行と同じ判断(`decide_from_state`、
//! `classify_and_decide`)を、実行の代わりに、仮定した終了コードで進める(…今のまま…)。

// crates/core/src/types.rs:440(直した後)
    /// 以降のジョブは、スケジューラが呼んだときに `decide_from_state` が `Skip(AfterEnd)` を
```

- 名前はリンクにしない(今の書き方のまま)。`cargo doc` の警告は基準の 5 件のまま(写しで確かめた)。
- 完了の判定: `grep -n "decide_before_run" crates/core/src/plan/mod.rs crates/core/src/types.rs` が 0 件。`git grep -n "decide_before_run" -- crates ':!crates/core/src/flow/decide.rs'` が 0 件(HEAD は 2 件)。

### 2.2 項目 2 の形(仕様書 rust 02)

- S1: 2.4 の関数の表の `resolve_run_id` の行のシグネチャを `(option: Option<&RunIdArg>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>` にする(`crates/cli/src/run_id.rs:37-41` のとおり)。
- S2・S3: 2.1 の「再公開(`lib.rs`):」の一覧の先頭に `consistency::{ConsistencyResult, MismatchKind, check_consistency}` を、「公開モジュール:」の一覧の先頭に `consistency` を足す。再公開の一覧は `lib.rs` の `pub use` の並び(`consistency`、`definition`、`envfile`、`error`、`flow`、`plan`、`run_id`、`types`)にする。42 行目の「v0.9.0 以降は、さらに `envfile::{…}` と `plan::{…}` を再公開する」は今の形の再公開の一覧でもあるので、版の履歴に分けるとき、`envfile` と `plan` の一覧(中身はそのまま)を「今の形」の再公開の一覧に統合して残し、版の履歴には「v0.9.0 以降は、…も再公開する」の注記だけを置く(S4)。
- S4〜S6: 3 段落から版の注記をすべて抜き、各段落の直後に「版の履歴(古い順):」と時系列の箇条書きを置く(`inventory.md` §2.3 の表の文のとおり。中身は移すだけ)。並べ方は、注記が述べる変わり目の版の古い順。
- 直す段落の一覧(この 3 つだけ。2 章のほかの段落・表の行は変えない。D7):

| 段落 | 節 | 抜く注記の数 | 版の履歴の箇条書き |
|---|---|---|---|
| 42 行目の「再公開(`lib.rs`):」 | 2.1 `conductor-core` | 3 | 3 |
| 105 行目の「再公開(`lib.rs`):」 | 2.2 `conductor-runtime` | 3 | 3 |
| 137 行目の「再公開(`lib.rs`):」 | 2.4 `conductor-cli` | 1 | 1 |

- 「今の形」の段落に「v0.」で始まる版の注記が残らないことを確かめる検索(§7 にも書く):

```sh
grep -c '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md               # 3(今の形の 3 段落)
grep -n '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md | grep -c 'v0\.'   # 0(HEAD は 3)
grep -c '^版の履歴(古い順):$' docs/spec/rust/02-architecture.md            # 3
awk '/^版の履歴\(古い順\):$/{h=1;next} h&&/^- /{print NR": "$0} h&&/^#/{h=0}' docs/spec/rust/02-architecture.md   # 箇条書き 7 行。各節の中で版が古い順
```

### 2.3 項目 3 の形

`plan_simulate_perf.rs` の変わり方(T2):

- モジュールの説明の最後に、空の `//!` 行と、次の 2 行を足す:「あわせて、すべてのジョブが `env` と `env_file` を持つ 1,000 ジョブの計画を、1 回の計測で 1 秒以内に作れることを確かめる(…)。」括弧の中は、`plan_perf.rs:15` のモジュールの説明の括弧の中(Unit と規則の番号)をそのまま写す。
- import: `use common::median;` を `use common::{add_env_file, median};` に。`conductor_core` の import に `EnvEntry`、`EnvFileContent` を足す(`cargo fmt` が並べる)。
- ファイルの最後(P2 の後)に、`plan_perf.rs:30-79` の `#[test]` と関数を、一字も変えずに置く。`N` は両ファイルで同じ `1000`。`Path` は P2 が使うので残る。
- `plan_perf.rs` を `git rm` する(T1 で 1 件を消した後のコミットで)。

`init_types.rs` の変わり方(T3):

```rust
//! `init` の型(…1 行目は今のまま…)のテスト。
//!
//! オプションの既定(`InitOptions::default()` と、引数から取り出した既定)は、`args_grammar.rs` の
//! `init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` が確かめる。

use std::ffi::OsString;

use crate::types::InitTarget;

#[test]
fn ハイフンだけが標準出力でそれ以外はファイル() { /* 今のまま */ }
```

- C4: `crates/cli/src/types.rs:221` の「指定のない値で、コマンドを作る(引数の解釈と、テストが使う)。」を「指定のない値で、コマンドを作る(引数の解釈が使う)。」に。

### 2.4 項目 4 の形(`tests/nfr_source.rs`。T4)

```rust
    // 製品のファイル(src/tests/ の外)から、#[path] や include の 3 つのマクロで、別のファイル
    // (src/tests/ やテストのデータ)を読む道が無い
    let paths = find(
        &|line| {
            let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            ["#[path", "include!", "include_str!", "include_bytes!"]
                .iter()
                .any(|word| compact.contains(word))
        },
        &|path| is_unit_test(path),
    );
    assert!(paths.is_empty(), "{paths:#?}");
```

- 今の `:160-162`(`#[path` だけを見る 3 行)を、上の形に置き換える。止める語は D4 の決定(マクロの名前と `!`。区切りの `(`・`[`・`{` のどれでも、`include! (` のように空白があっても当たる。推奨のとおり。ユーザー、2026-11-25)。
- `include!` は `include_str!`・`include_bytes!` の部分の文字列ではないので、3 つを並べる。
- モジュールの説明 `:19-20` の「製品のファイルに `#[path` が無いことを確かめる」を、「製品のファイルに `#[path`・`include!`・`include_str!`・`include_bytes!` が無いことを確かめる」にする(行が長くなるので 2 行に分ける)。
- テストの数(6 件)と名前は変えない(★ なし。`inventory.md` §4.2)。
- 既知の限界: 空白を除いた部分一致なので、`my_include!` のような別のマクロ、文字列や `/* */` のコメントの中の語にも当たる(誤検出の側。`//` 以降のコメントは除く)。`#[cfg_attr(…, path = "…")]` の形の `path` は見逃す。今の製品のコードには、どの形も無い。
- 写しで、今の製品のコードで 6 件が通り、本番のファイルに `include_str! ["../Cargo.toml"]` を足すとこのテストだけが失敗することを確かめた(`inventory.md` §0.2)。

## 3. 実物で確かめて初期の記述と違った点、範囲から外したもの

- **記述と違った点・足した点**(実物に従った):
  - 項目 1 の `decide_before_run` を公開のものとして書く説明は、コードでは初期の記述の 2 か所だけ。仕様書 rust 05 に、呼ぶ側の関数として書く 2 か所がある(X1。D5)。同じ種類の古い説明が core の `lib.rs:38` にある(C3。D1)。
  - 項目 2 の 105 行目の段落の非公開のモジュールの一覧が欠けている(S7。D2)。rust 06 の実装の所在の一覧が古い(X2。D6)。
  - 項目 3 の `plan_perf.rs` の消すテストは、P2 と定義・ステート・入力・確かめが同じで、P2 の上限の方が厳しい(`inventory.md` §3.1)。`plan_perf` は言語に依らない仕様書 11 章の、1,000 ジョブの計画の性能の受け入れテスト(979 行目)の根拠の欄に名前がある(S12。D3)。
  - 項目 3 で `init_types.rs` の 2 件を消すと、`CliCommand::new` の説明の「テストが使う」が古くなる(C4)。import も 1 つに減らす。
- **範囲から外したもの**: テストのファイルの統合(`init_types.rs` の残る 1 件の置き場所を含む)、補助の写しの解消、init のひな形の原本の重複、本番の小さなモジュールの統合。rust 02 のほかの段落・表の行と、ほかの章の版の注記(D7)。

## 4. 消す・移す・変えるテストの一覧(T1〜T4。ユーザーが設計の承認で認めた、2026-11-25)

Q1: A に従い、消すテストには、同じことを確かめて残るテストを添える(§4.6)。移すテストと残るテストの確かめる中身(呼ぶ関数、入力、期待の値)は変えない。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §6。

```sh
git grep -n "plan_perf\|plan_simulate_perf\|init_types\|signals_order" -- ':!aidlc'
git grep -n "CliCommand::new\|InitOptions::default\|InitTarget::from_path" -- crates
grep -c "#\[test\]" crates/core/tests/plan_perf.rs crates/core/tests/plan_simulate_perf.rs crates/cli/src/tests/init_types.rs crates/cli/tests/nfr_source.rs   # 2、2、3、6
cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$"
```

### 4.1 T1: 消す(core 1 件)

| ID | ファイル(HEAD) | テスト名 | 扱い |
|----|----------------|----------|------|
| T1 | `crates/core/tests/plan_perf.rs:81-134`(前の空行を含む) | `終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる` | `#[test]` から関数の終わりまでを消す。使われなくなる import(`Path`、`JobExecution`、`RunMode`、`apply_outcome`、`classify_and_decide`、`initial_state`)も消す(残すと `unused_imports` で clippy が止まる)。消すテストは 1 回の計測で 1 秒以内、残る P2 は 5 回の中央値で 150 ms 以内なので、残るテストの方が強い |

### 4.2 T2: 移す(core 1 件。名前と本体は変えない)

| ID | 元(HEAD) | テスト名 | 移す先 | 変える行 |
|----|-----------|----------|--------|----------|
| T2 | `crates/core/tests/plan_perf.rs:30-79` | `千個のジョブがすべて実行する計画は_1秒以内に作れる` | `crates/core/tests/plan_simulate_perf.rs` の最後 | §2.3 の import 2 か所とモジュールの説明。本体は一字も変えない。`plan_perf.rs` を消す |

- パスは変わらない(統合テストの `--list` は関数の名前だけを出す)。実行ファイルは `plan_perf` から `plan_simulate_perf` に変わる。

### 4.3 T3: 消す(cli 2 件)

| ID | ファイル(HEAD) | テスト名 | 扱い |
|----|----------------|----------|------|
| T3a | `crates/cli/src/tests/init_types.rs:21-30` | `init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml` | 消す |
| T3b | 同 `:44-48` | `コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ` | 消す |
| — | 同 `:32-42` | `ハイフンだけが標準出力でそれ以外はファイル` | 残す(変えない) |

- import を `use crate::types::InitTarget;` にし、モジュールの説明に、既定を確かめる場所の 2 行を足す(§2.3)。

### 4.4 T4: ソースの検査の確かめを広げる(cli。数と名前は変えない)

| ID | ファイル | テスト名(変えない) | 変える内容 |
|----|----------|--------------------|------------|
| T4 | `crates/cli/tests/nfr_source.rs` | `テスト専用の経路を実行ファイルに持たない` | §2.4。止める語に `include!`・`include_str!`・`include_bytes!` を足し、空白を除いて比べる。モジュールの説明とコメントを合わせる |

### 4.5 残すテスト(変えない)

- cli の `tests/signals_order.rs`(Q2: B)、`tests/signals_real.rs`。
- core の `plan_simulate_perf.rs` の P1・P2(本体は変えない。ファイルの import とモジュールの説明だけ変わる)。

### 4.6 消すテストと、同じことを確かめて残るテスト(ユーザーが設計の承認で認めた、2026-11-25)

| 消すテスト | 確かめていたこと | 同じことを確かめて残るテスト | 残るテストの方が強い・同じ所 | 残るテストで確かめられない点 |
|------------|------------------|------------------------------|-----------------------------|------------------------------|
| core `plan_perf::終点の後に千個近いジョブをスキップする計画は_1秒以内に作れる` | `run j0` で `@end` に着いた後のステート(`j0` 成功、`end_reached`、残り未実行)から、1,000 ジョブの計画を作ると、先頭が `AlreadyExecuted`、残り 999 個が `SkippedAfterEnd`、終わりが `EndReached`。1 回の計測で 1 秒以内 | core `plan_simulate_perf::p2_終点のあとの_999_個のスキップの計画は_150ms_以内` | 定義の文字列・ステートの作り方・入力・4 つの確かめが同じ(`inventory.md` §3.1)。時間は 5 回の中央値が 150 ms 以内で、1 秒より厳しい | なし(消すテストは 1 回の計測、P2 は 5 回の中央値で、上限も厳しい) |
| cli `tests::init_types::init_のオプションの既定は_my_pipeline_と_force_なしと_conductor_toml` | `InitOptions::default()` の名前が `my-pipeline`、`force` なし、出力先が `File("conductor.toml")` | cli `tests::args_grammar::init_の既定は_conductor_toml_と_my_pipeline_で_force_なし` | `init` だけの引数から取り出したオプションが `InitOptions::default()` と等しく、かつ出力先・名前・`force` の 3 つの値を同じく確かめる(等しいので、既定の 3 つの値も確かめたことになる) | なし |
| cli `tests::init_types::コマンドから取り出した_init_のオプションの既定は_init_のオプションの既定と同じ` | `CliCommand::new(Init).init_options()` が `InitOptions::default()` と等しい | 同上 | 引数の解釈は `CliCommand::new(kind)` から作り(`args.rs:488`)、`init` だけの引数は何も変えないので、同じ等しさを、本番の解釈の経路を通して確かめる | 引数の解釈を通さない `CliCommand::new(Init)` を直接呼んで確かめる所が無くなる(`CliCommand::new` は引数の解釈の中でだけ使われ、`init` だけの引数で解釈が何も変えないことに頼る) |

### 4.7 名前を変えるテスト(★)

**なし**。移す 1 件と、確かめを広げる `nfr_source.rs` の 1 件は、名前を変えない。

## 5. 足すテスト

なし。項目 4 の確かめは、既存のテストの中の 1 つの確かめを広げるだけで、テストの数は増えない。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 112 | **269**(270 − 1) | 381 | −1(T1) |
| runtime | 69 | 128 | 197 | 0 |
| converter | 96 | 19 + 1 無視 | 115 + 1 無視 | 0 |
| cli | **162**(164 − 2) | 294 | 456 | −2(T3) |
| 全体 | | | **1,149 通過、1 無視、0 失敗** | **−3** |

基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた(`inventory.md` §0.2)。Windows でも、消す 3 件と移す 1 件は OS に依らないので、差は同じ −3。

## 7. 確かめ方

- 実装の前と後で、`cargo test --workspace --locked` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,152 − 消したテスト 3 + 足したテスト 0 = 1,149**(と無視 1)」に一致すること。移した 1 件は件数に数えない [Q1]。
- テスト名の一覧の差: 前後で `cargo test --workspace --locked -- --list 2>/dev/null | grep ": test$" | sed 's/: test$//; s/.*:://' | sort` を比べ、差が、消した 3 件の名前(前にだけある)だけであること。移した 1 件は、実行ファイルごとの件数(`plan_perf` 2 → 無し、`plan_simulate_perf` 2 → 3)で確かめる。
- 移したテストの本体が変わっていないこと: 実装の前の `plan_perf.rs` と後の `plan_simulate_perf.rs` で、`sed -n '/^fn 千個のジョブがすべて実行する/,/^}/p'` の出力が一致する。
- **完了の判定**。最後のコミットのあとで、次をすべて行う:

```sh
# 項目 1
git grep -n "decide_before_run" -- crates ':!crates/core/src/flow/decide.rs'   # 0 件
# 項目 2(仕様書 rust 02)
grep -n "resolve_run_id" docs/spec/rust/02-architecture.md | grep -c "Option<&RunIdArg>"   # 1
grep -n '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md | grep -c 'consistency::{ConsistencyResult, MismatchKind, check_consistency}'   # 1
grep -c '公開モジュール: `consistency`、`definition`' docs/spec/rust/02-architecture.md    # 1
grep -c '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md                              # 3
grep -n '^再公開(`lib.rs`)' docs/spec/rust/02-architecture.md | grep -c 'v0\.'             # 0(今の形に版の注記が残らない)
grep -c '^版の履歴(古い順):$' docs/spec/rust/02-architecture.md                           # 3
awk '/^版の履歴\(古い順\):$/{h=1;next} h&&/^- /{print NR": "$0} h&&/^#/{h=0}' docs/spec/rust/02-architecture.md   # 7 行。各節の中で古い順
# 項目 3
test ! -e crates/core/tests/plan_perf.rs
grep -c "#\[test\]" crates/core/tests/plan_simulate_perf.rs crates/cli/src/tests/init_types.rs   # 3、1
git grep -n "plan_perf" -- ':!aidlc'                       # 0 件
grep -n "init_types.rs" docs/spec/rust/11-quality-and-build.md   # S11 の文
# 項目 4
grep -n 'include!\|include_str!\|include_bytes!' crates/cli/tests/nfr_source.rs   # 確かめの 1 行と説明
grep -rn "include\|#\[path" crates/cli/src | grep -v "^crates/cli/src/tests/"     # 0 件
grep -n "include" docs/spec/rust/11-quality-and-build.md                         # S13、S14
# 範囲の外が変わっていない
git diff --stat 6ce7d45 -- Cargo.lock '*Cargo.toml' .github README.md crates/runtime crates/converter crates/core/src/flow   # 空
git diff --stat 6ce7d45 -- docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell   # 空
git diff --stat 6ce7d45 -- crates/cli/tests/signals_order.rs crates/cli/tests/signals_real.rs   # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4)のまま。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた)。実行は CI の `test`(windows)で確かめる。
- 本番の形: `cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings` が通る。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(写しで 98.06%)、MSRV 1.100 でのテスト(ローカルの rustc が 1.100.0)[memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。版の表記(`0.9.8`)は変えない。
- 利用者から見える動き: 本番の処理のコードが変わらない(変わるのは説明の行だけ。`git diff 6ce7d45 -- crates/*/src` の差がコメントの行だけ)ことと、全テストが期待を変えずに通ることで確かめる。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

各項目は互いに依らないので、どの順でもビルドとテストは通る。説明 → 仕様書 → テスト → 検査の順にする。テストのコミット(7〜10)で、そのテストに当たる仕様書の行も同じコミットで直す。

**各コミットの確かめ**: `cargo fmt --all`(整えてから)、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、変えたクレートのテスト(`cargo test -p <クレート> --locked`。件数がコミットごとの見込みと合うこと)。仕様書だけのコミット(3〜6)は、§7 の該当の検索。最後の確かめ(11)で全テストと §7 を行う。

| # | コミット | 変えるもの | 全体の件数の見込み |
|---|----------|------------|--------------------|
| 1 | core: 計画と次の動きの説明の、起動前の判断の関数の名前を直す | C1、C2。仕様書 rust `05:341`、`05:850` の `decide_before_run` を `decide_from_state` に(D5 の決定) | 1,152 |
| 2 | core: ステートの更新の関数を runtime が再公開するという説明を直す | C3(D1 の決定) | 1,152 |
| 3 | docs: 仕様書 rust 02 の `resolve_run_id` の引数の型を実装に合わせる | S1 | 1,152 |
| 4 | docs: 仕様書 rust 02 の core の公開の一覧に `consistency` を足す | S2、S3 | 1,152 |
| 5 | docs: 仕様書 rust 02 の公開の段落の版の注記を、版の履歴に分ける | S4、S5、S6、S7(D2 の決定) | 1,152 |
| 6 | docs: 仕様書 rust 06 の実装の所在を直す | X2(D6 の決定) | 1,152 |
| 7 | core: 計画の模擬の性能のテストと同じ場面の、終点の後のスキップの性能のテストを消す | T1 | 1,151(core 統合 269) |
| 8 | core: env と env_file を持つ計画の性能のテストを、計画の模擬の性能のテストのファイルへ移す | T2、`plan_perf.rs` を消す。仕様書 S8、S9、S10(rust 11 の 217 行目の `plan_simulate_perf.rs` の行に、移したテスト(すべてのジョブが `env` と `env_file` を持つ 1,000 ジョブの計画の生成、1 秒以内)を足す)、S12(D3 の決定) | 1,151 |
| 9 | cli: 引数の文法のテストと重なる init の型のテスト 2 件を消す | T3、C4。仕様書 S11 | 1,149(cli 単体 162) |
| 10 | cli: ソースの検査で、製品のコードの include の 3 つのマクロも止める | T4。仕様書 S13、S14 | 1,149 |
| 11 | (確かめ)§7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、本番の形の clippy、カバレッジの記録 | コミットなし。記録はコード生成の記録に書く | — |

合計 **10 コミット**(D1・D6 の決定で 2・6 を含む。11 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。言語に依らない仕様書は S12(D3 の決定)のほかは変えない。
- 版の更新、タグとリリース。README。CI の設定。
- `crates/core/src/flow/decide.rs`、`signals_order.rs`・`signals_real.rs`、T 表の外のテスト。

## 10. 決めてほしい点

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | core の `lib.rs:38` の「`conductor-runtime` は、これらと `check_consistency` を、同じ名前で再公開する。」(実物は再公開しない。C3) | **直す**(コミット 2。「再公開しない(使う側は、この crate から使う)」に) | 直さない(初期の記述の範囲の外として、別の intent に回す) | 項目 1 と同じ種類(実物と食い違う説明 1 行)で、動きに関わらない。runtime の `lib.rs:37` と仕様書 rust 02 の 2.2 は、すでに「core の関数を使う」と書いており、core の説明だけが食い違う | 決定: 直す(コミット 2。推奨のとおり。ユーザー、2026-11-25) |
| D2 | 仕様書 rust 02 の 105 行目の段落の非公開のモジュールの一覧に、`bounded_read`・`temp_file`・`wait` が無い(S7) | **直す**(コミット 5 で、同じ段落を書き換えるときに「モジュールはすべて非公開(11 個を並べる)」にする) | 直さない(版の注記だけを動かす) | 同じ段落を書き換えるので、差は 1 文。runtime の `lib.rs` に `pub mod` は無く、「すべて非公開」が実物どおり | 決定: 直す(コミット 5 で「モジュールはすべて非公開」にする。推奨のとおり。ユーザー、2026-11-25) |
| D3 | 言語に依らない仕様書 11 章の、1,000 ジョブの計画の性能の受け入れテスト(979 行目)の根拠の欄 `plan_perf`(T2 でファイルが無くなる。S12) | **`plan_simulate_perf` に直す**(コミット 8) | 直さない(言語に依らない仕様書は触らない、という前の intent の方針に合わせる) | 前の intent で直さなかったのは、ファイル名とテスト名が変わらず根拠の欄が正しいままだったから。今回はファイル名が変わり、根拠の欄が存在しないファイルを指すことになる。凍結した仕様書ではない(この 3 回の intent でも直している) | 決定: `plan_simulate_perf` に直す(コミット 8。推奨のとおり。ユーザー、2026-11-25) |
| D4 | ソースの検査で止める語(T4) | **マクロの名前と `!`(`include!`・`include_str!`・`include_bytes!`)を、行の空白を除いて探す**。区切りが `[`・`{` の形や、`include! (` も止める | 初期の記述・Q3 の字句どおり `include!(`・`include_str!(`・`include_bytes!(` を、空白を除かずに探す | Rust のマクロは `(`・`[`・`{` のどれでも呼べ、`!` の後に空白も置ける。推奨は Q3 の「すべて止める」を、書き方によらず満たす。今の製品のコードには、どの形も 0 件で、文字列の中にもこれらの語は無い(写しで通ることを確かめた)。代案は `include_str! [..]` などを見逃す | 決定: マクロの名前と `!` を、行の空白を除いて探す(推奨のとおり。ユーザー、2026-11-25) |
| D5 | 仕様書 rust 05 の `:341`(`pipeline` は「`decide_before_run` で決める」)と `:850`(計画は「実行と同じ判断(`decide_before_run`、…)」) | **`decide_from_state` に直す**(コミット 1。C1 と同じ文の直し)。ほかの `decide_before_run`(4 章の見出し、`:191`、`:283`、`:899`、12 章)は、非公開の本体を説明しているので変えない | 直さない(仕様書は判定の本体の名前で書いてよい) | `:341` はすぐ下の擬似コードが `decide_from_state` を呼び、`:850` は C1 で直すコードの説明と同じ文。どちらも「呼ぶ側の関数」として書いており、02 章の「公開は `decide_from_state`」と食い違う | 決定: `05:341`・`05:850` を `decide_from_state` に直す(コミット 1。ほかは変えない。推奨のとおり。ユーザー、2026-11-25) |
| D6 | 仕様書 rust 06 の `:5` の実装の所在(`runtime/src/state/{…,record,consistency,…}.rs`。実物に無い 2 ファイル。X2) | **直す**(コミット 6。「`crates/runtime/src/state/{reader,writer,reset_all}.rs`、…。照合と記録は `crates/core/src/{consistency,flow/record}.rs`」) | 直さない(別の intent に回す) | 項目 2 と同じ種類(仕様書と実装の食い違い)で、1 行。02 章の 68 行目・280 行目は正しい所を書いている | 決定: 直す(コミット 6。推奨のとおり。ユーザー、2026-11-25) |
| D7 | rust 02 のほかの段落・表の行(`02:150`、`02:152` など)と、ほかの章(05、07、09 など)の版の注記 | **今回は直さない**(この 3 段落だけを分ける。初期の記述の範囲) | 2 章のすべての版の注記を分ける / 全章で分ける | ほかの注記は、表の行の括弧の中に機能の版(「v0.9.0 以降」)として書くものが多く、分け方(表に履歴の列を作るか、節の終わりにまとめるか)を決める必要がある。範囲が大きく、小さな直しの intent に入れると 1 コミットが大きくなる | 決定: 今回は直さない(3 段落だけ。推奨のとおり。ユーザー、2026-11-25) |
| D8 | 移すテストの名前(`plan_simulate_perf.rs` のほかの 2 件は `p1_`・`p2_` で始まる) | **変えない**(★ なし。件数と名前の差の確かめが単純で、言語に依らない仕様書の根拠の欄の書き方にも影響しない) | `p6_` などの番号を付ける(★ 1 件) | `p1`〜`p5` は性能の直しの intent の計測の番号で、このテストはその計測に含まれない(1 回の計測、上限 1 秒)。番号を付けると、別の番号の体系と混ざる | 決定: 変えない(★ なし。推奨のとおり。ユーザー、2026-11-25) |

D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-25)**: §4 の T 表(T1〜T4。消す 3 件と、§4.6 の同じことを確かめて残るテスト、移す 1 件、名前は変えない)、§2.4 のソースの検査の広げ方、§2.2 の版の注記の分け方。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 古い説明: core の `plan/mod.rs` と `types.rs` の説明に `decide_before_run` が残らない(`flow/decide.rs` の中は除く) | BR1.1、BR1.2 |
| SM2 | 仕様書 rust 02: `resolve_run_id` の型が実装と一致し、公開の一覧に `consistency` がある。版の注記は「今の形」と「版の履歴」に分かれ、版の並びが時系列。直す段落と、「今の形」に「v0.」が残らないことを確かめる検索を設計で決める | BR2.1、BR2.2、BR2.3、BR2.4 |
| SM3 | テストの重複: `plan_perf.rs` の 1 件を消し、残る 1 件を移して `plan_perf.rs` が無くなる。`init_types.rs` の 2 件を消す。`signals_order.rs` は残す。消すテストは、残るテストを添えて承認を取る | BR3.1、BR3.2、BR3.3、BR3.4、BR3.5 |
| SM4 | ソースの検査: `src/tests/` の外の製品のコードで `#[path`・`include!(`・`include_str!(`・`include_bytes!(` を見つけたら失敗する。今の製品のコードでは通る | BR4.1、BR4.2 |
| SM5 | 件数が「基準 − 消したテスト + 足したテスト」に一致し、テスト名の差が消したテストと移したテストだけ | BR9.2、BR9.3 |
| SM6 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM7 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
