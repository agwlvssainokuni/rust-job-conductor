# 機能の仕様(Functional Spec): テストの都合で曲げられた本番のコードと、残りの整理

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A、Q4 A、Q5 A)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、使った検索のコマンド)と、基準の件数は `inventory.md`**(HEAD c5f1a3a で作った)。この文書は、方針、段ごとの設計、**承認が要るテストの変更(§4 の T 表)と消すテストの対の表(§4.6)**、足すテスト(§5 の N 表)、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない**(前の intent 261123-leftovers-and-fifo の Q3 のユーザーの判断)。本番から、テストのためだけの関数・項目・引数・公開範囲・差し替え口をなくす。テストは次のどれかで代わりを持つ [desc] [Q2]:
  - (a) 公開の入口から確かめる
  - (b) 偽物の側で記録する
  - (c) テストの補助に同じ処理を書く
  - (d) 単体テストへ移す
- 依存の注入の継ぎ目(`Runtime`・`Sink`・`Env`・`Signals` のトレイト、`run_cli_with`、`Deps`)は、設計上の継ぎ目なので残す。
- 2 つの段を順に進める。段 1 はテストの都合で曲げられた所、段 2 はふつうの整理。どちらも、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` と `graph` の出力)を変えない。版は上げない [desc]。
- 各段は小さなコミットの積み重ねで、どのコミットでもビルド、`clippy -D warnings`、テストが通る [Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は段の順に進め、各段の終わりにテストの件数を記録し、結果は最後にまとめて見せる [Q1]。
- 既存のテストは、書き方を変える・移す・消すことを認める。**利用者から見える動きの確かめと、期待する出力の文字列は変えない** [Q2]。消すテストは §4.6 で理由と、同じことを確かめて残るテストを示す。テストの名前を変えるのは ★ の 3 つ(確かめる中身の一部が、消す関数・場面と一緒に無くなるため)だけ。
- 新しい依存を足さない [memory:M5]。`unsafe` は runtime の `platform/unix.rs` だけ。`cfg(unix)`/`cfg(windows)` は src の `platform` の外に置かない。runtime の製品のコード(src の `#[cfg(test)]` より前の部分)に `.unwrap()`・`.expect(`・`panic!(`・`unreachable!(`・`todo!(`・`unimplemented!(` を書かない(runtime `tests/nfr_source.rs`)。cli の src には `panic!(` を書かない(cli `tests/nfr_source.rs` の `テスト専用の経路を実行ファイルに持たない`。1h で cli の src に初めて単体テストを置くので、単体テストの中でも書かない)。誤りの表示に入力の値を含めない。converter は core に依存しない。MSRV 1.100。テスト名とドキュメントコメントは日本語。
- 仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` を直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。**版を上げないので、「v0.9.8 までは…」などの版の注記は足さず、今の形だけを書く**(既存の版の注記は残す)。

## 2. 段ごとの設計(詳細は `inventory.md` の同じ番号)

### 2.1 段 1(テストの都合で曲げられた所)

| 項目 | 本番から消すもの | テストの代わり(方針) |
|------|------------------|------------------------|
| 1a | cli `Reporter::language_resolved`・`color_resolved`(`#[doc(hidden)]`) | (b) 偽の `Env` の記録。既存の `FakeEnv::locale_read()` と、足す `color_read()`(`NO_COLOR` を読んだ、または端末を問い合わせた) |
| 1b | runtime `ResetAllReport.removed`。付随して `remove_state_file` の戻り値の `bool` | (a) ファイルの一覧と有無(7 か所のうち 6 か所は同じテストにすでにある) |
| 1c | core `graph::is_lane_drawable` と再公開 | (a) `matches!(layout_graph(&g), GraphLayout::Lanes(_))` |
| 1d | core `LoadedEnvFiles::add` | (c) core の `tests/common` の `add_env_file`(`push_file` → `link`)。cli の 1 か所はその場で 2 行 |
| 1e | runtime `remove_stale_temps` の引数 `now` | (d) のまま(単体テストは更新時刻を 48 時間前にしてから呼ぶ) |
| 1f | core `DefinitionErrorKind::code`・`EnvFileErrorKind::code`、converter `ConversionErrorKind::code` | (a) 種類そのもの(`HashSet<Kind>`)で数える。識別子が重ならないことの確かめは関数と一緒に消える。仕様の識別子との対応は、型の説明に規則(変種の名前の先頭を小文字にしたもの)を 1 文書く |
| 1g | `ALL` 5 つ(`DefinitionErrorKind`、`EnvFileErrorKind`、`ConversionErrorKind`、runtime `CreateError`、cli `UsageErrorKind`) | (c) テストの側の写し `tests/common/kinds.rs`(core・converter・cli)。網羅の `match` と、並びと重複のコンパイル時の確かめつき(§2.3。D1・D2) |
| 1h | cli `on_panic`・`install_panic_hook_with` と再公開 | (d) 本体を非公開の `write_internal_error(&mut dyn Write) -> i32` にし、2 件を単体テストへ。フックの設定の 1 件は消す(確かめられなくなる。§2.4) |
| 1i | cli `SignalError::AlreadyRegistered`、`static REGISTERED` | テストは 1 つで 1 回だけ登録する(`signals_real.rs` の 2 回目の登録を消す)。場面の無くなるテスト 1 件を消す(§2.5) |
| 1j | core `graph::node_state_of`・`flow_edges` の公開 | (a) `build_graph(def, None).edges`、`overlay_from_state` |

### 2.2 段 2(ふつうの整理)

| 項目 | 要点 |
|------|------|
| 2a | 古くなった説明 12 か所(`inventory.md` 2a の表): `.env` の重複キーの警告は `validate` と `graph` が出す(cli のコメントと、仕様書 rust・language-agnostic の 03・08・10 の 6 か所。調査は 1 か所だった)、`run_validate` の説明に `.env` の検査、runtime のモジュールの表(`bounded_read` の行、`state::reader`・`state::writer`・`temp_file` の責務)、「自動の再試行はしない」の 5 か所の主語を限る、converter のモジュールの表(`raw`・`scalar` の行と `intake` の形 B)、`env_file_refs` の `Some` の場合、`Reporter::show` の `notice` |
| 2b | `PipelineState.definition_key` と `job_order` を型から消す。ファイルの形(両キーの書き出し、`definition_key` の必須の確認、`job_order` と `[[job]]` の一致の検査、`resume_job_id` が含まれることの検査、誤りの理由)は 1 バイトも変えない。照合は `job_states` の ID の並びで行う(読み込みで一致を確かめ済みで、作る所でも同じ並び) |
| 2c | runtime `CommandError.reason`(`kind` から決まる写し)を消し、`CommandErrorKind::reason()` に。`Display` は `.kind.reason()`。`CommandError::new` も消す(D6) |
| 2d | core `ExitCodeClass` を消し、`classify_exit_code` が `JobOutcome` を直接返す |
| 2e | 小さい整理: core の `EXIT_*` を `classify.rs` の中だけの非公開に、cli の `run_dry_run` を非公開に、runtime `reader.rs:110` の常に真の条件、runtime `writer.rs` のテストだけの `use` を `mod tests` の中へ、converter `lib.rs:126` の `Path` の包み直し |
| 2f | ステートが無いときの判断の重複(cli `step::decide` と core `plan_run`)を、core の `flow::decide_from_state(job_id, definition, Option<&PipelineState>)` の 1 か所に(D7)。cli の `step::decide` は消す |

