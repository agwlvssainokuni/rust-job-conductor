# 変える場所の一覧(Inventory): 動きを変えない整理の第 2 弾

HEAD a2f036f(v0.9.5)の時点で、実物を読んで作った一覧。コードは変えていない。行番号はこの時点のもの。調査の候補と実物が違ったところは「(調査との違い)」と書く。

## 0. 基準のテスト件数(`cargo test --workspace --locked`、macOS、rustc 1.100.0、debug)

| クレート | 単体(src) | 統合(tests/) | 合計 |
|---|---|---|---|
| core | 105 | 258 | 363 |
| runtime | 71 | 124 | 195 |
| converter | 53 | 63 通過 + 1 無視 | 116 通過 + 1 無視 |
| cli | 0 | 454 | 454 |
| 全体 | | | **1,128 通過、1 無視、0 失敗** |

この intent に関わるファイルの件数: `args_grammar.rs` 60、`flow_run.rs` 52、`flow_pipeline.rs` 33、`report_messages.rs` 35、`escape_table.rs` 4、`flow_sequences.rs` 9、`plan_run.rs` 5、`plan_perf.rs` 2、`runtime/tests/flow_with_state.rs` 20。

cli の前提(前回と同じ): `ctx`・`step`・`run`・`pipeline`・`dry_run`・`aux_commands`・`graph`・`init`・`retry` は非公開のモジュールで、テストから見えない。テストが見る cli の型は、公開の `report::DisplayEvent`、`report::escape`、`args::resolve_run_id`、`CliCommand` など。

---

## 段 1(説明と制約)

### 1a. `apply_skip` の説明(完了の判定は `apply_skip` が行う)

- 今: 次の 4 か所が「スキップの記録のあと、完了の判定に `resume_point_on_interrupt` を使う」と書く。
  - `crates/core/src/lib.rs:62-66`(「呼び出しの順序」の段落。調査の `:64` を含む)
  - `crates/core/src/flow/decide.rs:45-48`
  - `crates/core/src/flow/next_step.rs:36-41`
  - `crates/core/src/flow/resume.rs:28-31`
- 実際: 完了と再開位置は `apply_skip`(`crates/core/src/flow/record.rs:120-175`。`:171-173` で `completed` と `resume_job_id` を書く)が決める。cli が `resume_point_on_interrupt` を呼ぶのは `crates/cli/src/pipeline.rs:109`(ジョブのあいだの中断)だけ。
- 目標: 4 か所の文を「完了と再開位置は [`apply_skip`] が書く(最後のジョブをスキップとして記録し、`pending` が無ければ `completed = true`)。`resume_point_on_interrupt` は、`pipeline` のジョブのあいだの中断にだけ使う」に直す。コードは変えない。
- 影響するテスト: なし(コメントだけ)。

### 1b. `StateContext::Plain`、`OutputFailed`、`EXIT_STATE`、`config_path`、`read_state_raw` の説明

| 今 | 実際 | 目標の文 |
|---|---|---|
| `crates/cli/src/report/mod.rs:66` `Plain`「補足なし(`reset` コマンド、`watch`)」 | `watch` は `read_context`(`crates/cli/src/aux_commands.rs:38`)で `Read`/`ReadRunId`。`Plain` は `reset`(`aux_commands.rs:73`、`:85`)だけ | 「補足なし(`reset` と `reset --all`)」 |
| `docs/spec/rust/08-output-and-i18n.md:205` `Plain`(`reset`・`watch`) | 同上(同じ節の `:207` は正しい) | `Plain`(`reset`・`reset --all`) |
| `docs/spec/language-agnostic/08-output-and-i18n.md:211` `Plain`(`reset`・`watch` の一部) | 同上 | `Plain`(`reset`・`reset --all`) |
| `crates/cli/src/report/mod.rs:123` `OutputFailed`「`watch` は 65、`convert` は 67」 | `graph`(`graph.rs:64-65`)と `--dry-run`(`dry_run.rs:104-105`)も 65 | 「`watch`・`graph`・`--dry-run` は 65、`convert` は 67」 |
| `crates/cli/src/exit.rs:23` `EXIT_STATE`「…と、`watch` の出力の失敗」 | `graph`・`--dry-run` の出力の失敗(`OutputFailed`)と `init` の失敗(`init.rs:41`、`:64`)も 65 | 「…と、`watch`・`graph`・`--dry-run` の標準出力の失敗、`init` の書き込みの失敗」 |
| `crates/cli/src/types.rs:143` `config_path`「run・pipeline・watch・reset のときのみ」 | `--config` は `convert` 以外のすべて(`crates/cli/src/args.rs:127` の `ALL_BUT_CONVERT`)。`init` では出力先 | 「`convert` 以外のすべて(`init` では出力先。`-` は標準出力)」 |
| `crates/cli/src/deps.rs:108` `read_state_raw`「`watch` が使う」 | `graph --state` も使う(`ctx.rs:300`、`graph.rs:79`) | 「`watch` と `graph --state` が使う」 |

- 影響するテスト: なし。

### 1c. escape の説明(同じ規則の関数は 3 つ)

