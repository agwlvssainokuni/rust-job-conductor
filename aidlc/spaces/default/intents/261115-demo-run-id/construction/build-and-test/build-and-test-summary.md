# Build and Test Summary

## 状態

- ビルド(安定版 1.97 と最小の 1.85)、fmt、clippy、Windows 向けの型検査: すべて成功。
- テスト: 666 件成功、失敗 0(前回 663 件 + `unix_demo.rs` の 3 件)。行カバレッジ 97.74%(下限 80%)。
- UNIX 版デモ: 日本語・英語・利用者の環境を設定した状態の 3 通りとも完走し、場面 7〜9 の終了コードと表示を確かめた。
- 未確認: Windows 版 `run-demo.cmd` の Windows 実機での実行(人の確認の項目)。`windows_demo.rs` の実行(CI の Windows)。

## 作成したテストの種類

| 種類 | 内容 |
|------|------|
| 統合(UNIX、`unix_demo.rs`) | デモが 0 で完走し場面 0〜9 と要点の行が出る(3 件)。利用者の環境に左右されない。一時ディレクトリが残らない |
| 統合(Windows、`windows_demo.rs`) | 場面 7〜9 の検証 6 件(`validate`、run ID の分離と再開、`--run-id` の優先と `""` の固定、`reset --run-id` と `reset --all`)。`Demo::run` が `CONDUCTOR_RUN_ID` と `DEMO_RUN_ID` を外す。実行は CI の Windows |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture | 行カバレッジ 80% 以上(全体) | 97.74% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions | 全テストが成功(既存を含む) | 666 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-03 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-04 | rust-version 1.85 | 1.85 でビルドとテストが成功 | 成功(666 件) | `cargo +1.85 build/test` | build-and-test | Met |
| BT-05 | requirements FR1・FR3・FR4(UNIX 版デモ) | 場面 7〜9 が動き、終了コードと表示が仕様どおり。環境の影響を受けない | 3 通りとも完走。表示と終了コードを確認 | `test-results.md` のデモの実行と、`unix_demo.rs` | build-and-test | Met |
| BT-06 | requirements FR6.2 | UNIX 版のデモの自動テストが成功 | 3 件が成功 | `cargo test -p conductor-cli --test unix_demo` | build-and-test | Met |
| BT-07 | requirements FR6.1(Windows 版の検証) | `windows_demo.rs` の新しい検証が型検査を通り、CI で成功する | 型検査は成功。実行は CI の Windows | `cargo check --target x86_64-pc-windows-msvc` | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` の `test (windows-2022)` が実行する) |
| BT-08 | requirements NFR1 | `conductor` 本体・ルートの README・版・`.github/` が変わらない | 変更なし | `git diff 1d8a461 HEAD` | build-and-test | Met |
| BT-09 | requirements NFR2・NFR3 | 既存の場面とテストが成功のまま。`docs-check` の語句が残る | 既存のテストが成功。7 語句すべてあり | `cargo test`、README の検索 | build-and-test | Met |
| BT-10 | requirements NFR4(Windows 版の形式) | `run-demo.cmd` が UTF-8・CRLF | 230 行すべて CRLF | `file` と `grep -c` | build-and-test | Met |
| BT-11 | requirements FR2(Windows 版デモ) | Windows の実機で、場面 0〜9 が最後まで動く | 未実行(macOS では実行できず、CI でも実行されない) | `run-demo.cmd` を既存の場面の書き方に合わせて読み直した | なし(人の確認) | Unverified |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS。Linux・Windows は既存の CI)。
- deployment-ready: リリースはしない(版は 0.5.0 のまま。デモだけの変更)。

## 既知の制限と残りの項目

- **BT-11(Windows 版デモの実機での実行)は未確認:** 要件の承認のとき、「`run-demo.cmd` は CI で実行されないため、実機での確認があなたの作業」と合意した。確認の点: `bad.toml` を `echo` で作ること、`--run-id ""` が空の引数として渡ること、`dir /b` が一致なしで `errorlevel 1` を返すことへの依存。
- UNIX 版の表示では、`--run-id ""` の空の引数が見えず `--run-id  nightly` と出る(動作は正しい)。
- BT-07 は、push 後の CI の結果で確かめる。
