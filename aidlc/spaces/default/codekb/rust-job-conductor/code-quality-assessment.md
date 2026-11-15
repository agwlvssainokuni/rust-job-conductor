# Code Quality Assessment

## テスト
- 各クレートの `tests/` に統合テストと、補助バイナリ。今回の数え直し(`#[test]` の数の grep、ソース内の単体テストを含む。概数): cli 17 ファイル/204 件、core 6/122、runtime 10/165、converter 4/115。`cargo test --workspace --locked` を ubuntu・macOS・windows-2022 で実行し、MSRV(1.85)でも実行する(前回の記録)。
- カバレッジ: cargo-llvm-cov で行 80% 未満なら失敗(補助バイナリは除外。前回の記録)。
- Windows の Ctrl+C は自動テストなし。リリース前の手動チェックリストのみ(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)。
- Windows のデモは `crates/cli/tests/windows_demo.rs`(`cfg(windows)`)が、`demo/windows` の場面 1〜6 の終了コードを確かめる。
- 引数の文法は `args_grammar.rs`(26 件)、使い方の文言の先頭は `report_messages.rs`・`flow_aux.rs`、終了コードは `exit_codes.rs` が固定している。コマンドや使い方の文言を変えると、これらの期待値の更新が要る。
- ステートファイルの名前 `conductor.toml.state` は、`crates/cli/tests/common/mod.rs`(`Harness::state_path`)、`bin_support/mod.rs`、`flow_*.rs`、`binary_basic.rs` などが前提にする。ステートの TOML の形は、`runtime/src/state/reader.rs` の単体テストと `runtime/tests/{state_read,state_write,nfr_content}.rs` が固定する。
- `reset` が壊れた定義でも動くことは `flow_aux.rs:157-163` が固定する(`runtime.log() == ["reset"]`)。

## 静的検査と CI/CD
- `cargo fmt --check`、`cargo clippy -D warnings`(設定ファイルなし)。docs-check が README の必須語を検査。startup-bench は警告のみ(前回の記録)。
- release.yml: タグ v* が main 上にあり、クレートの版と一致することを確認 → CI を再利用 → 4 ターゲットをビルド → 展開して `--version` を確認(x86_64 macOS を除く)→ SHA256SUMS 付きの下書きリリース(前回の記録)。

## 文書
- README.md は詳細で、日本語。各 `lib.rs` に設計の表。`demo/README.md` に UNIX 版と Windows 版の説明あり。使い方の文言が `messages.rs` の `USAGE` と `README.md` の 2 か所にある。

## 技術的負債・懸念
- Windows の Ctrl+C は手動確認のみ。Windows の Ctrl+Break とコンソールを閉じた通知は対象外。
- doc コメントがリポジトリ外の仕様 ID(BR・NFR・W など)を参照している。
- Intel macOS は動作未確認(クロスビルド)。
- `PipelineState.definition_key` は「仮」とコメントされ、`definition_path` と同じ値で書かれ、照合に使われない(`crates/core/src/types.rs:192`)。
- ステートの全フィールドと定義の全フィールドを、テストが構造体リテラルで直接書くため、フィールドの追加でテストの修正が多数要る。
- `Runtime` trait の `read_state`・`write_state`・`reset_state` が定義のパスだけを受け取り、2 種類のテスト用の偽(`ScriptedRuntime`、`MemRuntime`)が同じ署名を持つ。
- 解消済み(前回の懸念): `Ctx::prepare` から定義の検査が切り出された(`check_definition`)。`PipelineDefinition.jobs` は `job` に改名された。