- 今: `crates/cli/src/report/escape.rs:17-19`「conductor-core(`sanitize`)・conductor-runtime・conductor-converter の、同じ規則の関数」「4 つの Unit に重複」。`crates/cli/tests/escape_table.rs:15`、`:17`、`:22`(テストのファイルのドキュメントコメント)も runtime を挙げる。`docs/spec/rust/11-quality-and-build.md:151`、`:238` も「core・runtime・converter と揃える」。
- 実際: 同じ規則の関数は `crates/core/src/error.rs:232`(`sanitize`)、`crates/converter/src/error.rs:159`(`sanitize`。非公開)、`crates/cli/src/report/escape.rs:31`(`escape_display`)の 3 つ。runtime には無い(`grep "fn is_format_char"` で 3 件)。
- 目標: 段 2 の 2i で cli の写しを core に寄せた後の姿に合わせて書く(「本体は conductor-core の `escape_display`。conductor-converter は core に依存しないため、同じ規則の写しを持ち、`tests/escape_table.rs` と converter の単体テストの同じ期待値の表で揃える」)。段 1 では、まず「3 つ」「runtime なし」に直し、2i で最終の文にする。
- 影響するテスト: `escape_table.rs` のコメントだけ(functional-spec.md §4 の T8。承認が要る)。

### 1d. `lib.rs` の表とモジュールの説明の古さ

- `crates/core/src/lib.rs:43-56`(モジュールの表): `envfile`、`graph`、`plan`、`run_id` が無い。目標: 4 行を足す(`envfile` = `.env` の解析と層の統合、`graph` = 図の構造と配置、`plan` = `--dry-run` の計画、`run_id` = run ID の型と規則)。`:22-35` の関数の表は「契約 C2」の表として残し、表の下に「ほかの公開の関数は各モジュールにある」と 1 文を足す。
- `crates/runtime/src/state/mod.rs:15`「ファイルに触れるのは、`reader`、`writer` だけ」: `reset_all`(`state/reset_all.rs`)もファイルに触れる。目標: 「`reader`、`writer`、`reset_all`」。
- `crates/runtime/src/lib.rs`:
  - `:24-41` 関数の表: `create_file`(`init`。`lib.rs:122` で公開)が無い。`read_state_raw` の「`watch` 用」は「`watch` と `graph --state` 用」に。
  - `:47-49`「ファイルに触れるのは、`read_state`、`write_state`、`reset_state` だけ」: `read_state_raw`、`reset_all_states`、`load_env_files`、`create_file` も触れる。
  - `:53-64` モジュールの表: `envfile`、`file_create`、`wait`、`state::reset_all` が無い。
  - `:84-85`「`watch` だけは `read_state_raw`」: `graph --state` も。
- `crates/cli/src/lib.rs`:
  - `:17` コマンドの並びに `init` が無い。
  - `:37` `Runtime` の責務の並びに `load_env_files`、`wait`、`read_state_raw`、`reset_all_states`、`run_id_env`、`create_file` が無い(`deps.rs:79-139` が実物)。
  - `:51` 流れの行に `graph`、`init`、`retry` が無く、`aux_commands` の説明に `validate` が無い。段 3 で足す `run_id` も載せる。
  - `:62-63` 終了コードの並びに 69(`init` の作る先が既にある)が無い。
- 影響するテスト: なし。

### 1e. `definition_key` の「仮」の注記

- 今: `crates/core/src/types.rs:313`「仮。確定は conductor-runtime の Functional Design。」
- 実際: 確定済み。`initial_state` が `definition_path` と同じ文字列を入れ(`record.rs:41-45`)、ステートファイルの必須のキー(`runtime/src/state/reader.rs:221`、`:265`)で、照合には使わない(`core/src/consistency.rs:41`、`docs/spec/rust/06-state-file.md:39`)。
- 目標: 「定義ファイルのパス(`definition_path` と同じ文字列)。ステートファイルの必須のキーで、照合には使わない(06 章)」。フィールドは消さない。
- 影響するテスト: なし。

### 1f. 仕様書 11 章のテストの件数

- 今: `docs/spec/rust/11-quality-and-build.md` の表に、ファイルごとの件数が 53 か所(`(49)` など)と、`:166-167` の「(5 件: …)」「(6 件: …)」。
- 実物との違い(基準の測定): `args_grammar.rs` 49→60(`:236`)、`flow_run.rs` 51→52、`flow_pipeline.rs` 31→33(`:244`)、`plan_perf.rs` 1→2、`plan_pipeline.rs` 9→10。また OS で変わる(`windows_demo.rs` は macOS で 0 件、`runner_basic.rs` は Windows 専用 3 件を含む)。
- **決定: 件数を書くのをやめる**(表のファイル名と目的は残し、`(N)` を消す。`:166-167` は確かめる中身の並びを残し、「N 件」だけ消す)。理由: 機能を足すたびにずれる(今回 5 か所)、OS で変わる、件数は `cargo test` の出力で分かる。`docs/spec/language-agnostic/11` には件数が無い(0 か所)ので変えない。
- 影響するテスト: なし(この章を読むテストは無い)。

### 1g. core と converter の `#![forbid(unsafe_code)]`

- 今: `crates/cli/src/lib.rs:82` だけが `forbid`。runtime は `#![deny(unsafe_code)]`(`runtime/src/lib.rs:107`。`platform/unix.rs` で許す)。core(`crates/core/src/lib.rs`)と converter(`crates/converter/src/lib.rs`)には無い。両クレートの `src` に `unsafe` は 0 件。
- 目標: `crates/core/src/lib.rs` と `crates/converter/src/lib.rs` の、ドキュメントコメントの直後(最初の `mod` の前)に `#![forbid(unsafe_code)]`。各 `lib.rs` の「守る性質」に 1 行足す。
- 影響するテスト: なし。

### 1h. 理由の文の上限の数と定数を、コンパイル時の検査で結ぶ

