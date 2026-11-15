# コード品質の評価

## テスト

- 統合テスト 41 ファイル(cli 20、runtime 11、core 6、converter 4)と、各 `src/` 内の単体テスト。`#[test]` はおよそ 681 個。
- バイナリを実際に動かすテスト(`binary_*.rs`、`signals_real.rs`、`unix_demo.rs`、`windows_demo.rs`)と、性能(`nfr_perf.rs`。CI の `startup-bench` は警告のみ、`STARTUP_BENCH_ENFORCE=true` で強制)、堅牢性(`nfr_robust.rs`)、セキュリティ(`nfr_security.rs`)のテストがある。
- カバレッジ: CI の `coverage` が、行 80% 未満で失敗(`cargo llvm-cov --workspace --locked --fail-under-lines 80`。補助の bin は除外)。実測値はこの走査では確認していない(実行しない約束)。
- README・デモを固定するもの: `unix_demo.rs`、`windows_demo.rs`(デモ)、`report_messages.rs`(USAGE ほかの文言)。README の本文を読むテストはない。

## 静的解析・整形

- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(`ci.yml` の `lint`)。設定ファイルなし。
- `#![forbid(unsafe_code)]`(cli・core)、`#![deny(unsafe_code)]` + UNIX の `kill` 1 か所だけ許可(runtime)。

## CI/CD

| ジョブ | 内容 |
|--------|------|
| `lint` | fmt・clippy(stable) |
| `build-check` | ubuntu・windows、Rust 1.85.0 で全ターゲットをビルド。C の依存の禁止 |
| `test` | ubuntu・macos・windows、stable |
| `msrv-test` | ubuntu、1.85.0 でテスト |
| `coverage` | 行 80% |
| `docs-check` | README に 7 語があること(`ci.yml:114-127`): `SHA-256`・`Gatekeeper`・`終了コードの表`・`ステートファイルの場所`・`at-least-once`・`シグナル`・`convert`。`SHA-256`・`終了コードの表`・`ステートファイルの場所`・`at-least-once` は、README の見出しにだけ現れる |
| `startup-bench` | 起動の測定。`continue-on-error` |
| `audit.yml` | `cargo audit`(PR と毎週月曜。警告のみ) |
| `release.yml` | タグ `v*` → `verify-tag`(main の祖先、版の一致)→ `checks`(ci.yml を流用)→ 4 ターゲットのビルドとスモークテスト → `SHA256SUMS` → 下書きのリリース |

## ドキュメントの質

- doc コメントは、契約・順序・保証・守る性質を詳しく書く。README は日本語で、網羅的だが、run ID の記述が 9 か所に散る(developer-scan.md (1))。
- README の主張は、走査した範囲ではコードと一致した。直すべき点: アーカイブ展開後のパス(README 47-50 行)、`validate` のシグナルの扱いの書き漏らし(251 行)、`reset --all` の `<名>.state` の扱いのあいまいさ(235 行)、断片的な「(68)」(99 行)(詳細は developer-scan.md (2))。

## 技術的負債

- エスケープ関数が 4 クレートに重複(`tests/escape_table.rs` で揃えを確認)。
- README と USAGE の二重管理。
- 要件 ID(BR/NFR/FR/W)を指す doc コメントがあるが、対応する文書がリポジトリにない。
- Intel macOS のバイナリは、クロスビルドで、スモークテストをしない(`release.yml:84`)。
- Windows の Ctrl+Break・コンソールを閉じた通知、ジョブのタイムアウト、排他制御は、対象外。
