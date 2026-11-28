# ビルドとテストのまとめ: v0.10.4

出典:
- `construction/code-generation/` の `code-generation-plan.md`(Testing Contract、R-01・R-02、付録 A)、`unit-test-instructions.md`、`code-summary.md`
- 要件: `requirements.md` の FR1〜FR5、NFR1〜NFR6
- project.md の Mandated・Forbidden
- `build-instructions.md`、`test-results.md`

## 全体の状態と前提

- ビルド: 成功した。
- テスト: 手元で 1,449 passed、0 failed、1 ignored だった。CI 46131106543 では、12 のジョブがすべて成功した。
- 3 つの大きなファイルを、10 の子のファイルに分けた。あわせて、`build_command` と `render` を関数に分けた。動き、公開の範囲、依存は変えていない。
- Test Strategy は Minimal、scope は refactor。そのため、結合・性能・セキュリティのテストの手順の文書は作らない。

## テストの種類

| 種類 | この作業で足したもの |
|---|---|
| 既存のテスト | 無い。名前と期待の値を変えずに通る(版の期待と人の判断 A を除く) |
| 一覧の直し(FR4.4) | `with_all_events` に 2 つの変種を足した。`#[test]` の数は変わらない |

## カバレッジ

- プロジェクト全体の行カバレッジは 98.25%(下限は 80%)。Units は無い。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| T-COV | project.md Mandated、NFR3 | 行カバレッジ 80% 以上 | 98.25% | `test-results.md`、CI の `coverage` | build-and-test | Met |
| T-SUITE | Testing Contract の scope_floor、NFR1 | 既存のテストがすべて、数を変えずに通る | 1,449 passed、`#[test]` 1,472 | `test-results.md` | build-and-test | Met |
| T-SPLIT | FR1.1・FR2.1・FR3.1、R-01 | 移す項目が `mod.rs` に無い | 3 つとも満たす(411・341・195 行) | `test-results.md`、`code-summary.md` | build-and-test | Met |
| T-DISPATCH | FR1.2・FR3.2 | `render` は網羅の振り分け、`build_command` は手順の呼び出しだけ | `render` 64 行、`build_command` 10 行 | `code-summary.md`、コード生成のレビュー | build-and-test | Met |
| T-LAZY | FR1.3 | 言語と色の判定は後回し | `report_messages.rs` の 694・726 行のテストが通る | `test-results.md` | build-and-test | Met |
| T-ORDER | NFR1 | 誤りの順が変わらない | `args_grammar` の 69 テストと core の定義のテストが通る | `test-results.md` | build-and-test | Met |
| T-SURFACE | FR1.4・FR2.3・FR3.3、NFR2 | 公開の範囲と道筋が変わらない。テストのための `pub` が無い | 呼ぶ側の `use` は変えていない。`env.rs`・`retry.rs` の差分は 0 | コード生成のレビュー | build-and-test | Met |
| T-SRC | NFR5 | ソースの検査が通る | `nfr_source` が 3 つの OS で通る | CI 46131106543 | build-and-test | Met |
| T-LINT | project.md Mandated | fmt と clippy が通る | どちらも exit 0、CI の `lint` も成功 | `test-results.md` | build-and-test | Met |
| T-DEP | NFR4 | 依存を足さない | package 46 | `test-results.md` | build-and-test | Met |
| T-DOC | FR4.1〜FR4.3 | 仕様書、文書コメント、team.md の 1 行、古い名前を直す | 直した(2c284c3、c293fa3、bdde159 ほか) | コード生成のレビュー | build-and-test | Met |
| T-MSRV | project.md Mandated | 1.100 でビルドとテストが通る | `msrv-test` が成功 | CI 46131106543 | build-and-test | Met |
| T-VER | FR5.1 | 版が 0.10.4 で、ad775bc と同じ 22 ファイル | `conductor 0.10.4`、22 ファイル | 3852f7a | build-and-test | Met |
| T-CI | project.md Mandated | CI のすべてのジョブが通る | 12 のジョブが成功 | CI 46131106543 | build-and-test | Met |

## 準備の度合い

- build-ready: はい
- test-ready: はい(手元と CI)
- deployment-ready: はい(版は 0.10.4)

## 残っていること・限り

- 手元のカバレッジの計測で、1 回目だけ `runner_log` の `br4_1_…` が落ちた。走らせ直すと通り、CI の `coverage` も成功した。
  - 原因: 出力を写す処理を待つ上限(`DRAIN_LIMIT` 2 秒)を、計測の負荷で超えたと見ている。
  - v0.10.3 に続いて 2 度目なので、次の整理の候補(リファクタリングの候補の A-3)とする。
- `args/mod.rs` は、参考値の 340 行を 1 行超えた(341 行)。
- 後の候補として残すもの:
  - 単体テストの置き場所の決まり
  - `report/messages.rs` の分割
  - ほかの長い関数(`run_job_with_retry`、`Walker::deliver`、`run_pipeline`)