形は converter の既存の `const _: () = assert!(crate::raw::MAX_DEPTH == 32);`(`crates/converter/src/error.rs:143`)と同じ。文は 1 文字も変えない。

| 文の場所 | 文の数 | 定数 | 足す検査 |
|---|---|---|---|
| `crates/core/src/definition/intake.rs:128`(`too_large`。調査の `:135` は関数の終わり) | 1 MiB | `MAX_DEFINITION_BYTES`(`intake.rs:27`) | `const _: () = assert!(MAX_DEFINITION_BYTES == 1024 * 1024);` |
| `crates/core/src/definition/version.rs:41`、`:50`、`:56` | 版は 1 | `SUPPORTED_FORMAT_VERSION`(`version.rs:28`) | `assert!(SUPPORTED_FORMAT_VERSION == 1)` |
| `crates/core/src/envfile/mod.rs:126` | 1 MiB | `MAX_ENV_FILE_BYTES`(`:38`) | `assert!(MAX_ENV_FILE_BYTES == 1_048_576)` |
| `crates/core/src/envfile/mod.rs:128` | 10,000 | `MAX_ENV_ENTRIES`(`:40`) | `assert!(MAX_ENV_ENTRIES == 10_000)` |
| `crates/converter/src/error.rs:131` | 1 MiB | `intake::MAX_INPUT_BYTES`(`converter/src/intake.rs:29`) | `assert!(crate::intake::MAX_INPUT_BYTES == 1024 * 1024)` |
| `crates/converter/src/error.rs:132` | 100000 | `raw::MAX_NODES`(`raw.rs:24`) | `assert!(crate::raw::MAX_NODES == 100_000)` |
| `crates/converter/src/error.rs:133` | 16 MiB | `raw::MAX_EXPANDED_BYTES`(`raw.rs:31`) | `assert!(crate::raw::MAX_EXPANDED_BYTES == 16 * 1024 * 1024)` |
| (調査に無い)`crates/runtime/src/error.rs:103` | 1 MiB | `state::reader::MAX_STATE_BYTES`(`reader.rs:32`) | `assert!(crate::state::reader::MAX_STATE_BYTES == 1024 * 1024)` |
| (調査に無い)`crates/cli/src/report/messages.rs:324`、`:326`(英語の文) | 1 MiB、10,000 | core の `envfile::MAX_ENV_FILE_BYTES`、`MAX_ENV_ENTRIES` | `assert!(conductor_core::envfile::MAX_ENV_ENTRIES == 10_000)` など |

- 影響するテスト: なし(コンパイル時の検査。実行時の件数は変わらない)。

---

## 段 2(小さな重複と計算量)

### 2a. 位置で状態を引く処理の共有と、`pending_ids` の O(n²)

- 今:
  - `crates/core/src/flow/decide.rs:20-27` の `status_at`(非公開)。
  - `crates/core/src/flow/record.rs:152-161` の同じ考えのクロージャ(記録が無ければ `Pending`)。
  - `crates/core/src/flow/next_step.rs:132-138` の `pending_ids` が、範囲の各ジョブで `record_of`(線形の探索。`flow/mod.rs:39-45`)を呼ぶ。`@end` に着いたとき(`:143`)に、後ろの全ジョブで O(n²)。`:162` の遷移の飛び越しも同じ。
  - (調査に無い)`crates/core/src/flow/resume.rs:36-45` も `record_of` を各ジョブで呼ぶ(O(n²)。ジョブのあいだの中断で 1 回だけ)。
- 目標(`crates/core/src/flow/mod.rs`):
  ```rust
  /// 記述順の位置 `index` のジョブ `job_id` の状態。記録が同じ位置にあればそれを使い(O(1))、
  /// なければ [`record_of`] と同じに引く(記録が無ければ pending)。
  pub(crate) fn status_at(states: &[JobState], index: usize, job_id: &str) -> JobStatus
  ```
  `decide.rs` は自分の `status_at` を消して使う。`record.rs` のクロージャを `status_at(&out.job_states, i, &j.id)` に。`next_step.rs` の `pending_ids` は、範囲の先頭の位置 `start` を受け取り、`status_at(job_states, start + k, &j.id)` で引く(`pending_ids(index + 1, &jobs[index + 1..])`、`pending_ids(index + 1, &jobs[index + 1..t])`)。`resume.rs` も `enumerate` して `status_at` で引く。
- 結果は変わらない(位置が合わないときは今と同じ線形の探索)。
- 性能のテスト: A2(functional-spec.md §5)。
- 影響するテスト: なし(非公開の関数)。

### 2b. `merge_env` の O(E²)(BR4.3 の並びを保つ)と、`.env` の解析の重複の除去

- 今: `crates/core/src/envfile/merge.rs:77-84` の `put` が、足すたびに `entries.retain(|e| e.name != name)` を回す。1 ファイル 10,000 項目で約 5,000 万回の比較。
- 並びの規則(BR4.3、`merge.rs:23-26`): 完全に同じ名前は、後ろの層が勝って 1 つになり、**最後に足した位置**に並ぶ。大文字小文字だけが違う名前は別の名前。
- 目標: 2 段に分ける。
  ```rust
  /// 層の順に (名前, 値, 出どころ) を並べる(重複を含む)。
  fn collect_layers<'a>(def, job, files, ctx, run_id_text) -> Vec<(&'a str, &'a str, EnvOrigin)>  // 形は実装で調整してよい
  /// 名前ごとの最後の位置を表(`HashMap<&str, usize>`)で引き、最後の位置の項目だけを、並びの順に残す。
  fn keep_last(items: Vec<...>) -> Vec<EffectiveEntry>
  ```
  O(E)。`CONDUCTOR_ATTEMPT` の値(`ctx.attempt.to_string()`)のように、その場で作る文字列は、先に作って借りるか、`String` で持つ。公開の関数 `merge_env` のシグネチャと戻り値は変えない。
