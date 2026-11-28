# コード生成のまとめ: v0.10.4 の大きなファイルの分割(Step 1〜17)

この記録は、承認済みの計画(`code-generation-plan.md`)の結果です。Step 1〜8 は前の依頼で、Step 9〜17 は次の依頼で行いました。Step 9〜17 の結果は、後ろの「Step 9〜17 の結果」にあります。

## Step 1: 出発点(変更の前。`9081c03`)

テストの数(`cargo test --workspace --locked`、macOS):

| 項目 | 数 |
|---|---|
| テストの束(`test result` の行) | 90 |
| 通ったテスト | 1,449 |
| 落ちたテスト | 0 |
| 無視したテスト | 1 |
| `#[test]` の数(`crates` の下の `.rs` で `^\s*#\[test\]` の行) | 1,472 |

`#[test]` の数と走ったテストの数(1,450)の差の 22 は、macOS では動かない `cfg(windows)` のテストなどで、変更の前と後で同じ数え方をする。

3 つのファイルの行数(`wc -l`):

| ファイル | 行 |
|---|---|
| `crates/cli/src/report/mod.rs` | 746 |
| `crates/core/src/definition/walk/mod.rs` | 695 |
| `crates/cli/src/args.rs` | 679 |

計画の値(746・695・679、1,472)と一致した。

## 各ステップの結果

どのステップも、コミットの前に `unit-test-instructions.md` のテストを走らせ、すべて通った。ほかに、各ステップで `cargo build`、`cargo fmt --all -- --check`、`cargo clippy -D warnings`(そのクレート)も通した。期待の値やテストは変えていない。

| Step | コミット | 件名 | 走らせたテストと結果 |
|---|---|---|---|
| 2 | `5622157` | `core: walk の値の読み取りを values.rs に移す` | `cargo test -p conductor-core`(472 通過)、`spec_numbers`(12 通過) |
| 3 | `82bdb61` | `core: walk の遷移を transition.rs に移す` | 同上(472・12 通過) |
| 4 | `075fbd6` | `core: walk のジョブをまたぐ検査を cross_check.rs に移す` | 同上(472・12 通過) |
| 5 | `17ac829` | `cli: args.rs を args/mod.rs にする` | `tests::args_grammar`(69 通過)。差分は名前の変更だけ(100%) |
| 6 | `fe0cbad` | `cli: args のオプションの表を options.rs に移す` | `tests::args_grammar`(69 通過) |
| 6 | `661c36d` | `cli: args の字句を tokens.rs に移す` | `tests::args_grammar`(69 通過) |
| 6 | `1839a43` | `cli: args の値の解釈を values.rs に移す` | `tests::args_grammar`(69 通過) |
| 7 | `5e0e031` | `cli: build_command を手順ごとの関数に分ける` | `tests::args_grammar`(69 通過)、`cargo test -p conductor-cli`(606 通過) |
| 8 | `d73f133` | `cli: lib.rs のモジュールの表を args の分割に合わせる` | `tests::args_grammar`(69 通過) |

層の終わりの確かめ:

- walk の後(Step 4): `cargo test -p conductor-core --locked`(472 通過)、`cargo test -p conductor-cli --test spec_numbers --locked`(12 通過)。
- args の後(Step 8): `cargo test -p conductor-cli --locked`(30 の束、606 通過。`nfr_source.rs` を含む)。
- 全体(Step 8 の後): `cargo test --workspace --locked` は 90 の束、1,449 通過、0 失敗、1 無視(出発点と同じ)。`#[test]` は 1,472 のまま。`cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` も通った。

各ステップで守ったこと:

- 移すコミットでは、中身の行を変えていない。変えたのは、`use` の行、`pub(super)` の付け足し、`mod` の宣言、新しいファイルのライセンスと `//!` だけ。`pub(super)` の付け足しで 100 文字を超えた行は、`cargo fmt` の形に折った(`walk/values.rs` の `exit_codes` の宣言、`args/mod.rs` の `use options::{…}` の並び)。
- 文書コメントの直しは、その対象を移すコミットに含めた(Step 4 の `walk/mod.rs` の `//!` と `crates/core/src/lib.rs` の walk の行)。`crates/cli/src/lib.rs` の args の行は Step 8 で直した。
- `crates/core/src/definition/walk/env.rs` と `retry.rs` には差分が無い(`git diff 9081c03 -- …` が空)。`retry.rs` の `use super::{…, exit_code_of, integer_of}` は、`mod.rs` の `use values::{exit_code_of, integer_of};` で引ける。
- キーの表 4 つは `walk/mod.rs` に残した。`const PIPELINE_KEYS`(56 行)と `const JOB_KEYS`(67 行)より前に、その語を含む行は無い。
- 子の項目は、親や兄弟が使うものだけを `pub(super)` にした。`options.rs` の `Allowed`、`ALL_BUT_CONVERT`、`RUN_AND_PIPELINE`、`OPTIONS`、`is_long_spelling`、`OptSpec` の `long`・`allowed`、`transition.rs` の `transition_on`、`cross_check.rs` の `check_exit_codes` は private のまま。クレートの外に見える名前は変わらない(`lib.rs` の `pub` の行は変えていない)。
- 本番のコードに `unreachable!`、`unwrap`、`expect`、`panic!`、`#[path]`、`include!`、`cfg(…)` を足していない(`git diff 9081c03 HEAD -G … -- crates` が空)。依存は足していない。

