# 機能の仕様(Functional Spec): テストだけが使う本番のコードの削除

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A、Q4 A、Q5 A)。**変える場所の正確な一覧(ファイルと行、目標の形、完了の判定、使った検索のコマンド)と、基準の件数、写しの木での検証は `inventory.md`**(HEAD 9e91512 で作った)。この文書は、方針、項目ごとの設計、**承認が要るテストの変更(§4 の T 表)と消すテストの対の表(§4.6)**、件数の見込み、確かめ方、コミットの並び、範囲の外、決めてほしい点を書く。

## 1. 方針

- **テストのコードの都合で本番のコードを曲げない。本番に、テストのための要素を新しく足さない**(ユーザーの判断)。本番のコードから、テストだけが使う関数・実装・公開範囲を消す。消す・狭める項目を使っていたテストは、次のどれかで代わりを持つ [desc] [Q1]:
  - (a) 公開の入口から確かめる(例: `decide_before_run` の代わりに `decide_from_state`)
  - (b) 偽物の側で確かめる(この intent では該当なし)
  - (c) テストの補助に、本番と同じ作り方を書く(例: `"c".into()` の代わりに `Command::Line("c".into())`)
  - (d) 同じことを確かめる残りの確かめに任せる。消す関数・実装だけを試していた確かめは、それと一緒に消える(§4.6)
- 利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` と `graph` の出力)を変えない。版は上げない(`0.9.8` のまま)[desc]。
- 1 項目 1 コミットの小さな並びで、どのコミットでもビルド、`clippy -D warnings`、テストが通る [Q1] [memory:M1] [memory:M4]。
- 承認は、この設計とコードの計画の 1 回。実装は小さなコミットで進め、結果は最後にまとめて見せる [Q1]。
- 既存のテストの変更は、**書き方の置き換えだけ**。確かめる利用者から見える動きと、**期待する出力の文字列は変えない** [Q1]。消すテストは §4.6 に理由と、同じことを確かめて残るテストを示す。テストの名前を変えるもの(★)は無い。
- 新しい依存を足さない [memory:M5]。`unsafe` と src の `cfg(unix)`/`cfg(windows)` を足さない。runtime の製品のコード(src の `#[cfg(test)]` より前の部分)に `.unwrap()`・`.expect(`・`panic!(` などを書かない。cli の src に `panic!(` を書かない。converter は core に依存しない。MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。誤りの表示に入力の値を含めない。
- 仕様書は `docs/spec/rust` と `docs/spec/language-agnostic` を直す。凍結した `docs/spec/{go,python,typescript,csharp,powershell}` は触らない。**版を上げないので、「v0.9.8 までは…」などの版の注記は足さず、今の形だけを書く**(既存の版の注記は残す)。language-agnostic の関数の名前は「仮の名前」なので、論理が変わらない限り変えない。
- ライブラリの誤りの型(`DefinitionError`、`EnvFileError`、`CommandError`、`StateError`、`CreateError`、`ConversionError`)の `Display` と `std::error::Error` は残す [Q5]。

## 2. 項目ごとの設計(詳細は `inventory.md` の同じ番号)

### 2.1 削除

| 項目 | 本番から消すもの | テストの代わり(方針) |
|------|------------------|------------------------|
| 1a | core `Command` の `From<&str>`・`From<String>` | (c) `Command::Line(..)` を書く。runtime の補助は §2.4(D5)。`From` だけを試す単体テスト 1 件は消す(§4.6) |
| 1b | cli `RunIdArg::as_str` | (a) `RunIdArg` の値で比べる(`PartialEq`)。`args_grammar.rs` はファイルの中の補助 `run_id(text)` |
| 1c | runtime `InterruptSignal` の `is_raised`・`PartialEq`・`Eq`(`Default` は残す。D1。決定(ユーザー、2026-11-24)) | (a) `signal_number()`(`None` は立っていない)。`Debug` の `raised` は `signal_number().is_some()` で書く(D8)。等価性だけの確かめ 2 行は消す |
| 1d | 本番で使われない `Default`: core `JobStatus`、runtime `Durability`、core `graph::StateOverlay`、cli `Language` | (c) 空の `StateOverlay` は項目を書いて作る。`Default` を試すテスト 2 件は消す(§4.6。D3)、`JobStatus::default()` の 1 行は消す |
| 1e | core `sanitize` と再公開 | 呼ぶ所(本番 2 か所、単体テスト 6 か所)は `escape_display(x, Some(n))`。cli の再公開と core の包みが同じことの確かめ 1 件は消す(§4.6) |
| 1f | core `EnvFileError::reason()` | (c) `err.kind.reason()`。写しを確かめる 1 行は消す |

### 2.2 公開範囲を狭める

| 項目 | 今 | 目標 | テストの代わり |
|------|----|------|----------------|
| 2a | core `decide_before_run`(公開、`flow` と根で再公開) | `decide.rs` の中だけの非公開の `fn`(D6) | (a) `decide_from_state`。core の模型 `Sim::decide` はステートを組んで渡す。runtime の `decide` の写しは消す(D7) |
| 2b | core `flow::classify`(公開) | `pub(super)`(`next_step.rs` が使う。D6) | 外のテストの使用は 0 件 |
| 2c | core `MAX_RUN_ID_LEN`・`is_valid_env_name`・`is_reserved_env_name`(根で再公開)、`JobDefinition::effective_success_codes`(公開) | `MAX_RUN_ID_LEN` は `run_id.rs` の中だけ、ほかの 3 つは `pub(crate)`(D6) | `effective_success_codes` の 2 か所だけ: 1 行は消し、1 行は `success_exit_codes` の値で比べる |
| 2d | cli `report::escape` の `escape_display` の再公開 | 非公開の `use` | (a) テストは本体の `conductor_core::escape_display` を直接使う |
| 2e | cli `lib.rs` の `pub mod args`・`deps`・`panic_hook`・`types` | `mod`(中の公開の項目は根で再公開済み) | 使用 0 件 |
| 2f | cli 根の `GraphOptions`・`DryRunOptions` の再公開 | 再公開を消す | 使用 0 件 |

### 2.3 判断が要ったもの