- (任意)`crates/core/src/envfile/parse.rs:83-85` の重複キーの除去(`position` + `remove` で O(n))。同じ「最後の位置だけ残す」で、解析の後に 1 回で除く形にする。警告(`DuplicateKey`)の行と順は変えない。測定(基準): 10,000 行のうち 5,000 が重複で debug 40.6 ms(性能のテストは足さない。§5 の注)。
- 性能のテスト: A3(functional-spec.md §5)。
- 影響するテスト: なし(`env_merge.rs`、`env_file_parse.rs` などの既存のテストがそのまま通ること)。

### 2c. 既定の定義ファイル名を core に寄せる

- 今: `crates/cli/src/ctx.rs:40`(`DEFAULT_DEFINITION`)と `:125-131`(`definition_path`)、`crates/cli/src/types.rs:60`(`DEFAULT_INIT_PATH`、公開)が、core の `crates/core/src/definition/intake.rs:31`(`DEFAULT_DEFINITION_FILE`)と `:33-39`(`resolve_path`)の写し。
- 目標: `ctx::definition_path(cmd)` を `conductor_core::definition::intake::resolve_path(cmd.config_path.as_deref().map(Path::new))` に。`DEFAULT_DEFINITION` を消す。`pub const DEFAULT_INIT_PATH: &str = conductor_core::definition::intake::DEFAULT_DEFINITION_FILE;`(名前と値は変えない。公開の道を保つ)。
- 影響するテスト: なし。

### 2d. `"@end"` の文字列と `DisplayEvent::Transition.target` を型に

- 今: `"@end"` が `crates/cli/src/ctx.rs:478`、`crates/cli/src/report/graph_text.rs:48`、`:89`、`crates/cli/src/report/graph_mermaid.rs:68`、core の `crates/core/src/definition/walk/mod.rs:62`(`END_TARGET`。非公開。`:433`、`:574` で使う)。`DisplayEvent::Transition { target: &'a str }`(`crates/cli/src/report/mod.rs:175-181`)。
- 目標(core の `types.rs`):
  ```rust
  impl TransitionTarget {
      /// 遷移先に書く、終了を表す語(`@end`)。
      pub const END_LABEL: &'static str = "@end";
      /// 定義と表示に使う語(ジョブID、または `@end`)。
      pub fn label(&self) -> &str
  }
  ```
  walk は `END_TARGET` を消して `TransitionTarget::END_LABEL` を使う。cli: `DisplayEvent::Transition { on, target: &'a TransitionTarget, skipped }`。表示は `escape_name(target.label())`(`@end` はエスケープしても同じ)。`ctx`(段 3 では `step`)は `&transition.next` をそのまま渡す。graph の text は `EdgeTarget`(段 4 で `TransitionTarget` の別名)の `label()` を、今と同じ `escape_path`(64 文字で切らない)でエスケープ(遷移の表示の `escape_name` と使い分けを保つ。graph を `escape_name` にすると 65 文字以上のジョブ ID で出力が変わるので、しない。確かめ方: 65 文字以上のジョブ ID の graph の text の出力が今と同じであることを、既存のテストか、足すテストで確かめる)、終点の節は `TransitionTarget::END_LABEL`。Mermaid は `format!("  nend([\"{}\"])\n", TransitionTarget::END_LABEL)`。
- 影響するテスト: `crates/cli/tests/report_messages.rs:252-261`、`:553-557`(functional-spec.md §4 の T1。承認が要る)。

### 2e. 表示の `<file>:<line>` と `show_path`

- 今: `crates/cli/src/report/mod.rs:508-512`(`definition_line`)、`:522-526`(`env_file_lines`)、`:593-597`(`conversion_line`)が同じ `match e.line { Some(line) => format!("{file}:{line}"), None => file }`。`:543-545`(`env_warning_line`)も `{}:{}`。`escape_path(&X.display().to_string())` が `:326`、`:346`、`:385`、`:568` にあり、同じ中身の `show_path`(`:477-480`)と重なる。
- (調査との違い)`show_path` は未使用ではない。`InitCreated`・`InitReplaced`・`InitExists`・`InitFailed`(`:450-463`)が使う。重複は「`show_path` があるのに、4 か所が同じ式を書く」こと。
- 目標:
  ```rust
  /// `<ファイル>[:<行>]`(ファイル名はエスケープする。切らない)。
  fn place_of(file: &str, line: Option<usize>) -> String
  ```
  3 か所と `env_warning_line`(`place_of(&w.file, Some(w.line))`)が使う。4 か所の `escape_path(&p.display().to_string())` を `show_path(p)` に。
- 影響するテスト: なし(出力は 1 文字も変わらない)。

### 2f. graph の text と Mermaid の共通部分

