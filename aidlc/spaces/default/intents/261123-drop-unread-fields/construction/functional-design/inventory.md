# 変える場所の一覧(Inventory): 読まれない項目の削除

HEAD 79ab4d0(drop-unread-fields の意図の把握の記録のコミット)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。初期の記述と実物が違ったところは「(記述との違い)」と書く。設計の要点、テストの変更の一覧(T…)、確かめの行を消すものの表、件数の見込み、コミットの並び、決めてほしい点(D…)は `functional-spec.md`。

方針(ユーザーの判断。前の intent の Q3 と、この intent の Q1〜Q3): **テストのコードの都合で本番のコードを曲げない**。本番に、テストのための要素を新しく足さない。消す項目を読む・書くテストは、残る項目(名前の並び、値、位置、辺の行き先と種類)で同じ意味を確かめる書き方にする。消す項目だけを確かめていた行は、項目と一緒に消える(`functional-spec.md` §4.6 に、同じ意味を確かめて残る行を添える)。利用者から見える動き(出力、終了コード、ステートファイル、`--dry-run` と `graph` の出力)は変えない。版は上げない。

決めてほしい点(`functional-spec.md` §10 の D1〜D3)は、ユーザーが決めた(2026-11-24)。D1 は**代案**(`EnvNameEntry { name }` の名前だけの構造体を残す。cli は変えない)、D2・D3 は推奨のとおり。この文書の「(D1)」〜「(D3)」は、その決定のとおりの形を書いている。★ 2 件の個々の名前の変更と、`functional-spec.md` §4.6 の確かめの行の削除(合計 13 行。コメントの行を含めて数える)は、設計の承認で認めてもらう(承認待ち)。

範囲の外(Q3: B と初期の記述): 定義の `format_version` と `description`(`PipelineDefinition` の項目として残す)、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分。

## 0. 基準と、設計の検証

### 0.1 テストの件数(`cargo test --locked --workspace`、macOS、rustc 1.100.0、debug、HEAD 79ab4d0)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 112 | 270 | 382 |
| runtime | 69 | 128 | 197 |
| converter | 52 | 64 通過 + 1 無視 | 116 通過 + 1 無視 |
| cli | 2 | 456 | 458 |
| 全体 | | | **1,153 通過、1 無視、0 失敗** |

前の intent(drop-test-only-code)の結果 1,153 と一致する。この intent に関わるファイルの件数(基準): core `graph_edges.rs` 9、`graph_lookup_perf.rs` 2、`graph_drawable.rs` 21、`graph_types.rs` 4、`env_merge.rs` 8、`env_merge_perf.rs` 2、`env_parse_perf.rs` 1、`env_file_parse.rs` 10、`env_types.rs` 6、`plan_types.rs` 3、`plan_perf.rs` 2。cli `plan_report.rs` 7。runtime と converter のテストは変えない。

数え方: 出力の `Running unittests src/lib.rs`(単体)と `Running tests/*.rs`(統合)の各 `test result:` の `passed` と `ignored` を、クレートごとに足した。テスト名の一覧は `cargo test --workspace --locked -- --list` の `: test` の行(1,154 行。無視の 1 件を含む)。作業の出力は `target/drop-uf/` の下に置き、終わったあと消した。

### 0.2 設計の検証(写しの木での試し)

T 表を検索だけに頼らないために、HEAD 79ab4d0 の写し(`git archive 79ab4d0` を `target/drop-uf/work` に展開)に、この一覧の全項目(本番)と `functional-spec.md` §4 の T 表を当て、次を確かめた。D1 の決定(代案)のあと、写しを作り直して、決定の形でもう一度すべてを確かめた(下の結果は、決定の形のもの。推奨の形 `Vec<String>` でも、件数と名前の差は同じだった)。写しは確かめたあと消した。リポジトリのコードは変えていない。仕様書(`docs/spec`)の書き換えは、コンパイルとテストに関わらない(`crates` に `docs/spec` を読むテストは無い)ので、写しには当てていない。

