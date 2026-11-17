# Build and Test Summary

対象: 最新の Rust に合わせたリファクタリング。入力: `code-generation-plan.md`、`unit-test-instructions.md`、`code-summary.md`(`construction/code-generation/`)。

## 全体の状態

- ビルド: 成功。書式、静的解析(`-D warnings`)、依存の確認(`-sys` と C のビルド依存がない)もすべて問題なし。
- テスト: 705 件のうち 704 件が成功、失敗 0 件、無視 1 件(既存)。詳細は `test-results.md`。
- カバレッジ: 行 91.81%(下限 80% を満たす)。
- 前提: Rust 1.100.0 以降、`git`、`cargo-llvm-cov`(カバレッジの測定のみ)。手順は `build-instructions.md`。

## テストの種類

Test Strategy が Minimal のため、追加のテスト手順書(統合、性能、セキュリティ)は作らない。ユニット・統合テストは、既存のテストとコード生成で追加した `msrv_consistency.rs` が担う。

## カバレッジの見込み

- プロジェクト全体: 行 80% 以上(チームの方針と CI の `coverage` ジョブ)。実測 91.81%。
- Unit ごとの目標はない(この作業は Unit に分けていない)。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| TC-COVERAGE-FLOOR | `code-generation-plan.md` の Testing Contract と `team.md` の Testing Posture(行カバレッジ 80%) | 行カバレッジ 80% 以上 | 91.81% | `test-results.md` の「カバレッジ」(`cargo llvm-cov ... --fail-under-lines 80` が終了コード 0) | build-and-test | Met |
| TC-SUITE-GREEN | Testing Contract の scope floor(既存のテストが通ったままであること) | 失敗 0 件 | 失敗 0 件(成功 704、無視 1) | `test-results.md` の「テスト」 | build-and-test | Met |
| TC-CI-BEFORE-MERGE | Testing Contract(マージ前に CI でテストを実行する) | — | — | この計画は CI Pipeline(`ci-pipeline`)を実行しない(状態で SKIP) | — | N/A(the plan runs no CI Pipeline) |

NFR2(行カバレッジ 80%)は、`code-generation` の `traceability.json` で `Deferred`(この段階で確認する)としていた。この段階で測って満たした(TC-COVERAGE-FLOOR)。

## 準備の状態(readiness)

- build-ready: はい(ローカルの macOS で確認)。
- test-ready: はい(ローカルの macOS で確認)。
- deployment-ready: 次の条件つき。CI(Linux・Windows・macOS、`@1.100.0` の `build-check` と `msrv-test`)の結果をプッシュ後に確認する。

## 既知の制限・未確認の項目

- Linux と Windows のテスト、CI の `@1.100.0` のジョブは、このローカルでは実行していない。要件 NFR1(3 OS で通る)と NFR4(新しい MSRV での `build-check` と `msrv-test`)は、ローカル(macOS)の範囲で確認し、CI の範囲はプッシュ後の CI で確認する。これらは、この段階の Target Verification Matrix の対象(Testing Contract と NFR の設計成果物)には入らない。
- 変更前の基準が 701 件、変更後が 704 件(新規 3 件)。計画にあった「約 716 件」との差の理由は確かめていない。
- `cross-unit-traceability.md` に、`code-generation` の記録で `Deferred` のままの NFR2 が未カバーとして残る(上記のとおり、この段階で満たした)。
