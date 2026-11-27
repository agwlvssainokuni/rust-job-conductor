# ビルドとテストのまとめ: v0.10.2

出典: `construction/code-generation/code-generation-plan.md`(Testing Contract、付録 A・B)、`unit-test-instructions.md`、`code-summary.md`、要件(`requirements.md` の FR1〜FR5、NFR1〜NFR5)、project.md の Mandated・Forbidden、`build-instructions.md`、`test-results.md`。

## 全体の状態と前提

- ビルド: 成功。テスト: 手元で 1,437 passed、0 failed。CI 46079873817 の 12 のジョブがすべて成功。
- この版の変更は文書(`docs/spec/rust/` 13 章、`docs/spec/language-agnostic/` 13 章、`team.md` の 1 行)と、`crates/runtime/src/lib.rs` のドキュメントコメントと版だけ。コードの動きは変わらない。
- Test Strategy は Minimal、scope は refactor。追加のテストの手順の文書(結合・性能・セキュリティ)は作らない。

## テストの種類

| 種類 | この作業で足したもの |
|---|---|
| 単体・統合・実バイナリのテスト | なし(既存がすべて通る) |
| 文書のテスト(`src/tests/docs.rs`) | なし(既存が通る) |
| 仕様書の確かめ | 計画の付録 B の `grep` と数(テストではない) |

## カバレッジ

- プロジェクト全体の行カバレッジ: 98.22%(下限 80%)。Units は無い。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| T-COV | project.md Mandated | 行カバレッジ 80% 以上 | 98.22% | `test-results.md`(`cargo llvm-cov … --fail-under-lines 80` が exit 0)、CI の `coverage` | build-and-test | Met |
| T-SUITE | Testing Contract の scope_floor、NFR1 | 既存のテストがすべて通る | 1,437 passed、0 failed | `test-results.md` | build-and-test | Met |
| T-UNIT | `unit-test-instructions.md` | 4 つのコマンドがすべて通る | すべて exit 0 | `test-results.md` | build-and-test | Met |
| T-LINT | project.md Mandated(NFR4) | fmt と clippy が通る | どちらも exit 0 | `test-results.md` | build-and-test | Met |
| T-DEP | project.md Forbidden(NFR5) | 依存を足さない | package 46 | `test-results.md` | build-and-test | Met |
| T-NOCODE | NFR1、BR6.1 | `crates/` の差分はドキュメントコメントと版だけ | 一致(コード生成のレビューで `git diff` を確かめた) | `code-summary.md`、コード生成のレビュー | build-and-test | Met |
| T-SPEC-COUNT | NFR3、BR1.3、BR2.4 | 「33 種・13 種」が履歴の行だけ | 3 行(rust 12:105・107、language-agnostic 12:129) | `test-results.md` の付録 B | build-and-test | Met |
| T-SPEC-CONTRA | NFR3、FR1.12 | 実装と逆の語が「残してよい記述」の行だけ | 7 行、すべて残してよい記述 | `test-results.md` の付録 B | build-and-test | Met |
| T-SPEC-AC | FR2.2、BR2.2、BR2.3 | AC-1〜573 に欠番・重複が無い。書き換えた既存の AC に「(v0.10.0 で変更)」 | 欠番・重複なし、17 か所 | `code-summary.md`、コード生成のレビュー | build-and-test | Met |
| T-TEAM | FR3、BR3.1 | `team.md` の差分が 1 行で、数が実装と一致 | 1 行(36・16・26・2、12・4、1,460) | `git show c005e87 --stat`、`code-summary.md` | build-and-test | Met |
| T-VER | FR5.1、BR5.1 | 版が 0.10.2 で、152c4a1 と同じ 22 ファイル | `conductor 0.10.2`、22 ファイル | `test-results.md`、`code-summary.md` | build-and-test | Met |
| T-CI | project.md Mandated、NFR4 | CI のすべてのジョブが通る | 12 のジョブがすべて成功 | CI 46079873817 | build-and-test | Met |

## 準備の度合い

- build-ready: はい
- test-ready: はい(手元と CI)
- deployment-ready: はい(版は 0.10.2)

## 残っていること・限り

- 仕様書の正しさは、付録 B の `grep` と数、コード生成のレビューの抜き取りで確かめた。仕様書を読む自動の検査は無い(要件の前提のとおり)。
- コード生成のレビューの Minor の 2 件(6342a8d に監査ファイルが紛れたこと、11 章の根拠のテスト名の `_` の書き方の不揃い)は、受け入れた危険として残る。
- 範囲の外として次に回したもの: ほかの 5 つの言語版の仕様書、`team.md` の他の古い記述(タグの範囲など)。