- 本番のコードだけを変えた段階(1、2a、2b、3a、3b を当てた形。D1 は決定の `EnvNameEntry { name }`)で、`cargo check --workspace --locked --all-targets --keep-going` のエラーは **src に 0 件**(cli の src も変えずに通る)、テストに 30 件。エラーの出た場所は、すべて T 表にある(`graph_edges.rs` の呼び出しの引数の数の誤りは、構造体の項目の誤りの陰で出ないが、`edge` の補助の定義の行 `:49` が出て、T1 で呼び出しをすべて直す)。
- T 表を当てたあと、`cargo fmt --all -- --check`(当てたあと `cargo fmt` で整形した形。整形で変わったのは `use` の並びの折り返しだけ)、`cargo clippy --workspace --all-targets --locked -- -D warnings` が、macOS と `--target x86_64-pc-windows-msvc` の両方で通る。
- `cargo test --workspace --locked` が **1,153 通過、1 無視、0 失敗**(core 112 + 270、runtime 69 + 128、converter 52 + 64、cli 2 + 456。§6 の見込みと一致)。
- テスト名の一覧の差(HEAD の写しの `cargo test -- --list` と比べた。どちらも 1,154 行)は、★ の 2 件の名前の変更だけ(`functional-spec.md` §4.5)。消えたテスト、増えたテストは無い。
- `cargo doc --workspace --no-deps --locked` の警告は、基準と同じ 5 件(core 1、runtime 4。この intent と無関係)。消す名前へのドキュメントのリンクは無い(`EnvOrigin` などは文の中の名前だけ)。
- 写しで変わったファイル: 本番 9(すべて core)、テスト 12(core 11、cli 1)。cli の src、runtime、converter は変わらない。

---

## 1. core の `FlowEdge.explicit`

- 今: `crates/core/src/graph/mod.rs:59-60`(説明と項目 `pub explicit: bool`)。書くのは `crates/core/src/graph/edges.rs` の `target_position`(`:26-41`。戻り値 `(usize, bool)` の 2 つ目)と `flow_edges`(`:59-77`、`:92`。2 つの辺を 1 つにまとめるときの「どちらかが `true`」)。
- 本番の読み取り: 0 件(cli の `report/graph_text.rs`・`graph_mermaid.rs`、`graph/layout.rs`、`graph/overlay.rs` は `from_job`・`to`・`kinds`・`skipped_jobs` だけを読む。`grep -rn "explicit" crates/*/src` は edges.rs と mod.rs だけ)。
- 目標: 項目と説明を消す。`target_position` は位置だけを返す(`-> usize`)。`TransitionResolution::Default` の区別は、位置(`index + 1`)を決めるのに残る(`End` は `jobs.len()`、`Job(t)` は `t`)。`flow_edges` の `pairs` は `(位置, 種類)` の組にする。並べ方、まとめ方、飛び越すジョブは変えない。
- 説明: `edges.rs:27` の「戻り値の 2 つ目は、遷移が書かれていたか。」を消す。
- テストの使用(T1〜T5): core `tests/graph_edges.rs:37-51`(補助 `edge` の引数と項目)とその呼び出し 14 か所、`tests/graph_lookup_perf.rs:146`、`:150`、`:156`、`:160`(確かめの行)、`tests/graph_drawable.rs:264`、`tests/graph_types.rs:32`、`tests/graph_common/mod.rs:135`(構造体を直接作る所)。runtime・converter・cli: 0 件。
- 仕様書: rust `05-execution-flow.md:738`(15.1 の `FlowEdge` の行)、`:754-758`(15.2 の `target_position` の擬似コード)、`:761`(「`explicit` = どちらかが `true`」)。language-agnostic `05-execution-flow.md:747`、`:761-765`、`:768`(同じ 3 か所)。
- 完了の判定: `grep -rn "explicit" crates/core crates/cli/src` が 0 件(converter には別の意味の `explicit` は無いが、検索から外す)。

## 2. core の `EnvOrigin` と `origin` の項目

### 2a. 計画の `EnvNameEntry.origin`(計画の名前を名前だけにする)

