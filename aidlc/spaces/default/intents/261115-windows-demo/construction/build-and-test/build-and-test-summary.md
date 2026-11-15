# Build and Test のまとめ

## 状態

- ビルド: 成功(release、`--locked`)。
- テスト: 566 件成功、失敗 0、無視 1(以前からのもの)。この作業のテスト 5 件を含む。
- 静的検査: `cargo fmt --check` と `cargo clippy -D warnings` は問題なし。
- 行カバレッジ: 96.43%(下限 80%)。
- 実機のデモ: Windows 11 で場面 0〜6 を最後まで確認。

## テストの種類

最小の方針(Minimal)のため、追加の手順書は作っていない(結合・性能・セキュリティ)。この作業のテストは、`crates/cli/tests/windows_demo.rs` の統合テスト 5 件(Windows 専用)。

## カバレッジの見込み

プロジェクト全体で行 80% 以上(チームの方針)。この作業は Rust のコードを変えていないので、測る行は増えない。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| TC-STRATEGY | code-generation-plan.md の Testing Contract(Minimal) | 要件ごとに 1 件の確認、構成要素ごとに正常系 1 件 | 統合テスト 5 件と実機の実行が、FR1.3、FR2.1、FR2.2 などに対応 | `test-results.md`、`cross-unit-traceability.md` | build-and-test | Met |
| TC-EXISTING-GREEN | 同上(express の下限) | 既存のテストが緑のまま | 566 件成功、失敗 0 | `cargo test --workspace --locked` の終了コード 0 | build-and-test | Met |
| TM-COVERAGE-80 | `team.md` の Testing Posture(行 80%) | 行カバレッジ 80% 以上 | 96.43% | `cargo llvm-cov --workspace --locked …` の終了コード 0 | build-and-test | Met |
| TM-CI-BEFORE-MERGE | Testing Contract の「CI で実行してからマージ」 | 計画に CI Pipeline がある場合のみ | 計画に CI Pipeline はない | — | — | N/A(この計画は CI Pipeline を実行しないため) |

## 準備の状態

- ビルドの準備: できている
- テストの準備: できている
- デプロイの準備: 該当は次の段階で決める(デプロイ用のパイプラインは、この計画ではスキップ)

## 既知の制限

- Windows での実行は、このマシン(Windows 11)だけ。CI の windows-2022 では、push 後に確かめる。
- Linux・macOS では、`windows_demo` は 0 件として動く。
- 前提 A1(待ち時間は整数の秒)と A2(Windows の `watch` に色なし)は、実行で確かめ済み。