| 項目 | 目標 | テストの代わり |
|------|------|----------------|
| 3a(Q2) | core `RunIdError` を種類のない誤りの型 `pub struct RunIdError;` に(D2)。`RunId::parse` は 4 つの条件のどれかで `Err(RunIdError)` | 単体テスト 3 件は `Err(RunIdError)` と比べる。4 つの場合の入力は残す |
| 3b(Q3) | runtime の core の再公開(`check_consistency`、`apply_interrupt`・`apply_outcome`・`apply_skip`、`initial_state`、`ConsistencyResult`、`MismatchKind`)をやめる | 使う所(cli の本番 5 ファイル、runtime のテスト 4 ファイル、cli のテスト 1 ファイル)の `use` を `conductor_core` に |
| 3c(Q4) | core `DefinitionErrorKind`・`EnvFileErrorKind` の `Display` と `Error`、converter `ConversionErrorKind` の `Error` を消す | (c) `to_string()` を `label()` に(2 か所) |
| 3d(Q4) | `Hash` を消す: core `DefinitionErrorKind`・`EnvFileErrorKind`、converter `ConversionErrorKind`・`TargetFormat` | (c) `HashSet<Kind>` を `Vec<Kind>`(`push` と `contains`)に(2 ファイル) |

各項目のコミットで、`docs/spec/rust`(と、該当があれば `docs/spec/language-agnostic`)の行と、消す・狭める名前を指すドキュメントコメントを合わせる(`inventory.md` の各項目の「仕様書」「ドキュメントコメント」)。仕様書の変更は内部の名前と公開範囲だけで、利用者から見える挙動の記述は変えない。

### 2.4 1a のテストの補助(D5)

runtime のテストには、`impl Into<Command>` を受け取る補助が 3 つある。`From` が無くなると、文字列を渡す呼び出しがすべて止まる(写しのコンパイルで 30 か所ほど)。呼び出しの書き換えを少なくするため、補助の引数の形を、呼ばれ方に合わせて変える:

| 補助 | 呼ばれ方 | 変え方 | 呼び出しの変更 |
|------|----------|--------|----------------|
| `tests/common/mod.rs` の `command_line(command)` | 文字列だけ(`runner_env`、`runner_signal`、`nfr_robust`、`runner_basic` の閉包) | 引数を `impl Into<String>` にし、中で `Command::Line(command.into())` | なし |
| `tests/common/mod.rs` の `job(id, command)` | 文字列だけ(`flow_with_state`、`nfr_content`、`nfr_performance`) | 同上 | なし |
| `tests/runner_basic.rs` の `run(command)` | 文字列と `Command::Argv(..)` の両方 | 引数を `Command` にし、中で `check_command(&command).expect("分けられる")` | 文字列を渡す 10 か所を `Command::Line(..)` で包む(うち 2 か所は Windows でだけコンパイルされる) |

core のテストの補助(`common::job`、`graph_common::definition`、単体の `test_support::job`)は、`"c".into()` などを `Command::Line(..)` に書き換える(引数の形は変えない)。

### 2.5 3a の設計(`RunIdError`。Q2: A)

```rust
/// run ID の検査に通らなかった(値は持たない。理由は区別しない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunIdError;

/// 値を検査して、run ID にする。空、64 バイト超、`A-Za-z0-9._-` 以外の文字、先頭が `.` のどれかなら誤り。
pub fn parse(value: &str) -> Result<RunId, RunIdError> {
    if value.is_empty()
        || value.len() > MAX_RUN_ID_LEN
        || !value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        || value.starts_with('.')
    {
        return Err(RunIdError);
    }
    Ok(RunId(value.to_string()))
}
```

- 本番の呼ぶ所(cli 2 か所、runtime 1 か所)は今も種類を見ないので、変わらない。単体テストは `Err(RunIdError)` と比べ、4 つの場合の入力(空、65 文字、`.a`・`..`・`.`、`a/b` などの 14 個)が通らないことを確かめる [Q2]。
- 仕様書 rust 05 の 6.1 の表は「誤り」の列を消し、4 つの条件を残す。language-agnostic は D4。

### 2.6 3b の設計(runtime の再公開。Q3: A)

- 再公開をやめる 7 つの名前は、どれも core の `consistency.rs` と `flow/record.rs` が定める純粋な関数と型で、runtime の本番は使わない。cli は core にも直接依存しているので、`use` を `conductor_core` に移すだけで中身は変わらない。
- 再公開が無くなると、`conductor_runtime` を通した参照が残ればコンパイルが止まる。終わりの基準(`conductor_runtime::` を通した参照が 0 件)は、ビルドが通ることと、`inventory.md` §8 の `awk` の検索で確かめる [Q3]。
- runtime のモジュールの説明(`lib.rs` の関数の表と流れの説明)から、これらへのリンクを外し、「conductor-core の関数を使う」と書く(`cargo doc` のリンクの警告を増やさない)。

## 3. 実物で確かめて調査と違った点、範囲から外したもの

- **調査と違った点**(実物に従った):
  - 1a: `From` の使用は、名前の検索(`Command::from`)では 2 か所しか見えないが、`.into()` と `impl Into<Command>` の形で、core・runtime のテストと単体テストに約 45 か所ある(写しのコンパイルで網羅した)。そのうち runtime の補助(`common::job`・`common::command_line`)を文字列で呼ぶ 13 か所は、補助の引数を変えれば変えずに済む(§2.4)。
  - 1c: `InterruptSignal` の `Default` を消すと、clippy の `new_without_default` で `-D warnings` が止まる。D1 で残すと決めた(ユーザー、2026-11-24)。`Debug` は `is_raised()` を呼ぶので、`signal_number()` で書き直す(D8)。
  - 1d-3: `StateOverlay::default()` を使うテストが 2 つある(`graph_overlay.rs` の空の重ね合わせと、`graph_types.rs` の `Default` を試すテスト)。
  - 2c-4: `effective_success_codes` の使用は、core の統合テスト `definition_valid.rs` の 2 か所(調査は使用の場所を挙げていなかった)。
  - 2e: 仕様書 rust 02 の cli の公開モジュールの列に、今も公開の `run_id` が無い。狭めるついでに合わせる。
  - 3a: `RunIdError` を捨てる本番の場所は、cli 2 か所に加えて runtime の `reset_all.rs` の `.ok()` もある(種類を見ないので変わらない)。
  - 3b: cli の本番で再公開を使うのは、`ctx.rs`・`pipeline.rs`・`step.rs`・`report/mod.rs`・`report/messages.rs` の 5 ファイル。runtime のテストは `common/mod.rs`・`flow_with_state.rs`・`nfr_content.rs`・`nfr_performance.rs`、cli のテストは `report_messages.rs`。
  - 文書: 消す・狭める名前へのドキュメントのリンクが 20 か所ある(core 4、runtime 13、cli 3)。直さないと `cargo doc` の警告が基準の 5 件から 25 件に増える(CI は `cargo doc` を走らせないが、説明の正しさとして直す)。
