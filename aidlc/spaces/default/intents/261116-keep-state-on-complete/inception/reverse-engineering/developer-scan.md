## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `crates/cli/src/`(`pipeline.rs`、`run.rs`、`args.rs`、`ctx.rs`、`aux_commands.rs`、`deps.rs`、`exit.rs`、`lib.rs`、`types.rs`、`report/mod.rs`、`report/watch.rs`、`report/messages.rs` の watch 関連)
  - `crates/core/src/`(`types.rs`、`run_id.rs`、`definition/walk.rs` の `[pipeline]` 部分、`flow/decide.rs`、`flow/next_step.rs` の完了判定)
  - `crates/runtime/src/`(`lib.rs`、`state/reader.rs`、`state/writer.rs`、`state/reset_all.rs`、`state/record.rs` の完了判定、`types.rs` の `StateLocation`)
  - `Cargo.toml`、`crates/*/Cargo.toml`、`.github/workflows/`(`ci.yml`、`release.yml`、`audit.yml`)
  - `README.md`(コマンド、定義、ステート、終了コードの節)、`demo/README.md`、`demo/run-demo.sh`、`demo/windows/run-demo.cmd`(場面 8・9)
  - テストの該当箇所: `crates/cli/tests/{flow_pipeline,flow_run,flow_run_id,flow_aux,binary_basic,binary_run_id,unix_demo,windows_demo,common/mod}.rs`
- **Skimmed only**:
  - `crates/converter/src/`(`lib.rs` の説明だけ。`[pipeline]` のキーを知らない汎用変換であることを確認)
  - `crates/core/src/definition/{intake,parse,version,depth}.rs`、`crates/core/src/flow/{classify,resume}.rs`、`crates/core/src/error.rs`
  - `crates/runtime/src/{runner,split,error,platform}`、`crates/runtime/src/state/consistency.rs`
  - `crates/cli/src/{platform,panic_hook}`、`crates/cli/src/report/escape.rs`、`crates/cli/src/bin/`
  - 上記以外の `crates/*/tests/`、`crates/*/tests/fixtures/`、`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`
  - `demo/conductor.toml`、`demo/scripts/`、`demo/windows/conductor.toml`、`demo/windows/scripts/`、`LICENSE`、`Cargo.lock`
- **Left out**: `target/`(`.gitignore` で除外。ビルド出力)、`local-docs/`(`.gitignore` で除外)、`.idea/`(IDE の設定)、`.claude/`・`aidlc/`(AI-DLC 自身のインストールとワークスペース)

### Packages Found
- conductor-core — ライブラリ — Rust — 定義の読み込みと検証、フローの判断(分類、起動前の判断、次の動き)
- conductor-runtime — ライブラリ(+ テスト用バイナリ) — Rust — ジョブの起動とシグナル転送、ステートの読み書き・照合・初期化・記録
- conductor-converter — ライブラリ — Rust — 定義ファイルの TOML と YAML の相互変換
- conductor-cli — ライブラリ + バイナリ `conductor`(+ テスト用バイナリ) — Rust — 引数の解釈、流れの組み立て、シグナル、表示、終了コード

### Build System
- **Type**: Cargo ワークスペース(`resolver = "3"`、edition 2024、rust-version 1.85)
- **Config Files**: `Cargo.toml`、`crates/*/Cargo.toml`、`Cargo.lock`。`rustfmt.toml`・`clippy.toml` はない(既定)
- **Build Dependencies**: cli → core, runtime, converter / runtime → core / converter は core に依存しない

### APIs Discovered
- CLI(6 コマンド: `run`・`pipeline`・`watch`・`reset`・`validate`・`convert`) — `crates/cli/src/args.rs` — オプション `-v`・`--config`・`--run-id`・`--all`・`--to`
- 差し替え口 trait `Runtime`(9 メソッド)・`Sink`・`Env`・`Signals` — `crates/cli/src/deps.rs`
- ライブラリの公開関数(契約 C3) — `crates/runtime/src/lib.rs`: `read_state`・`write_state`・`reset_state`・`reset_all_states`・`initial_state`・`apply_*`・`check_consistency` ほか
- ステートファイル(TOML、`format_version = 1`) — `crates/runtime/src/state/{reader,writer}.rs`

### Frameworks & Libraries
- lexopt — 0.3.2 — 引数の字句分け
- toml — 1.2.1 — 定義とステートの読み書き
- thiserror — 2.0.21 — エラー型
- saphyr-parser — 0.3.4 — YAML の解析(converter)
- signal-hook 0.4.6(UNIX)、ctrlc 3.5.2(Windows)、libc 0.2(runtime の UNIX)
- tempfile 3.27.0 — テスト用(dev)

### Test Coverage
- **Test Directories**: `crates/{cli,core,runtime,converter}/tests/`、各 `src/` 内の `#[cfg(test)]`(`#[test]` は全体で約 681 件。cli 248、core 133、runtime 185、converter 115)
- **Test Frameworks**: 標準の `cargo test`(日本語名のテスト関数、偽の `Runtime`/`Sink`/`Env`/`Signals` を使う `flow_*`、実バイナリを動かす `binary_*`・`signals_*`・`unix_demo`・`windows_demo`)
- **Coverage Config**: CI の `coverage` ジョブが行カバレッジ 80% を要求(`ci.yml`)