### Step 7 の `build_command` の分け方(R-02 のとおり)

- `collect_tokens(tokens) -> Result<(Given, Vec<OsString>), _>`: 字句を集め、知らないオプションを誤りにする。
- `command_kind(&mut positionals) -> Result<CommandKind, _>`: 位置引数のイテレータ(`vec.into_iter()`)の先頭を取る。
- `check_options(&given, kind) -> Result<(), _>`: `CHECKS` の順の検査と、`--all` と `--run-id`、`graph` の `--run-id`。`--run-id` は `given.count(Opt::RunId)` で見るだけで、値は取り出さない。
- `fill_options(&mut given, kind) -> Result<(CliCommand, Vec<OsString>), _>`: 最初に `given.take(Opt::RunId)` で値を取り出し、今と同じ順(`--assume` → `--format` → `--interval` → `--run-id`)で解釈する。`--to` の値も返す。
- `check_operands(kind, &mut positionals) -> Result<Option<OsString>, _>`: 同じイテレータの残りから、1 つ目の位置引数と余りの引数を見る。
- `fill_operand(kind, &mut command, operand, to) -> Result<(), _>`: `match kind` の 5 つの枝。`Convert` と `Init` は `convert_operand`・`init_operand` を呼ぶ。

`CHECKS` の後は `--run-id` は 0 回か 1 回で、値を取るオプションの指定はいつも `Some` を積むので、`given.count(Opt::RunId) > 0` は前の `run_id.is_some()` と同じ判定になる。誤りの種類、対象、最初に返す誤りの順は変わらない(`args_grammar` の「報告の順」のテスト 7 つを含む 69 のテストが通る)。

## 行数の実測(Step 8 の後。`wc -l`)

| ファイル | 行 | 参考値(R-01) |
|---|---|---|
| `crates/core/src/definition/walk/mod.rs` | 411 | 420 |
| `crates/core/src/definition/walk/values.rs` | 171 | — |
| `crates/core/src/definition/walk/transition.rs` | 90 | — |
| `crates/core/src/definition/walk/cross_check.rs` | 115 | — |
| `crates/cli/src/args/mod.rs` | 341 | 340(機能の設計の 5 章は 300) |
| `crates/cli/src/args/options.rs` | 323 | — |
| `crates/cli/src/args/tokens.rs` | 62 | — |
| `crates/cli/src/args/values.rs` | 100 | — |

- `build_command` は 10 行(`fn` の行から閉じの `}` まで。前の文書コメント 10 行を除く)。参考値の 40 行、目安の 30 行より短い。3.2 の 6 つの関数を順に呼ぶだけになっている。
- `args/mod.rs` は参考値の 340 行を 1 行超える。R-01 のとおり、行数は参考値なので止めずに記録する。超えた主な理由は、新しい 8 つの関数の文書コメント(手順の番号と誤りの種類)と、`cargo fmt` が折った関数の宣言。
- 分け終わりの主な基準(R-01): `functional-spec.md` の 2.1・3.1 の移す項目は、`walk/mod.rs`・`args/mod.rs` に無い(`grep -n 'fn string\|fn cross_check\|fn transitions' walk/mod.rs` は 0 件。`args/mod.rs` の最上位の項目は `mod`・`use`・`parse_args`・`build_command` と 3.2 の関数だけ)。

## Step 9〜17 の結果

### 各ステップ

どのステップも、コミットの前に `unit-test-instructions.md` のテストを走らせ、すべて通った。ほかに、各ステップで `cargo clippy -p conductor-cli --all-targets -D warnings` と `cargo fmt --all -- --check` も通した。