- 今: 辺の種類のラベル `crates/cli/src/report/graph_text.rs:77-83`(`kind_label`)と `crates/cli/src/report/graph_mermaid.rs:86-90`(同じ `match`)。主線の辺の判定 `graph_text.rs:252` と `graph_mermaid.rs:83`(`edge.kinds.len() == 2 && edge.skipped_jobs.is_empty()`)。
- 目標(`graph_text.rs`。Mermaid はすでに `state_symbol` をここから使っている):
  ```rust
  /// 辺の種類のラベル(成功だけ、警告だけ、両方)。
  pub(crate) fn kind_label(lang: Language, kinds: &[EdgeKind]) -> &'static str
  /// 主線の辺(次のジョブへ、成功と警告の両方)か(BR3.2、BR5.3)。
  pub(crate) fn is_main_edge(edge: &FlowEdge) -> bool
  ```
- 影響するテスト: なし。

### 2g. 使い方の誤り(68)、出力の失敗(65)、分類から終了コード の補助

- 使い方の誤りを表示して 68: `crates/cli/src/ctx.rs:212-214`、`:227-229`、`:282-283`、`crates/cli/src/dry_run.rs:42-44`、`crates/cli/src/aux_commands.rs:34-35`、`:79-80`、`:108-110` の **7 か所**(調査の「9 か所」との違い。`crates/cli/src/lib.rs:144-146` は `Ctx` を作る前なので別の形で残す)。
  - 目標: `impl Ctx { pub(crate) fn usage_error(&mut self, error: &UsageError) -> i32 }`(表示して `EXIT_USAGE` を返す)。
- 出力に失敗して 65: `aux_commands.rs:48-54`(watch)、`crates/cli/src/graph.rs:60-66`、`dry_run.rs:101-106` が「`show(出力)` が偽なら `OutputFailed` を表示して `EXIT_STATE`」。`crates/cli/src/init.rs:37-42` は失敗の表示が `InitStdoutFailed`(別の文)で 65。
  - 目標: `impl Ctx { pub(crate) fn show_output(&mut self, output: &DisplayEvent<'_>, failed: &DisplayEvent<'_>) -> i32 }`(書けたら `EXIT_OK`、書けなければ `failed` を表示して `EXIT_STATE`)。4 か所が使う(init は `failed = &DisplayEvent::InitStdoutFailed`)。`convert` の出力の失敗(67。`aux_commands.rs:114-119`)は終了コードが違うので、そのまま。
- 分類から終了コード: `crates/cli/src/run.rs:89-90`、`crates/cli/src/pipeline.rs:98`・`:101`、`crates/cli/src/ctx.rs:451-452`。
  - 目標(`crates/cli/src/exit.rs`): `pub fn outcome_exit_code(outcome: &JobOutcome) -> i32`(`exit_code_for(outcome.reported_exit_code, outcome.classification == Some(Classification::Error))`)。pipeline の停止の判定(`is_error || classification.is_none()`)は残す。
- 影響するテスト: なし(`exit_codes.rs` は既存の関数だけを確かめる。足すなら §6 の新しいテスト)。

### 2h. runtime のステートの削除と「親が空なら `.`」

- 削除の重複: `crates/runtime/src/state/reset_all.rs:119-131`(`remove`)と `crates/runtime/src/state/writer.rs:118-129`(`reset_state` の中)が、「`remove_file`、`NotFound` は成功、ほかは `RemoveFailed`(`reason::REMOVE_FAILED`)」。
  - 目標(`writer.rs`): `pub(crate) fn remove_state_file(path: &Path) -> Result<bool, StateError>`(`true` は消した、`false` は無かった)。`reset_state` は結果の真偽を捨て、`reset_all` は `true` のとき `removed += 1`。
- 「親が空なら `.`」: `crates/runtime/src/state/reset_all.rs:58-61`、`crates/runtime/src/state/writer.rs:171-175`(`remove_stale_temps`)、`crates/runtime/src/file_create.rs:121-124`、`crates/runtime/src/platform/unix.rs:34-38`(`sync_dir`)の 4 か所。
  - 目標: runtime の非公開の関数 `pub(crate) fn dir_or_current(dir: &Path) -> &Path`(空なら `Path::new(".")`)を 1 か所(`crates/runtime/src/state/mod.rs`、または新しい `fs_util.rs`)に。`reset_all` と `file_create` は `dir_or_current(path.parent().unwrap_or(Path::new("")))`。`platform/unix.rs` は `unsafe` の個所に触れず、`sync_dir` の先頭だけを置き換える(`cfg` は増やさない)。`write_state_file` の `create_temp(dir, …)`(空のまま `join`)は、一時ファイルのパスの文字列が変わるため、変えない。
- 影響するテスト: なし(`reset_all.rs`・`writer.rs` の単体テストと runtime の統合テストがそのまま通ること)。

### 2i. cli の escape を core の `sanitize` に寄せる

- 今: `crates/cli/src/report/escape.rs:31-79`(`escape_display`、`escape_char`、`is_format_char`)が `crates/core/src/error.rs:229-270`(`sanitize`、`escape_char`、`is_format_char`)と同じ規則。違いは cli が「上限なし」(`None`)を持つこと。
- 目標(core の `error.rs`):
  ```rust
  /// 入力に由来する文字列を、表示用に整える(上限 `limit` は、エスケープ後の文字数。`None` は切らない)。
  pub fn escape_display(input: &str, limit: Option<usize>) -> String
  pub fn sanitize(input: &str, max_chars: usize) -> String { escape_display(input, Some(max_chars)) }
  ```
  cli の `escape.rs` は `pub use conductor_core::escape_display;`、`escape_name`、`escape_path` の薄い包みと、`pub const MAX_DISPLAY_CHARS: usize = conductor_core::MAX_KEY_DISPLAY_CHARS;`(値 64 は同じ)だけにする。converter の写し(`crates/converter/src/error.rs:159-197`)は残す(converter は core に依存しない)。
