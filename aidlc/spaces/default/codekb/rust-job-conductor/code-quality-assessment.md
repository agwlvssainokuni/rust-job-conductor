# Code Quality Assessment

## テスト
- 各クレートの `tests/` に統合テストと、補助バイナリ。今回の数え直し(`#[test]` の数、ソース内の単体テストを含む): cli 16 ファイル/192 件、core 6/122、runtime 10/165、converter 4/115。`cargo test --workspace --locked` を ubuntu・macOS・windows-2022 で実行し、MSRV(1.85)でも実行する(前回の記録)。
- カバレッジ: cargo-llvm-cov で行 80% 未満なら失敗(補助バイナリは除外。前回の記録)。
- Windows の Ctrl+C は自動テストなし。リリース前の手動チェックリストのみ(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)。
- Windows のデモは `crates/cli/tests/windows_demo.rs`(`cfg(windows)`)が、`demo/windows` の場面 1〜6 の終了コードを確かめる。
- 引数の文法は `args_grammar.rs`、使い方の文言は `report_messages.rs`、終了コードは `exit_codes.rs` が固定している。コマンドや使い方の文言を変えると、これらの期待値の更新が要る。

## 静的検査と CI/CD
- `cargo fmt --check`、`cargo clippy -D warnings`(設定ファイルなし)。docs-check が README の必須語を検査。startup-bench は警告のみ(前回の記録)。
- release.yml: タグ v* が main 上にあり、クレートの版と一致することを確認 → CI を再利用 → 4 ターゲットをビルド → 展開して `--version` を確認(x86_64 macOS を除く)→ SHA256SUMS 付きの下書きリリース(前回の記録)。

## 文書
- README.md は詳細で、日本語。各 `lib.rs` に設計の表。`demo/README.md` に UNIX 版と Windows 版の説明あり。使い方の文言が `messages.rs` の `USAGE` と `README.md` の 2 か所にある。

## 技術的負債・懸念
- 実行前の検査(`Ctx::prepare`)が、シグナル・定義・command・ジョブID・ステートを一続きに行い、定義の検査だけを呼ぶ入口がない。
- `PipelineDefinition.jobs` が、定義ファイルのキー `job` と名前が食い違う(Rust のフィールドだけ)。使用箇所は製品コード 14 か所前後、テスト多数(行の一覧は、この intent の `developer-scan.md`)。
- Windows の Ctrl+C は手動確認のみ。Windows の Ctrl+Break とコンソールを閉じた通知は対象外。
- doc コメントがリポジトリ外の仕様 ID(BR・NFR・W など)を参照している。
- Intel macOS は動作未確認(クロスビルド)。