各項目のコミットで、`docs/spec/rust` と `docs/spec/language-agnostic` の該当の行を合わせる(`inventory.md` の各項目の「仕様書」)。仕様書の変更は内部の名前と形だけで、利用者から見える挙動の記述は変えない(2a-2 の「`validate` と `graph` が出す」は、04 章にすでに書かれた今の挙動に、ほかの章を合わせるもの)。

### 2.3 1g の設計(誤りの種類の一覧の写し。Q5: A)

置き場所(D2。決定(ユーザー、2026-11-23)): 各クレートのテストの共通の補助の下位モジュール `tests/common/kinds.rs`(`tests/common/mod.rs` に `pub mod kinds;`)。

| クレート | ファイル | 持つ一覧 | 使うテスト |
|----------|----------|----------|------------|
| core | `crates/core/tests/common/kinds.rs` | `DEFINITION_ERROR_KINDS: [DefinitionErrorKind; 33]`、`ENV_FILE_ERROR_KINDS: [EnvFileErrorKind; 13]` | `env_types.rs`、`nfr_security.rs` |
| converter | `crates/converter/tests/common/kinds.rs` | `CONVERSION_ERROR_KINDS: [ConversionErrorKind; 6]` | `nfr_security.rs` |
| cli | `crates/cli/tests/common/kinds.rs` | 上の 3 つと、`CREATE_ERRORS: [CreateError; 5]`、`USAGE_ERROR_KINDS: [UsageErrorKind; 8]` | `report_messages.rs` |

形(D1。決定(ユーザー、2026-11-23))。一覧ごとに、次の 3 つを並べて置く:

```rust
/// 定義の誤りの種類のすべて(テストの写し。種類を足したら、下の `match` がコンパイルで止まるので、ここにも足す)。
pub const DEFINITION_ERROR_KINDS: [DefinitionErrorKind; 33] = [K::FileMissing, /* … */ K::InvalidEnvValue];

/// 種類の、一覧の中の位置(網羅の `match`。`_` を使わない)。
const fn definition_position(kind: DefinitionErrorKind) -> usize {
    match kind { K::FileMissing => 0, /* … */ K::InvalidEnvValue => 32 }
}

// 一覧の i 番目の位置が i であること(並びの誤りと重複を、コンパイルで止める)。
const _: () = {
    let mut i = 0;
    while i < DEFINITION_ERROR_KINDS.len() {
        assert!(definition_position(DEFINITION_ERROR_KINDS[i]) == i);
        i += 1;
    }
};
```

- 種類が増えると `match` がコンパイルで止まり(Q5 の「コンパイルで気づける」)、開いた所に一覧がある。一覧の並びの誤りや重複も、`const` の `assert!` でコンパイルで止まる(Rust 1.57 以降。MSRV 1.100 で使える)。どれも `cargo test` を走らせる前に分かるので、テストの件数は増えない。
- 本番の型に `Hash` などの derive は足さない(3 つの種類はすでに `Hash` を持つ。`CreateError`・`UsageErrorKind` は数えるだけで `Hash` を要らない)。
- core の一覧は、core と cli のテストに写しができる(Q5 の背景で承知済み)。写しどうしの食い違いは、どちらも網羅の `match` で止まるので、片方だけが古くなることは無い。

### 2.4 1h の設計(パニックのフック。Q3: A)

```rust
/// フックの本体。固定の英語の文を `err` に書き、終了コード 70 を返す(書き込みの失敗は無視する)。
fn write_internal_error(err: &mut dyn Write) -> i32 { /* writeln!、flush、EXIT_INTERNAL */ }

/// パニックのフックを設定する(`main` の最初に呼ぶ)。パニックのとき、本体を標準エラーに書き、70 で終わる。
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        std::process::exit(write_internal_error(&mut std::io::stderr()));
    }));
}
```

- 単体テスト(`panic_hook.rs` の `#[cfg(test)] mod tests`。cli の src で初めての単体テスト): `フックの本体は固定の英語の文を書き_70_で終わる`(`Vec<u8>` に書き、文と 70)、`標準エラーに書けなくても_70_で終わる`(書けない書き手で 70)。名前と確かめる中身は今のまま。
- **確かめられなくなること**: `install_panic_hook` がフックを設定し、パニックのとき 70 で `process::exit` を呼ぶこと。差し替え口が無いと、テストのプロセスが終わってしまうので自動のテストで確かめられず、リリースのバイナリにはパニックを起こす経路を持たせない方針なので実バイナリでも確かめない(Q3 で承知済み)。残る確かめは、本体の 2 件と、`main.rs`(3 行)が最初に `install_panic_hook()` を呼ぶことを読んで確かめること。この点を仕様書 rust 10 と 11、language-agnostic 11 の AC-368 に書く。

### 2.5 1i の設計(シグナル。Q4: A)

