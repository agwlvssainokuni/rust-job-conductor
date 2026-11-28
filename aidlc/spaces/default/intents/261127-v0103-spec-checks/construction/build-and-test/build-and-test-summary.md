# ビルドとテストのまとめ: v0.10.3

出典: `construction/code-generation/code-generation-plan.md`(Testing Contract、付録 A・B)、`unit-test-instructions.md`、`code-summary.md`、要件(`requirements.md` の FR1〜FR4、NFR1〜NFR5)、project.md の Mandated・Forbidden、`build-instructions.md`、`test-results.md`。

## 全体の状態と前提

- ビルド: 成功した。
- テスト: 手元で 1,449 passed、0 failed、1 ignored だった。CI 46092396041 では 12 のジョブがすべて成功した。
- この版で変えたもの:
  - 仕様書の小さな粗の直し(A-1)と `team.md` の古い記述の書き換え(A-2)
  - 仕様書の数を突き合わせるテストの追加(A-3、`crates/cli/tests/spec_numbers.rs`)
  - 版
- 本番のコードの動きは変わらない。
- Test Strategy は Minimal で、scope は refactor。そのため、結合・性能・セキュリティのテストの手順の文書は作らない。

## テストの種類

| 種類 | この作業で足したもの |
|---|---|
| 統合テスト(文書とソースの突き合わせ) | `spec_numbers.rs` の 12 個(数ごとに 1 つ) |
| 効くことの確かめ | 3 通りに一時的にずらし、毎回落ちることを確かめた(コミットしない) |
| 既存のテスト | 直したのはテスト名 1 つ(`env_types.rs`)。すべて通る |

## カバレッジ

- プロジェクト全体の行カバレッジは 98.22% で、下限は 80%。Units は無い。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| T-COV | project.md Mandated、NFR3 | 行カバレッジ 80% 以上 | 98.22% | `test-results.md`、CI の `coverage` | build-and-test | Met |
| T-SUITE | Testing Contract の scope_floor、NFR3 | 既存のテストがすべて通る | 1,449 passed、0 failed | `test-results.md` | build-and-test | Met |
| T-SPEC-NUM | FR3.1〜FR3.3 | 12 の数が仕様書と実装で一致する | 12 passed | `test-results.md` | build-and-test | Met |
| T-SPEC-BITE | FR3.4 | ずらすと落ちる(3 通り) | 3 通りとも落ちた | `code-summary.md` | build-and-test | Met |
| T-OS | NFR5、FR3.4 (3) | 3 つの OS で通る | ubuntu、macos、windows で成功 | CI 46092396041 | build-and-test | Met |
| T-LINT | project.md Mandated | fmt と clippy が通る | どちらも exit 0。CI の `lint` も成功 | `test-results.md` | build-and-test | Met |
| T-DEP | project.md Forbidden、NFR4 | 依存を足さない | package 46 | `test-results.md` | build-and-test | Met |
| T-NOCODE | NFR1、NFR2 | `crates/*/src/` は版の外を変えない | 一致 | コード生成のレビュー | build-and-test | Met |
| T-A1 | FR1.1〜FR1.4 | テスト名の書き方、AC-430、性能の表、env_types のテスト名が直る | 直った(AC-214 も直した) | `code-summary.md`、d646030、7b7db42 | build-and-test | Met |
| T-TEAM | FR2.1〜FR2.3 | `team.md` の差分が付録 B の行だけ | 一致 | 91edf57、b299046、コード生成のレビュー | build-and-test | Met |
| T-MSRV | project.md Mandated | 1.100 でビルドとテストが通る | `msrv-test` 成功 | CI 46092396041 | build-and-test | Met |
| T-VER | FR4.1 | 版が 0.10.3 で、10ce84e と同じ 22 ファイル | `conductor 0.10.3`、22 ファイル | `test-results.md`、ad775bc | build-and-test | Met |
| T-CI | project.md Mandated | CI のすべてのジョブが通る | 12 のジョブが成功 | CI 46092396041 | build-and-test | Met |

## 準備の度合い

- build-ready: はい
- test-ready: はい(手元と CI)
- deployment-ready: はい(版は 0.10.3)

## 残っていること・限り

- 手元のカバレッジの計測の 1 回目で、`runner_log` のテストが 1 つ落ちた。7 回走らせ直しても再現しなかった。時間に敏感なテストが計測の負荷で落ちたと見ている。CI の `coverage` は成功した。
- 受け入れた危険として残るもの:
  - コード生成のレビューの R-03: 試行の理由の数は、`///` に「試行」と書いたものだけを数える。
  - 全体の探しで検査されない行: 別の話の注記がある行。
- 後の候補として残すもの:
  - `team.md` の 60 行目の `#[allow]` の記述
  - 数の検査の範囲の外の数(使い方の誤り 8 種など)