- **範囲から外したもの**: 初期の記述の「範囲の外」(読まれない項目(`FlowEdge.explicit`、`EnvOrigin`、`EnvFileContent.path` と `EnvEntry.line`、`format_version` と `description`)、converter の `testing` モジュール、cli のライブラリの公開範囲の大きい部分)と、ライブラリの誤りの型の `Display`・`Error`(Q5)、converter の `sanitize`(core に依存しないための写し)、runtime の `InterruptSignal` の `Default`(clippy の `new_without_default` が求める慣例の実装。D1。決定(ユーザー、2026-11-24))。
- **2f で残る形(記録)**: `GraphOptions`・`DryRunOptions` は、再公開を消すと名前で書けなくなるが、`CliCommand` の公開の項目 `graph`・`dry_run` の型として残る(rustc の `unnameable_types` は既定で警告しない)。`CliCommand` の形を変えるのは、範囲の外の「cli のライブラリの公開範囲の大きい部分」で扱う。

## 4. 変えるテストの一覧(ユーザーの承認が要る)

Q1: A に従い、変えるのは**消す・狭める名前の置き換え(公開の入口、本番と同じ作り方、`use` の移動)**だけで、**確かめる利用者から見える動きと、期待する出力の文字列は変えない**。テストの名前を変えるもの(★)は無い。件数が変わるのは §4.6 の消すテスト(−4)だけ。

### 4.0 網羅に使った検索のコマンド

全体は `inventory.md` §8。T 表は、**消す・狭める名前のすべての参照**を、次の `grep -rn` で `crates` の全体(`tests/` の下の補助のディレクトリと、src の `#[cfg(test)]` の単体テストを含む)を検索して作り、さらに **HEAD の写しの木で本番のコードだけを変えてコンパイルし、エラーの出た場所がすべて T 表にあること**を確かめた(`inventory.md` §0.2。前の intent で、型を定義するファイルの中の単体テストが検索から漏れたため)。

```sh
grep -rn "From<&str> for Command\|From<String> for Command\|Command::from\|Into<Command>\|command: .*\.into()" crates
grep -rn "as_str" crates/cli
grep -rn "is_raised\|InterruptSignal::default\|assert_ne!(s, InterruptSignal" crates
grep -rn "JobStatus::default\|Durability::default\|StateOverlay::default\|Language::default\|Default::default()" crates
grep -rn "sanitize" crates
grep -rn "\.reason()" crates | grep -v "kind.reason()\|kind().reason()"
grep -rn "decide_before_run" crates
grep -rn "flow::classify\|[^_]classify(" crates | grep -v "classify_and_decide\|classify_exit_code"
grep -rn "MAX_RUN_ID_LEN\|is_valid_env_name\|is_reserved_env_name\|effective_success_codes" crates
grep -rn "escape_display" crates
grep -rn "conductor_cli::\(args\|deps\|panic_hook\|types\)\|GraphOptions\|DryRunOptions" crates
grep -rn "RunIdError" crates
for f in $(grep -rl "conductor_runtime" crates); do awk -v F=$f '/use conductor_runtime/{u=1} u{print F":"NR": "$0} /;/{u=0}' $f; done \
  | grep -E "check_consistency|apply_interrupt|apply_outcome|apply_skip|initial_state|ConsistencyResult|MismatchKind"
grep -rn "to_string()" crates/core/tests/env_types.rs
grep -rn "HashSet" crates/*/tests crates/*/src
# 写しの木(本番だけを変えた段階)で、エラーの場所を出す
cargo check --workspace --locked --all-targets --keep-going --message-format=short 2>&1 | grep -E "^crates/.*: error"
```

検索で参照が見つからず、テストを変えずに通るもの: 1d-4(`Language`)、2b(`classify`)、2c-1〜2c-3、2e、2f、3c-3、3d-2、3d-4。

### 4.1 core(`crates/core/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T1 | `src/types.rs:603`(単体テスト `新しいジョブの状態は_pending`) | 1 | `assert_eq!(JobStatus::default(), JobStatus::Pending);` を消す(`:602` が新しいジョブの状態が `Pending` であることを確かめる) | 1d-1 |
| T2 | `src/test_support.rs:18`(import)、`:24`(単体テストの補助 `job`) | 2 | `command: "c".into()` を `Command::Line("c".into())` に、import に `Command` を足す | 1a |
| T3 | `src/error.rs:281`、`:285`、`:317`、`:324`、`:329`、`:335`(単体テスト `書式文字と双方向制御文字もエスケープする`、`キー名の制御文字はエスケープされる`、`キー名は_64_文字で切られる`、`エスケープ後も_64_文字を超えない`) | 6 | `sanitize(x, n)` を `escape_display(x, Some(n))` に(名前、入力、期待は変えない) | 1e |
| T4 | `src/run_id.rs:134`、`:135`、`:140`、`:141`、`:142`、`:152`(単体テスト `長さの境界は_64_と_65`、`先頭のドットは拒否する`、`使えない文字は拒否する`) | 6 | `Err(RunIdError::TooLong)` などを `Err(RunIdError)` に(4 つの場合の入力は変えない) | 3a |
| T5 | `tests/common/mod.rs:25-27`(import)、`:62`(補助 `job`) | 2 | `"c".into()` を `Command::Line("c".into())` に | 1a |
| T6 | `tests/graph_common/mod.rs:24-26`(import)、`:37`(補助 `definition`) | 2 | `format!("echo {id}").as_str().into()` を `Command::Line(format!("echo {id}"))` に | 1a |
| T7 | `tests/definition_valid.rs:49`、`:79` | 2 | `"…".into()` を `Command::Line("…".into())` に(`Command` は import 済み) | 1a |
| T8 | `tests/definition_valid.rs:52`、`:57`(テスト `正しい定義をファイルから読める`) | 2 | `:52` の `extract.effective_success_codes() == [0]` を消す(`:51` が `success_exit_codes == None` を確かめ、`None` を `[0]` とすることは単体テスト `types.rs:524` と `classify.rs` が確かめる)。`:57` を `transform.success_exit_codes == Some(vec![0, 2])` に。消す `:52` で減るのは「定義ファイルから読んだ定義の既定の成功コードを値で見る」確かめだが、読み取りが省略を `None` で持つこと(`:51`)と、`None` を `[0]` として扱うこと(単体 `types.rs:524`、`classify.rs` の単体テスト)の間に別の変換は無い(読み取り → `JobDefinition` → `classify`)ので、2 つを合わせて同じことが確かめられる(§4.6 の下の注) | 2c-4 |
| T9 | `tests/nfr_security.rs:23`(import)、`:34` | 2 | `command.into()` を `Command::Line(command.into())` に | 1a |
| T10 | `tests/nfr_security.rs:20`(import)、`:192`、`:202`、`:213`、`:218`、`:223`、`:230`(テスト `nfr4_6_すべての種類で表示の文に入力の値が出ない`) | 7 | `HashSet<DefinitionErrorKind>` を `Vec<DefinitionErrorKind>` に、`insert` を `push` に(`:233` の `contains` はそのまま) | 3d-1 |
| T11 | `tests/env_file_parse.rs:179` | 1 | `err.reason()` を `err.kind.reason()` に | 1f |
| T12 | `tests/env_types.rs:270`(テスト `誤りの理由は種類の固定の文で_種類の文は変更前と同じ`) | 1 | `assert_eq!(e.reason(), kind.reason(), …)` を消す(直後の `:271` が、表示に種類の理由の文を含むことを確かめる) | 1f |
| T13 | `tests/env_types.rs:59`、`:69` | 2 | `k.to_string()` を `k.label()` に | 3c |
| T14 | `tests/flow_sequences.rs:23-27`(import)、`:68-76`(模型 `Sim::decide`) | 2 | `decide_before_run(id, &self.def, &self.states, self.resume.as_deref(), self.end_reached)` を、模型の項目から組んだ `PipelineState`(名前、パス `t.toml`、状態の並び、再開位置、`end_reached`、`completed`、run ID なし)で `decide_from_state(id, &self.def, Some(&state))` に。模型の項目と、各テストは変えない | 2a |
| T15 | `tests/graph_overlay.rs:110` | 1 | `StateOverlay::default()` を `StateOverlay { states: vec![], resume_job_id: None }` に | 1d-3 |
| T16 | `tests/graph_types.rs:22`(import) | 1 | 使わなくなる `StateOverlay` を消す(テスト `状態の重ね合わせの既定は空で_再開位置もない` を消すため。§4.6) | 1d-3 |

