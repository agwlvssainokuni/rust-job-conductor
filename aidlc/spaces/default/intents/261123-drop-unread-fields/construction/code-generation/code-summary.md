# コード生成のまとめ(Code Summary): 読まれない項目の削除

計画 `code-generation-plan.md` の Step 1〜7 を実装した。基準は HEAD 172d8c6(コードの計画のコミット。コードは 79ab4d0 と同じ)。コードと仕様書のコミットは 5 件で、`functional-spec.md` §8 の並び 1〜5 と 1 対 1 に対応する(6 はコミットなしの確かめ)。ほかに、この記録のコミットが 1 件。push、タグ、リリース、版の更新はしていない(版は 0.9.8 のまま)。

方針(ユーザーの判断): テストのコードの都合で本番のコードを曲げない。型から読まれない項目を消し、テストは名前の並び、値、行き先など、残る項目で確かめる。本番に、テストのための要素を新しく足さない。

## コミットの一覧

| # | §8 | 項目 | ハッシュ | 内容 |
|---|----|------|----------|------|
| 1 | 1 | 1 | `9a51c93` | core: `FlowEdge.explicit` を消し、`target_position` は位置だけを返す(T1〜T5、§4.6 の T2 の 4 行。仕様書 S1〜S4) |
| 2 | 2 | 2a | `9fc8d95` | core: 計画の `EnvNameEntry` から `origin` を消し、名前だけの構造体 `EnvNameEntry { name }` にする(D1)。`merge_env_names` は `Vec<&str>` を返す(T6、★1。仕様書 S6〜S11。S5 は変えない) |
| 3 | 3 | 2b | `f90fe73` | core: `EnvOrigin` と `EffectiveEntry.origin` を消し、統合の 1 項目 `Item` を `(名前, 値)` にする(T7〜T9、★2、§4.6 の T7 の 6 行と T8 の 1 行。仕様書 S12〜S16) |
| 4 | 4 | 3a | `10b862e` | core・cli: `EnvFileContent.path` を消す(T10〜T13、T19、§4.6 の T10 の 1 行。仕様書 S17、S18、S21(D3)) |
| 5 | 5 | 3b | `b660947` | core・cli: `EnvEntry.line` を消す。重複キーの警告の行(`EnvWarning.line`)は変えない(T14〜T18、T20、§4.6 の T14 の 1 行。仕様書 S19、S20) |
| — | 6 | 確かめ | (コミットなし) | §7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy、カバレッジ(下の記録) |