- 今: `crates/core/src/plan/mod.rs:96-101`(`EnvNameEntry { name, origin }`)、`:115-116`(`PlanEntry.env_names: Vec<EnvNameEntry>`)、`:25`(`use crate::envfile::EnvOrigin`)、`:21`(モジュールの説明「名前と出どころだけ」)。作るのは `plan/simulate.rs:305-312`(`merge_env_names` の `(name, origin)` から作る)、説明 `:290`。`merge_env_names` は `envfile/merge.rs:56-72`(戻り値 `Vec<(&str, EnvOrigin)>`)。再公開 `crates/core/src/lib.rs:110`。
- 本番の読み取り: cli `src/report/plan_text.rs:161-165` が `e.name` だけを読む(`--dry-run` の「環境変数」の行は名前だけ)。`origin` を読む所は 0 件(`Debug` の出力だけに出る)。
- 目標(D1。決定: 代案。ユーザー、2026-11-24): `EnvNameEntry` は名前だけの構造体 `EnvNameEntry { name: String }` として残し、`origin` の項目と `use crate::envfile::EnvOrigin` を消す。`PlanEntry.env_names: Vec<EnvNameEntry>` と再公開(`lib.rs:110`)は今のまま。`merge_env_names` は `Vec<&str>` を返し、`simulate.rs` は `EnvNameEntry { name: name.to_string() }` を作る。cli の `plan_text.rs` は変えない(`e.name` を読むまま)。
- 説明: `plan/mod.rs:21`(「名前と出どころだけ」を「名前だけ」に)、`:96`(「環境変数の名前と出どころ」を「環境変数の名前」に)、`:115`(「名前と出どころ」を「名前」に)、`simulate.rs:290`、`:305`(同じ)、`merge.rs:56`(「名前と出どころだけを返す」を「名前だけを返す」に)。
- テストの使用(T6): core `tests/plan_types.rs:21`(import)、`:54`(テスト名 ★)、`:87-101`(名前と出どころの組の比較を、`e.name` の並びの比較に)。`:37` の `env_names: Vec::new()` は変えずに通る。cli `tests/plan_report.rs` は表示の文字列で確かめるので、この項目では変えない。
- 仕様書: rust `05-execution-flow.md:724`、`:860`(`EnvNameEntry` の行)、`:861`(`env_names: Vec<EnvNameEntry>(…名前と出どころ)`)、`:899`(「名前と出どころ」)、`12-rebuild-guide.md:129`。language-agnostic `02-architecture.md:62`、`05-execution-flow.md:733`、`:867`(`list<(name, origin)>`)、`:904`、`12-rebuild-guide.md:145`。rust `02-architecture.md:42` の再公開の列の `EnvNameEntry` は、型を残すので変えない。
- 完了の判定: `grep -rn "origin" crates/core/src/plan` が 0 件、`grep -n "pub struct EnvNameEntry" -A 3 crates/core/src/plan/mod.rs` の項目が `name` だけ。

### 2b. `EffectiveEntry.origin` と `EnvOrigin`

- 今: `crates/core/src/envfile/mod.rs:264-272`(`EnvOrigin` の 5 つの変種)、`:279`(`EffectiveEntry.origin`)、`:287`(手書きの `Debug` の `origin` の項目)。書くのは `envfile/merge.rs:19`(import)、`:47-51`(`merge_env`)、`:74-75`(`Item` = `(名前, 値, 出どころ)`)、`:84-87`(`collect_layers`)、`:99-106`(`push_builtins`)、`:115`、`:121`(`keep_last_positions` の組の分解)、`:125-144`(`collect_vars`・`collect_files` の引数 `origin`)。再公開 `lib.rs:101`。
- 本番の読み取り: 0 件(runtime の起動と cli の流れは `additions()` の名前と値だけを使う。`origin` は `Debug` の出力だけに出る)。
- 目標: `EnvOrigin`、`EffectiveEntry.origin`、`Debug` の `origin` の項目、再公開を消す。`Item` は `(名前, 値)` の組にし、`collect_vars`・`collect_files` から `origin` の引数を消す。`push_builtins` は名前と値だけを足す。並べ方、「最後に残す」規則、自動の変数の扱いは変えない。`EffectiveEntry` の手書きの `Debug` は残し(値を `"***"` にする)、`name` と `value` の 2 項目にする。
- 説明: `merge.rs:56`(「名前と出どころだけを返す」を「名前だけを返す」に。2a のコミット)、`:74`(「(名前, 値, 出どころ)」を「(名前, 値)」に)、`envfile/mod.rs:264`(型と一緒に消える)。
- テストの使用(T7〜T9): core `tests/env_merge.rs:21`(import)、`:53-58`(補助 `origins`)、`:95-100`(層ごとの出どころの確かめの 6 行。コメント 1 行、`let origin` 1 行、`assert_eq!` 4 行)、`:173`(テスト名 ★)、`:178`、`:180`、`:181`、`tests/env_merge_perf.rs:30`(import)、`:106`、`:123`、`:147`、`tests/env_types.rs:24`(import)、`:42`。cli・runtime: 0 件。
- 仕様書: rust `02-architecture.md:42`(再公開の列の `EnvOrigin`)、`:283`(`envfile/mod.rs` の型の列の `EnvOrigin`)、`05-execution-flow.md:644`(`EnvOrigin` の行)、`:645`(`{ name, value, origin }`)、`:646`(「名前と出どころ(値を含まない要約)は `entries` から取り出せる」)、`:664-672`(14.3 の擬似コードの `origin`)、`11-quality-and-build.md:216`(「結果の中身(並び、出どころ)」)。language-agnostic `05-execution-flow.md:656`、`:657`、`:676-687`(14.3 の擬似コード)。
- 完了の判定: `grep -rn "EnvOrigin\|\.origin\b\|origin:" crates/core crates/cli crates/runtime` が 0 件。

