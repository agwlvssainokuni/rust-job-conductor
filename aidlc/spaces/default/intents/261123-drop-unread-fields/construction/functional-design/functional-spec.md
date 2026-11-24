# 機能の仕様(Functional Spec): 読まれない項目の削除

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 B)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、直す仕様書の節と語句 S1〜S21、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD 79ab4d0 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表、★ の名前の変更、§4.6 の確かめの行を消すもの)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない。本番に、テストのための要素を新しく足さない**(ユーザーの判断)。core の型から、書かれるが本番で読まれない項目(`FlowEdge.explicit`、`EnvOrigin` と `origin` の項目、`EnvFileContent.path`、`EnvEntry.line`)を消す [desc] [Q1]。
- 消す項目を読む・書くテストは、**残る項目で同じ意味を確かめる**書き方にする: 辺は行き先・種類・飛び越すジョブ、環境変数は名前の並びと値、`.env` の項目は名前と値と並び(位置)、ファイル名は警告の `file`。消す項目だけを確かめていた行は、項目と一緒に消える(§4.6 に、同じ意味を確かめて残る行を示す)[Q1]。
- 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` と `graph` の出力)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- 1 項目 1 コミットの小さな並びで、どのコミットでもビルド、`clippy -D warnings`、テストが通る [Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は小さなコミットで進め、結果は最後にまとめて見せる [Q1]。
- 既存のテストの変更は、**書き方の置き換えだけ**。**期待する出力の文字列は変えない** [Q1]。テストは消さない。名前を変えるテスト(★)は 2 件で、名前が消す項目(「出どころ」)を含むため(§4.5。承認待ち)。
- 新しい依存を足さない [memory:M5]。`unsafe` と src の `cfg(unix)`/`cfg(windows)` を足さない。runtime の製品のコードは変えない(`.unwrap()` などを足す余地が無い)。cli の src に `panic!(` を書かない。converter は core に依存しない(converter は変えない)。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。誤りの表示に入力の値を含めない(誤りと警告の型は変えない)。
- 値を持つ型(`EnvEntry`、`EffectiveEntry`)の手書きの `Debug` は残し、値を `"***"` と表示する(項目が減るだけ)。
- 仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` を直す(Q2: A)。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。**版を上げないので、「v0.9.8 までは…」などの版の注記は足さず、今の形だけを書く**(既存の版の注記は残す)。計画の表示が名前だけであること、重複キーの警告の行、図の描き方の記述は残す。
- 定義の `format_version` と `description` は型に残す(Q3: B。範囲の外)。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

| 項目 | 本番から消すもの | 本番の形の変わり方 | テストの代わり |
|------|------------------|--------------------|----------------|
| 1 | core `FlowEdge.explicit` | `target_position` は位置だけ返す(`-> usize`)。`TransitionResolution::Default` は `index + 1` を決めるのに残る。`flow_edges` の組は `(位置, 種類)` | 辺の比較は行き先・種類・飛び越すジョブで。`explicit` だけの確かめ 4 行は消す(§4.6) |
| 2a | 計画の `EnvNameEntry.origin` | `EnvNameEntry { name }` の名前だけの構造体として残す(D1。決定: 代案)。`PlanEntry.env_names: Vec<EnvNameEntry>` と再公開は今のまま。`merge_env_names` は `Vec<&str>` を返す。cli は変えない | `e.name` の並びを比べる(★ 1 件) |
| 2b | core `EnvOrigin`、`EffectiveEntry.origin` | 統合の 1 項目 `Item` は `(名前, 値)`。`collect_vars`・`collect_files` は `origin` の引数を持たない。`Debug` は `name` と `value`(伏せ字) | 層は値で確かめる(すでにある)。出どころだけの確かめ 7 行は消す(§4.6。`env_merge.rs` の 6 行と `env_merge_perf.rs` の 1 行)。性能のテストは値の接頭辞で(★ 1 件) |
| 3a | core `EnvFileContent.path` | `parse_env_file(text, file)` の `file` は、誤りと警告のファイル名に使うので残る | ファイル名は警告の `file` で確かめる(すでにある)。1 行は消す(§4.6) |
| 3b | core `EnvEntry.line` | 重複キーの警告の行(`EnvWarning.line` = `line_no`)は変えない。`Debug` は `name` と `value`(伏せ字) | 項目は名前・値・並びで確かめる(すでにある)。1 行は消す(§4.6) |

各項目のコミットで、`docs/spec/rust` と `docs/spec/language-agnostic` の該当の行(`inventory.md` §4 の S1〜S21。S5 は変えない)と、消す名前を指すドキュメントコメントを合わせる。

### 2.1 1 の形(`graph/edges.rs`)

```rust
/// 結果 `on` で終わったときの行き先の位置を決める。ジョブは 0 から、終点は `jobs.len()`。
/// `positions` は、ジョブ ID から記述順の位置を引く表。
fn target_position(definition: &PipelineDefinition, positions: &HashMap<&str, usize>, index: usize, on: TransitionOn) -> usize {
    let job = &definition.job[index];
    match resolve_transition(job, on, index, |id| positions.get(id).copied()) {
        TransitionResolution::End => definition.job.len(),
        TransitionResolution::Job(t) => t,
        // 遷移がない。記述順で次のジョブ(最後のジョブなら終点)。(この行のコメントは今のまま)
        TransitionResolution::Default => index + 1,
    }
}
```

`flow_edges` は `on_success == on_warning` なら `(位置, [Success, Warning])` の 1 つ、違えば 2 つの組を作り、位置で並べ、辺を作る(今と同じ。`explicit` の計算だけが無くなる)。

### 2.2 2a・2b の形(`envfile/merge.rs`、`plan`)

```rust
/// 層の順に並べた 1 項目(名前, 値)。同じ名前を重ねて含む。
type Item<'a> = (&'a str, &'a str);

pub(crate) fn merge_env_names<'a>(def: &'a PipelineDefinition, job: &'a JobDefinition, files: &'a LoadedEnvFiles, ctx: &EnvContext<'_>) -> Vec<&'a str> {
    let mut items = collect_layers(def, job, files);
    // 値は読まないので、自動の変数の値は空にする(名前と並びだけが要る)。
    push_builtins(&mut items, "", "", ctx.run_id.map(|_| ""), "");
    keep_last_positions(&items).map(|(name, _)| *name).collect()
}
```

- 2a のコミットでは、`Item` はまだ `(名前, 値, 出どころ)` のままで、`merge_env_names` だけが名前を返す(`.map(|(name, _, _)| *name)`)。2b のコミットで `Item` から出どころを外す。
- `plan/simulate.rs` の項目は `merge_env_names(..).into_iter().map(|name| EnvNameEntry { name: name.to_string() }).collect()`(D1。決定: 代案。ユーザー、2026-11-24)。`plan/mod.rs` の `EnvNameEntry` は次の形:

```rust
/// 環境変数の名前(U1 の名前の要約の 1 項目。値は持たない)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvNameEntry {
    pub name: String,
}
```

- cli の `report/plan_text.rs` は変えない(`e.name` を読むまま。写しで、cli の src を変えずにコンパイル、clippy、テストが通ることを確かめた)。

## 3. 実物で確かめて初期の記述と違った点、範囲から外したもの

- **記述と違った点**(実物に従った):
  - 2a: 計画の `EnvNameEntry` を作るのは `merge_env_names` で、統合 `merge_env` とは別の crate の中の関数(値を読まない)。出どころを外すと、この関数の戻り値も名前だけになる。
  - 2b: `EffectiveEntry` と `EnvEntry` の `Debug` は手書き(値を伏せ字にするため)で、`origin`・`line` を書いている。項目と一緒に `Debug` の行を消す。`Debug` の出力の形を確かめるテストは無い(`env_types.rs` は値が出ないことと名前が出ることだけを確かめる)。
  - 3a・3b: cli のテスト `plan_report.rs` も `EnvFileContent` と `EnvEntry` を直接作る(初期の記述は core だけを挙げていた)。
  - 仕様書: rust 05 の 14.1 の `LoadedEnvFiles` の行に、実物に無い `add(記述, content)` がある(読まれない項目とは別の食い違い。D3。決定: 3a のコミットで消す)。
- **範囲から外したもの**: 定義の `format_version` と `description`(Q3: B)、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分(初期の記述)。誤りと警告の型(`EnvFileError`、`EnvWarning`)の形。runtime と converter(変更 0 件)。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q1: A に従い、変えるのは**消す項目の書き換え(構造体を作る所から項目を外す、比べる値を残る項目にする)と、消す項目だけの確かめの行の削除**だけで、**期待する出力の文字列は変えない**。テストの件数は変わらない(消すテスト 0 件)。名前を変えるもの(★)は 2 件。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §6。T 表は、**消す名前のすべての参照**を、次の `grep -rn` で `crates` の全体(`tests/` の下の補助のディレクトリと、src の `#[cfg(test)]` の単体テストを含む)を検索して作り、さらに **HEAD の写しの木で本番のコードだけを変えてコンパイルし、エラーの出た場所がすべて T 表にあること**を確かめた(`inventory.md` §0.2。D1 の決定のあと、決定の形で写しを作り直して確かめ直した。エラーは 30 件)。

```sh
grep -rn "explicit" crates/core crates/cli crates/runtime
grep -rn "EnvOrigin\|\.origin\b\|origin:\|origins(" crates
grep -rn "EnvNameEntry\|env_names" crates
grep -rn "EnvFileContent\b" crates
grep -rn "EnvEntry\b" crates
grep -rn "\.path\b" crates/core crates/cli crates/runtime/tests | grep -v "base\.path\|checked\.path\|\.path()"
grep -rn "\.line\b\|line: " crates/core crates/cli/tests/plan_report.rs
# 写しの木(本番だけを変えた段階)で、エラーの場所を出す
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```

src の単体テストの参照は 0 件(core の `envfile`・`graph`・`plan` の単体テストは、消す項目を読まない。写しのコンパイルでも src にエラーは無かった)。runtime と converter のテストの参照も 0 件。

### 4.1 項目 1(`FlowEdge.explicit`。core)

| ID | ファイル:行 | 箇所 | 変える内容 |
|----|-------------|------|------------|
| T1 | `tests/graph_edges.rs:37-51`(補助 `edge`)と呼び出し `:59`、`:60`、`:61`、`:75`、`:76`、`:90`、`:91`、`:104`、`:119`、`:120`、`:131`、`:132`、`:144`、`:153-159`(9 件のテストのうち 8 件) | 15 | 補助 `edge` の引数 `explicit: bool` と項目を消し、14 の呼び出しから最後の `true`/`false` を消す。比べる行き先・種類・飛び越すジョブは変えない。`:141` のコメント「成功は明示、警告は既定だが、どちらも終点なので 1 つにまとまる」は、定義の遷移の書き方の説明なので残す |
| T2 | `tests/graph_lookup_perf.rs:146`、`:150`、`:156`、`:160`(テスト `p4_千個のジョブの辺の取り出しを_100_回くり返しても_600ms_以内`) | 4 | `assert!(success.explicit)` など 4 行を消す(§4.6) |
| T3 | `tests/graph_drawable.rs:264`(テスト `辺の一覧に残る形_後ろ向きの辺`) | 1 | `FlowEdge { .. }` から `explicit: true,` を消す |
| T4 | `tests/graph_types.rs:32`(テスト `辺は_kinds_を1つ以上持ち_行き先はジョブか終点`) | 1 | 同上 |
| T5 | `tests/graph_common/mod.rs:135`(補助 `graph_with_extra_edge`) | 1 | 同上 |

### 4.2 項目 2a(計画の名前。core)

| ID | ファイル:行 | 箇所 | 変える内容 |
|----|-------------|------|------------|
| T6 | `tests/plan_types.rs:21`(import)、`:54`(テスト名 ★)、`:87-101` | 3 | 名前と出どころの組の並び(`Vec<(&str, EnvOrigin)>`、`.map(\|e\| (e.name.as_str(), e.origin))`)を、名前の並び(`Vec<&str>`、`.map(\|e\| e.name.as_str())`)にし、`["FROM_FILE", "TOKEN", "CONDUCTOR_PIPELINE", "CONDUCTOR_JOB_ID", "CONDUCTOR_ATTEMPT"]` と比べる(並びと名前は今の期待のまま。D1 の決定で `EnvNameEntry` が残るので、`e.name` を読む)。★ テスト名 `計画は環境変数の値を持たない_名前と出どころだけ` を `計画は環境変数の値を持たない_名前だけ` に。import から `EnvOrigin` を消す |

### 4.3 項目 2b(`EnvOrigin`、`EffectiveEntry.origin`。core)

| ID | ファイル:行 | 箇所 | 変える内容 |
|----|-------------|------|------------|
| T7 | `tests/env_merge.rs:21`(import)、`:53-58`(補助 `origins`)、`:95-100`(テスト `層の優先順位は_pipeline_env_file_から_job_env_の順に後ろが勝つ`)、`:173`(テスト名 ★)、`:178`、`:180`、`:181` | 7 | 補助 `origins` を消す。`:95-100` の 6 行(コメント「層ごとの出どころ」1 行、`let origin` 1 行、`assert_eq!` 4 行)を消す(§4.6)。★ テスト名 `names_は値を含まず出どころを持ち_additions_は並びの順` を `names_は値を含まず_additions_は並びの順` に。`:178` は `format!("{:?} {:?}", names(&env), env)`、`:180` は `names(&env).contains(&"P")`、`:181` は `env.additions().len() == env.entries.len()`(足す環境が項目ごとに 1 つ) |
| T8 | `tests/env_merge_perf.rs:30`(import)、`:106`(補助 `assert_order`)、`:123`(テスト `a3a_一万項目の_env_file_一つの統合は_50ms_以内`)、`:147`(テスト `a3b_同じ名前の一万項目の_env_file_二つの統合は_100ms_以内で_後ろの層の並びになる`) | 4 | `:106` の `e.origin == Builtin` を消す(§4.6。直前の行が名前を確かめる)。`:123` の `e.origin == PipelineEnvFile` を `e.value.starts_with("p.env-")` に(値は `content` が `"{ファイル}-{位置}"` で作るので、どのファイルの値かが分かる)。`:147` は `e.origin == JobEnvFile && e.value.starts_with("j.env-")` を `e.value.starts_with("j.env-")` に |
| T9 | `tests/env_types.rs:24`(import)、`:42`(テスト `値を持つ型の_debug_は値を伏せる`) | 2 | `EffectiveEntry { .. }` から `origin: EnvOrigin::JobEnv,` を消し、import から `EnvOrigin` を消す |

### 4.4 項目 3a・3b(`EnvFileContent.path`、`EnvEntry.line`。core・cli)

| ID | ファイル:行 | 箇所 | 変える内容 | 項目 |
|----|-------------|------|------------|------|
| T10 | core `tests/env_file_parse.rs:120`(テスト `重複は後ろが勝ち警告に行と名前を持つ`) | 1 | `assert_eq!(c.path, "t.env")` を消す(§4.6) | 3a |
| T11 | core `tests/env_merge_perf.rs:45`(補助 `content`) | 1 | `path: path.into(),` を消す(引数 `path` は値の接頭辞に使うので残す) | 3a |
| T12 | core `tests/plan_perf.rs:51`(テスト `千個のジョブがすべて実行する計画は_1秒以内に作れる`) | 1 | `path: "p.env".into(),` を消す | 3a |
| T13 | core `tests/plan_types.rs:66` | 1 | 同上 | 3a |
| T14 | core `tests/env_parse_perf.rs:66`(テスト `p5_一万行で後半が逆順の重複の_env_の解析は_100ms_以内`) | 1 | `assert_eq!(entry.line, HALF + k + 1)` を消す(§4.6) | 3b |
| T15 | core `tests/env_merge_perf.rs:52`(補助 `content`) | 1 | `line: i + 1,` を消す | 3b |
| T16 | core `tests/plan_perf.rs:56` | 1 | `line: i + 1,` を消す | 3b |
| T17 | core `tests/plan_types.rs:70` | 1 | `line: 1,` を消す | 3b |
| T18 | core `tests/env_types.rs:37`(テスト `値を持つ型の_debug_は値を伏せる`) | 1 | `line: 3,` を消す | 3b |
| T19 | cli `tests/plan_report.rs:416`(テスト `環境変数の名前は_層の順で_最後に_conductor_の変数`) | 1 | `path: reference.into(),` を消す | 3a |
| T20 | cli `tests/plan_report.rs:420`(同上) | 1 | `line: 1,` を消す | 3b |

### 4.5 名前を変えるテスト(★。名前を変える方針は決定(D2。推奨のとおり。ユーザー、2026-11-24)。個々の名前もユーザーが設計の承認で認めた、2026-11-24)

| ID | 今の名前 | 新しい名前 | 理由 |
|----|----------|------------|------|
| ★1(T6) | core `plan_types.rs` `計画は環境変数の値を持たない_名前と出どころだけ` | `計画は環境変数の値を持たない_名前だけ` | 計画は出どころを持たなくなり、今の名前は事実と違う。確かめる内容(値を持たない、名前の並び)は同じ |
| ★2(T7) | core `env_merge.rs` `names_は値を含まず出どころを持ち_additions_は並びの順` | `names_は値を含まず_additions_は並びの順` | 統合の結果は出どころを持たなくなる。確かめる内容(値が出ない、名前に `P` がある、項目ごとに足す、`PATH` を含まない)は同じ |

D2 は「名前を変える」に決まった(推奨のとおり。ユーザー、2026-11-24)。上の 2 つの新しい名前そのものは、ユーザーが設計の承認で認めた(2026-11-24)。

### 4.6 確かめの行を消すもの(テストは消さない。Q1: A。ユーザーが設計の承認で認めた、2026-11-24)

| 消す行 | 確かめていること | 消す理由 | 同じ意味を確かめて残る行 | 残る行で確かめられない点 |
|--------|------------------|----------|--------------------------|--------------------------|
| T2: `graph_lookup_perf.rs:146`、`:150` の 2 行(`success.explicit`、`warning.explicit`) | 書いた遷移の辺が `explicit` | 項目が無くなる(1) | 同じ繰り返しの `to`(`j{i+1}`、`j{i+2}`)と `kinds`(`[Success]`、`[Warning]`)が、成功と警告で別の行き先に分かれたこと(= 書いた遷移を読んだこと)を確かめる | **辺が「書いた遷移」か「記述順の次」かの区別**。残る `to` と `kinds` は、記述順の次を指す遷移を書いた定義(たとえば `j998` に `on_success = "j999"`)でも同じ値になるので、区別はテストで確かめられなくなる。本番はこの区別を読まない(読み取り 0 件)。位置の計算(書いた遷移の行き先と、記述順の次の位置)は `to` で確かめられる |
| T2: `graph_lookup_perf.rs:156`、`:160` の 2 行(`!j998.explicit`、`!j999.explicit`) | 遷移の無いジョブの辺が `explicit` でない | 同上 | `j998` の `to`(`j999`)と `kinds`(`[Success, Warning]`)、`j999` の `to`(`End`)が、記述順の次の位置への辺であることを確かめる | 上の行と同じ(「書いた遷移」か「記述順の次」かの区別は確かめられない。本番は読まず、位置は `to` で確かめられる) |
| T7: `env_merge.rs:95-100` の 6 行(コメント 1 行、`let origin` 1 行、`assert_eq!` 4 行) | `V`・`W`・`P`・`CONDUCTOR_JOB_ID` の出どころが `JobEnv`・`JobEnvFile`・`PipelineEnvFile`・`Builtin` | 項目が無くなる(2b) | 同じテストの `:86-93` が、値で層を確かめる(`V` は `job-env`、`W` は `job-file-2`、`P` は `p`。どれも層ごとに違う値)。自動の変数の値は `自動の変数は_run_id_がないとき_conductor_run_id_を作らず試行を反映する` が確かめる | なし |
| T8: `env_merge_perf.rs:106`(`e.origin == Builtin`) | 末尾の 4 つが自動の変数 | 同上 | 直前の行 `e.name == b` が、末尾の 4 つが自動の変数の名前であることを確かめる | なし(利用者は `CONDUCTOR_` の名前を書けないので、名前で決まる) |
| T10: `env_file_parse.rs:120`(`c.path == "t.env"`) | 内容が渡したファイル名を持つ | 項目が無くなる(3a) | 同じテストの `:115-118` が、警告の `file` が `"t.env"` であることを確かめる(表示に使うファイル名はこちら) | なし |
| T14: `env_parse_perf.rs:66`(`entry.line == HALF + k + 1`) | 残った項目が後半の行のもの | 項目が無くなる(3b) | 直前の 2 行が、名前の並び(`K4999` … `K0`)と値(`b{i}`。後半だけの値)を確かめる。重複の警告 5,000 件は `:68-75` が確かめる | なし |

- 消す行の合計: **13 行**(T2 の 4 行、T7 の 6 行、T8 の 1 行、T10 の 1 行、T14 の 1 行。行数はコメントの行も含めて数える)。
- 件数: 消すテストは 0 件。件数は変わらない。
- `explicit` の区別(書いた遷移か、記述順の次か)そのものは、図の配置や表示に使われていない(本番の読み取り 0 件)。項目を消したあとは、この区別をテストで確かめる手段が無くなる(上の T2 の行)。区別が要る判断(`TransitionResolution::Default`)は本番に残り、その結果の位置(記述順の次)は行き先 `to` で確かめられる(`graph_edges.rs` の 9 件)。

## 5. 足すテスト

なし。消す項目の代わりは、既存のテストの書き換えと、残る項目の既存の確かめで持つ(§4)。本番の動きは変わらないので、新しい確かめは要らない。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | **112** | **270** | **382** | 0 |
| runtime | **69** | **128** | **197** | 0 |
| converter | **52** | **64 + 1 無視** | **116 + 1 無視** | 0 |
| cli | **2** | **456** | **458** | 0 |
| 全体 | | | **1,153 通過、1 無視、0 失敗** | 0 |

基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた(`inventory.md` §0.2)。テスト名の一覧の差は ★ の 2 件だけ。D1 の決定(代案。名前だけの構造体)の形で、写しを作り直して確かめ直した(推奨の形で試したときと、件数と名前の差は同じ)。

## 7. 確かめ方

- 実装の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,153 − 消したテスト 0 + 足したテスト 0 = 1,153**」に一致すること [Q1]。
- テスト名の一覧の差: 前後で `cargo test --locked --workspace -- --list 2>/dev/null | grep ": test$" | sort` の差が、§4.5 の ★ 2 件の名前の変更だけであること。
- テストのファイルの変更は §4 の T 表だけ。コミットごとに `git diff --stat -- 'crates/*/tests'` を見る。src の単体テストは変えない。
- **項目ごとの完了の判定**(`inventory.md` の各項目)。最後のコミットのあとで、次をすべて行う [Q1]:

```sh
# コード
grep -rn "explicit" crates/core crates/cli crates/runtime                                     # 0 件
grep -rn "EnvOrigin\|\.origin\b\|origin:" crates/core crates/cli crates/runtime                  # 0 件
grep -n -A 3 "pub struct EnvNameEntry" crates/core/src/plan/mod.rs                             # 項目は `pub name: String` だけ(D1)
grep -rn "名前と出どころ\|出どころを持" crates                                                  # 0 件
git diff --stat 79ab4d0 -- crates/cli/src                                                      # 空(cli の src は変えない)
grep -n "pub path\|pub origin\|pub explicit" crates/core/src/envfile/mod.rs crates/core/src/graph/mod.rs crates/core/src/plan/mod.rs  # 0 件
grep -n "pub line" crates/core/src/envfile/mod.rs                                              # EnvFileError と EnvWarning の 2 件だけ
grep -rn "^ *path: \|c\.path\|entry\.line\|line: i + 1\|line: 1,\|line: 3," crates/core/tests crates/cli/tests/plan_report.rs  # 0 件
grep -n "line: line_no" crates/core/src/envfile/parse.rs                                       # 警告の 1 件(:88)だけ

# 仕様書(直した後に語句が残らないこと。S1〜S21)
grep -rn "EnvOrigin" docs/spec/rust docs/spec/language-agnostic                                 # 0 件
grep -rn "EnvNameEntry" docs/spec/rust docs/spec/language-agnostic                              # rust の 4 行だけ(02:42 の再公開、05 の 14.6 の 1 行、16.1 の 2 行)。どれも `origin` を含まない
grep -rn "list<(name, origin)>\|env_names: list<string>" docs/spec/language-agnostic             # 0 件(la は `list<{ name }>`)
grep -rn "origin" docs/spec/rust docs/spec/language-agnostic | grep -v "origin/main"            # 0 件(11 の verify-tag の git のブランチ名は別物)
grep -rn "explicit" docs/spec/rust docs/spec/language-agnostic                                  # 0 件
grep -rn "出どころ" docs/spec/rust docs/spec/language-agnostic                                   # 00-README の 2 件(rust :75、la :91。層の用語の説明)だけ
grep -rn "名前、値、行\|name, value, line\|EnvFileContent { path\|path: String, entries\|解決済みのパス、項目\|\`path\` は与えられた" docs/spec/rust docs/spec/language-agnostic  # 0 件
# 残すべき記述が残っていること
grep -n "EnvWarning { file, line, name, kind" docs/spec/rust/03-definition-format.md            # 1 件(重複キーの警告の行)
grep -n "警告(重複キー: ファイル、後ろの行、名前)" docs/spec/language-agnostic/03-definition-format.md  # 1 件
grep -n "add(記述, content)" docs/spec/rust/05-execution-flow.md                                 # 0 件(D3。S21)
# 凍結した仕様書が変わっていないこと
git diff --stat 79ab4d0 -- docs/spec/go docs/spec/python docs/spec/typescript docs/spec/csharp docs/spec/powershell  # 空
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4。この intent と無関係)のまま増えないこと。
- Windows の対象のコンパイル: `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` をローカルの macOS で走らせる(写しで通ることを確かめた。この intent は Windows でだけコンパイルされるコードに触れない)。実行は CI の `test`(windows)で確かめる。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`。消すのは本番とテストの両方の行で、割合はほぼ変わらない)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。`unsafe` と src の `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。版の表記(`0.9.8`)は変えない(`git diff -- '*Cargo.toml' Cargo.lock` が空)。README と `docs-check` の語句に触れる変更は無い。
- 利用者から見える動き: 既存のテスト(T 表の書き換えを除く)が期待を変えずに通ることで確かめる。とくに `--dry-run` の環境変数の行(cli `plan_report.rs` の 7 件、`docs_dry_run.rs`)、`graph` の出力(cli `graph_report.rs`、`docs_graph.rs`、core `graph_*.rs`)、`.env` の誤りと警告の表示(cli `report_messages.rs`、runtime `env_loader.rs`)は、変えずに通る。ステートファイル、表示の文、終了コードを作るコードは、どの項目でも変わらない(消すのは書かれるだけの項目)。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

1 項目 1 コミット。各コミットに、その項目のテストの書き換え(T)、確かめの行の削除(§4.6)、★ の名前の変更、ドキュメントコメント、仕様書の行(S)を含める(本番を変えたコミットでテストのコンパイルが止まるため、本番とテストを同じコミットで変える。test-after の「実装の後にテスト」は、新しいテストが無いので当てはまらない)。

**各コミットの確かめ**: コミットごとに、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked --no-run`(全テストのコンパイル)を通し、そのコミットで変えたクレートのテストを実行する。写しの木では、全項目を当てた最後の形でだけ確かめたので、途中の形はこのコミットごとの `--no-run` で確かめる。最後の確かめ(6)で全テストを実行する。