各コミットで、失敗で止まる確認のスクリプト(`set -eu`)で、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked --no-run`(全テストのコンパイル)と、そのコミットで変えたクレートのテストを通した(整形は `cargo fmt --all`)。cli の src は変えていないので、1〜3 では core のテストを、cli のテストのファイルを変えた 4・5 では core と cli のテストを実行した。2 では、`--dry-run` の表示が変わらないことの念のため、cli の `plan_report.rs`(7 件)と `docs_dry_run.rs`(2 件)も実行した。最後に全テストを実行した。

## 件数の移り変わり(macOS、rustc 1.100.0 = MSRV、debug)

| 時点 | core 単体 | core 統合 | runtime 単体 | runtime 統合 | converter 単体 | converter 統合 | cli 単体 | cli 統合 | 合計 |
|------|-----------|-----------|--------------|--------------|----------------|----------------|----------|----------|------|
| 基準(172d8c6) | 112 | 270 | 69 | 128 | 52 | 64 + 1 無視 | 2 | 456 | **1,153 通過、1 無視、0 失敗** |
| 最後(b660947) | 112 | 270 | 69 | 128 | 52 | 64 + 1 無視 | 2 | 456 | **1,153 通過、1 無視、0 失敗**(見込みどおり。1,153 − 0 + 0) |

コミットごとの、実行したクレートの件数(失敗は常に 0):

| # | core | cli |
|---|------|-----|
| 1 | 382 | — |
| 2 | 382 | (`plan_report.rs` 7、`docs_dry_run.rs` 2) |
| 3 | 382 | — |
| 4 | 382 | 458 |
| 5 | 382 | 458 |

消したテスト 0 件、足したテスト 0 件(BR8.1)。

## テストの変更

### テスト名の一覧の比較

`cargo test --locked --workspace -- --list 2>/dev/null | grep ": test$" | sort` を基準(172d8c6)と最後(b660947)で比べた(どちらも 1,154 行。無視の 1 件を含む)。差は §4.5 の ★ 2 件の名前の変更だけ。消えたテスト、増えたテストは無い。

| ID | 前の名前 | 後の名前 |
|----|----------|----------|
| ★1(T6) | core `plan_types.rs` `計画は環境変数の値を持たない_名前と出どころだけ` | `計画は環境変数の値を持たない_名前だけ` |
| ★2(T7) | core `env_merge.rs` `names_は値を含まず出どころを持ち_additions_は並びの順` | `names_は値を含まず_additions_は並びの順` |

### 書き換えたテスト(T1〜T20)

| ID | ファイル | 変えた内容 |
|----|----------|------------|
| T1 | core `graph_edges.rs` | 補助 `edge` の引数 `explicit` と項目を消し、14 の呼び出しから最後の `true`/`false` を消した(整形で、短くなった 3 つの呼び出しが 1 行にまとまった)。`:141` のコメントは残した |
| T2 | core `graph_lookup_perf.rs` | `explicit` の確かめの 4 行を消した(§4.6) |
| T3〜T5 | core `graph_drawable.rs`、`graph_types.rs`、`graph_common/mod.rs` | `FlowEdge { .. }` から `explicit: true,` を消した |
| T6 | core `plan_types.rs` | 名前と出どころの組を、`e.name` の並び(`["FROM_FILE", "TOKEN", "CONDUCTOR_PIPELINE", "CONDUCTOR_JOB_ID", "CONDUCTOR_ATTEMPT"]`)の比較にした。import から `EnvOrigin` を消した。★1 |
| T7 | core `env_merge.rs` | 補助 `origins` を消した。出どころの確かめの 6 行を消した(§4.6)。★2 のテストは `names(&env)`、`names(&env).contains(&"P")`、`env.additions().len() == env.entries.len()` にした。import から `EnvOrigin` を消した |
| T8 | core `env_merge_perf.rs` | `assert_order` の `e.origin == Builtin` を消した(§4.6)。`a3a` は `e.value.starts_with("p.env-")`、`a3b` は `e.value.starts_with("j.env-")` にした。import から `EnvOrigin` を消した |
| T9 | core `env_types.rs` | `EffectiveEntry` から `origin` を、import から `EnvOrigin` を消した |
| T10 | core `env_file_parse.rs` | `assert_eq!(c.path, "t.env")` を消した(§4.6) |
| T11〜T13 | core `env_merge_perf.rs`、`plan_perf.rs`、`plan_types.rs` | `EnvFileContent { .. }` から `path` を消した(T11 の補助 `content` の引数 `path` は値の接頭辞に使うので残した) |
| T14 | core `env_parse_perf.rs` | `assert_eq!(entry.line, HALF + k + 1)` を消した(§4.6) |
| T15〜T18 | core `env_merge_perf.rs`、`plan_perf.rs`、`plan_types.rs`、`env_types.rs` | `EnvEntry { .. }` から `line` を消した |
| T19、T20 | cli `plan_report.rs` | `EnvFileContent` から `path`、`EnvEntry` から `line` を消した |

期待する出力の文字列は、どのテストでも変えていない。テストのファイルの変更は 12 ファイル(core 11、cli 1)で、T 表のファイルだけ。src の `#[cfg(test)]` の単体テストは変えていない。

### 消した確かめの行(§4.6。合計 13 行)