### 4.2 runtime(`crates/runtime/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T17 | `src/types.rs:158`、`:160`(単体テスト `合図は最初は立っていない`)、`:168`、`:170`(`合図は立ったら下りず最初の番号を保つ`)、`:177`(`予約値の番号でも立つ`) | 5 | 5 行を消す。`:158`・`:168`・`:177` の `is_raised()` は、すぐ後ろの行が同じことを `signal_number()` で確かめる。`:160`(`== InterruptSignal::default()`)・`:170`(`assert_ne!(s, InterruptSignal::new())`)は等価性の確かめ(`:160` は `Default` を残しても `==` が無くなるので消す)。`合図はスレッド間で共有できる` の `raised: true` の確かめは変えない(D8) | 1c |
| T18 | `src/runner.rs:229`(単体テストの補助 `line`) | 1 | `&command.into()` を `&conductor_core::Command::Line(command.into())` に(このファイルは `std::process::Command` を import しているため、パスを書く) | 1a |
| T19 | `src/split.rs:271`、`:274`(単体テスト `check_command_は分けた結果を返す`) | 2 | `&"…".into()` を `&Command::Line("…".into())` に | 1a |
| T20 | `tests/common/mod.rs:36-38`(`command_line`)、`:68-71`(`job`)、`:113`(`without_run_id::initial_state`) | 3 | `command_line`・`job` の引数を `impl Into<String>` にし、中で `Command::Line(command.into())`(§2.4。D5)。`conductor_runtime::initial_state` を `conductor_core::initial_state` に | 1a、3b |
| T21 | `tests/runner_basic.rs:23-30`(補助 `run`)、`:41`、`:125-128`、`:146`、`:153`、`:162`、`:190`、`:200`、`:230`、`:292`、`:332`、`:171`、`:173` | 14 | `run` の引数を `Command` にし、中で `check_command(&command).expect("分けられる")`。文字列を渡す 10 か所を `run(Command::Line(..))` に(`:292`・`:332` は `#[cfg(windows)]` のテスト)。`:171` は `command_line(helper_command(..))`、`:173` は `Command::Line("'".into())` | 1a |
| T22 | `tests/nfr_robust.rs:20`(import)、`:150`、`:160`、`:163`、`:171` | 5 | `check_command(&s.into())` などを `check_command(&Command::Line(s))` に | 1a |
| T23 | `tests/nfr_content.rs:20-23`(import)、`:88`、`:93` | 3 | `apply_outcome` の import を `conductor_core` に、`Command` を import に足す。`&input.into()`・`&"   ".into()` を `Command::Line(..)` に | 1a、3b |
| T24 | `tests/nfr_performance.rs:25-26`(import) | 1 | `ConsistencyResult`・`check_consistency` を `conductor_core` から | 3b |
| T25 | `tests/flow_with_state.rs:15`(モジュールの説明)、`:30-39`(import)、`:140-156`(補助 `decide`)、`:256`、`:350` | 5 | 補助 `decide`(core の `decide_from_state` と同じ `match` の写し)を消し、2 か所は `decide_from_state` を直接呼ぶ(D7)。import は `decide_before_run` を `decide_from_state` に、再公開の 5 つ(`ConsistencyResult`、`MismatchKind`、`apply_outcome`、`apply_skip`、`check_consistency`)を `conductor_core` に。説明の `decide_before_run` を `decide_from_state` に | 2a、3b |

### 4.3 converter(`crates/converter/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T26 | `tests/nfr_security.rs:19`(import)、`:314`(テスト `表示は_どの種類でも入力の値を含まない`) | 2 | `HashSet<K>` を `Vec<K>` に(`:316` の `contains` はそのまま)。import から `HashSet` を消す | 3d-3 |

### 4.4 cli(`crates/cli/`)

| ID | ファイル:行 | 箇所 | 変える内容 | 理由(項目) |
|----|-------------|------|------------|------------|
| T27 | `tests/run_id_arg.rs:22`、`:25`、`:27`、`:32`(テスト `run_id_arg_は空を固定_規則どおりを値_規則違反を値を含めない誤りにする`) | 2 | `:25` の `empty.as_str() == ""` を消す(`:24` が `RunIdArg::Empty` であることを確かめる)。`:32` の `arg.as_str() == ok` を `assert!(matches!(&arg, RunIdArg::Id(id) if id.as_str() == ok))` に(書かれた値をそのまま持つこと)。コメント(`:22` の「書かれた値は空文字」、`:27` の「返す」)を合わせる | 1b |
| T28 | `tests/args_grammar.rs:21-26`(import と補助)、`:494-498`、`:503-516`(テスト `run_id_は_4_つのコマンドで解釈される`)、`:702`(`graph_は_全オプションを受け取り_既定は_text_で_状態なし`)、`:795`(`dry_run_と_assume_は_run_と_pipeline_で_複数回指定できる`) | 5 | ファイルの中に補助 `fn run_id(text: &str) -> Option<RunIdArg>` を置き、`cmd.run_id.as_ref().map(RunIdArg::as_str) == Some("r1")` を `cmd.run_id == run_id("r1")` に、`Some("")` との比較を `Some(RunIdArg::Empty)` との比較に。import に `conductor_core::RunId` | 1b |
| T29 | `tests/signals_real.rs:46` | 1 | `assert!(!signal.is_raised())` を `assert_eq!(signal.signal_number(), None)` に | 1c |
| T30 | `tests/signals_order.rs:38` | 1 | `while !signal.is_raised() && …` を `while signal.signal_number().is_none() && …` に | 1c |
| T31 | `tests/escape_table.rs:15-19`(モジュールの説明)、`:21`(import) | 2 | 説明から「core の `sanitize` も同じ関数を使う」「cli の `escape_display` は…再公開」を消す。`escape_display` を `conductor_core` から import する(`escape_full`・`escape_name` は cli から) | 1e、2d |
| T32 | `tests/report_messages.rs:29-36`(import) | 1 | `MismatchKind` を `conductor_core` から | 3b |