| Step | コミット | 件名 | 走らせたテストと結果 |
|---|---|---|---|
| 9 | `450da59` | `cli: report の出来事の定義を event.rs に移す` | `tests::report_messages`(45 通過)、`tests::escape_table`(3 通過) |
| 10 | `08ee54f` | `cli: report のエスケープを escape.rs に移す` | 同上(45・3 通過) |
| 11 | `13647fa` | `cli: report の 1 行の組み立てを lines.rs に移す` | 同上(45・3 通過) |
| 12 | `f903860` | `cli: render の枝を表示の形ごとのメソッドに分ける` | 同上(45・3 通過。694 行と 726 行のテストを含む)、`cargo test -p conductor-cli`(30 の束、606 通過)、`nfr_source`(6 通過) |
| 13 | `bdde159` | `core: error.rs の文書コメントの古い名前を直す` | `cargo test -p conductor-core`(472 通過) |
| 13 | `334c574` | `cli: with_all_events に足りない 2 つの変種を足す` | `tests::report_messages`(45 通過) |
| 14 | `2c284c3` | `docs: rust 版の仕様書のモジュールの記述を分けた後の形にする` | `spec_numbers`(12 通過)、`tests::docs`(22 通過) |
| 15 | `c293fa3` | `aidlc: team.md の使い方の表の場所を args の分割に合わせる` | `git diff` で 1 行だけの変更(1 挿入・1 削除)を確かめた |
| 16 | `3852f7a` | `版を 0.10.4 にする` | 下の「最後の確かめ」 |

### 変えたファイル(Step 9〜16)

- 新しいファイル: `crates/cli/src/report/event.rs`、`escape.rs`、`lines.rs`、`render.rs`
- `crates/cli/src/report/mod.rs`(移した項目を除き、`mod`・`use`・`pub use`・`pub(crate) use` を置いた。`render` は振り分けだけにした。`//!` を分けた後の形に直した)
- `crates/cli/src/lib.rs`(モジュールの表の `report` の行)
- `crates/core/src/error.rs`(`escape_display` の文書コメントの `report::escape` を `report::escape_full`(`report/escape.rs`)に)
- `crates/cli/src/tests/report_messages.rs`(`with_all_events` の一覧と、エラーの行のテストの例外の条件。下の「人の判断」)
- `docs/spec/rust/02-architecture.md`、`07-process-and-signals.md`、`08-output-and-i18n.md`、`12-rebuild-guide.md`
- `aidlc/spaces/default/memory/team.md`(付録 A の 1 行)
- 版の 22 ファイル(`git show --stat ad775bc` と同じ範囲、37 行。`Cargo.lock` は 4 つのクレートの版の行だけ。`crates/cli/tests/binary_basic.rs` の `--version` の期待を含む)

### Step 12 の `render` の分け方

- `render` は網羅の `match` のままで、各枝は `render.rs` のメソッドを 1 つ呼ぶだけ(R-01 の基準)。5 つの枝は、`cargo fmt` が `{ … }` か、パターンの複数行に折った。
- `render.rs` は、`functional-spec.md` の 4.2 の 9 つの節(`// ---- A. 固定の文 ----` から `// ---- I. 標準出力 ----`)に、変種ごとの `pub(super)` のメソッドを並べた。メソッドは取り出した中身を引数で受け取り、`&DisplayEvent` は受け取らない。
- 言語は、メソッドの中で `self.language()` を呼んで判定する。A の節と `pass_through` は呼ばない。色は `watch_table` だけが `self.color_enabled()` を呼ぶ。`watch_frame` は `self.render` を呼び直す。
- 中身は枝のとおりに写した。違いは、値の引数で受け取るために要る直しだけで、文、改行、`format!` の形は変えていない。
  - `lang()` を `self.language()` にした
  - `*last_exit_code` などの `*` が外れた
  - `WatchFrame` の中の `now: *now` が `now` になった
  - `(*text).to_string()` が `text.to_string()` になった

### 移すコミットで守ったこと(Step 9〜11)

- 移した項目の中身の行は変えていない(`git show` の前の行と並べて確かめた)。違うのは `pub(super)` の付け足しと、それで 100 文字を超えた `command_line` の宣言を `cargo fmt` が折ったところだけ。
- `lines.rs` の `place_of` は `lines.rs` の中でしか使わないので、Step 1〜8 の前例(親や兄弟が使うものだけ `pub(super)`)に合わせて private にした。`CLEAR_SCREEN` は前のまま `pub(crate)` で、`event.rs` の中でだけ使う。
- 今の道筋は保った: `crate::report::{DisplayEvent, Reporter, StateContext, ResultDetail, FramePrefix, escape_name, escape_full}`、子の `super::{escape_full, escape_name, push_line}`、`messages.rs` の `crate::report::StateContext`。`report/messages.rs` は動かしていない。
- 本番のコードに `unreachable!`、`unwrap`、`expect`、`panic!`、`#[path]`、`include!`、`cfg(…)` を足していない(新しい 4 つのファイルで grep して 0 件)。依存は足していない。

### Step 14 の残りの確かめ

`grep -rn 'args\.rs\|report/mod\.rs の\|walk/mod\.rs' docs/spec/rust/` の残りは次の 3 種類で、今の形として古い場所を書いた行は無い。

