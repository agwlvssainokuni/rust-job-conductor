# Build and Test Summary

## 状態

- ビルド: 成功(安定版のデバッグとリリース、Rust 1.85)。
- 既存の検査: fmt、clippy(警告 0)、`cargo test --workspace`(701 件成功、失敗 0、無視 1)、すべて成功。
- カバレッジ: 行 97.39%(下限 80%)。
- 実機の確認: 完了後にステートが残ること、`watch` の完了の行(英語・日本語)、`--remove-on-complete` と `[pipeline].remove_on_complete`、68・64 の誤り、run ID、`reset`・`reset --all`、`run` の単独の完了、`conductor 0.7.0`、すべて期待どおり(`test-results.md`)。
- 未確認: Windows のテストとデモの実行、Linux・Windows の `cargo test`(CI で実行する)。

## 作成したテストの種類

Minimal の方針。追加のテスト指示のファイル(integration・performance・security)は、作らない。要件ごとのテストは、code-generation で足した(`code-summary.md`)。

## カバレッジの目標

プロジェクト全体で行 80% 以上(team.md)。実測 97.39%。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Code Style・requirements NFR3 | fmt・clippy(警告 0)が成功 | 成功 | `test-results.md` | build-and-test | Met |
| BT-02 | requirements NFR3・NFR5 | 既存と新規のテストがすべて成功 | 701 件成功、失敗 0、無視 1 | `test-results.md` | build-and-test | Met |
| BT-03 | team.md Testing Posture・requirements NFR3 | 行カバレッジ 80% 以上 | 97.39% | `test-results.md`(`cargo llvm-cov`) | build-and-test | Met |
| BT-04 | requirements NFR3 | Rust 1.85 のビルドが成功 | 成功 | `test-results.md` | build-and-test | Met |
| BT-05 | requirements NFR4 | docs-check の 7 語句が README に残る | すべてある | `test-results.md` | build-and-test | Met |
| BT-06 | requirements FR1.1・FR1.3 | 完了後にステートが残り、再実行は先頭から始まる | `pipeline`・`run` の両方で、そのとおり | `test-results.md`(実機) | build-and-test | Met |
| BT-07 | requirements FR2.1〜FR2.4・FR2.6 | オプション・定義で削除される。型の誤りは 64。環境変数は見ない | そのとおり | `test-results.md`(実機) | build-and-test | Met |
| BT-08 | requirements FR2.5 | `watch`・`reset`・`validate`・`convert` で 68 | そのとおり | `test-results.md`(実機) | build-and-test | Met |
| BT-09 | requirements FR3.1〜FR3.3 | 表の下に完了の行(英語 `completed`、日本語 `完了`)。run ID あり・なし | そのとおり | `test-results.md`(実機) | build-and-test | Met |
| BT-10 | requirements FR4.1 | `reset`・`reset --all` が完了のステートも消す | そのとおり | `test-results.md`(実機) | build-and-test | Met |
| BT-11 | requirements FR6.1 | `conductor --version` が 0.7.0 | `conductor 0.7.0` | `test-results.md`(実機) | build-and-test | Met |
| BT-12 | requirements FR5.1・FR5.2 | README とデモが新しい挙動に合う | README のクイックスタートは出力が一致(code-generation で実行)。UNIX のデモはテストが成功 | `code-summary.md`、`unix_demo` のテスト | build-and-test | Met |
| BT-13 | requirements FR5.2・FR5.3(Windows) | `windows_demo.rs` と `run-demo.cmd` が、Windows で成功 | 未実行 | — | なし(CI の Windows と、人の確認) | Unverified |
| BT-14 | team.md(Linux・Windows のテストを CI で実行) | CI の 3 OS でテストが成功 | — | — | なし(この計画は CI Pipeline を持たない。既存の `ci.yml` が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が実行する) |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS では全件成功)。deployment-ready: リリースはしない(版は 0.7.0 に上げた。タグとリリースは利用者が行う)。

## 既知の制限と残りの項目

- **BT-13 は未確認**: `windows_demo.rs` と `demo\windows\run-demo.cmd` は、Windows でしか動かない。push 後の CI の `test`(windows-latest)の結果で、`windows_demo.rs` を確かめる。`run-demo.cmd` の実行は、Windows の実機で、人が確かめる(これまでと同じ)。
- BT-14: push 後の CI の結果で確かめる。
- 範囲外として決めたもの(掃除機能、完了済みの run を何もせず終える仕様、`--keep-on-complete`、環境変数での指定)は、含めていない。