- 影響するテスト: なし(`escape_table.rs` は `conductor_cli::report::escape::{escape_display, escape_name, escape_path}` を使い、道は再公開で保つ)。

### 2j. `aux_commands.rs:112-113` の細かな冗長

- 今: `let target: TargetFormat = target;`(型を書き直すだけ)と、`std::path::Path::new(input)`(フルパス)。
- 目標: `:112` を消し、`use std::path::Path;` にして `Path::new(input)`。`TargetFormat` の import が要らなくなれば消す。
- 影響するテスト: なし。

### 2k. 段 2 のあとの仕様書(docs/spec/rust)

- `02-architecture.md:151`、`:370`(escape の置き場所)、`05-execution-flow.md:191`(`status_at` の置き場所)、`08-output-and-i18n.md:563`、`11-quality-and-build.md:151`、`:238`(escape の揃え)を、新しい置き場所に合わせる。

---

## 段 3(cli の ctx)

### 3a. `report_execution` と `result_detail` を ctx.rs の外へ

- 今: `crates/cli/src/ctx.rs:440-491`(`Ctx::report_execution`)と `:523-548`(`result_detail`)。呼ぶのは `crates/cli/src/step.rs:143` だけ。
- 目標: `step.rs` に `fn report_execution(ctx: &mut Ctx<'_>, job: &JobDefinition, command: &CommandLine, execution: &JobExecution, outcome: &JobOutcome, next: &NextStep)` と `fn result_detail(execution: &JobExecution) -> ResultDetail` を移す(非公開。中身は 2d、2g のあとの形)。`ctx.rs` の先頭の説明(`:15-18`)と `step.rs` の表(`:19-24`)を直す。
- `definition_errors`(`ctx.rs:401-404`、`pub(crate)`)は `ctx.rs` の中だけで使う(`:353`)ので、非公開(`fn`)にする。
- 影響するテスト: なし。

### 3b. `Checked` と `Session` の重複

- 今: `crates/cli/src/ctx.rs:60-70`(`Checked`)と `:72-89`(`Session`)が 7 つのフィールドを重ねて持ち、`Session::new`(`:91-105`)が 1 つずつ写す。
- 目標:
  ```rust
  pub(crate) struct Session {
      /// 実行の前の検査を終えた材料。
      pub checked: Checked,
      /// 中断の合図(シグナルの監視が書き、`run_job` が読む)。
      pub signal: Arc<InterruptSignal>,
  }
  ```
  `Session::new` を消す。`session.definition` などの読み(`run.rs` 5 か所、`pipeline.rs` 6 か所、`step.rs` 11 か所(説明の文を含む)、`retry.rs` 5 か所)を `session.checked.definition` などに。フィールドの説明(`remove_on_complete` の FR2.3 など)は `Checked` に移す。
- 影響するテスト: なし。

### 3c. `prepare_run` と `prepare_checked` の重複

- 今: `crates/cli/src/ctx.rs:143-153`(`prepare_run`)と `:186-191`(`prepare_checked` の `Run` の枝)が「`check_definition` → `require_job` → `check_run_inputs`」を重ねて持つ(違いは `require_known` の有無と位置の返し方)。`prepare_pipeline`(`:157-166`)と `Pipeline` の枝(`:181-185`)も同じ。
- 目標:
  ```rust
  /// `run <job-id>` の検査(定義 64 → command 64 → ジョブID 68 → `--assume` 68 → run ID 68 → `.env` 64 → ステート 65)。材料と位置を返す。
  fn check_job(&mut self, cmd: &CliCommand, job_id: &str, also_known_jobs: &[&str]) -> Result<(Checked, usize), i32>
  /// `pipeline <name>` の検査(ジョブIDを除く同じ順)。
  fn check_pipeline(&mut self, cmd: &CliCommand, name: &str, also_known_jobs: &[&str]) -> Result<Checked, i32>
  ```
  `prepare_run` = `start_signals` → `check_job(cmd, job_id, &[])`。`prepare_pipeline` = `start_signals` → `check_pipeline(cmd, name, &[])`。`prepare_checked` は `target` で 2 つを呼び分ける(`Run` は位置を捨てる)。`require_known` に空の並びを渡すと何もしない(表示もしない)ので、**検査の順序と表示は変わらない**。
- 影響するテスト: なし。

### 3d. `resolve_run_id` の置き場所

- 今: `crates/cli/src/args.rs:585-608`(公開の `resolve_run_id`)。引数の解釈ではなく、定義を読んだ後(`ctx.rs:279`)と `watch`・`reset`(`ctx.rs:507-521` の `aux_run_id`)で使う。仕様書 `docs/spec/rust/02-architecture.md:149`、`:349`、`12-rebuild-guide.md:76` が「`args`」と書く。
- 目標: 新しい公開のモジュール `crates/cli/src/run_id.rs`(`pub mod run_id;`)に、`resolve_run_id`(4a の新しいシグネチャ)と、`ctx.rs` の `aux_run_id`(`pub(crate)`)を移す。`definition_run_id`(表示して 68 を返す)は `Ctx` に残す。仕様書の 3 か所を直す。
- 影響するテスト: `crates/cli/tests/args_grammar.rs:621-624`(呼ぶ道。4a と一緒に。functional-spec.md §4 の T7)。

### 3e. 段 3 のあとの仕様書