**複数のコミットにまたがるファイル**(各コミットの終わりの形):

| ファイル | 触るコミット | 各コミットの後 |
|----------|--------------|----------------|
| core `tests/plan_types.rs` | 2(T6)、4(T13)、5(T17) | 2 の後: import から `EnvOrigin` を消し、`e.name` の並びで比べる(`EnvFileContent { path, entries: [EnvEntry { name, value, line }], .. }` は今のまま)。4 の後: `path` を消す。5 の後: `line` を消す |
| core `tests/env_merge_perf.rs` | 3(T8)、4(T11)、5(T15) | 3 の後: import は `EffectiveEnv, EnvContext, EnvEntry, EnvFileContent, LoadedEnvFiles, PipelineDefinition, merge_env, parse_definition`。4 の後: 補助 `content` から `path` を消す。5 の後: `line` を消す |
| core `tests/env_types.rs` | 3(T9)、5(T18) | 3 の後: import から `EnvOrigin` を消し、`EffectiveEntry` から `origin` を消す。5 の後: `EnvEntry` から `line` を消す |
| core `tests/plan_perf.rs` | 4(T12)、5(T16) | 4 の後: `path` を消す。5 の後: `line` を消す |
| cli `tests/plan_report.rs` | 4(T19)、5(T20) | 同上 |
| core `src/envfile/merge.rs` | 2、3 | 2 の後: `merge_env_names` は `Vec<&'a str>` を返す(`.map(|(name, _, _)| *name)`)。説明の「名前と出どころだけ」を「名前だけ」に。3 の後: `Item` は `(名前, 値)` |
| core `src/envfile/mod.rs` | 3、4、5 | 3 の後: `EnvOrigin` と `EffectiveEntry.origin` が無い。4 の後: `EnvFileContent.path` が無い。5 の後: `EnvEntry.line` が無い |
| core `src/envfile/merge.rs` と `src/plan/mod.rs` の `EnvOrigin` | 2、3 | 2 の後: `plan/mod.rs` は `EnvOrigin` を使わない(`use` を消す)。`envfile` の `EnvOrigin` は `Item` と `EffectiveEntry` がまだ使う。3 の後: `EnvOrigin` は無い |
| core `src/lib.rs` | 3 | 3 の後: `envfile` の再公開から `EnvOrigin` を消す(`plan` の再公開の `EnvNameEntry` は残す。D1) |
| rust `docs/spec/05-execution-flow.md` ほか | 1〜5 | 各コミットで、その項目の S の行だけを直す(`inventory.md` §4 の「項目」の列) |