## 3. core の `EnvFileContent.path` と `EnvEntry.line`

### 3a. `EnvFileContent.path`

- 今: `crates/core/src/envfile/mod.rs:206-207`(説明と項目)。書くのは `envfile/parse.rs:110`(`path: file.to_string()`)。
- 本番の読み取り: 0 件(cli の `ctx.rs:382` は `files()` の `warnings` だけ、`LoadedEnvFiles` は `by_ref` の位置で引く。警告と誤りの表示のファイル名は `EnvWarning.file`・`EnvFileError.file` が持つ)。
- 目標: 項目を消す。`parse_env_file(text, file)` の `file` は、誤りと警告のファイル名に使うので残る。
- テストの使用(T10〜T13、T19): core `tests/env_file_parse.rs:120`(確かめの行)、`tests/env_merge_perf.rs:45`、`tests/plan_perf.rs:51`、`tests/plan_types.rs:66`、cli `tests/plan_report.rs:416`(構造体を直接作る所)。
- 仕様書: rust `03-definition-format.md:364`(出力の形の `path`)、`:377`(手順 4 の「(`path` は与えられた `file`)」)、`05-execution-flow.md:640`(`{ path: String, … }`)。language-agnostic `05-execution-flow.md:653`(「解決済みのパス、」)。
- 完了の判定: `grep -rn "^ *path: \|c\.path" crates/core/src/envfile crates/core/tests crates/cli/tests/plan_report.rs` が 0 件、`grep -n "pub path" crates/core/src/envfile/mod.rs` が 0 件。

### 3b. `EnvEntry.line`

- 今: `crates/core/src/envfile/mod.rs:189-190`(説明と項目)、`:198`(手書きの `Debug` の `line` の項目)。書くのは `envfile/parse.rs:104`(`line: line_no`)。
- 本番の読み取り: 0 件(重複キーの警告の行は `EnvWarning.line` が持ち、`parse.rs:88` が今の `line_no` を書く。これは変えない)。
- 目標: 項目と `Debug` の `line` の項目を消す。`EnvEntry` の手書きの `Debug` は残し(値を `"***"`)、`name` と `value` の 2 項目にする。重複キーの警告(`EnvWarning { file, line, name, kind }`)は変えない。
- テストの使用(T14〜T18、T20): core `tests/env_parse_perf.rs:66`(確かめの行)、`tests/env_merge_perf.rs:52`、`tests/plan_perf.rs:56`、`tests/plan_types.rs:70`、`tests/env_types.rs:37`、cli `tests/plan_report.rs:420`(構造体を直接作る所)。
- 仕様書: rust `03-definition-format.md:364`(`EnvEntry { name, value, line }`)、`05-execution-flow.md:641`(`{ name, value, line }`)。language-agnostic `03-definition-format.md:530`(「項目(名前、値、行)の並び」)、`05-execution-flow.md:652`(「名前、値、行」)。警告の「ファイル、後ろの行、名前」は残す。
- 完了の判定: `grep -rn "entry\.line\|line: i + 1\|line: 1,\|line: 3,\|line: line_no" crates/core crates/cli/tests/plan_report.rs` が `parse.rs` の警告の 1 件(`:88`)だけ。