- `02-architecture.md:335`、`:398`: 分けた後の表の行(`walk/mod.rs` の行と、「v0.10.3 までは `args.rs` の 1 つのファイルだった」の注記)
- `07-process-and-signals.md:44`: 「v0.10.3 までは `definition/walk/mod.rs`」の注記
- `12-rebuild-guide.md:76`・`:105`・`:117`・`:127`・`:137`・`:150`・`:179`・`:211`: 段の時点のファイルの記述(履歴として残した)
  - 段 7(`:76`)に、v0.10.4 で分けたことを注記した。
  - `:105`・`:150` の `definition/walk/mod.rs` の `PIPELINE_KEYS`・`JOB_KEYS` は、分けた後も `walk/mod.rs` にあり、今の形とも合う。

language-agnostic 版は、版の行(Step 16)のほかは変えていない。

## 分け終わりの判定(R-01)

主な基準(`functional-spec.md` の 2.1・3.1・4.1 の移す項目が `mod.rs` に無いこと)は、3 つとも満たした。

- `walk/mod.rs`: `grep -n 'fn string\|fn cross_check\|fn transitions'` は 0 件。
- `args/mod.rs`: 最上位の項目は、`mod`・`use`・`parse_args`・`build_command` と 3.2 の関数だけ。
- `report/mod.rs`: 4.1 の移す項目(4 つの型、`CLEAR_SCREEN`、`MAX_DISPLAY_CHARS`、エスケープの 3 つ、1 行の組み立ての 12 の関数)の定義を grep して 0 件。残るのは、`//!`、`mod` の宣言(11)、`pub use`・`pub(crate) use`・`use`、`Reporter`、`new`、`language`、`color_enabled`、`show`、`render`、`detect_language` だけ。

`render` の各枝はメソッドを 1 つ呼ぶだけで、`build_command` は 3.2 の関数を順に呼ぶだけ(10 行)。

行数の実測(最後。`wc -l`):

| ファイル | 行 | 参考値(R-01) |
|---|---|---|
| `crates/core/src/definition/walk/mod.rs` | 411 | 420 |
| `crates/cli/src/args/mod.rs` | 341 | 340(機能の設計の 5 章は 300) |
| `crates/cli/src/report/mod.rs` | 195 | 250(機能の設計の 5 章は 220) |
| `crates/cli/src/report/event.rs` | 293 | — |
| `crates/cli/src/report/render.rs` | 351 | — |
| `crates/cli/src/report/lines.rs` | 184 | — |
| `crates/cli/src/report/escape.rs` | 39 | — |

- `render` は 64 行(文書コメントを除き、`fn` の行から閉じの `}` まで)。前は 188 行だった。
- 子のファイルは 10 個(`walk` に 3、`args` に 3、`report` に 4)。

## テストの数(前後で同じ)

最後の確かめ(Step 16 の後。macOS):

- `cargo build --locked` は通った。
- `cargo test --workspace --locked` は 90 の束で、1,449 通過、0 失敗、1 無視。出発点(Step 1)と同じ。
- `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` も通った。
- `#[test]` は 1,472 のまま(`grep -rcE '^\s*#\[test\]' crates --include='*.rs'` の合計)。テストの名前と期待の値は、次の 2 つのほかは変えていない。
  - 版の期待(`binary_basic.rs`)
  - FR4.4 の一覧と、それに伴う例外の条件(下の「人の判断」)

## 残った事項

- **人の判断(2026-11-28、A)**: `with_all_events` に `ResetAllKept` を足すと、`エラーの行は_conductor_error_で始まり_場所を含む` が落ちた。
  - 理由: `ResetAllKept` は水準が `Error` で、表示は `conductor: warning: ` で始まり、名前の行も続く。
  - 人は A を選んだ。このテストの例外の条件を `matches!(event, DisplayEvent::EnvWarnings(_) | DisplayEvent::ResetAllKept { .. })` に広げ、2 つの変種を一覧に足した(`334c574`。理由はコミットの本文に書いた)。
  - 変えたのは、一覧とこの例外の条件だけ。テストの名前とほかの期待の値は変えていない。
- **`args/mod.rs` の行数**: 341 行で、参考値の 340 行を 1 行超えた。R-01 のとおり行数は参考値なので、止めずに記録する。主な理由は、新しい 8 つの関数の文書コメントと、`cargo fmt` が折った関数の宣言。
- **`place_of` の見える範囲**: 機能の設計の 4.1 の表は `pub(super)` だが、前例に合わせて private にした(上の「移すコミットで守ったこと」)。
- Step 5 の `git mv` の前に、移す先のディレクトリ `crates/cli/src/args/` を `mkdir` で作った(`git mv` は、無いディレクトリへは移せないため)。ファイルの中身は書いていない。
- 行カバレッジ(`cargo llvm-cov`)は、計画どおり Build and Test で測る。この段階では測っていない。
- Windows と Linux のテストは、CI で確かめる(team.md)。