### 4.5 テストを変えずに通るもの(参考)

- 1d-4、2b、2c-1〜2c-3、2e、2f、3c-3、3d-2、3d-4(検索でも写しのコンパイルでも、テストの参照が 0 件)。
- 1a のうち、runtime の補助 `common::job`・`common::command_line` を文字列で呼ぶ 13 か所(`flow_with_state.rs:70`、`:735`、`nfr_content.rs:100`、`nfr_performance.rs:36`、`nfr_robust.rs:180`、`:187`、`runner_env.rs:29`、`runner_signal.rs:38`、`:118`、`:154`、`:196`、`runner_basic.rs:218`、`:261`)は、補助の引数を `impl Into<String>` にするので変えない(D5)。

**合計**: 変えるテストのファイルは core 13(src の単体テストと補助 4: `types.rs`、`test_support.rs`、`error.rs`、`run_id.rs`。統合 9: `common/mod.rs`、`graph_common/mod.rs`、`definition_valid.rs`、`nfr_security.rs`、`env_file_parse.rs`、`env_types.rs`、`flow_sequences.rs`、`graph_overlay.rs`、`graph_types.rs`)、runtime 9(src の単体テスト 3: `types.rs`、`runner.rs`、`split.rs`。統合 6: `common/mod.rs`、`runner_basic.rs`、`nfr_robust.rs`、`nfr_content.rs`、`nfr_performance.rs`、`flow_with_state.rs`)、converter 1(`nfr_security.rs`)、cli 6(`run_id_arg.rs`、`args_grammar.rs`、`signals_real.rs`、`signals_order.rs`、`escape_table.rs`、`report_messages.rs`)。消すファイルは無い。名前が変わるテスト(★)は無い。

### 4.6 消すテストと、同じことを確かめて残るテスト(Q1: A。ユーザーが設計の承認で認めた、2026-11-24)

| 消すテスト | 確かめていること | 消す理由 | 同じことを確かめて残るテスト | 残るテストで確かめられない点 |
|------------|------------------|----------|-------------------------------|-----------------------------|
| core `src/types.rs` の単体テスト `文字列から_line_を作れる`(`:590-597`) | `Command::from("echo a")` と `Command::from(String)` が `Command::Line` になる | `From` を消す(1a)。`From` だけを試していた | 文字列の `command` が `Command::Line` として読まれることは、`tests/definition_valid.rs` の `コマンドの文字列はそのまま保持される` と `nfr_security.rs` の `nfr4_5_コマンドの文字列はそのまま保持され展開も起動もされない` が確かめる | なし(変換そのものが無くなる) |
| core `tests/graph_types.rs` の `状態の重ね合わせの既定は空で_再開位置もない`(`:43-53`。D3) | `StateOverlay::default()` が空で再開位置なし。項目を書いて作った値の状態を読める | `StateOverlay` の `Default` を消す(1d-3)。後半は項目を読むだけ(型が保つ) | 空の重ね合わせの扱いは `graph_overlay.rs` の `ステートがないときは_すべてのジョブが未実行で_再開位置もない`(T15 で空の値を項目で書く)、状態から重ね合わせを作ることは `graph_overlay.rs` のほかの 4 件と `graph_types.rs` の `ジョブの状態は成功と警告を_ok_にまとめる` | なし(`Default` が無くなる) |
| runtime `src/types.rs` の単体テスト `耐久性の既定は_durable`(`:227-230`) | `Durability::default()` が `Durable` | `Durability` の `Default` を消す(1d-2)。`Default` だけを試していた | (既定の値という場面が無くなる)。書き込みのたびに `Durable`・`Fast` を明示して渡すことは、runtime の `state_write.rs` と cli の流れのテストが確かめる | なし |
| cli `tests/escape_table.rs` の `u1_core_の_sanitize_と同じ結果`(`:56-65`) | cli の再公開の `escape_display` と core の `sanitize` が、表の入力に同じ結果を返す | `sanitize` を消し(1e)、cli の再公開もやめる(2d)。比べる 2 つが同じ 1 つの関数になる | 同じファイルの `期待値の表のとおりにエスケープされ_切り詰められる`(T31 のあとは core の `escape_display` を直接確かめる)、core の単体テスト(T3 の 4 件)、converter の写しの単体テスト(同じ表) | なし |

- 件数: core −2(単体 −1、統合 −1)、runtime −1(単体)、cli −1(統合)。
- 注(テストは消さず、確かめの行を消すもの。ユーザーが設計の承認で認めた、2026-11-24): T8 の `definition_valid.rs:52`(`extract.effective_success_codes() == [0]`)。定義ファイルから読んだ定義の既定の成功コードを値で見る行が 1 つ減る。足りる理由: 読み取りが `success_exit_codes` の省略を `None` で持つことは同じテストの `:51` が、`None` を `[0]` として扱うことは単体テスト `types.rs:524` と `classify.rs` の単体テストが確かめ、読み取りから分類までの間に `success_exit_codes` を変える処理は無い。ほかの行の削除(T1、T12、T17、T27)は、どれもすぐ近くの行が同じことを確かめる(T 表の各行)。

## 5. 足すテスト

なし。消す・狭める項目の代わりは、既存のテストの書き換えと、公開の入口の既存のテストで持つ(§4)。本番の動きは変わらないので、新しい確かめは要らない。実装で足す必要が出たら、その数と理由を記録する。

## 6. 件数の見込み

| クレート | 単体(src) | 統合(tests/) | 合計 | 基準からの差 |
|---|---|---|---|---|
| core | 113 − 1 = **112** | 271 − 1 = **270** | **382** | −2 |
| runtime | 70 − 1 = **69** | **128** | **197** | −1 |
| converter | **52** | **64 + 1 無視** | **116 + 1 無視** | 0 |
| cli | **2** | 457 − 1 = **456** | **458** | −1 |
| 全体 | | | **1,153 通過、1 無視、0 失敗** | −4 |