- 本番で 2 回登録しない根拠(`inventory.md` 1i): `Signals::start` → `Ctx::start_signals`(1 か所)→ `Ctx::prepare_session`(1 か所)→ `run.rs:41`・`pipeline.rs:43`(1 回の `run_cli` で 1 つ)。`main.rs` は `run_cli` を 1 回だけ呼ぶ。
- `SignalError` は `Failed` だけの列挙型(D5)。`Signals::start` の説明に「プロセスで 1 回だけ呼ぶ。2 回呼んだときの動きは契約の外(UNIX 系は監視が 2 つになり、Windows は `Failed`)」と書く。
- 公開の継ぎ目 `run_cli_with`(と `run_cli`)の説明にも、「シグナルの監視の登録はプロセスで 1 回だけ。本番の `SystemSignals` を渡して、`run`・`pipeline` をプロセスで 2 回実行すると、2 回目の登録は契約の外(Windows は登録に失敗して 70 で終わる。UNIX 系は監視が 2 つになる)」と書く。偽の `Signals` を渡すテストには当たらない。`Signals` トレイト(`deps.rs`)と仕様書 rust 02(公開の関数の表)・07(シグナルの登録)にも同じ文を書く。
- テスト: `signals_real.rs` は 1 回だけ登録し、最初の番号が残ることだけを確かめる(★ 名前の変更)。もともと 1 つのファイルに 1 つのテストで、ファイルごとに別のプロセスなので、ファイルを分けなくてよい。`signals_order.rs` は今も 1 回だけ(変えない)。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - 1a: 偽の `Env` は、`var_calls`・`terminal_calls` に加えて、読んだ名前 `names` とロケールを読んだかの `locale_read()` をすでに持つ。足すのは `color_read()` だけ。
  - 1b: `remove_state_file` の `bool` を読むのは `removed` の数えだけ。`removed` と一緒に `Result<(), StateError>` にする。
  - 1f: 3 つの種類のすべてで、仕様の識別子は変種の名前の先頭を小文字にしたもの。`code()` の表の代わりに、この規則を型の説明に書く。
  - 1h: cli の `nfr_source.rs` が cli の src のすべての行で `panic!(` を禁じる。消す 1 件(`panic!` を使う)以外の 2 件は `panic!` を使わないので、src に移せる。
  - 1i: `signals_real.rs` は、もともと 1 つのファイルに 1 つのテストで、2 回目の `start` の確かめだけが「1 回」を超えていた。ファイルを分ける必要は無い。
  - 1j: `node_state_of` を `pub(crate) use` で再公開すると、使われない import になる(使うのは同じファイルの `overlay_from_state` だけ)。再公開を消し、`overlay.rs` の中だけの非公開にする。P4 の性能のテストは `flow_edges` を直接測っていたので、`build_graph(def, None).edges` を測る形にする(上限 600 ms、今の中央値 117.5 ms)。
  - 2a-2: 「`.env` の重複キーの警告は `validate` だけが出す」は、仕様書 rust・language-agnostic の 03・08・10 の 6 か所にある(調査は rust 08 の 1 か所)。
  - 2a-9: runtime の `runner.rs:96` の「再試行しない」は `run_job` 1 回の範囲の文で、正しい(変えない)。
  - 2b: 照合(`consistency.rs:51`)が `job_order` を読む(調査は「常に `job_states` の並びと同じ」まで)。照合を `job_states` の ID の並びに替えても、読み込みで一致を確かめ済みで、作る所(`initial_state`)でも同じ並びなので、結果は同じ。テストで手で組む 13 か所もすべて同じ並び(確かめた)。
  - 2e-1: `EXIT_*` は前の intent で `pub(crate)` になっており、使うのは `classify.rs` の中だけ。
- **範囲から外したもの**: 初期の記述の「範囲の外」(converter の `testing` モジュール、cli のライブラリの公開範囲(`parse_args`、`Reporter` 等)、上限つき読み取りの core と runtime の共有、`skip_reason_kind`、エスケープの確保、`JobExecution` の包み、辺の間のジョブの複製)と、`EdgeList`、`CommandKind::ALL`、core・converter の src のテストの `cfg(unix)`、凍結した仕様書。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q2: A に従い、変えるのは**消す名前の置き換え(公開の入口、偽物の記録、テストの補助、単体テストへの移動)と、型の変更に伴う組み立ての直し**だけで、**確かめる利用者から見える動きと、期待する出力の文字列は変えない**。テストの名前を変えるのは ★ の 3 つ。件数が変わるのは §4.6 の消すテスト(−5)と §5 の足すテスト(+1)だけ。単体テストの移動(1h の 2 件)は件数を変えない。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §8。T 表は、**消す・変える名前のすべての参照**を、次の `grep -rn` で網羅して作った(`crates` の全体。`tests/` の下の補助のディレクトリ(`common/`、`graph_common/`、`signal_support/` など)も含む)。

```sh
grep -rn "language_resolved\|color_resolved" crates docs/spec/rust docs/spec/language-agnostic README.md
grep -rn "\.removed\|removed:\|ResetAllReport {" crates
grep -rn "is_lane_drawable" crates
grep -rn "\.add(" crates
grep -rn "remove_stale_temps" crates
grep -rn "\.code()\|::ALL\b" crates
grep -rn "install_panic_hook_with\|on_panic\|panic_hook" crates
grep -rn "AlreadyRegistered\|SignalMode::Already\|REGISTERED" crates
grep -rn "node_state_of\|flow_edges" crates
grep -rn "definition_key\|job_order" crates
grep -rn "PipelineState {" crates
grep -rn "CommandError {\|CommandError::new" crates
grep -rn "ExitCodeClass\|classify_exit_code" crates
grep -rn "run_dry_run\|EXIT_START_FAILURE\|EXIT_SIGNAL_BASE\|EXIT_UNLISTED_ZERO" crates
grep -rn "decide(\|decide_before_run" crates/cli crates/core/src/plan
# 補助のディレクトリに、消す名前の参照が無いこと(core の common の state() の 2 項目と、cli の common の SignalMode::Already だけが当たる)
grep -rn "flow_edges\|is_lane_drawable\|node_state_of\|\.add(\|code()\|::ALL\|language_resolved\|\.removed\|AlreadyRegistered\|on_panic\|install_panic_hook_with\|job_order\|definition_key" crates/*/tests/*/
```

**注(T31 の漏れ)**: `grep -rn "CommandError {\|CommandError::new" crates` は runtime `src/error.rs:150`、`:152` も拾っていたが、型を定義するファイルの中の当たりを、製品のコード(`split.rs` と同じ組み立て)として読み、ファイルの後ろの `#[cfg(test)] mod tests` の中の単体テストだと見分けなかった。src の当たりは、`#[cfg(test)]` より後ろかを行番号で確かめる(確認担当の指摘で T31 を足した。ほかの項目の src の当たりも同じ見方で確かめ直し、ほかの漏れは無かった)。

検索で参照が見つからず、テストを変えずに通るもの: 1e 以外の単体テストに参照の無い項目、2a(コメントと仕様書)、2d(`classify` の単体テストは `classify` を呼ぶ)、2e(2e-4 を除く)、2f(呼ぶ所は製品のコードだけ)。