- `docs/spec/rust/02-architecture.md:190`、`:224`、`:349`、`:352`、`04-cli.md:238-239`、`05-execution-flow.md:273`、`:426`、`:707`、`:713`、`07-process-and-signals.md:288`、`12-rebuild-guide.md:76`、`:127` の `prepare_*`、`Checked`、`Session`、`report_execution`、`resolve_run_id` の置き場所を新しい形に。

---

## 段 4(型とテスト)

### 4a. `CliCommand.run_id` を型で表す

- 今: `crates/cli/src/types.rs:147-149` `pub run_id: Option<String>`(空文字が「run ID なしに固定」)。`args.rs:576-583` の `parse_run_id_option` が `RunId::parse` で検査して文字列で持ち、`resolve_run_id`(`args.rs:592-608`)がもう一度 `RunId::parse` する(二重の検査)。
- 目標(`types.rs`、`lib.rs` で再公開):
  ```rust
  /// `--run-id` の指定(検査済み)。
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum RunIdArg {
      /// `--run-id ""`: run ID なしに固定する(環境変数を見ない)。
      Empty,
      /// 規則を満たす run ID。
      Id(RunId),
  }
  impl RunIdArg {
      /// `--run-id` の値を検査する(空は `Empty`。規則に反する値は `invalidValue`、対象は `--run-id`、値は含めない)。
      pub fn parse(text: &str) -> Result<Self, UsageError>
      /// 書かれた値(`Empty` は空文字)。
      pub fn as_str(&self) -> &str
  }
  pub struct CliCommand { …, pub run_id: Option<RunIdArg>, … }
  ```
  `parse_run_id_option` は UTF-8 の検査のあと `RunIdArg::parse` を呼ぶだけにする(誤りの種類と対象は今のとおり)。`resolve_run_id(option: Option<&RunIdArg>, env_name: &str, env: &dyn Env) -> Result<Option<RunId>, UsageError>`: `Some(Empty)` は `None`、`Some(Id(r))` は `Some(r.clone())`(検査しない)、`None` は環境変数を読んで `RunId::parse`(今のとおり)。
- 影響するテスト: `crates/cli/tests/args_grammar.rs:483`、`:488`、`:492`、`:700`、`:793`(読み方)と `:621-624`(補助)(functional-spec.md §4 の T7)。

### 4b. スキップの理由と拒否の理由の型を分ける

- 今: `crates/core/src/types.rs:328-350` の `DecisionReason { SkipTarget, AfterEnd, AlreadyExecuted, OutOfOrder }` を `Decision::Skip` と `Decision::Reject` の両方が使う。そのため、到達しない分岐がある: `crates/core/src/flow/record.rs:137`(`OutOfOrder` を `apply_skip` に渡す)、`crates/core/src/plan/simulate.rs:247-249`(`OutOfOrder` のスキップ)、`crates/cli/src/report/messages.rs:476-479`(拒否の理由に `SkipTarget`・`AfterEnd`・`AlreadyExecuted`)、`:513`(スキップの理由に `OutOfOrder`)。`PlanRejection.reason`(`plan/mod.rs:187`)も同じ型。
- 目標(`core/src/types.rs`):
  ```rust
  /// 起動前の判断でスキップする理由(BR3.5 の 1〜3)。
  pub enum SkipCause { SkipTarget, AfterEnd, AlreadyExecuted }
  /// 起動前の判断で拒否する理由(BR3.5 の 7、BR3.8)。
  pub enum RejectReason { OutOfOrder }
  pub enum Decision { Run, Skip(SkipCause), Reject(RejectReason) }
  ```
  (derive は今の `DecisionReason` と同じ `Debug, Clone, Copy, PartialEq, Eq`。ステートに書く `SkipReason { ByTransition, ByEnd }` とは別の型のまま。)`DecisionReason` は消す。`apply_skip(state, job_id, reason: SkipCause, definition)`、`PlanRejection { job_id, reason: RejectReason }`、cli の `DisplayEvent::Rejected { reason: RejectReason }`・`JobSkipped { reason: SkipCause }`、`step::record_skip(…, reason: SkipCause)`、`messages::reject_reason(RejectReason)`・`skip_reason(SkipCause)`。到達しない 4 つの分岐と、その文(`"skip target"`、`"after @end"`、`"already executed"` の拒否の文、`"out of order"` のスキップの文)が消える。実際に出る文は 1 文字も変わらない。
- 影響するテスト: functional-spec.md §4 の T2〜T6。

### 4c. 同じ意味の列挙型の重複、`From`、`TransitionOn::parse`

| 今 | 目標 |
|---|---|
| `crates/core/src/plan/mod.rs:117-124` `PlanMode { Pipeline, Individual }` と `crates/core/src/types.rs:410-417` `RunMode { Individual, Pipeline }` | `PlanMode` の定義を消し、`pub use crate::types::RunMode as PlanMode;`(別名。`ExecutionPlan.mode` の型は `RunMode`)。`simulate.rs:153`、`:219` は `RunMode::…` |
| `crates/core/src/graph/mod.rs:34-39` `EdgeKind { Success, Warning }` と `types.rs:157-162` `TransitionOn` | `pub use crate::types::TransitionOn as EdgeKind;` |
| `graph/mod.rs:41-48` `EdgeTarget { Job(String), End }` と `types.rs:174-181` `TransitionTarget` | `pub use crate::types::TransitionTarget as EdgeTarget;` |
| `crates/core/src/flow/record.rs:90-94` の分類 → 状態の `match` | `impl From<Classification> for JobStatus`(`types.rs`)。`record.rs` は `JobStatus::from(classification)` |
| `crates/core/src/definition/walk/mod.rs:350-352` の `"success"`・`"warning"` の文字列の比較 | `impl TransitionOn { pub fn parse(text: &str) -> Option<Self> }`(`as_str` の逆)。walk は `TransitionOn::parse(&text)` を先に見て、`None` のとき `"error"` か(`Classification::Error.as_str()`)で誤りの種類を分ける。誤りの文と種類は変えない |

