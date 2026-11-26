# Build and Test のまとめ: v0.10.0

出典: 各ユニットの `code-generation/code-generation-plan.md`(Testing Contract)、`code-summary.md`、`unit-test-instructions.md`、`nfr-requirements/`、`nfr-design/`、team.md の Testing Posture、project.md の Mandated・Forbidden。結果は `test-results.md`。

## ビルドの状態

- 成功。`conductor 0.10.0` のリリースのビルド、fmt、clippy(`-D warnings`)が通る。新しい依存は無い。

## テストの種類

| 種類 | 手順 |
|---|---|
| ビルド | `build-instructions.md` |
| 単体テスト(ユニットごと) | 各ユニットの `unit-test-instructions.md` |
| 結合テスト(流れ、実バイナリ、デモ、文書の一致) | `integration-test-instructions.md` |

- テストの方針は Standard。性能とセキュリティは、各ユニットの非機能のテスト(`*_perf.rs`、`nfr_*.rs`)が単体・結合テストの中で確かめるため、別の手順の文書は作っていない。

## カバレッジの期待

- プロジェクト全体で行カバレッジ 80% 以上(Testing Contract の scope floor、team.md、project.md の Mandated)。ユニットごとの下限は無い。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| T-COV | 各ユニットの Testing Contract(scope floor)、project.md の Mandated | 行カバレッジ 80% 以上(全体) | 98.18% | `test-results.md` のカバレッジ(`cargo llvm-cov ... --fail-under-lines 80`、終了コード 0) | build-and-test | Met |
| T-TESTS | 各ユニットの `unit-test-instructions.md`、Testing Contract | すべてのテストが通る | 1,429 件通過、失敗 0(3 回) | `test-results.md` の全体のテストと各ユニットのコマンド | build-and-test | Met |
| T-FMT | 各ユニットの NFR7.1、project.md の Mandated | `cargo fmt --check` が通る | 差分なし | `test-results.md` のビルド | build-and-test | Met |
| T-CLIPPY | 各ユニットの NFR7.1、project.md の Mandated | `clippy -D warnings` が通る | 警告なし | `test-results.md` のビルド | build-and-test | Met |
| T-DEPS | 各ユニットの NFR4.1、project.md の Forbidden | 新しい依存が無い | package 46 のまま | `test-results.md` の依存 | build-and-test | Met |
| T-SRC | 各ユニットの NFR5.1(`unsafe`・`cfg` の場所、ネットワーク無し) | `nfr_source.rs` が通る | 通過 | `cargo test --workspace --locked` | build-and-test | Met |
| T-SECRET | 各ユニットの NFR6.x(誤りの表示に値を出さない、ステートに時刻のほかを書かない) | `nfr_security.rs`、`nfr_content.rs`、各ユニットの表示のテストが通る | 通過 | 各ユニットのコマンド、全体のテスト | build-and-test | Met |
| T-PERF | 各ユニットの NFR1.3 など(`*_perf.rs` の上限) | 今の上限を変えずに通る | 通過 | `definition_perf`、`env_merge_perf`、`env_refs_perf` ほか(全体のテスト) | build-and-test | Met |
| T-COMPAT | 各ユニットの NFR1.1・NFR1.2(新しいキーを書かない定義は今と同じ) | 既存のテストが通る(設計で認めた期待の直しを除く) | 通過 | 全体のテスト | build-and-test | Met |
| T-DEMO | 各ユニットの NFR3.2 | `unix_demo.rs` が通る | 通過 | 各ユニットのコマンド | build-and-test | Met |
| T-DOCS | 各ユニットの FR8.1、BR の README(使い方の表の一致、必須の語句) | `tests::docs` が通る | 通過 | u1・u4・u7 のコマンド | build-and-test | Met |
| T-VERSION | u7 の BR5.2、FR8.2 | `--version` が 0.10.0 | `conductor 0.10.0` | `test-results.md` のビルド | build-and-test | Met |
| T-CI | 各ユニットの Testing Contract(「Run the selected tests in CI before merge.」)、NFR3.1(3 つの OS と MSRV、Windows だけのテスト) | CI(ubuntu・macos・windows、msrv-test、coverage)で通る | — | — | ci-pipeline | N/A(CI Pipeline, later in this plan, sets up CI to run these tests before merge) |

## 準備の状態

- ビルド: できる。
- テスト: 手元(macOS)ですべて通り、カバレッジの下限を満たす。
- 配る準備: CI(3 つの OS と最小の版)を通してから。タグ `v0.10.0` とリリースの下書きは、人が確かめて付ける(project.md の Mandated)。

## 残っていること

- Windows だけのテストと最小の版での実行は CI Pipeline の段階で確かめる。
- 各ユニットのレビューの Minor(例: README の「止まった」の注意の言い方、u5 の予約名の検査、u6 の `DURATION` の見出し)は、ふるまいを変えない注記・改善として残る。
- u1 の単体テストの手順の書き方を 1 か所直した(`test-results.md`)。