ファイルごと: core `graph_types.rs` 4、cli `escape_table.rs` 3。ほかのファイルの件数は変わらない。基準と見込みは macOS の件数。写しの木での試しで、この件数になることを確かめた(`inventory.md` §0.2)。D1(`InterruptSignal` の `Default` を残す)は件数に影響しない(`Default` を使うテストの行 `types.rs:160` は等価性 `==` の確かめでもあり、`Eq` を消すので T17 のまま消す。テストの削除は無い)。

## 7. 確かめ方

- 実装の前と後で、`cargo test --locked --workspace` の件数をクレートごとに記録する(基準 `inventory.md` §0.1)。合格は、失敗 0 で、全体が「**基準 1,157 − 消したテスト 4 + 足したテスト 0 = 1,153**」に一致すること [Q1]。
- テスト名の一覧の差: 前後で `cargo test --locked --workspace -- --list 2>/dev/null | sort` の差が、§4.6 の 4 つの削除だけであること(名前の変更は無い)。
- テストのファイルの変更は §4 の T 表と §4.6 だけ。コミットごとに `git diff --stat -- 'crates/*/tests'` と、src の単体テストの差分を見る。
- **項目ごとの完了の判定**(`inventory.md` の各項目)。最後のコミットのあとで、次をすべて行う [Q1]:

```sh
grep -rn "From<&str> for Command\|From<String> for Command\|Command::from(\|Into<Command>" crates             # 0 件
grep -rn "RunIdArg::as_str\|empty.as_str()\|arg.as_str()" crates                                             # 0 件
grep -rn "is_raised\|impl PartialEq for InterruptSignal\|impl Eq for InterruptSignal" crates docs/spec/rust   # 0 件
grep -rn "JobStatus::default\|Durability::default\|StateOverlay::default\|Language::default" crates           # 0 件
grep -rn "#\[default\]" crates/*/src                                                                         # cli types.rs の GraphFormat の 1 件
grep -rn "sanitize" crates/core crates/cli docs/spec/rust                                                    # 0 件(converter の写しは対象外)
grep -n "pub fn reason(&self)" crates/core/src/envfile/mod.rs                                                # 0 件
grep -rn "decide_before_run" crates/*/tests crates/cli crates/runtime                                        # 0 件
grep -rn "pub fn decide_before_run\|pub use classify::classify\|pub fn classify(" crates/core/src            # 0 件
grep -rn "pub fn is_valid_env_name\|pub fn is_reserved_env_name\|pub fn effective_success_codes\|pub const MAX_RUN_ID_LEN" crates  # 0 件
grep -rn "pub use conductor_core::escape_display\|report::escape::escape_display\|escape::{escape_display" crates  # 0 件
grep -n "^pub mod" crates/cli/src/lib.rs                                                                     # exit、report、run_id の 3 件
grep -n "GraphOptions\|DryRunOptions" crates/cli/src/lib.rs                                                  # 0 件
grep -rn "RunIdError::" crates docs/spec/rust                                                                # 0 件
grep -rnwE "TooLong|InvalidChar|LeadingDot" crates docs/spec/rust                                            # 0 件(:: の付かない変種名。rust 02:99、05 の 6.1、10:125 を含む)
grep -rn "RunIdError.*Empty\|\`Empty\`[・,、] *\`TooLong\`" crates docs/spec/rust                               # 0 件(Empty は CommandErrorKind・RunIdArg にもあるので、文脈で絞る)
grep -rn "pub use conductor_core" crates/runtime/src                                                         # 0 件
grep -rn "Display for DefinitionErrorKind\|Display for EnvFileErrorKind\|Error for DefinitionErrorKind\|Error for EnvFileErrorKind\|Error for ConversionErrorKind" crates  # 0 件
grep -rn -A1 "Hash)\]" crates/core/src crates/converter/src | grep "enum \(DefinitionErrorKind\|EnvFileErrorKind\|ConversionErrorKind\|TargetFormat\)"  # 0 件
for f in $(grep -rl "conductor_runtime" crates); do awk -v F=$f '/use conductor_runtime/{u=1} u{print F":"NR": "$0} /;/{u=0}' $f; done \
  | grep -E "check_consistency|apply_interrupt|apply_outcome|apply_skip|initial_state|ConsistencyResult|MismatchKind"  # 0 件
```

- `cargo doc --workspace --no-deps --locked` の警告が、基準の 5 件(core 1、runtime 4。この intent と無関係)のまま増えないこと(消す・狭める名前へのリンクの直し漏れを見つける)。
- Windows でだけコンパイルされるテスト(`runner_basic.rs` の `cmd_` の 3 件)は、ローカルの macOS で `cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings` を走らせてコンパイルを確かめ(写しで通ることを確かめた)、実行は CI の `test`(windows)で確かめる。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分が無い)[memory:M5]。`unsafe` と src の `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の `Cargo.toml` に core を足さない。版の表記(`0.9.8`)は変えない(`git diff -- '*Cargo.toml' Cargo.lock` が空)。README と `docs-check` の語句に触れる変更は無い。
- 利用者から見える動き: 既存のテスト(T 表の書き換えを除く)が期待を変えずに通ることで確かめる。ステートファイルの形、表示の文、終了コードを作るコードは、どの項目でも変わらない(消すのはテストだけが使う実装、狭めるのは公開範囲、`use` の移動は同じ実体)。

## 8. コミットの並び(小さな単位。各コミットでビルド、clippy、テストが通る)

1 項目 1 コミット。各コミットに、その項目のテストの書き換え(T)、消すテスト(§4.6)、ドキュメントコメント、仕様書の行を含める(テストを先に変えないと、本番を変えたコミットでビルドが止まるため、本番とテストを同じコミットで変える。test-after の「実装の後にテスト」は、新しいテストが無いので当てはまらない)。

**各コミットの確かめ**: コミットごとに、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked --no-run`(全テストのコンパイル)を通し、そのコミットで変えたクレートのテストを実行する。2 つのコミットにまたがるファイル(下の表)は、途中のコミットでもコンパイルが通ることを `--no-run` で確かめる。最後の確かめ(25)で全テストを実行する。

**2 つのコミットにまたがる import の塊(T20・T23・T25)**。各コミットの終わりでの形(並びは `cargo fmt` に従う):

