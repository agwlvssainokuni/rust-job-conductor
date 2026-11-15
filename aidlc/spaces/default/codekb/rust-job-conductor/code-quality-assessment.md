# Code Quality Assessment

## テスト
- 各クレートの `tests/` に統合テスト(cli 15 ファイル前後、core 6、runtime 3、converter 5)と、補助バイナリ。`cargo test --workspace --locked` を ubuntu・macOS・windows-2022 で実行し、MSRV(1.85)でも実行する。
- カバレッジ: cargo-llvm-cov で行 80% 未満なら失敗(補助バイナリは除外)。
- Windows の Ctrl+C は自動テストなし。リリース前の手動チェックリストのみ(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)。

## 静的検査と CI/CD
- `cargo fmt --check`、`cargo clippy -D warnings`(設定ファイルなし)。docs-check が README の必須語を検査。startup-bench は警告のみ。
- release.yml: タグ v* が main 上にあり、クレートの版と一致することを確認 → CI を再利用 → 4 ターゲットをビルド → 展開して `--version` を確認(x86_64 macOS を除く)→ SHA256SUMS 付きの下書きリリース。

## 文書
- README.md は詳細で、日本語。各 `lib.rs` に設計の表。`demo/README.md` あり。

## 技術的負債・懸念
- デモが UNIX 系専用(`demo/`)。Windows 版が無い(本 intent の対象)。
- Windows の Ctrl+C は手動確認のみ。Windows の Ctrl+Break とコンソールを閉じた通知は対象外。
- doc コメントがリポジトリ外の仕様 ID(BR・NFR・W など)を参照している。
- Intel macOS は動作未確認(クロスビルド)。