### 4.1 core(`crates/core/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T1 ★(承認待ち) | `tests/env_types.rs:54-65`(`定義の誤りの種類は_33_種で識別子が重複せず表示が空でない`) | 1 | **名前を `定義の誤りの種類は_33_種で表示が空でない` に**。`ALL.len()`、識別子の集合、3 つの識別子の確かめを消し、`common::kinds::DEFINITION_ERROR_KINDS` のすべてで `to_string()` が空でないことを確かめる(33 種であることと重複の無いことは、写しの型とコンパイル時の確かめが保つ) | 1f、1g(D4) |
| T2 ★(承認待ち) | `tests/env_types.rs:67-78`(`env_file_の誤りの種類は_13_種で識別子と理由が重複しない`) | 1 | **名前を `env_file_の誤りの種類は_13_種で理由が重複しない` に**。識別子の集合を消し、`ENV_FILE_ERROR_KINDS` で理由の集合が 13 個、表示と理由が空でないことを確かめる | 1f、1g(D4) |
| T3 | `tests/env_types.rs:274`、`:281`、`:287` | 3 | `EnvFileErrorKind::ALL`・`DefinitionErrorKind::ALL` を `common::kinds` の写しに | 1g |
| T4 | `tests/nfr_security.rs:229`、先頭 | 2 | `mod common;` を足し、`DefinitionErrorKind::ALL` を `DEFINITION_ERROR_KINDS` に | 1g |
| T5 | `tests/common/mod.rs`(`pub mod kinds;` と `add_env_file` を足す。`:69-89` の `state` から 2 項目の行を消す)、新しい `tests/common/kinds.rs` | 2 | 既存の補助の名前と引数は変えない | 1d、1g、2b |
| T6 | `tests/graph_drawable.rs:22`(import)、`:26-28`(補助 `drawable`)、`:210`、`:225`、`:238`、`:267` | 6 | 補助の本体を `matches!(layout_graph(&build_graph(..)), GraphLayout::Lanes(_))` に。4 か所の `assert!(!is_lane_drawable(&graph));` を消す(直後の行の `assert_eq!(layout_graph(&graph), GraphLayout::EdgeList)` が同じことを確かめる) | 1c |
| T7 | `tests/graph_layout_oracle.rs:30`、`:182` | 2 | `assert!(matches!(layout_graph(&graph), GraphLayout::Lanes(_)), "{}", example.heading)` に | 1c |
| T8 | `tests/plan_types.rs:62`、`tests/env_merge_perf.rs:115`、`:138`、`:139`、`tests/plan_perf.rs:44`(と先頭に `mod common;`)、`tests/env_merge.rs:78`、`:79`、`:80`、`:108`、`:109`、`:110`、`:172` | 12 | `files.add(r, c)` を `add_env_file(&mut files, r, c)` に(同じ並びで同じ内容が引ける) | 1d |
| T9 | `tests/graph_edges.rs:19`(import)、`:50`、`:63`、`:78`、`:93`、`:107`、`:122`、`:134`、`:145`、`:160` | 10 | ファイルの中の補助 `fn edges(def) -> Vec<FlowEdge> { build_graph(def, None).edges }` を置き、`flow_edges(` を `edges(` に | 1j |
| T10 | `tests/graph_lookup_perf.rs:27`、`:140`、`:165` | 3 | `flow_edges(&def)` を `build_graph(&def, None).edges` に(P4 の名前、上限、結果の中身の確かめは変えない。測る処理に節を作る時間が加わる。前後の時間を記録する) | 1j |
| T11 | `tests/graph_types.rs:91-98`(`ジョブの状態は成功と警告を_ok_にまとめる`)、先頭に `mod common;` | 1 | `node_state_of` を、公開の `overlay_from_state(&common::state(&[(a, Success), (b, Warning), (c, Skipped), (d, Error), (e, Pending)], None, false)).states` の 5 つの状態(`Ok, Ok, Skipped, Error, Pending`)の確かめに(D8) | 1j |
| T12 | `src/flow/record.rs:294`、`:300-301`(単体テスト `初期ステートはすべて_pending`) | 2 | `s.job_order` を `job_states` の ID の並びの確かめに。`definition_key` の 2 行を消す(同じ文字列で書かれることは runtime の `state_write.rs` と `writer.rs` の単体テストが確かめる) | 2b |
| T13 | `src/consistency.rs:78-90`(単体テストの補助 `state`) | 1 | 2 項目の行を消す | 2b |

### 4.2 runtime(`crates/runtime/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T14 | `tests/run_id_state.rs:230`、`:259`、`:276`、`:297`、`:313`、`:340` | 6 | `assert_eq!(report.removed, N)` を消す。同じテストの中のファイルの確かめ(`:230` は `names(p)` の一覧、`:259`・`:276`・`:340` は `exists`、`:297` は `!exists`)が、消したことと消さなかったことを確かめる。**`:313` だけは、`assert!(exists(p, "x.toml.big.state"))` と `assert!(exists(p, "x.toml.ver.state"))` を足す**(残したファイルの有無の確かめが無かったため) | 1b |
| T15 | `tests/run_id_state.rs:33-41`(補助 `state`) | 1 | 2 項目の行を消す | 2b |
| T16 | `tests/state_read.rs:290` | 1 | `assert_eq!(report.removed, 0)` を消す(直後の `:292` の `pipe.exists()` が確かめる) | 1b |
| T17 | `tests/state_write.rs:30-34`(補助 `sample`) | 1 | 2 項目の行を消す(ファイルのキーの確かめ `:104`、`:107` は変えない) | 2b |
| T18 | `src/state/writer.rs:236-240`(単体テストの補助 `state`)、`:324`、`:333-337`、先頭の `:25-29` | 4 | 2 項目の行を消す。`remove_stale_temps` の呼び出しから `SystemTime::now()` を外す。`JobStatus`・`OpenOptions` の `use` を `mod tests` の中へ | 1e、2b、2e-4 |
| T19 | `src/state/reader.rs:376`(単体テスト) | 1 | `state.job_order` を `job_states` の ID の並びの確かめに | 2b |
| T31 | `src/error.rs:150`、`:152`(単体テスト `コマンドエラーの表示は固定の文`) | 2 | `CommandError::new(CommandErrorKind::UnclosedQuote)` を `CommandError { kind: CommandErrorKind::UnclosedQuote }` に(`Empty` も同じ)。名前、確かめる中身(`to_string()` が固定の理由の文と同じ)、期待の文字列(`reason::UNCLOSED_QUOTE`、`reason::EMPTY_COMMAND`)は変えない。確認担当の指摘で足した(§4.0 の注) | 2c |

### 4.3 converter(`crates/converter/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T20 | `tests/nfr_security.rs:313-319`(`表示は_どの種類でも入力の値を含まない`) | 1 | 作った誤りの種類を `HashSet<ConversionErrorKind>` で集め、`common::kinds::CONVERSION_ERROR_KINDS` のすべてが含まれることを確かめる(失敗の文は `{kind:?} のテストがない`)。残りの確かめ(秘密の値を含まない)は変えない | 1f、1g |
| T21 | `tests/common/mod.rs`(`pub mod kinds;`)、新しい `tests/common/kinds.rs` | 1 | 足すだけ | 1g |