| ファイル | コミット 1(1a)または 10(2a)の後 | コミット 20(3b)の後 |
|----------|-------------------------------------|----------------------|
| runtime `tests/common/mod.rs`(T20) | 1 の後: `use conductor_core::{Command, JobDefinition, PipelineDefinition};`、`use conductor_runtime::{CommandLine, check_command};` は今のまま(変えるのは `command_line`・`job` の引数と中身)。`:113` は `conductor_runtime::initial_state` のまま | `:113` を `conductor_core::initial_state(d, def, None)` に(パスで書くので import は変えない) |
| runtime `tests/nfr_content.rs`(T23) | 1 の後: `use conductor_core::{Classification, Command, JobExecution, JobOutcome, NextStep, PipelineDefinition};`、`use conductor_runtime::{CommandErrorKind, Durability, StateErrorKind, apply_outcome, check_command};` | `apply_outcome` を移し、`use conductor_core::{Classification, Command, JobExecution, JobOutcome, NextStep, PipelineDefinition, apply_outcome};`、`use conductor_runtime::{CommandErrorKind, Durability, StateErrorKind, check_command};` |
| runtime `tests/flow_with_state.rs`(T25) | 10 の後: `conductor_core` の塊は `decide_before_run` を `decide_from_state` に替えた形(`Classification, Decision, ExecutionResult, JobDefinition, JobOutcome, JobStatus, NextStep, PipelineDefinition, PipelineState, RunMode, SkipCause, Transition, TransitionOn, TransitionTarget, classify_and_decide, decide_from_state`)。`conductor_runtime` の塊は今のまま(`ConsistencyResult, Durability, InterruptSignal, MismatchKind, StateError, StateErrorKind, apply_outcome, apply_skip, check_command, check_consistency, run_job`)。補助 `decide` は消してある | `ConsistencyResult`・`MismatchKind`・`apply_outcome`・`apply_skip`・`check_consistency` を `conductor_core` の塊へ移し、`conductor_runtime` の塊は `Durability, InterruptSignal, StateError, StateErrorKind, check_command, run_job` |

コミット 20 で使わなくなる名前が import に残ると `-D warnings` で止まるので、移すときは両方の塊を同時に直す。写しの木では、全項目を当てた最後の形でだけ確かめたので、途中の形はこのコミットごとの `--no-run` で確かめる。

削除(1a〜1f):

1. core・runtime: `Command` の `From<&str>`・`From<String>` を消す(1a。T2、T5、T6、T7、T9、T18、T19、T20 の一部、T21、T22、T23 の一部、§4.6 の `文字列から_line_を作れる`)
2. cli: `RunIdArg::as_str` を消す(1b。T27、T28。仕様書 rust 02)
3. runtime・cli: `InterruptSignal` の `is_raised`・`PartialEq`・`Eq` を消す(`Default` は残す。D1)。`Debug` は `signal_number()` で書く(1c。T17、T29、T30。仕様書 rust 02・07)
4. core: `JobStatus` の `Default` を消す(1d-1。T1)
5. runtime: `Durability` の `Default` を消す(1d-2。§4.6 の `耐久性の既定は_durable`。仕様書 rust 02)
6. core: `StateOverlay` の `Default` を消す(1d-3。T15、T16、§4.6 の `状態の重ね合わせの既定は空で_再開位置もない`)
7. cli: `Language` の `Default` を消す(1d-4。仕様書 rust 02)
8. core・cli: `sanitize` を消す(1e。T3、T31 の説明、§4.6 の `u1_core_の_sanitize_と同じ結果`。仕様書 rust 02・08・11・12)
9. core: `EnvFileError::reason()` を消す(1f。T11、T12。仕様書 rust 02・10)

公開範囲を狭める(2a〜2f):

10. core・runtime: `decide_before_run` を非公開に(2a。T14、T25 の一部。ドキュメントコメント。仕様書 rust 02・12)
11. core: `flow::classify` を非公開に(2b。仕様書 rust 02)
12. core: `MAX_RUN_ID_LEN` を `run_id.rs` の中だけに(2c-1。仕様書 rust 02)
13. core: `is_valid_env_name` を crate の中に(2c-2。仕様書 rust 02)
14. core: `is_reserved_env_name` を crate の中に(2c-3。仕様書 rust 02)
15. core: `effective_success_codes` を crate の中に(2c-4。T8。仕様書 rust 02)
16. cli: `report::escape` の `escape_display` の再公開をやめる(2d。T31 の import。仕様書 rust 02・08・11)
17. cli: `pub mod` 4 つを crate の中に(2e。ドキュメントコメント。仕様書 rust 02)
18. cli: 根の `GraphOptions`・`DryRunOptions` の再公開をやめる(2f。仕様書 rust 02)

判断が要ったもの(3a〜3d):

19. core: `RunIdError` を種類のない誤りの型 1 つに(3a。T4。仕様書 rust 02・05・10・12。language-agnostic は D4)
20. runtime・cli: runtime の core の再公開をやめ、使う所を `conductor_core` に(3b。T20 の一部、T23 の一部、T24、T25 の一部、T32。ドキュメントコメント。仕様書 rust 02・05・06・10)
21. core: `DefinitionErrorKind`・`EnvFileErrorKind` の `Display` と `Error` を消す(3c-1、3c-2。T13。仕様書 rust 03・08・10)
22. converter: `ConversionErrorKind` の `Error` を消す(3c-3)
23. core: `DefinitionErrorKind`・`EnvFileErrorKind` の `Hash` を消す(3d-1、3d-2。T10)
24. converter: `ConversionErrorKind`・`TargetFormat` の `Hash` を消す(3d-3、3d-4。T26)
25. (確かめ)§7 の検索、件数、`cargo doc` の警告、Windows の対象の clippy の記録(コミットなし。記録はコード生成の記録に書く)

合計 **24 コミット**ほど(25 はコミットなし。実装で分けたりまとめたりしたら記録する)。版は上げない。タグとリリースはしない。

## 9. 範囲の外(書かない)

- §3 の「範囲から外したもの」。
- 凍結した仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)。
- 版の更新、タグとリリース。

## 10. 決めてほしい点

D1〜D8 は、ユーザーがすべて推奨のとおりに決めた(2026-11-24)。§4 の T 表の書き換えと、§4.6 の 4 つのテストの削除は、ユーザーが設計の承認で認めた(2026-11-24)。