## 4. 仕様書(`docs/spec/rust`、`docs/spec/language-agnostic`)

直す節と語句の一覧(各項目のコミットで直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。版を上げないので、版の注記は足さず今の形を書く)。

| # | ファイル:行(HEAD) | 節 | 今の語句 | 直した後 | 項目 |
|---|---|---|---|---|---|
| S1 | rust `05:738` | 15.1 `FlowEdge` | `…, explicit: bool }`、「`explicit` は、定義に遷移として書かれた辺か(…`false`)」 | `{ from_job, to, kinds, skipped_jobs }`。`explicit` の文を消す | 1 |
| S2 | rust `05:754-758` | 15.2 擬似コード | `End → (n, explicit = true)` など | `End → n`、`Job(t) → t`、`Default → i+1`(注釈「記述順の次。最後のジョブなら終点(n)」は残す) | 1 |
| S3 | rust `05:761` | 15.2 | 「、`explicit` = どちらかが `true`」 | その句を消す | 1 |
| S4 | la `05:747`、`:761-765`、`:768` | 15.1、15.2 | S1〜S3 と同じ | S1〜S3 と同じ(`list<…>`、小文字の種類の綴りは今のまま) | 1 |
| S5 | rust `02:42` | 再公開 | `plan::{Assumption, EnvNameEntry, …}` | **変えない**(D1 の決定で型を残す。番号は欠番にせず、変えないことの記録として残す) | 2a |
| S6 | rust `05:724` | 14.6 | 「計画は、名前と出どころ `EnvNameEntry` だけ」 | 「計画は、名前 `EnvNameEntry` だけ」 | 2a |
| S7 | rust `05:860`、`:861` | 16.1 計画の型 | `EnvNameEntry` の行 `{ name, origin: EnvOrigin }。値を持たない`、`env_names: Vec<EnvNameEntry>(試行 1 で渡される名前と出どころ)` | `{ name }。値を持たない`、`env_names: Vec<EnvNameEntry>(試行 1 で渡される名前。並びの順)` | 2a |
| S8 | rust `05:899` | 16.2 計画の項目 | 「(名前と出どころ。値は持たない。…)」 | 「(名前だけ。値は持たない。…)」 | 2a |
| S9 | rust `12:129`、la `12:145` | 確認の観点 | 「環境変数は名前と出どころだけを計画に持つ」 | 「環境変数は名前だけを計画に持つ」 | 2a |
| S10 | la `02:62`、`05:733` | 依存の規則、14.6 | 「(名前と出どころだけ)」「計画は、名前と出どころだけ」 | 「(名前だけ)」「計画は、名前だけ」 | 2a |
| S11 | la `05:867`、`:904` | 16.1、16.2 計画のデータ | `env_names: list<(name, origin)>(…名前と出どころ)`、「の名前と出どころ(値は持たない)」 | `env_names: list<{ name }>(試行 1 で渡される名前。並びの順)`(rust の `EnvNameEntry { name }` と同じ形)、「の名前(値は持たない)」 | 2a |
| S12 | rust `02:42`、`:283` | 再公開、ファイルの表 | `envfile::{…, EnvOrigin, …}`、型の列の `EnvOrigin` | `EnvOrigin` を消す | 2b |
| S13 | rust `05:644-646` | 14.1 | `EnvOrigin` の行、`{ name, value, origin }`、「名前と出どころ(値を含まない要約)は `entries` から取り出せる」 | 行を消す、`{ name, value }`、「名前(値を含まない要約)は `entries` から取り出せる」 | 2b |
| S14 | rust `05:664-672` | 14.3 擬似コード | `(name, value, origin)` と各行の `PipelineEnvFile` など | `(name, value)` の組にし、層の名前は行末の注釈(`# [pipeline].env_file` など)で示す | 2b |
| S15 | rust `11:216` | テストの表 | 「結果の中身(並び、出どころ)も確かめる」 | 「結果の中身(名前の並び、値)も確かめる」 | 2b |
| S16 | la `05:656-657`、`:676-687` | 14.1、14.3 | S13・S14 と同じ(`list<{ name, value, origin }>`、`put(name, value, origin)`) | S13・S14 と同じ形(`list<{ name, value }>`、`put(name, value)`) | 2b |
| S17 | rust `03:364`、`:377` | 6.7 書式と解析 | `EnvFileContent { path, entries: … }`、「(`path` は与えられた `file`)」 | `EnvFileContent { entries: …, warnings: … }`、その句を消す | 3a |
| S18 | rust `05:640`、la `05:653` | 14.1 | `{ path: String, … }`、「解決済みのパス、項目の並び…」 | `{ entries: Vec<EnvEntry>, warnings: Vec<EnvWarning> }`、「項目の並び(書いた順)、警告の並び」 | 3a |
| S19 | rust `03:364`、`05:641` | 6.7、14.1 | `EnvEntry { name, value, line }`、`{ name, value, line }` | `EnvEntry { name, value }`、`{ name, value }`(警告の `EnvWarning { file, line, name, kind }` は残す) | 3b |
| S20 | la `03:530`、`05:652` | 7.7、14.1 | 「項目(名前、値、行)の並び」、「名前、値、行」 | 「項目(名前、値)の並び」、「名前、値」(警告の「ファイル、後ろの行、名前」は残す) | 3b |
| S21 | rust `05:642` | 14.1 `LoadedEnvFiles` | 「`link(記述, 位置)`、`add(記述, content)`(偽の実装・テスト用)、`get(記述) …`」 | 「`link(記述, 位置)`、`get(記述) …`」(D3。決定: 推奨のとおり) | 3a |

