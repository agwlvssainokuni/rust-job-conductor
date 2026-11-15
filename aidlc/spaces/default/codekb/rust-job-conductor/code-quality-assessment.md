# Code Quality Assessment

## テスト
- 各クレートの `tests/` に統合テストと、補助バイナリ。件数の概数は前回の数え直し(cli 204 件、core 122 件、runtime 165 件、converter 115 件)で、run ID の実装前の数。run ID 関連のテストが加わった(cli: `flow_run_id.rs`・`binary_run_id.rs`、runtime: `run_id_state.rs`)。`cargo test --workspace --locked` を ubuntu・macOS・windows-2022 で実行し、MSRV(1.85)でも実行する(前回の記録)。
- カバレッジ: cargo-llvm-cov で行 80% 未満なら失敗(補助バイナリは除外。前回の記録)。
- Windows の Ctrl+C は自動テストなし。リリース前の手動チェックリストのみ(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)。
- Windows のデモは `crates/cli/tests/windows_demo.rs`(`cfg(windows)`)が、`demo/windows` の定義とスクリプトを一時ディレクトリへ写し、本物の `conductor`(`LANG=C`、`DEMO_SLEEP=0`、30 秒の上限)で場面 1〜6 の終了コードと、ジョブの出力(`[<job>] 実行中`)を確かめる(`run-demo.cmd` 自体は実行しない)。run ID の場面は検査していない。`CONDUCTOR_RUN_ID` を環境から外さないため、環境に設定があると影響を受けうる。macOS・Linux の開発機では実行されない。
- UNIX 版のデモ(`run-demo.sh`)を検査する自動テストは、ない。デモの出力を `grep` する検査もない。
- 並行する 2 つの run ID は、`binary_run_id.rs` が、スレッドで同時に実行して検査している(デモ向けの出力の順序は見ていない)。
- 引数の文法は `args_grammar.rs`、使い方の文言の先頭は `report_messages.rs`・`flow_aux.rs`、終了コードは `exit_codes.rs` が固定している。コマンドや使い方の文言を変えると、これらの期待値の更新が要る。
- ステートファイルの名前 `conductor.toml.state` は、`crates/cli/tests/common/mod.rs`(`Harness::state_path`)、`bin_support/mod.rs`、`flow_*.rs`、`binary_basic.rs` などが前提にする。run ID ありの名前は `flow_run_id.rs`・`binary_run_id.rs` が固定する。ステートの TOML の形は、`runtime/src/state/reader.rs` の単体テストと `runtime/tests/{state_read,state_write,nfr_content,run_id_state}.rs` が固定する。
- `reset` が壊れた定義でも動くことは `flow_aux.rs` が固定する(`runtime.log() == ["reset"]`)。

## 静的検査と CI/CD
- `cargo fmt --check`、`cargo clippy -D warnings`(設定ファイルなし)。docs-check が README の必須語を検査。startup-bench は警告のみ(前回の記録。`.github/` は今回未走査)。docs-check が `demo/README.md` を見るかは、未確認。
- release.yml: タグ v* が main 上にあり、クレートの版と一致することを確認 → CI を再利用 → 4 ターゲットをビルド → 展開して `--version` を確認(x86_64 macOS を除く)→ SHA256SUMS 付きの下書きリリース(前回の記録)。

## 文書
- README.md は詳細で、日本語。run ID の節(「run ID でステートを分ける(v0.5.0 以降)」)あり。各 `lib.rs` に設計の表。`demo/README.md` に UNIX 版と Windows 版の説明あり(run ID の記述は、まだない)。使い方の文言が `messages.rs` の `USAGE` と `README.md` の 2 か所にある。

## 技術的負債・懸念
- Windows の Ctrl+C は手動確認のみ。Windows の Ctrl+Break とコンソールを閉じた通知は対象外。
- doc コメントがリポジトリ外の仕様 ID(BR・NFR・W・FR など)を参照している。
- Intel macOS は動作未確認(クロスビルド)。
- `PipelineState.definition_key` は「仮」とコメントされ、`definition_path` と同じ値で書かれ、照合に使われない(`crates/core/src/types.rs`)。
- ステートの全フィールドと定義の全フィールドを、テストが構造体リテラルで直接書くため、フィールドの追加でテストの修正が多数要る。
- 2 種類のテスト用の偽(`ScriptedRuntime`、`MemRuntime`)が、`Runtime` trait の署名(9 メソッド)を持つ。
- デモの落とし穴(この intent の `developer-scan.md` も参照): どちらのデモも `CONDUCTOR_RUN_ID` を環境から消さない。UNIX 版は `LC_ALL` を消さないので、利用者の `LC_ALL` が `DEMO_LANG` に勝ちうる。Windows 版は、途中で中断すると一時ディレクトリとコードページが戻らない(`trap` に当たるものがない)。`windows_demo.rs` は `run-demo.cmd` を実行しないので、スクリプトの記述の誤りは検出されない。
- 解消済み(前回の懸念): `Ctx::prepare` から定義の検査が切り出された(`check_definition`)。`PipelineDefinition.jobs` は `job` に改名された。`watch`・`reset` が定義を読まない設計と `run_id_env` の両立は、「`--run-id` があれば読まない、なければ `run_id_env` だけをゆるく読む」で解決した。
