# Build and Test Summary

## 状態

- ビルド(安定版 1.97 と最小の 1.85)、リリースビルド、fmt、clippy、Windows 向けの型検査: すべて成功。
- テスト: 585 件成功、失敗 0(前回 580 件 + 新しい 5 件)。行カバレッジ 97.80%(下限 80%)。
- 実際のバイナリ: ステートが `[[job]]` で書かれ、`watch` が読め、古い `[[jobs]]` は 65 で拒否される。デモは完走。
- 未実行: Linux・Windows でのテスト実行(既存の CI で行う)。

## 作成したテストの種類

| 種類 | 内容 |
|------|------|
| 単体 | reader 2(`[[job]]` が読める、`[[jobs]]` は `corrupt`)、error 1(文面に「jobs」がない) |
| 統合 | `state_read.rs` 2(version 1 の `[[job]]` が読める、`[[jobs]]` は `corrupt` で拒否)。書き出しは、既存の `state_write.rs` を強化(`[[job]]` があり `jobs` がないことを確かめる) |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture | 行カバレッジ 80% 以上(全体) | 97.80% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions | 全テストが成功(既存のテストを含む) | 585 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-03 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-04 | rust-version 1.85 | 1.85 でビルドとテストが成功 | 成功(585 件) | `cargo +1.85 build/test` | build-and-test | Met |
| BT-05 | requirements FR1〜FR4 | 要件ごとのテストが成功 | runtime 5 件が成功 | `traceability.json`(code-generation) | build-and-test | Met |
| BT-06 | requirements FR1〜FR2(実際のバイナリ) | `[[job]]` で書かれ、`[[jobs]]` は 65 で拒否される | そのとおり | `test-results.md` の実際のバイナリでの確認 | build-and-test | Met |
| BT-07 | requirements NFR1 | 変更が、ステートのキー名に限られる | runtime のステートの読み書きと文面、テスト、版だけ。core・cli・converter の `src/` は変更なし | `git diff --stat` | build-and-test | Met |
| BT-08 | requirements NFR3 | 版が 0.3.0 | `conductor 0.3.0` | `target/release/conductor --version` | build-and-test | Met |
| BT-09 | requirements FR5 | README・デモにステートのキーの記述が残っていない | 記述なし(変更不要) | README・デモの検索 | build-and-test | Met |
| BT-10 | team.md(Linux・Windows のテストを CI で実行) | CI でテストが成功 | — | — | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が実行する) |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS。Linux・Windows は既存の CI)。
- deployment-ready: 次のステージで、版 0.3.0 のリリースを、人が決める。

## 既知の制限と残りの項目

- v0.3.0 は、ステートのキーが変わる破壊的変更。止まったままの古い `.state`(`[[jobs]]`)は、「壊れたステート」(65)になる。`conductor reset` で消してやり直す(互換なしは、依頼で決定済み)。
- BT-10 は、push 後の CI の結果で確かめる。