### 4.4 cli(`crates/cli/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T22 | `tests/common/mod.rs`: `impl FakeEnv`(`color_read()` を足す)、`:133-137`・`:162`(`SignalMode::Already` を消す)、`:807-815`(`MemRuntime::reset_all_states` の数えを消す。ステートを消すことは変えない)、`pub mod kinds;`、新しい `tests/common/kinds.rs`(5 つの一覧) | 4 | 既存の補助の名前と引数は変えない | 1a、1b、1g、1i |
| T23 | `tests/report_messages.rs:664`、`:669`、`:673`、`:704` | 4 | `reporter.language_resolved()` を `env.locale_read()` に、`reporter.color_resolved()` を `env.color_read()` に(同じテストの `var_calls`・`terminal_calls` の確かめは変えない) | 1a |
| T24 | `tests/report_messages.rs:79`、`:92`、`:134`、`:193`、`:272`、`:1043`、`:1053`、`:1067` | 8 | 5 つの `ALL` を `common::kinds` の写しに | 1g |
| T25 | `tests/report_messages.rs:49-55`(`pipeline_state`)、`tests/graph_report.rs:297-301`、`tests/nfr_robust.rs:166-170`、`tests/flow_pipeline.rs:441-445`、`tests/plan_report.rs:86-90` | 5 | `PipelineState` の組み立てから 2 項目の行を消す | 2b |
| T26 | `tests/report_messages.rs:116-119`、`:123-126`、`:469-472` | 3 | `CommandError { kind, reason }` の `reason` の行を消す(表示は種類の文だけなので、期待は変わらない) | 2c |
| T27 | `tests/flow_run_id.rs:309-312` | 1 | `ResetAllReport { removed: 2, kept: … }` の `removed` の行を消す | 1b |
| T28 | `tests/plan_report.rs:417` | 1 | `files.add(reference, content)` を `let i = files.push_file(content); files.link(reference, i);` に | 1d |
| T29 | `tests/panic_hook.rs` の 2 件 → `src/panic_hook.rs` の `#[cfg(test)] mod tests` | 2 | `フックの本体は固定の英語の文を書き_70_で終わる` と `標準エラーに書けなくても_70_で終わる` を、名前と確かめる中身を変えずに移す(`on_panic(&mut err, &mut \|c\| code = Some(c))` を `let code = write_internal_error(&mut err)` に。`code` の型は `Option<i32>` から `i32` に)。`tests/panic_hook.rs` は消す(3 件目は §4.6) | 1h |
| T30 ★(承認待ち) | `tests/signals_real.rs:15-19`(説明)、`:28`(import の `SignalError`)、`:44`(名前)、`:57-65` | 1 | **名前を `最初のシグナルの番号だけが残る` に**。2 回目の登録の部分(`SystemSignals.start(&other)` が `AlreadyRegistered` で、`QUIT` が配線されない)を消し、説明から「2 回目の `start` が…」を消す。1 回の登録と、最初の番号が残る確かめは変えない。**確かめなくなること**: 2 回目の登録が別の合図に配線されないこと(本番で起こらない場面で、印と一緒に無くなる) | 1i |

### 4.5 テストを変えずに通るもの(参考)

- 2a(コメントと仕様書だけ)、2d(`classify` の単体テストと流れのテストがそのまま確かめる)、2e-1〜2e-3・2e-5、2f(cli の流れのテストと core の計画のテストがそのまま確かめる)。
- 1i の本番の変更(`REGISTERED` を消す): `signals_order.rs` は 1 回だけ登録するので変えない。

**合計**: 変えるテストのファイルは core 13(統合 10: `env_types.rs`、`nfr_security.rs`、`graph_drawable.rs`、`graph_layout_oracle.rs`、`plan_types.rs`、`env_merge_perf.rs`、`plan_perf.rs`、`env_merge.rs`、`graph_edges.rs`、`graph_lookup_perf.rs`、`graph_types.rs` と `common/mod.rs`。src の単体テスト 2: `flow/record.rs`、`consistency.rs`)と新しい `tests/common/kinds.rs`、runtime 6(統合 3: `run_id_state.rs`、`state_read.rs`、`state_write.rs`。src の単体テスト 3: `state/writer.rs`、`state/reader.rs`、`error.rs`(T31))、converter 2 と新しい `tests/common/kinds.rs`、cli 9(`common/mod.rs`、`report_messages.rs`、`graph_report.rs`、`nfr_robust.rs`、`flow_pipeline.rs`、`plan_report.rs`、`flow_run_id.rs`、`signals_real.rs`、`src/panic_hook.rs` の単体テスト)と新しい `tests/common/kinds.rs`。消すファイルは cli `tests/panic_hook.rs`。消すテストのあるファイルは core 1、converter 1、cli 3(§4.6)。名前が変わるのは ★ T1、T2、T30。

### 4.6 消すテストと、同じことを確かめて残るテスト(Q2: A。承認待ち)

消す方針(D3・D10)はユーザーが決めた(2026-11-23)。下の 5 件の個々の削除は、設計の承認で認めてもらう(承認待ち)。

| 消すテスト | 確かめていること | 消す理由 | 同じことを確かめて残るテスト | 残るテストで確かめられない点 |
|------------|------------------|----------|-------------------------------|-----------------------------|
| core `src/error.rs` の単体テスト `全種類に識別子と表示があり重複しない` | `ALL` の 33 種の `code()` が重ならない、`Display` が空でない | `code()` と `ALL` を消す(テストのための関数だけを試していた)。src の単体テストには一覧の写しを置かない | core `tests/env_types.rs` の ★ T1(33 種すべての表示が空でない)。種類の数と重複の無いことは、`tests/common/kinds.rs` の型とコンパイル時の確かめ | 識別子(`code()`)が重ならないこと(関数と一緒に無くなる) |
| converter `src/error.rs` の単体テスト `表示は_どの種類でも入力の値を含まない`(D3) | 6 種のそれぞれで、`ConversionError::new("in.yaml", Some(3), None, kind, "固定の文")` の表示が `in.yaml:3:` と理由を含む、`code()` が空でない | `ALL` と `code()` を消す。src の単体テストに一覧の写しを置かない | converter `tests/nfr_security.rs` の同じ名前のテスト(本物の 6 種の誤りを作り、表示に秘密の値を含まないこと。T20 で全種類の網羅も確かめる)、`tests/convert_ac.rs:179`(行つきの表示 `:7: 文法の誤りです: …`)、`src/error.rs` の `行がなく_キーの場所がある表示` | 種類ごとの「行つきの表示の形」(形は種類によらず同じ組み立てで、上の 2 つが行あり・なしを確かめる) |
| cli `tests/panic_hook.rs` の `設定したフックは_パニックのとき終了の関数を_70_で呼ぶ` | `install_panic_hook_with` で差し替えた終了の関数が、パニックのとき 70 で呼ばれる | 差し替え口を消す(Q3: A) | `src/panic_hook.rs` の単体テスト 2 件(T29。本体の文と 70) | **フックの設定そのもの(パニックのとき 70 で終わること)。確かめられなくなる**(§2.4。Q3 で承知済み。仕様書に記録する) |
| cli `tests/flow_run.rs` の `すでに登録済みでも_流れは成功と同様に続ける` | 偽の `Signals` が `AlreadyRegistered` を返しても、流れが 0 で終わる | `AlreadyRegistered` を消す(Q4: A)。場面そのものが無くなる | (場面が無い)。登録の成功で流れが続くことは、`SignalMode::Noop` を使うすべての流れのテスト、登録の失敗で 70 は `flow_run.rs` の `シグナルの準備に失敗したら_70_で_固定の英語の文`(`SignalMode::Fail`) | なし |
| cli `tests/args_grammar.rs` の `使い方の誤りの全種類が網羅されている` | `UsageErrorKind::ALL.len() == 8` | `ALL` を消す(Q5: A)。写しの長さは型 `[UsageErrorKind; 8]` で決まり、テストが常に真になる(D10) | cli `tests/common/kinds.rs` の網羅の `match` とコンパイル時の確かめ(種類が増えればコンパイルで止まる。今より強い)、`report_messages.rs` の全種類の表示 | なし |