| 消した行 | コミット | 同じ意味を確かめて残る行 |
|----------|----------|--------------------------|
| `graph_lookup_perf.rs` の `assert!(success.explicit)`、`assert!(warning.explicit)` | 1 | 同じ繰り返しの `to` と `kinds` |
| `graph_lookup_perf.rs` の `assert!(!j998.explicit)`、`assert!(!j999.explicit)` | 1 | `j998`・`j999` の `to` と `kinds` |
| `env_merge.rs` のコメント「層ごとの出どころ」、`let origin`、`assert_eq!` 4 行(6 行) | 3 | 同じテストの値の確かめ(`V`、`W`、`P`、`Q`) |
| `env_merge_perf.rs` の `assert_eq!(e.origin, EnvOrigin::Builtin)` | 3 | 直前の `assert_eq!(e.name, b)` |
| `env_file_parse.rs` の `assert_eq!(c.path, "t.env")` | 4 | 同じテストの警告の `file` の確かめ |
| `env_parse_perf.rs` の `assert_eq!(entry.line, HALF + k + 1)` | 5 | 直前の名前と値の確かめ、重複の警告 5,000 件の確かめ |

## 本番の変更(すべて core)

- `src/graph/mod.rs`(`FlowEdge.explicit` と説明)、`src/graph/edges.rs`(`target_position -> usize`、`pairs` は `(位置, 種類)`)
- `src/plan/mod.rs`(`EnvNameEntry { name }`、`use crate::envfile::EnvOrigin` を消す、説明を「名前だけ」に)、`src/plan/simulate.rs`(`EnvNameEntry { name }` を作る、説明)
- `src/envfile/merge.rs`(`merge_env_names -> Vec<&str>`、`Item = (名前, 値)`、`collect_vars`・`collect_files` の `origin` の引数を消す)、`src/envfile/mod.rs`(`EnvOrigin`、`EffectiveEntry.origin`、`EnvFileContent.path`、`EnvEntry.line` と、手書きの `Debug` の該当の項目)、`src/envfile/parse.rs`(`path`・`line` を書く 2 行)、`src/lib.rs`(再公開から `EnvOrigin` を消す)

cli の src、runtime、converter は変えていない(`git diff 172d8c6 -- crates/cli/src crates/runtime crates/converter` は空)。

## 仕様書の変更

- rust: `02-architecture.md`(S12)、`03-definition-format.md`(S17、S19)、`05-execution-flow.md`(S1〜S3、S6〜S8、S13、S14、S18、S19、S21)、`11-quality-and-build.md`(S15)、`12-rebuild-guide.md`(S9)
- language-agnostic: `02-architecture.md`(S10)、`03-definition-format.md`(S20)、`05-execution-flow.md`(S4、S10、S11、S16、S18、S20)、`12-rebuild-guide.md`(S9)
- S5(rust 02 の再公開の `EnvNameEntry`)は D1 の決定で変えていない。版の注記は足していない。凍結した仕様書は変えていない。

## 確かめ(Step 6)

### 完了の判定の検索(`functional-spec.md` §7。最後のコミット b660947 で実行)