- 別名にする理由: 定義は 1 つになり(重複がなくなる)、`EdgeKind::Success`、`use EdgeKind::{Success, Warning}`、`PlanMode::Individual` のような既存の書き方が、そのまま通る(`pub use … as` は型の別名と違い、`use` の道にも使える)。変種の名前が同じなので、`Debug` の表示も変わらない。
- 影響するテスト: なし(`graph_edges.rs`、`graph_types.rs`、`graph_drawable.rs`、`graph_common/mod.rs`、`plan_run.rs:83` などは別名で通る)。別名を置かずに古い名前を消す代案は functional-spec.md §4 の注。

### 4d. `runtime/tests/flow_with_state.rs` を今の cli の流れに(Q3: A)

- 今(`crates/runtime/tests/flow_with_state.rs`):
  - `invoke_inner`(`:130-205`): スキップの記録を、変化が無くても常に書く(`:166-167`)。完了で常に `reset_state` する(`:168-171`、`:192-194`)。
  - `pipeline`(`:231-244`): 再開位置から `next_job_id` をたどって、1 回ずつ `invoke` を呼ぶ(呼ぶたびにステートを読み直す)。
  - 完了の後に「ステートファイルが無い」を期待するテスト: `ac4_2_1_個別実行で最後まで行くとステートが初期化される`(`:266`)、`ac4_2_2_…`(`:299`)、`ac4_2_3_…`(`:322`)、`遷移によるスキップのあと…`(`:345`)、`ac4_2_4_一連の実行の_end_で完了しステートが初期化される`(`:360`)、`ac4_3_1_ac4_3_2_エラーからの再開`(`:390`)。
- 今の cli(本物):
  - `run`(`crates/cli/src/run.rs:48-92`、`step.rs:77-102`、`:115-159`): スキップは、土台(無ければ初期ステート)に `apply_skip`、**変わったときだけ** `Durable` で書く。記録が変わって完了したときだけ、`remove_on_complete` が真なら消す。実行は、空なら初期ステートを `Fast` で書き、実行・分類・記録・`Durable` で書き、完了で `remove_on_complete` が真なら消す。
  - `pipeline`(`crates/cli/src/pipeline.rs:38-134`): ステートを 1 回だけ読み、**定義の記述順に**各ジョブを `decide_before_run` で決める(`next_job_id` でたどらない)。スキップを記録し(変わったときだけ書く)、完了なら消すかを決めて終える。実行でエラー(または分類なし)なら止まる。完了なら終える。
  - 完了のステートは、既定では残る(`remove_on_complete` が偽)。`read_state` は完了のステートを空として返す。
- 目標:
  - `invoke_inner` に `remove_on_complete: bool` を足す(位置引数の真偽値が 2 つになるので、`struct Call { remove_on_complete: bool, stop_before_record: bool }` にまとめてよい)。スキップは変わったときだけ書く。削除は `remove_on_complete` が真のときだけ(run の規則)。`Done.next_job` は使わなくなるので消す。
  - `pipeline(scene, definition, remove_on_complete)` は、cli の `run_pipeline` と同じ順(1 回読む → 照合 → 記述順に決める → スキップ・実行・停止・完了)で、各ジョブの `Done`(スキップは `ran = false`)を返す。
  - 期待: 完了の後は「ステートファイルが残り、`read_state` は `None`(空)」を確かめる。少なくとも 1 つのテスト(`ac4_2_1`)で、`remove_on_complete = true` のときにファイルが消えることも確かめる。
  - テスト名: `ac4_2_1_個別実行で最後まで行くとステートが初期化される` → `ac4_2_1_個別実行で最後まで行くと完了のステートが残る`、`ac4_2_4_一連の実行の_end_で完了しステートが初期化される` → `ac4_2_4_一連の実行の_end_で完了し完了のステートが残る`。ほかの 18 件の名前は今の動きと矛盾しないので変えない(language-agnostic/11 が参照する名前の先頭 `ac4_2_4` などは変わらない)。
  - `ac2_5_2_…`(`:596-630`)の許す失敗の種類 `WriteFailed | RemoveFailed` は、削除しない流れでは `RemoveFailed` が起きないが、許すだけなので残してよい(変えるなら `WriteFailed` だけに)。
  - 件数は 20 のまま(Q3: A)。
- `docs/spec/rust/11-quality-and-build.md:217` の目的の文は「今の cli と同じ順で」と補う。

### 4e. 段 4 のあとの仕様書

- `docs/spec/rust/02-architecture.md:83`、`:119`、`:149`、`05-execution-flow.md:183`、`:733-734`、`:858`、`:863`、`08-output-and-i18n.md:383`(理由の表を、スキップの理由 3 つと拒否の理由 1 つに分ける)、`03-definition-format.md:595`(変えなくてよい。`Transition` の形は同じ)。`docs/spec/language-agnostic` は型の名前を言語に依らない概念として書いているので、段 4 では変えない(段 1 の `08:211` だけ直す)。
