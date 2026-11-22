# ビルドとテストのまとめ(Build and Test Summary): v0.9.0 の機能拡張

## 全体の状況

- ビルド: 成功(`cargo build --workspace --locked`)。`cargo fmt` と `cargo clippy -D warnings` も通った。
- テスト: プロジェクト全体で **1,017 件が通り、失敗 0 件**。ユニットごとの範囲を絞った 15 のコマンドも、すべて通った。
- 行カバレッジ: **97.94%**(下限 80%)。
- 前提: Rust 1.100 以上(このマシンは `rustc 1.100.0`。MSRV と同じ)、`cargo-llvm-cov`。新しい外部ライブラリは足していない。
- 詳細は `build-instructions.md`、`integration-test-instructions.md`、`test-results.md`、`cross-unit-traceability.md`。

## テストの種類の一覧(Standard の戦略)

| 種類 | 内容 | 場所 |
|------|------|------|
| 単体 | 型、定義の検査、`.env` の解析、層の統合、辺の取り出し、レーンの配置(正解データ 537 例)、模擬と計画、ひな形 | 各ユニットの `unit-test-instructions.md` |
| 統合(偽の `Runtime`) | `.env` の誤りで何も実行しない、`graph`、`--dry-run`、`init` の流れ、実行との一致 | `crates/cli/tests/flow_*.rs` |
| 統合(実バイナリ) | 環境変数が子のプロセスに見える、値が出力に出ない、`graph`/`--dry-run`/`init` の出力、`init` → `validate` → `--dry-run` | `crates/cli/tests/binary_*.rs` |
| 統合(一時ディレクトリ) | `.env` の読み取り、排他的なファイルの作成 | `crates/runtime/tests/` |
| 非機能 | 1,000 ジョブの性能、ソースの規則(`unsafe`、`cfg`、依存) | `graph_perf.rs`、`plan_perf.rs`、`nfr_source.rs` |
| 文書 | README の新しい語句と出力例 | `crates/cli/tests/docs_*.rs` |

統合テストの手順は `integration-test-instructions.md`。性能と安全性の専用の手順書(Comprehensive の戦略)は、この戦略の範囲外で、作っていない。

## カバレッジの期待