使わなくなる名前が import に残ると `-D warnings` で止まるので、項目を消すコミットで import も同時に直す。

1. core: `FlowEdge.explicit` を消し、`target_position` は位置だけを返す(1。T1〜T5、§4.6 の T2。仕様書 S1〜S4)
2. core: 計画の `EnvNameEntry` から `origin` を消し、名前だけの構造体にする(D1。決定: 代案)(2a。T6、★1。仕様書 S6〜S11。S5 は変えない)
3. core: `EnvOrigin` と `EffectiveEntry.origin` を消す(2b。T7〜T9、★2、§4.6 の T7・T8。仕様書 S12〜S16)
4. core・cli: `EnvFileContent.path` を消す(3a。T10〜T13、T19、§4.6 の T10。仕様書 S17、S18、S21(D3。rust 05 の `add(記述, content)`))
5. core・cli: `EnvEntry.line` を消す(3b。T14〜T18、T20、§4.6 の T14。仕様書 S19、S20)
6. (確かめ)§7 の検索、件数、テスト名の差、`cargo doc` の警告、Windows の対象の clippy の記録(コミットなし。記録はコード生成の記録に書く)

合計 **5 コミット**(6 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」(定義の `format_version` と `description`、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分)。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- 版の更新、タグとリリース。
- rust と language-agnostic の仕様書の、この intent の項目に関わらない記述(D3 の 1 か所(S21)を除く)。
- cli の src(D1 の決定で、`plan_text.rs` を含めて変えない)。

## 10. 決めてほしい点

D1〜D3 は、ユーザーが決めた(2026-11-24)。D1 は代案、D2・D3 は推奨のとおり。★ 2 件の個々の名前の変更(§4.5)と、§4.6 の確かめの行の削除は、ユーザーが設計の承認で認めた(2026-11-24)。

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | 計画の環境変数の名前の型(2a) | **`EnvNameEntry` を消し、`PlanEntry.env_names: Vec<String>`**(並びの順の名前) | (B)`EnvNameEntry { name: String }` の名前だけの構造体にする(cli の `plan_text.rs` は変えない) | 1 項目だけの構造体は名前の文字列以上の意味を持たず、仕様書の「計画は名前だけを持つ」をそのまま表す。型と再公開が 1 つ減る。代案は差分が小さい(cli を変えない)が、中身が名前だけの包みが残る。どちらも利用者から見える動きとテストの件数は同じ(写しは両方の形で確かめた) | 決定: **代案**。`EnvNameEntry { name }` の名前だけの構造体を残し、cli の `plan_text.rs` は変えない(ユーザー、2026-11-24) |
| D2 | ★ の 2 件のテスト名(§4.5) | **名前を変える**(★1 `計画は環境変数の値を持たない_名前だけ`、★2 `names_は値を含まず_additions_は並びの順`) | 名前は今のまま(内容だけ書き換える) | 今の名前は「出どころ」を持つと書いており、項目を消すと事実と違う。確かめる内容は同じで、名前だけが変わる。テスト名の一覧の差は、この 2 件だけになる | 決定: 名前を変える(推奨のとおり。ユーザー、2026-11-24)。個々の名前は設計の承認で認めてもらう(承認待ち) |
| D3 | rust 05 の 14.1 の `LoadedEnvFiles` の行の `add(記述, content)`(偽の実装・テスト用。実物に無い) | **3a のコミット(同じ表の隣の行を直す)で消す**(実物のメソッドは `new`、`push_file`、`link`、`get`、`files`) | 範囲の外として記録だけし、直さない | 読まれない項目とは別の食い違いだが、同じ表の 1 語で、参照実装に合わせるという Q2 の方針と同じ向き。language-agnostic の同じ行には無い | 決定: 3a のコミットで消す(S21。推奨のとおり。ユーザー、2026-11-24) |

**設計の承認で認めてもらうもの**: §4 の T 表の書き換え(T1〜T20)、§4.5 の ★ 2 件の名前の変更(D2)、§4.6 の確かめの行の削除(合計 13 行: T2 の 4 行、T7 の 6 行(コメント 1 行を含む)、T8 の 1 行、T10 の 1 行、T14 の 1 行)。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた(字下げした項目は、親の項目に含める)。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | `FlowEdge.explicit`、`EnvOrigin` と `origin` の項目、`EnvFileContent.path`、`EnvEntry.line` が core の型から消え、本番とテストからの参照が 0 件になる。テストは名前の並び、値、位置など、残る項目で確かめる | BR1.1、BR2.1、BR2.2、BR3.1、BR3.2 |
| SM2 | rust と言語に依らない仕様書の該当する記述が、参照実装と一致する。計画の表示が名前だけ、重複キーの警告の行、図の描き方の記述は残る。直す節と語句を一覧にし、直した後に残らないことを検索で確かめる | BR4.1、BR4.2 |
| SM3 | 定義の `format_version` と `description` は型に残る | BR5.1 |
| SM4 | 確かめる手段: 項目ごとの今の場所・目標・完了の基準の一覧と、実装の後の検索 | BR9.2、BR9.3、BR9.6 |
| SM5 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM6 | 既存のテストの扱い(書き方を変える・消すテストは検索で網羅し、消すものは理由を添えて承認を取る。期待する文字列は変えない)と件数(基準 − 消したテスト + 足したテスト) | BR8.1、BR9.2、BR9.3 |
| SM7 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
