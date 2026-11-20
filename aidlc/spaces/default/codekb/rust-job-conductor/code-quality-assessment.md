# コード品質の評価

## 総評

小さく責務の分かれた、品質の高いコードベース。純粋関数と入出力の分離、固定の文によるエラー表示、網羅的なテスト(非機能のテストを含む)、3 OS の CI が揃っている。主な負債は、型のリテラルの散在と、複数の場所に同じ知識が重複する箇所である。

## 評価(3 段階: good / fair / poor)

| 観点 | 評価 | 根拠 |
|------|------|------|
| 命名の明快さ | good | Rust の標準の命名。日本語のドキュメントコメントが契約と規則の番号(BR/NFR/FR)に対応している |
| 関数の大きさ | fair | `walk.rs`(801 行)に検査が集中。`Walker` のメソッドは小さく分かれている。`args.rs::build_command` は約 150 行で長い |
| エラー処理 | good | 失敗は値で返す。入力に対してパニックしない。固定の文だけで表示し、入力の値を含めない |
| テストの有無 | good | 全クレートに単体と統合のテスト。実バイナリのテスト、シグナルの実テスト、非機能のテスト |
| 重複 | fair | `prepare`/`run_individual`/`run_pipeline` の流れが似ている。使い方の表が 3 か所。`peek_run_id_env` と `walk` の `run_id_env` 知識 |
| 不要なコード | good | 目立つ死んだコードは見当たらない(今回は読み取りのみで、警告は未確認) |

## テスト

- 配置は code-structure.md の「テストの配置」を参照。ファイルは core 10、runtime 13、cli 21、converter 5(いずれも tests/ の直下。補助を除く)。
- 流れのテスト(`flow_*.rs`)は偽の `Runtime`/`Sink`/`Env`/`Signals`(`crates/cli/tests/common/mod.rs`)を注入し、実際のプロセスを起動せずに検査する。実バイナリのテスト(`binary_*.rs`)は `conductor-cli-test-helper` を使う。
- 非機能のテスト(`nfr_*.rs`): ソースの検索(`unsafe`、ネットワーク、`cfg` の位置、依存)、性能(起動のオーバーヘッド)、堅牢性、セキュリティ(converter、core)、ステートの内容(runtime)。
- 網羅のテスト: `DefinitionErrorKind::ALL`、`UsageErrorKind::ALL`、`ConversionErrorKind::ALL` を使い、すべての種類に文があることを確かめる。
- カバレッジ: CI の `coverage` ジョブが行カバレッジ 80% 未満で失敗する(`cargo llvm-cov`)。実測値は今回確認していない(実行しない方針)。
- テスト名は日本語。チームの方針(test-after、行カバレッジ 80%)と一致する。

## Lint と整形

- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(CI `lint`)。設定ファイル(`rustfmt.toml`、`clippy.toml`)は無く、既定。
- `Cargo.lock` を追跡し、CI はすべて `--locked`。

## CI/CD(`.github/workflows/`)

| ファイル | ジョブ | 内容 |
|----------|--------|------|
| `ci.yml` | lint | fmt と clippy(ubuntu) |
| | build-check | ubuntu と windows。MSRV 1.100.0 で全ターゲットをビルド。Linux で C ビルド依存と `-sys` クレートを検査 |
| | test | ubuntu、macos、windows で `cargo test --workspace --locked` |
| | msrv-test | 1.100.0 でテスト |
| | coverage | `cargo llvm-cov --fail-under-lines 80` |
| | docs-check | README に 7 つの必須語句(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert) |
| | startup-bench | 3 OS。警告のみ(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`) |
| `release.yml` | verify-tag、checks、build(4 対象)、smoke test、draft-release | タグ `v*`。main 上のタグと cli の版の一致を確認。SHA256SUMS。下書き |
| `audit.yml` | cargo audit | PR と毎週月曜。`continue-on-error`、警告のみ |

トリガー: push(main)、pull_request(main)、`workflow_call`(release が CI を再利用)。`concurrency` で同じ参照の古い実行を取り消す。

## ドキュメントの質

- README(利用者向け)が、必須語句の検査つきで保たれている。
- 各 `lib.rs` に、契約、呼び出し順序、保証の範囲、守る性質が詳しく書かれている。
- `docs/spec/` は 7 版の仕様書。`docs/spec/rust/` は v0.8.0 の忠実な記述で、実装と README の差を明記している。今回の intent の機能が入ると、更新が要る。
- `demo/` に Unix 版・Windows 版の例。

## 技術的負債とリスク

1. 定義の型のリテラルの散在: `JobDefinition`、`PipelineDefinition` を構造体リテラルで直接作るテストが core(`types.rs`、`flow/decide.rs`、`flow/*`)、runtime(`runner.rs`)、cli(`tests/common/mod.rs` ほか)に多数ある。フィールドを足すと全箇所の修正が要る。
2. エラー種類の追加の連動: core の `error.rs`(`#[error]`、`code()`、`ALL: [_; 30]`)、cli の `definition_kind`(英語)、`report_messages.rs` などの網羅のテスト。
3. 使い方の表の重複(`args.rs` のコメント、`USAGE`、README)。
4. 3 か所の流れの重複(`prepare`、`run_individual`、`run_pipeline`)。`--dry-run` は、起動の直前と記録の前後の 3 か所に影響しうる。
5. `run_job` に環境変数を渡す口が無い(`Command` の環境は継承のみ)。`Runtime` trait の `run_job` の署名変更は、偽実装を持つテストに波及する。
6. `.idea/` が追跡されている。
7. `startup-bench` と `audit` は警告のみで、失敗しない。
8. macOS(Intel)の成果物は、クロスビルドで動作の確認をしていない。
9. セキュリティ上の既存の性質(ステートに環境変数を書かない、エラーに入力の値を含めない)は、`.env` の値の扱いを設計するときの制約になる。