- 件数: core −1(単体)、converter −1(単体)、cli −3(統合。`panic_hook.rs` の 1 件、`flow_run.rs` の 1 件、`args_grammar.rs` の 1 件)。`panic_hook.rs` のほかの 2 件は移動(統合 −2、単体 +2)。

## 5. 足すテスト

test-after(実装のコミットの後に、テストのコミット)。

| ID | 場所 | 確かめること | 件数 |
|----|------|--------------|------|
| N1 | `crates/core/src/flow/decide.rs` の単体テスト `ステートが無いときは_記録が空で再開位置も_end_もないときと同じ判断`(D11) | 3 つのジョブの定義で、(1)`decide_from_state(id, def, None)` が、すべてのジョブで `decide_before_run(id, def, &[], None, false)` と同じ(先頭は `Run`、ほかは `Reject(OutOfOrder)`)、(2)`end_reached = true` で先頭が成功のステートを渡すと、2 つ目が `Skip(AfterEnd)`(ステートの 3 つの項目が渡る)、(3)再開位置が 2 つ目のステートで、2 つ目が `Run`。定義にないジョブ ID は `None` でも `Some` でも `Reject(OutOfOrder)` | 1 |

- 合計の見込み: **+1 件**。消すテスト **−5 件**(§4.6)。移動 2 件(件数は変わらない)。実装で増減したら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 113 − 1(§4.6)+ 1(N1)= **113** | **271** | **384** | 0 |
| runtime | **70** | **128** | **198** | 0 |
| converter | 53 − 1(§4.6)= **52** | **64 + 1 無視** | **116 + 1 無視** | −1 |
| cli | 0 + 2(T29 の移動)= **2** | 462 − 3(`panic_hook.rs`)− 1(`flow_run.rs`)− 1(`args_grammar.rs`)= **457** | **459** | −3 |
| 全体 | | | **1,157 通過、1 無視、0 失敗** | −4(−5 + 1) |

ファイルごと: core `env_types.rs` 6(名前 2 つが変わる)、`graph_types.rs` 5。cli `flow_run.rs` 52、`args_grammar.rs` 60、`signals_real.rs` 1(UNIX。名前が変わる)、`panic_hook.rs`(統合)は無くなる。ほかのファイルの件数は変わらない。基準と見込みは macOS の件数。

## 7. 確かめ方

- 段の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,161 − 消したテスト 5 + 足したテスト 1 = 1,157**」に一致すること [Q1] [Q2]。
- テスト名の一覧の差: 段の前後で `cargo test --locked --workspace -- --list 2>/dev/null | sort` の差が、§4.6 の 5 つの削除、N1 の追加、★ の 3 つの名前の変更(T1、T2、T30)、T29 の 2 件の場所の変更(統合テスト `panic_hook` から `conductor_cli` の単体テスト `panic_hook::tests::…`。名前は同じ)だけであること。
- テストのファイルの変更は §4 の T 表、§4.6、§5 だけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、src の単体テストの差分を見る。
- **本番のコードからの参照が 0 件**(各項目の完了の判定。`inventory.md`)。段 1 の終わりに、次をすべて行う [Q1]:

```sh
grep -rn "language_resolved\|color_resolved" crates docs/spec/rust docs/spec/language-agnostic          # 0 件
grep -rn "\.removed\b\|removed:" crates                                                                    # 0 件
grep -rn "is_lane_drawable" crates docs/spec/rust docs/spec/language-agnostic                              # 0 件
grep -rn "\.add(" crates                                                                                   # 0 件
grep -n "remove_stale_temps(.*now\|now: SystemTime" crates/runtime/src/state/writer.rs                    # 0 件
grep -rn "fn code(self)\|kind\.code()\|k\.code()\|Kind::[A-Za-z]*\.code()" crates                         # 0 件(種類の code() の定義と呼び出しに限る。
#   `.code()` だけで探すと、プロセスの終了コード `status.code()` に当たる。改行で `.code(),` だけが行に残る
#   cli `tests/binary_run_id.rs:231`、`:276` などもプロセスの終了コードで、対象外)
grep -rn "::ALL\b" crates | grep -v "CommandKind::ALL\|Self::ALL"                                          # 0 件
grep -rn "pub const ALL" crates/*/src                                                                      # cli types.rs の CommandKind の 1 件
grep -rn "install_panic_hook_with\|on_panic" crates docs/spec/rust docs/spec/language-agnostic             # 0 件
grep -rn "AlreadyRegistered\|REGISTERED\|SignalMode::Already" crates docs/spec/rust docs/spec/language-agnostic  # 0 件
grep -rn "flow_edges\|node_state_of" crates | grep -v "^crates/core/src/graph/"                            # 0 件
grep -rn "doc(hidden)" crates/*/src                                                                        # converter の testing の 1 件と、その説明の文だけ
```

- 段 2 の終わりに、次を行う:

```sh
grep -rn "validate\` だけ\|validate だけ" crates/*/src docs/spec/rust docs/spec/language-agnostic           # 0 件
grep -rn "自動の再試行はしない\|自動の再試行もしない" crates/*/src                                          # 0 件
grep -rn "\.definition_key\|definition_key:\|\.job_order\|job_order:" crates                               # 0 件
grep -rn "pub reason: &'static str" crates/runtime/src/error.rs                                            # StateError の 1 件
grep -rn "ExitCodeClass" crates docs/spec                                                                  # 0 件
grep -n "pub(crate) const EXIT_\|pub(crate) fn run_dry_run" -r crates/core/src crates/cli/src               # 0 件
grep -n "if bytes.is_empty() =>" crates/runtime/src/state/reader.rs                                        # 0 件
grep -n "Path::new(&request" crates/converter/src/lib.rs                                                   # 0 件
grep -rn "fn decide(" crates/cli/src                                                                       # 0 件
```