### Code Quality Indicators
- **Linting**: `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings`(`ci.yml` の `lint`)
- **CI/CD**: `ci.yml`(lint、build-check、test、msrv-test、coverage、docs-check、startup-bench)、`release.yml`(タグ `v*` で 4 ターゲットをビルドして下書きリリース)、`audit.yml`(週次の cargo audit)
- **Documentation**: README(日本語)、`demo/README.md`、各クレートの `//!` の説明が充実。`docs-check` が README の必須の節を確認する

### Technical Debt Signals
- 「完了したら消す」が、1 か所の関数ではなく、`run.rs`(スキップ経路と実行経路の 2 か所)と `pipeline.rs`(`finish` 経由の 2 か所)の呼び出しに分かれている。`ctx.reset` を直接呼ぶ。切り替えの条件を足すには、この 4 つの呼び出しを同じ判断にそろえる必要がある。
- `read_state` は、`completed = true` のステートを `Ok(None)`(空)として返す(`reader.rs:169-172`)。`Runtime` trait に、完了のステートを区別して読む口がない。`watch` は `read_state` だけを使うので、完了のステートを残しても、いまのままでは `WatchEmpty`(`no state`)になる。
- `crates/core/src/definition/walk.rs` の `[pipeline]` は `PIPELINE_KEYS = ["name", "description", "run_id_env"]`(34 行)で、値の取り出しは `string` ヘルパーのみ。真偽値のヘルパーはない。`PipelineDefinition`(`types.rs`)に、削除の設定を持つ項目はない(依頼の `PipelineSection` という名前の型は、コードにない。実体は `PipelineHead` と `PIPELINE_KEYS`)。
- `args.rs` の `Token` と `build_command` は、コマンドごとの許可を `matches!` の連なりで持つ。`--all` は `reset` のみ許可、というのと同じ形で、新しいオプションを足す。文法の表は、`args.rs` の先頭の説明と `tests/args_grammar.rs` にもある。
- 既存テストの多くが「完了後にステートがない」を前提にする: `flow_pipeline.rs`(41、67、83、115、145、157、343 行付近)、`flow_run_id.rs`(53、71、143、233、357)、`binary_basic.rs`(55、312)、`binary_run_id.rs`(100、122、143、169、253)、`flow_aux.rs`(`watch_は完了のまま残ったステートを空として扱う`)、`unix_demo.rs`(75 行の `no state`)、`windows_demo.rs`(294 行)。`flow_pipeline.rs` の `完了後の初期化の失敗は_65`(187 行)は、`fail_reset` で初期化の失敗を作って 65 を確かめる。

## Handoff Summary
- **Intent-relevant finding**:
  - 完了判定は 3 経路で作られる: スキップ後の `apply_skip` の `completed`(`record.rs:164`)、実行後の `next.completed`(`classify_and_decide`、`flow/next_step.rs`)、中断時の `resume_point.completed`(`apply_interrupt`、`record.rs:176`)。
  - 削除の呼び出しは 4 か所: `pipeline.rs`(スキップ後 98-99 行、実行後 145-147 行。どちらも `finish`(179 行)から `ctx.reset`)、`run.rs`(スキップ後 76-80 行、実行後 128-132 行。`ctx.reset` を直接)。`ctx.reset`(`ctx.rs:208`)が `Runtime::reset_state` を呼び、`writer.rs:115` が `remove_file` する(存在しなければ成功)。失敗は `removeFailed`、終了コード 65。
  - 完了のステートが残る場合の読み: `read_state` が空(`None`)を返すので、`prepare`(`ctx.rs:120`)の後の `run`・`pipeline` は初期ステートを `Fast` で書き直して先頭から始まる。この挙動は変えない。
  - `watch`(`aux_commands.rs:29`)は `read_state` の `None` を `WatchEmpty`(`messages.rs` の `WATCH_EMPTY`)に、`Some` を `render_table`(`report/watch.rs`)にする。完了を示す表示はない。
  - `reset_all_states` は、run ID つきの完了のステートも `Inspected::Ours` として消す。run ID なしは読まずに消す。
  - 引数: `--remove-on-complete` はまだない。許可する場所は `run`・`pipeline` だけにするのが自然(`takes_verbose` と同じ形)。
- **Risks / follow-up**:
  - 完了のステートが残ると、`check_consistency`(定義との食い違い)は、読み返しが空になるので働かない。完了のステートが壊れている、または版が違う場合、`read_state` は `corrupt`/`versionMismatch` を返して 65 になる(`completed` の判定より前に検査される)。残したステートの扱いは設計で決める。
  - `watch` が完了を表示するには、`read_state` の「完了は空」という約束(README の「完了した run のステートは、これまでどおり消えます」、`reader.rs` の説明、`runtime/lib.rs` の説明)を壊さずに、完了のステートを読む別の入口が要る。
  - README(184-188、328、335、345 行)、`demo/README.md`、`run-demo.sh`・`run-demo.cmd` の場面 4・8(「完了したので、ステートファイルは無くなる」)・9、`unix_demo.rs`・`windows_demo.rs` の期待は、挙動の変更に合わせて直す。デモの場面 5 の `reset` と場面 9 の `reset --all` は、残ったステートの後始末としてそのまま意味を持つ。
  - converter は `[pipeline]` のキーを知らない汎用変換なので、新しい定義のキーを足しても、converter の変更は要らない見込み(`walk.rs` の検証だけ)。