- プロジェクト全体: 行カバレッジ 80% 以上(CI の `coverage` と同じコマンド)。結果は 97.94%。
- ユニットごとの個別の目標は、定めていない(全体のしきい値のみ)。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| T1-coverage | Testing Contract(各ユニットの `code-generation-plan.md`)、`team.md`、`project.md` の ALWAYS | プロジェクト全体の行カバレッジ 80% 以上 | 97.94%(9,308 行のうち未通過 192 行) | `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` が終了コード 0(`test-results.md`) | build-and-test | Met |
| T2-ci-before-merge | Testing Contract の「CI で実行してからマージ」、`team.md` | CI で、選んだテストを、マージの前に実行する | — | CI Pipeline(この計画の後の段階)が、CI でこれらのテストを実行する設定を受け持つ。この段階の時点で CI Pipeline は、まだ先にある(`[ ]`) | ci-pipeline | N/A(CI Pipeline は、この計画のあとの段階が、CI でこれらのテストを実行する設定を受け持つ) |
| T3-nfr1-perf | `requirements.md` の NFR1 | ジョブ 1,000 個で、`graph` と `--dry-run` が 1 秒以内 | `graph_perf`(2 件。入れ子の最大と交差する形)と `plan_perf`(1 件)が通過。`--dry-run` は、debug ビルドで約 49ms | `cargo test -p conductor-core --test graph_perf --test plan_perf`(ユニット U2、U3 の範囲を絞ったコマンドの結果。`test-results.md`) | build-and-test | Met |
| T4-nfr2-secrecy | NFR2、FR1.12、`project.md` の NEVER | 環境変数の値を、出力、エラーの表示、ステート、`Debug` の出力に出さない | 秘密の値の検査が、実バイナリのテスト(`binary_env`、`binary_graph`、`binary_dry_run`)と、型のテスト(`env_types`、`plan_types`)で、すべて通過 | `test-results.md` の 3、5、6、11、12、13 のコマンド | build-and-test | Met |
| T5-nfr3-cross-os | NFR3、`team.md`(他の OS は CI で確認) | Linux、macOS、Windows で、同じ挙動 | macOS で、全テストが通過。Windows と ubuntu は、このマシンでは未実行(`windows_demo.rs` は `cfg(windows)`) | CI の `test`(ubuntu、macos、windows)の行列。CI Pipeline(この計画の後の段階)が、受け持つ | ci-pipeline | N/A(CI Pipeline の仕事。ゲートで、未確認の事実として、人に知らせる) |
| T6-nfr4-quality | NFR4、`project.md` の ALWAYS(`cargo fmt`、`clippy -D warnings`、MSRV 1.100) | `fmt` 差分なし、`clippy` 警告なし、MSRV で通る | `fmt` 差分なし、`clippy` 警告なし。このマシンの `rustc` は 1.100.0 で、MSRV と同じ。全テストが通過 | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`(警告 0)、`rustc --version`(`test-results.md`) | build-and-test | Met |
| T7-nfr5-deps | NFR5、`project.md` の NEVER(新しい外部ライブラリを足さない) | `Cargo.toml`、`Cargo.lock` に差分なし | 差分 0 行 | `git diff 0674616 HEAD -- Cargo.toml Cargo.lock crates/*/Cargo.toml` が空 | build-and-test | Met |
| T8-nfr6-structure | NFR6、`project.md` の NEVER(`unsafe` は `platform/unix.rs` だけ、`cfg` は `platform` の外に置かない) | 新しいコードが、この規則を守る | `nfr_source`(cli 6 件、runtime 5 件)と `nfr_security`(core 5 件)が通過 | `cargo test -p conductor-cli --test nfr_source -p conductor-runtime --test nfr_source -p conductor-core --test nfr_security`(全体の実行でも通過) | build-and-test | Met |
| T9-nfr7-spec | NFR7 | 仕様書(`rust` と `language-agnostic`)を、実装に合わせて更新する | 両方の組の 13 ファイルずつを更新(26 ファイル、+2,927 行 / −240 行)。コミット `a893af4`(rust)と `5179659`(language-agnostic) | `git log`(`docs/spec` のコミット 2 件)、`git diff --stat` | build-and-test | Met |
| T10-docs-check | CI の `docs-check`(README の必須語句) | 7 つの必須語句が README に残る | すべて残っている | `grep -q -- '<語句>' README.md`(7 語句。`build-instructions.md`) | build-and-test | Met |

`Pending` の行は、残っていない。

## 準備の評価(Readiness)

- ビルドの準備: できている(ビルドと、`fmt`、`clippy`、全テストが、ローカルで通る)。
- テストの準備: できている(全体の 1,017 件と、ユニットごとの 15 のコマンドが通り、カバレッジは 97.94%)。
- デプロイの準備: この Build and Test の範囲の外。次の CI Pipeline 以降の段階と、別の依頼(版 0.9.0 への引き上げとリリース)が受け持つ。

## 既知の限界と残っていること

- **Windows と ubuntu、最小の Rust(1.100.0)の確認**: この環境では実行していない(T5、T2。CI Pipeline の仕事)。`windows_demo.rs`(場面 11〜14 のテスト)と `demo/windows/run-demo.cmd` は、特に、Windows の CI で、まだ一度も実行されていない。**マージ(リリース)の前に、GitHub Actions の `test`(windows と ubuntu)の結果が、緑であることを確かめる。**
- **版は 0.8.0 のまま**: 4 つのクレートの `Cargo.toml` の版と、`--version` の表示は、`0.8.0` のまま(版の引き上げとリリースは、別の依頼)。仕様書には、「v0.9.0 の機能を含む。部品の版は、版を上げるコミットまで 0.8.0」と明記した。版を上げたあとは、仕様書の 00、04 の 8 章、11 章の記述を、更新する。
- **揺れのあるテスト**: 計測つき(`cargo llvm-cov`)の実行では、マシンが忙しいとき、時間のかかる 2 つのテストが、30 秒の上限を超えて失敗することがある(失敗ではない揺れ。`test-results.md`)。
- **各ユニットのレビューの助言(受け入れた)**: `code-summary.md` の「既知の限界」に、ユニットごとに記録した(U1: 誤りの件数の上限なし、U2: 性能のテストが整形を含まない、U3: `status_at` の同等性のテスト、U4: パスの末尾が `/` のときの分類)。
- **仕様書の補足**: レーンの配置の規則は、実装から書き起こした(8,461 行の正解データの全例との照合は、していない。原本は正解データ)。新しい受け入れ基準(AC)の一部(AC-452、AC-496 など)は、参照実装に同じ形のテストがなく、規定から導いた(「派生」と明記済み)。
- **非機能要件の段階を省略した**ため、非機能要件は、ユニットごとの追跡(`cross-unit-traceability.md`)には載らない。代わりに、上のマトリクスで、直接確かめた。