- ステートファイルの形が変わらないこと(2b): runtime の `state_write.rs`、`state_read.rs`、`state_read_raw.rs`、`run_id_state.rs` と、cli の `flow_*.rs`・`binary_*.rs` が、変えずに(T 表の組み立ての直しを除く)通ること。あわせて、2b のコミットの前後で、`target/` の下の一時のディレクトリに置いた同じ定義(エラーで止まるジョブと、遷移でスキップするジョブを含む)で `conductor pipeline` と `conductor run` を実行して作ったステートファイルのバイト列が同じことを確かめて記録する(確かめたら消す)。
- P4 の時間(1j): `graph_lookup_perf.rs` の P4 の中央値を、変える前(117.5 ms)と後で記録する。上限 600 ms の 1/3(200 ms)を超えたら記録に理由を書く。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。`unsafe` と src の `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。版の表記(`0.9.8`)は変えない(`git diff -- '*Cargo.toml' Cargo.lock` が空)。README と `docs-check` の語句に触れる変更は無い。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

段 1(15 コミット):

1. cli: `FakeEnv::color_read` を足し、遅延のテストを偽物の記録で確かめ、`Reporter::language_resolved`・`color_resolved` を消す(1a。T22 の一部、T23。仕様書 rust 02)
2. runtime・cli: `ResetAllReport.removed` を消し、`remove_state_file` を `Result<(), _>` に(1b。T14、T16、T22 の一部、T27。仕様書 rust 02)
3. core: `is_lane_drawable` を消す(1c。T6、T7。仕様書 rust 02・05・12)
4. core・cli: `LoadedEnvFiles::add` を消す(1d。T5 の `add_env_file`、T8、T28)
5. runtime: `remove_stale_temps` の `now` を関数の中で取る(1e。T18 の一部)
6. core のテスト: 種類の一覧の写し `tests/common/kinds.rs` を足し、`ALL` の使用を置き換える(1g。T3、T4、T5 の `kinds`。T1・T2 の `ALL` の部分)
7. converter のテスト: 同じ(1g。T20 の `ALL` の部分、T21)
8. cli のテスト: 同じ(5 つの一覧。1g。T22 の `kinds`、T24)。`args_grammar.rs` の `使い方の誤りの全種類が網羅されている` を消す(§4.6)
9. core・runtime・converter・cli: 本番の `ALL` 5 つを消す。src の単体テスト 2 件を消す(§4.6 の core と converter)(1g。仕様書 rust 02)
10. core・converter: `code()` を消し、種類の説明に識別子の規則を書く。テストは種類で数える(1f。★ T1、★ T2、T20 の残り。converter `lib.rs` の説明。仕様書 rust 02・03・08・10・12)
11. cli: パニックのフックの差し替え口をなくし、本体を単体テストへ。`tests/panic_hook.rs` を消す(1h。T29、§4.6。仕様書 rust 02・10・11、language-agnostic 11)
12. cli: シグナルの `AlreadyRegistered` と登録済みの印をなくし、`Signals::start`・`run_cli_with`・`run_cli` の説明に登録はプロセスで 1 回だけと書く(1i。★ T30、T22 の一部、§4.6。仕様書 rust 02・07・10・11・12)
13. core: `node_state_of` を非公開に、`flow_edges` を `pub(crate)` に(1j。T9、T10(前後の時間を記録)、T11。仕様書 rust 02・05・11)
14. (段 1 の確かめ)`inventory.md` の完了の判定の検索と、件数の記録(コミットなし。記録はコード生成の記録に書く)

段 2(11 コミット):

15. cli・core・runtime・converter: 古くなった説明(2a-1、2a-3〜2a-12。コメントだけ)
16. 仕様書: `.env` の重複キーの警告は `validate` と `graph` が出す(2a-2。rust・language-agnostic の 03・08・10)
17. core・runtime・cli: `PipelineState` の `definition_key` と `job_order` を型から消す(2b。T5 の `state`、T12、T13、T15、T17、T18 の一部、T19、T25。ステートファイルのバイト列の前後の比較を記録。仕様書 rust 02・06・12、language-agnostic 06)
18. runtime・cli: `CommandError.reason` を `CommandErrorKind::reason()` に(2c。T26、T31。仕様書 rust 07・10)
19. core: `ExitCodeClass` を消し、`classify_exit_code` が `JobOutcome` を返す(2d)
20. core・cli: `EXIT_*` と `run_dry_run` を非公開に(2e-1、2e-2。仕様書 rust 02・10)
21. runtime: 常に真の条件と、テストだけの `use` の置き場所(2e-3、2e-4。T18 の一部)
22. converter: `Path` の包み直し(2e-5)
23. core・cli: `decide_from_state` を置き、cli の `step::decide` と計画の重複をまとめる(2f。仕様書 rust 02・05)
24. core のテスト: N1
25. 仕様書: rust 11 と language-agnostic 11 のテストのファイルの表と件数(消した `panic_hook.rs`、`kinds.rs`、テストの名前の変更)

合計 **24 コミット**ほど(14 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- 版の更新、タグとリリース。

## 10. 決めてほしい点

D1〜D12 は、ユーザーがすべて推奨のとおりに決めた(2026-11-23)。★ T1・T2・T30 の名前の変更と、§4.6 の 5 つのテストの削除は、設計の承認で認めてもらう(承認待ち)。

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | 誤りの種類の一覧の写しの網羅の形(1g) | **一覧の定数 + 位置を返す網羅の `match`(`const fn`)+ 並びと重複のコンパイル時の確かめ(`const _: () = { … assert! … }`)**(§2.3) | (B)一覧の定数と、何もしない網羅の `match`(`match kind { A \| B \| … => {} }`)だけ | 推奨は、種類の増加に加えて、一覧の並びの誤りと重複もコンパイルで止まる。代案は書き方が短いが、一覧と `match` の食い違い(片方だけに足す、同じ種類を 2 回書く)を止めない。どちらも実行時のテストは増えない | 決定: 一覧の定数 + 位置を返す網羅の `match` + 並びと重複のコンパイル時の確かめ(推奨のとおり。ユーザー、2026-11-23) |
| D2 | 写しの置き場所(1g) | **各クレートの `tests/common/kinds.rs`**(core 2 つ、converter 1 つ、cli 5 つ) | 各 `tests/common/mod.rs` に直に書く | cli の `common/mod.rs` はすでに約 850 行あり、一覧(約 70 行)を分けた方が読みやすい。クレートをまたいだ共有は、テスト用のクレートを新しく作らない限りできない(依存を足さない方針) | 決定: 各クレートの `tests/common/kinds.rs`(推奨のとおり。ユーザー、2026-11-23) |
| D3 | converter の src の単体テスト `表示は_どの種類でも入力の値を含まない`(`ALL` と `code()` を使う) | **消す**(§4.6) | 単体テストの `mod tests` の中に 4 つ目の写し(6 種。網羅の `match` つき)を置いて残す | 確かめる形(行つきの表示)は種類によらず同じ組み立てで、統合テストの同じ名前のテスト(本物の 6 種の誤り)と `convert_ac.rs`(行つきの表示)が確かめる。写しを増やさない | 決定: 消す方針(推奨のとおり。ユーザー、2026-11-23)。個々の削除は §4.6 として設計の承認で認めてもらう(承認待ち) |
| D4 | `env_types.rs` の 2 つのテストの名前(★ T1、T2) | **名前を変える**(「識別子」を外す) | 名前を変えない | 識別子(`code()`)を確かめる部分が無くなるので、名前と中身が食い違う。前の intent の ★ T23 と同じ考え方 | 決定: 名前を変える方針(推奨のとおり。ユーザー、2026-11-23)。★ T1・T2 の個々の名前は設計の承認で認めてもらう(承認待ち) |
| D5 | `SignalError` の形(1i) | **`Failed` だけの列挙型のまま**(`enum SignalError { Failed }`) | 単位構造体 `struct SignalError;` | 推奨は、呼ぶ所の `Err(SignalError::Failed)` が今のまま読め、仕様書 10 章の表の形(種類の一覧)が保てる。代案は 1 種類を素直に表すが、変更の範囲が広がる | 決定: `Failed` だけの列挙型のまま(推奨のとおり。ユーザー、2026-11-23) |
| D6 | `CommandError` の理由の取り出し(2c) | **`CommandErrorKind::reason()` だけを置き、`CommandError::reason()` は作らない**。`CommandError::new` は消す | `CommandError::reason(&self)` も置く | 理由を読むのは `Display` だけで、`.kind.reason()` で書ける(`StateError` の `.kind.label()` と同じ書き方。`EnvFileErrorKind::reason()` と同じ置き場所)。使われないメソッドを足さない | 決定: `CommandErrorKind::reason()` だけを置き、`CommandError::new` は消す(推奨のとおり。ユーザー、2026-11-23) |
| D7 | ステートが無いときの判断の関数(2f)の名前と場所 | **core の `flow::decide_from_state(job_id, definition, state: Option<&PipelineState>)`**(公開。`lib.rs` からも再公開) | (B)名前を `decide_before_run_with_state` に、(C)cli の `step::decide` を残し、core の `plan_run` だけを直す(重複は残る) | cli と core の計画の両方が使うので core に置く。名前は、既存の `decide_before_run`(項目を個別に受け取る)と並べて「ステートから決める」と読める短い形 | 決定: core の `flow::decide_from_state`(推奨のとおり。ユーザー、2026-11-23) |
| D8 | `node_state_of` のテスト(T11)の確かめ方 | **公開の `overlay_from_state` で確かめる**(core の `common::state` で 5 つの状態のステートを作る) | (B)`build_graph(def, Some(&overlay))` の節の状態で確かめる、(C)`overlay.rs` の単体テストに移す | 推奨は、`node_state_of` を呼ぶ唯一の公開の入口で、確かめる 5 つの対応がそのまま読める。(B)は定義も要り遠回り。(C)は公開の入口から確かめる方針 (a) から外れる | 決定: 公開の `overlay_from_state` で確かめる(推奨のとおり。ユーザー、2026-11-23) |
| D9 | `kept` だけになった `ResetAllReport`(1b) | **構造体のまま残す**(`ResetAllReport { kept: Vec<String> }`) | `reset_all_states` が `Vec<String>` を返す | 推奨は、cli の `report.kept` と `Runtime` トレイトの署名、テストの `Ok(Default::default())` が今のまま。代案は公開の関数の形が変わり、変更の範囲が広がる | 決定: 構造体のまま残す(推奨のとおり。ユーザー、2026-11-23) |
| D10 | `args_grammar.rs` の `使い方の誤りの全種類が網羅されている`(`ALL.len() == 8` だけ) | **消す**(§4.6) | 写しの長さの確かめとして残す | 写しは型 `[UsageErrorKind; 8]` なので、残すと常に真のテストになる(construction の規則「常に通るテストを作らない」)。網羅は写しのコンパイル時の確かめが保つ | 決定: 消す方針(推奨のとおり。ユーザー、2026-11-23)。個々の削除は §4.6 として設計の承認で認めてもらう(承認待ち) |
| D11 | 2f の足すテスト | **N1(`decide.rs` の単体テスト 1 件)を足す** | 足さない(既存の流れと計画のテストに任せる) | 新しい公開の関数の契約(`None` は空の記録・再開位置なし・`end_reached` 偽と同じ)を、直接 1 か所で確かめられる。正しい場合と、境界(`end_reached`、再開位置)と誤り(定義にないジョブ)を含む | 決定: N1 を足す(推奨のとおり。ユーザー、2026-11-23) |
| D12 | language-agnostic 06 の `definition_key` の文(2b) | **「読むときは別々に保持するが」を「読んだ値を保持するかは実装による」に直す**(ファイルの形と検査の記述は変えない) | language-agnostic は変えない(rust だけ直す) | 言語によらない仕様が「保持する」と言い切ると、rust の実装と食い違う。凍結した言語の仕様書は触らない | 決定: 「読んだ値を保持するかは実装による」に直す(推奨のとおり。ユーザー、2026-11-23) |

**設計の承認で認めてもらうもの(ユーザーが設計の承認で認めた、2026-11-23)**: §4 の T 表の置き換えの書き方(T6 の `matches!`、T8 の `add_env_file`、T9 の補助 `edges`、T10 の P4 の測る処理、T11 の `overlay_from_state`、T14 の `:313` の有無の確かめの追加、T29 の単体テストへの移動)、★ T1・T2・T30 の名前の変更、§4.6 の 5 つのテストの削除(うち 1 つは確かめられなくなる)、§5 の N1。

## 11. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | (1)テストの都合で曲げられた所: 挙げた項目のそれぞれについて、本番からテストのためだけの関数・項目・引数・公開範囲・差し替え口がなくなり、テストは 4 つの方針のどれかで代わりを持つ。パニックのフックは差し替え口をなくし、確かめられなくなることを記録する。シグナルの「登録済み」と印をなくし、テストは 1 回だけ登録する。誤りの種類の一覧はテストの側へ移し、網羅の `match` を付ける | BR1.1〜BR1.10 |
| SM2 | (2)ふつうの整理: 説明が実装と一致し、使われない型の項目と二重の処理がなくなる。ステートのファイルの形は変わらない | BR2.1〜BR2.6 |
| SM3 | 確かめる手段: 項目ごとの今の場所・目標・完了の基準の一覧と、実装の後の検索 | BR9.2、BR9.3 |
| SM4 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM5 | 既存のテストの扱い(T 表、消すテストの対の表、検索で網羅)と件数(基準 − 消したテスト + 足したテスト) | BR8.1、BR9.2、BR9.3 |
| SM6 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