| 検索 | 期待 | 結果 |
|------|------|------|
| `grep -rn "explicit" crates/core crates/cli crates/runtime` | 0 件 | 0 件 |
| `grep -rn "EnvOrigin\|\.origin\b\|origin:" crates/core crates/cli crates/runtime` | 0 件 | 0 件 |
| `grep -n -A 3 "pub struct EnvNameEntry" crates/core/src/plan/mod.rs` | 項目は `pub name: String` だけ | `pub name: String` だけ |
| `grep -rn "名前と出どころ\|出どころを持" crates` | 0 件 | 0 件 |
| `git diff --stat 79ab4d0 -- crates/cli/src` | 空 | 空 |
| `grep -n "pub path\|pub origin\|pub explicit" …envfile/mod.rs …graph/mod.rs …plan/mod.rs` | 0 件 | 0 件 |
| `grep -n "pub line" crates/core/src/envfile/mod.rs` | `EnvFileError` と `EnvWarning` の 2 件 | 2 件(`:141` `EnvFileError.line: Option<usize>`、`:178` `EnvWarning.line: usize`) |
| `grep -rn "^ *path: \|c\.path\|entry\.line\|line: i + 1\|line: 1,\|line: 3," crates/core/tests crates/cli/tests/plan_report.rs` | 0 件 | 0 件 |
| `grep -n "line: line_no" crates/core/src/envfile/parse.rs` | 警告の 1 件(`:88`) | 1 件(`:88`) |
| `grep -rn "EnvOrigin" docs/spec/rust docs/spec/language-agnostic` | 0 件 | 0 件 |
| `grep -rn "EnvNameEntry" docs/spec/rust docs/spec/language-agnostic` | rust の 4 行だけ、`origin` を含まない | 4 行(rust 02:42、05:723、05:859、05:860)。`origin` を含まない |
| `grep -rn "list<(name, origin)>\|env_names: list<string>" docs/spec/language-agnostic` | 0 件 | 0 件 |
| `grep -rn "origin" docs/spec/rust docs/spec/language-agnostic \| grep -v "origin/main"` | 0 件 | 0 件 |
| `grep -rn "explicit" docs/spec/rust docs/spec/language-agnostic` | 0 件 | 0 件 |
| `grep -rn "出どころ" docs/spec/rust docs/spec/language-agnostic` | 00-README の 2 件だけ | 2 件(rust 00-README:75、la 00-README:91) |
| `grep -rn "名前、値、行\|name, value, line\|EnvFileContent { path\|path: String, entries\|解決済みのパス、項目\|\`path\` は与えられた" …` | 0 件 | 0 件 |
| `grep -n "EnvWarning { file, line, name, kind" docs/spec/rust/03-definition-format.md` | 1 件 | 1 件(:364) |
| `grep -n "警告(重複キー: ファイル、後ろの行、名前)" docs/spec/language-agnostic/03-definition-format.md` | 1 件 | 1 件(:530) |
| `grep -n "add(記述, content)" docs/spec/rust/05-execution-flow.md` | 0 件 | 0 件 |
| `git diff --stat 79ab4d0 -- docs/spec/{go,python,typescript,csharp,powershell}` | 空 | 空 |

### そのほかの確かめ

| 確かめ | 結果 |
|--------|------|
| `cargo fmt --all -- --check` | 通る |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`(macOS) | 通る |
| `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` | 通る |
| `cargo test --locked --workspace` | 1,153 通過、1 無視、0 失敗 |
| `cargo doc --workspace --no-deps --locked` の警告 | 5 件(core 1、runtime 4)。基準と同じ(`check_command` から private の `split` へのリンク、redundant explicit link target、unresolved link 3 件) |
| `git diff 172d8c6 -- crates/cli/src crates/runtime crates/converter` | 空 |
| `git diff 172d8c6 -- '*Cargo.toml' Cargo.lock` | 空(依存と版は変わらない) |
| 足した行の `unsafe`・`cfg(unix)`・`cfg(windows)`・`panic!(` | 0 件 |
| `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | 通る。行カバレッジ 98.08%(TOTAL 9,315 行、未実行 179 行) |
| MSRV | ローカルの rustc が 1.100.0(MSRV)で、上のビルドとテストはすべてこの版 |

## 計画からのずれ

- なし(計画の書き換えはしていない。チェックボックスだけを付けた)。T1〜T20 の外の既存のテストの変更は要らなかった。
- 細かい点(記録): T7 で `env_merge.rs` の確かめの 6 行を消したとき、その塊の後ろの空行 1 行も一緒に消した(空行が 2 行続かないようにするため。確かめの行ではない)。T1 と import の折り返しは `cargo fmt` の整形で行の形が変わった(中身は同じ)。
- 仕様書 S14・S16 の擬似コードでは、`inventory.md` のとおり、層の名前を行末の注釈(`# [pipeline].env_file` など)で示した。自動の変数の層は、最初の行に `# 自動の変数` の注釈を付けた。
- 作業のファイル(確認のスクリプト、テスト名の一覧、出力)は `target/drop-uf/` の下に置き、終わったあと消した。