- 残す記述: 計画の表示が名前だけであること(rust・la の 08 の `--dry-run` の出力)、重複キーの警告の行(rust `03:364` の `EnvWarning`、`03:376` の手順 3.8、la `03:530` と `:542`)、図の描き方(15.3 以降の配置の規則)、環境変数の層の説明の「出どころ」(rust `00-README.md:75`、la `00-README.md:91`。層の用語の説明で、項目の名前ではない)。
- 記述との違い(記録。D3): rust `05:642` の `LoadedEnvFiles` の行に `add(記述, content)`(偽の実装・テスト用)とあるが、実物の `LoadedEnvFiles` に `add` は無い(`push_file` と `link` だけ。テストの補助 `common::add_env_file` が同じことをする)。読まれない項目とは別の食い違い。決定(D3。推奨のとおり。ユーザー、2026-11-24): 3a のコミットで、この 1 語(`add(記述, content)`(偽の実装・テスト用)、)を消す(S21。S18 と同じ表)。
- 完了の判定(直した後に語句が残らないこと): `functional-spec.md` §7 の検索。

## 5. 範囲の外で、触らないもの(確かめた)

- `PipelineDefinition` の `format_version` と `description`(Q3: B)。キーの検査、誤りの表示、仕様書の定義の形の記述も変えない。
- `EnvWarning`(`file`、`line`、`name`、`kind`)、`EnvFileError`(`file`、`line`、`kind`)、`LoadedEnvFiles`(`files`、`by_ref`)の形。
- cli `src/report/graph_text.rs:125` の「出どころのジョブ ID」(辺の元のジョブの意味の日本語。項目の名前ではない)。cli `tests/report_messages.rs:1028` の「種類の文の出どころ」(同上)。
- converter(`explicit`・`origin` の識別子は無い。`line` は converter 自身の誤りの型の項目で、別物)。
- runtime(この一覧のどの項目も使わない。写しの試しで変更 0 件)。

## 6. 網羅に使った検索のコマンド

```sh
grep -rn "explicit" crates/core crates/cli crates/runtime
grep -rn "EnvOrigin\|\.origin\b\|origin:\|origins(" crates
grep -rn "EnvNameEntry\|env_names" crates
grep -rn "EnvFileContent\b" crates
grep -rn "EnvEntry\b" crates
grep -rn "\.path\b" crates/core crates/cli crates/runtime/tests | grep -v "base\.path\|checked\.path\|\.path()"
grep -rn "\.line\b\|line: " crates/core crates/cli/tests/plan_report.rs
grep -rn "出どころ\|origin\|explicit\|EnvOrigin\|EnvNameEntry\|EnvFileContent\|EnvEntry\|EffectiveEntry\|env_names\|名前、値、行" docs/spec/rust docs/spec/language-agnostic
# 写しの木(本番だけを変えた段階)で、エラーの場所を出す
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```
