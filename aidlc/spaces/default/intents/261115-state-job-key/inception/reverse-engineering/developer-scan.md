## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `crates/runtime/` (`Cargo.toml`、`src/lib.rs`、`src/types.rs`、`src/error.rs`、`src/split.rs`、`src/runner.rs`、`src/state/{mod,reader,writer,record,consistency}.rs`、`src/bin/`、`src/platform/`、`tests/*.rs`、`tests/common/mod.rs`)
  - `README.md`(ステートファイルの節、定義の書式の節)
  - `demo/`(`conductor.toml`、`run-demo.sh`、`README.md`、`scripts/*.sh`)
- **Skimmed only**:
  - `crates/core/`(`types.rs` の `Command` 列挙型、`PipelineState`、`JobState` だけを確認)
  - `crates/converter/`(ステートに関わらないことをディレクトリ名と grep で確認)
  - `crates/cli/`(`tests/flow_run.rs` の `[[jobs]]` の 1 件だけを grep で確認)
  - `.github/workflows/`(ファイル名だけ)
- **Left out**: `target/`、`.idea/`、`.claude/`、`aidlc/`(ワークスペース)、`local-docs/`

### Packages Found
- conductor-runtime — library(+ テスト用 bin `conductor-test-helper`) — Rust — ジョブの起動・シグナル転送、ステートの読み書き・記録・照合
- conductor-core / conductor-converter / conductor-cli — Rust — 浅く確認のみ(前回の成果物のとおり)

### Build System
- **Type**: Cargo ワークスペース
- **Config Files**: `Cargo.toml`(ルート)、`crates/runtime/Cargo.toml`(v0.2.0。`toml` 1.2.1、`thiserror` 2.0.21、`libc` 0.2(unix)、dev: `tempfile` 3.27.0)
- **Build Dependencies**: runtime → core

### APIs Discovered
- Rust 公開関数 — `crates/runtime/src/lib.rs` — `run_job`、`check_command(&Command)`、`read_state`、`write_state`、`reset_state`、`check_consistency`、`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`(10 関数)
- ステートファイルの書式(TOML、`format_version = 1`)— `reader.rs`(`TOP_KEYS`、`JOB_KEYS`)と `writer.rs`(`serialize`)

### Frameworks & Libraries
- toml — 1.2.1 — ステートの読み書き(`toml::Table` と `toml::Value` で走査。`serde` 機能あり)
- thiserror — 2.0.21 — エラー型
- libc — 0.2 — UNIX のシグナル送信(`unsafe` は `platform/unix.rs` の 1 か所)

### Test Coverage
- **Test Directories**: `crates/runtime/tests/`(10 ファイル + `common/mod.rs`)、各 `src` の `#[cfg(test)]`
- **Test Frameworks**: 標準の `cargo test`、`tempfile`
- **Coverage Config**: CI の `cargo llvm-cov`(行 80% 以上。前回の成果物による)。runtime の `#[test]` は約 154 件(src 内 約 80、tests 内 約 80 を grep で数えた概数)

### Code Quality Indicators
- **Linting**: `cargo fmt --check`、`clippy -D warnings`(CI)。設定ファイルは無し
- **CI/CD**: `.github/workflows/{ci,release,audit}.yml`(ファイル名のみ確認)
- **Documentation**: 日本語の `//!` 文書が充実。README にステートの「場所」の節はあるが、ステートの内部の書式(キー名)の記述は無い

### Technical Debt Signals
- ステートの書式のキー名 `jobs` が、定義の `[[job]]` と食い違っている。同じ綴りが次の箇所に分散している(いずれも文字列リテラル)。
  - `crates/runtime/src/state/reader.rs`: `TOP_KEYS` の `"jobs"`(L134)、`walk` の `required(table, "jobs")`(L194)、テストの `VALID` と置換文字列(L292、L297、L396-400 付近)
  - `crates/runtime/src/state/writer.rs`: `serialize` の `"jobs".into()`(L260)
  - `crates/runtime/src/error.rs`: `reason::ORDER_MISMATCH` = 「jobs の並びが job_order と一致しません」(L117)。利用者に見える文
  - テスト: `tests/state_read.rs`(L33、L38)、`tests/nfr_robust.rs`(L47、L52、L74-75、L130-131)、`tests/toml_depth.rs`(L50、L55)、`tests/state_write.rs`(L109、L116)
- `state/writer.rs` の `serialize` は、`definition_key` に `state.definition_path` を入れている(別の値ではない)。テスト名に「`e3_` のキー」とあるなど、要件 ID が残っている

## Handoff Summary
- **Intent-relevant finding**: ステートの `[[jobs]]` は runtime の 3 か所と、runtime のテスト 4 ファイルに閉じている。読み取りは `reader.rs` L134(`TOP_KEYS`)と L194(`required(table, "jobs")`)、書き込みは `writer.rs` L260(`"jobs".into()` で配列テーブルを挿入。`toml::to_string` が `[[jobs]]` の形に出す)。`consistency.rs`・`record.rs` の `.jobs` は定義側(`PipelineDefinition.jobs`)の Rust フィールドで、ステートのキーとは無関係。`format_version` は `reader.rs` L27 の `FORMAT_VERSION = 1` で、版の確認は他の検査より先のため、旧形式(`[[jobs]]`)のファイルは、版が 1 のまま `required` の欠落(`jobs` が未知のキー、`job` が必須のキーの欠落)で `corrupt`(65)になる。旧形式の読み込み互換は無い設計と合う。
- **Risks / follow-up**:
  - `reason::ORDER_MISMATCH` の文面に `jobs` が入っている(`error.rs` L117)。キー名を変えるなら文面も揃える。
  - `README.md` のステートの節(L208 付近)は場所と動作だけで、キー名を書いていない。書式の記述を足すか否かは要件次第。L168 の「以前の版の `[[jobs]]` は `[[job]]` に変わりました」は定義ファイルの話で、ステートではない(混同に注意)。
  - `demo/` はステートの中身に触れない(`watch` の表示だけ)。変更は不要の見込み。
  - スキャン範囲外だが、`crates/cli/tests/flow_run.rs` L251 は定義の旧キー `[[jobs]]` が 64 になることを確認するテスト(ステートではない)で、影響しない。cli のテストにステートの `[[jobs]]` を直接書く箇所は grep では見つからなかった(浅い確認)。
  - ビルド・テストは実行していない。
  - 前回のストアは `command: String` を前提に書かれていたが、現在のコードは `conductor_core::Command`(`Line(String)` / `Argv(Vec<String>)`)。`check_command(&Command)`、`split.rs` の `command_line` で対応済み(runtime v0.2.0)。ストアの該当箇所を今回更新する。