| ID | 決めること | 推奨 | 代案 | 推奨どおりでよいかの判断の手がかり | 決定 |
|----|------------|------|------|------------------------------------|------|
| D1 | `InterruptSignal` の `Default`(1c) | **残す**(この項目だけ範囲から外し、理由を記録する) | (B)消して、`new` に `#[allow(clippy::new_without_default)]` を付ける | 引数のない `pub fn new()` を持つ公開の型に `Default` を持たせるのは Rust の慣例で、clippy の既定の検査(`new_without_default`)がそれを求める。消すと `-D warnings` で止まるので、消すには検査を黙らせる属性を本番に足すことになる(「本番を曲げない」方針に近いのは残す方)。`Default` は `new()` を呼ぶだけで、害はない | 決定: 残す(この項目だけ範囲から外す。推奨のとおり。ユーザー、2026-11-24) |
| D2 | `RunIdError` の形と、`RunId::parse` の判定(3a) | **単位構造体 `pub struct RunIdError;`。4 つの条件を 1 つの条件式にまとめる**(§2.5) | (B)変種 1 つの列挙型 `enum RunIdError { Invalid }`、(C)判定は今の 4 つの `if` の並びのまま(どれも同じ誤りを返す) | 単位構造体は「種類がない」をそのまま表し、呼ぶ所の `map_err(\|_\| ..)`・`.ok()` は変わらない。判定は結果が同じなので、1 つの式にした方が「どれかなら誤り」と読める。(C)は差分が小さい | 決定: 単位構造体 `pub struct RunIdError;`、4 つの条件を 1 つの条件式に(推奨のとおり。ユーザー、2026-11-24) |
| D3 | `graph_types.rs` の `状態の重ね合わせの既定は空で_再開位置もない`(1d-3) | **消す**(§4.6) | ★名前を `状態の重ね合わせは_状態と再開位置を持つ` に変え、後半(項目を書いて作り、読む)だけを残す | 前半は `Default` だけを試し、後半は型の項目を読むだけ(コンパイルが保つ)。空の重ね合わせの扱いと、状態から作る重ね合わせは、`graph_overlay.rs` の 5 件が公開の入口で確かめる | 決定: 消す方針(推奨のとおり。ユーザー、2026-11-24)。個々の削除は §4.6 として設計の承認で認めてもらう(承認待ち) |
| D4 | language-agnostic の run ID の 4 つの誤りの名前(05 の 6.1 の表、10 の文。3a) | **変えない**(rust の仕様書だけ直す) | language-agnostic も「誤り」の列を消し、条件だけにする | language-agnostic は言語によらない論理の仕様で、関数と型の名前は仮のもの。4 つの条件と「利用者には区別を見せず、すべて `invalid value`(68)」はそのまま正しい。凍結した言語の実装は 4 つの種類を持つ | 決定: language-agnostic は変えない(推奨のとおり。ユーザー、2026-11-24) |
| D5 | runtime のテストの補助の、`Command` の受け取り方(1a) | **`common::job`・`common::command_line` は `impl Into<String>`、`runner_basic.rs` の `run` は `Command`**(§2.4) | (B)3 つとも `Command` を受け取り、文字列を渡す約 25 か所を `Command::Line(..)` で包む、(C)テストの補助に `fn line(text) -> Command` を足し、呼ぶ所を `line(..)` に | 推奨は、補助を呼ぶ所のうち変えるのが `runner_basic.rs` の 12 か所で済み(13 か所は変えない)、補助の中で本番と同じ `Command::Line` を作る。`run` だけは配列の `Command::Argv` も受け取るので `Command` にする | 決定: `common::job`・`common::command_line` は `impl Into<String>`、`run` は `Command`(推奨のとおり。ユーザー、2026-11-24) |
| D6 | 狭める先の公開範囲(2a〜2c) | **使う範囲に合わせて最も狭く**: `decide_before_run` は `decide.rs` の中だけ(`fn`)、`classify` は `pub(super)`、`MAX_RUN_ID_LEN` は `run_id.rs` の中だけ、`is_valid_env_name`・`is_reserved_env_name`・`effective_success_codes` は `pub(crate)` | すべて `pub(crate)` | 推奨は、使う所がモジュールの外に増えたらコンパイルで気づける。どれも使う所を実物で確かめた(`inventory.md` 2a〜2c) | 決定: 使う範囲に合わせて最も狭く(推奨のとおり。ユーザー、2026-11-24) |
| D7 | runtime `flow_with_state.rs` の補助 `decide`(2a) | **消して、2 か所は `decide_from_state` を直接呼ぶ** | 補助を残し、本体を `decide_from_state` の呼び出しにする | 補助は core の `decide_from_state` と同じ `match` の写し(説明にも「core の `flow::decide_from_state` と同じ」とある)で、同じ形の関数を 2 つ持つ理由が無い | 決定: 補助を消し、`decide_from_state` を直接呼ぶ(推奨のとおり。ユーザー、2026-11-24) |
| D8 | `InterruptSignal` の `Debug` の `raised` の項目(1c) | **残し、`signal_number().is_some()` で書く**(出力は変わらない) | `raised` の項目を消し、`signal_number` だけを書く | 推奨は `Debug` の出力と、単体テスト `合図はスレッド間で共有できる` の `raised: true` の確かめが今のまま | 決定: 残し、`signal_number().is_some()` で書く(推奨のとおり。ユーザー、2026-11-24) |

**設計の承認で認めてもらうもの**: §4 の T 表の書き換え(T8 の `effective_success_codes` の 1 行の削除と値の比較、T12 の 1 行の削除、T14 の模型のステートの組み立て、T17 の 5 行の削除、T21 の `run` の引数、T25 の補助の削除、T27 の 1 行の削除と `matches!`、T28 の補助 `run_id`)、§4.6 の 4 つのテストの削除。

## 11. 成功の指標との対応(traceability.json)

`intent-statement.md` の「Success Metrics」の各項目に、上から ID を付けた。

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 初期の記述の「削除」の項目のそれぞれが本番のコードから消え、「公開範囲を狭める」の項目のそれぞれが外から見えなくなる。テストは偽物の側か公開の入口で代わりを持つ | BR1.1〜BR1.6、BR2.1〜BR2.6 |
| SM2 | `RunIdError` は種類のない誤りの型 1 つになり、テストは 4 つの場合の入力が通らないことを確かめる | BR3.1 |
| SM3 | runtime は core の関数と型を再公開しない。使う所は `conductor_core` から使う。`conductor_runtime::` を通した参照が 0 件 | BR4.1 |
| SM4 | 誤りの種類の `Display`・`Error` と、本番で使われない `Hash` がなくなる | BR5.1、BR5.2 |
| SM5 | ライブラリの誤りの型の `Display` と `Error` は残る | BR5.3 |
| SM6 | 確かめる手段: 項目ごとの今の場所・目標・完了の基準の一覧と、実装の後の検索 | BR9.2、BR9.3、BR9.6 |
| SM7 | 利用者から見える動きは変わらない。版は上げない | BR9.1、BR9.5 |
| SM8 | 既存のテストの扱い(書き方を変える・消すテストは検索で網羅し、消すものは理由を添えて承認を取る。期待する文字列は変えない)と件数(基準 − 消したテスト + 足したテスト) | BR8.1、BR9.2、BR9.3 |
| SM9 | 合格の条件(fmt、clippy、カバレッジ 80%、MSRV、依存なし) | BR9.4 |
