# CI の設定: v0.10.0

出典: `.github/workflows/ci.yml`、`release.yml`、`audit.yml`、Build and Test の結果(`construction/build-and-test/test-results.md`、`build-and-test-summary.md`)、`ci-pipeline-questions.md`、team.md。

## 方針

- v0.10.0 の 7 つの機能は、CI の設定を変えずに通る。新しいジョブ、新しい依存、新しい OS は要らない。`ci.yml` は変えていない。
- 起動は `main` への push と PR(`main` 向け)。チームは `main` に直接コミットし、push の CI で関門を効かせる。

## ジョブ

| ジョブ | OS | コマンド | 関門か |
|---|---|---|---|
| `lint` | ubuntu | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | はい |
| `build-check` | ubuntu、windows(MSRV 1.100.0) | `cargo build --workspace --locked --all-targets`、C のコンパイルを要する依存と `-sys` クレートの検査 | はい |
| `test` | ubuntu、macos、windows | `cargo test --workspace --locked` | はい |
| `msrv-test` | ubuntu(1.100.0) | `cargo test --workspace --locked` | はい |
| `coverage` | ubuntu | `cargo llvm-cov ... --fail-under-lines 80`(テスト専用の補助バイナリを除く) | はい |
| `docs-check` | ubuntu | README の必須の語句 | はい |
| `startup-bench` | 3 つの OS | `cargo test -p conductor-cli --release --locked --test nfr_perf` | いいえ(`continue-on-error`) |
| `audit`(`audit.yml`) | ubuntu | `cargo audit`(PR と毎週) | いいえ(警告のみ) |

## この版の CI の結果

| 実行 | コミット | 結果 |
|---|---|---|
| 45726115228 | `aidlc: Build and Test の承認`(4f54f4a) | 失敗: `test (windows-2022)` の `runner_log` の 1 件だけ。ほかは成功 |
| 45726504409 | `runtime: テストの補助の streams は 1 行を 1 回の書き込みで出す`(3139a5c) | **成功**(13 ジョブすべて) |

### Windows の失敗と直し

- `crates/runtime/tests/runner_log.rs` の `br4_1_標準出力と標準エラーの両方がファイルに残り_各流れの中の順を保つ` が、Windows でだけ、u5 以降の CI で毎回落ちていた(ログのファイルの中で 1 行が他の流れの行と混ざる)。
- 原因はテスト専用の補助バイナリ(`crates/runtime/src/bin/conductor-test-helper.rs` の `streams`)。標準エラーはバッファを持たないため、`writeln!` が 1 行を 3 回に分けて書き、間に標準出力の行が入っていた。conductor は設計どおり「子が書いた順にそのまま書く」。
- 直し: 補助バイナリが 1 行を 1 回の書き込みで出すようにした(3139a5c)。本番のコードは変えていない。配布物にも含まれない。
- 手元(macOS)では起きなかったため、Build and Test では見えなかった。Windows だけのテストを CI に任せた所が、ここで確かめられた。

### 揺れの記録

- 作業の途中の CI(45660860712、45713211039)で、`coverage` のジョブの性能のテスト `plan_simulate_perf.rs` の `p2_終点のあとの_999_個のスキップの計画は_150ms_以内` が 2 回落ちた。最新の 2 回の実行では通っている。CI のマシンの速さによる揺れとみられ、上限は変えていない。続けて見る。

## リリース(参考。この段階では行わない)

- タグ `v0.10.0` を `main` の上に付けると、`release.yml` が版の一致の検査、`ci.yml` の再利用、4 対象のビルド、下書きのリリースを作る。公開は人が確かめてから(project.md の Mandated)。
