# Build and Test Summary

## 状態

- ビルド(安定版 1.97 と最小の 1.85)、リリースビルド、fmt、clippy、Windows 向けの型検査: すべて成功。
- テスト: 597 件成功、失敗 0(前回 585 件 + 新しい 12 件)。行カバレッジ 97.79%(下限 80%)。
- 実際のバイナリ: `validate` が、定義の誤りを 64 で拒否し、何も実行せず、`.state` に触れない。デモは完走。README の必須の語句はすべてある。
- 未実行: Linux・Windows でのテスト実行(既存の CI で行う)。

## 作成したテストの種類

| 種類 | 内容 |
|------|------|
| 単体 | 引数 2(`args_grammar.rs`)、使い方の文言 1(`report_messages.rs`) |
| 流れ(偽の `Runtime`) | `flow_validate.rs` 8(ステート・実行・シグナルを呼ばない、定義と command の誤りは 64、名前の一致・不一致・省略、既定は無出力で `-v` は 1 行、存在しないプログラムは 0) |
| 統合(実際のバイナリ) | `binary_basic.rs` 1(正しい定義 0、誤りの定義 64、`.state` が作られない) |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture | 行カバレッジ 80% 以上(全体) | 97.79% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions | 全テストが成功(既存を含む) | 597 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-03 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-04 | rust-version 1.85 | 1.85 でビルドとテストが成功 | 成功(597 件) | `cargo +1.85 build/test` | build-and-test | Met |
| BT-05 | requirements FR1.1〜FR1.8 | `validate` の要件ごとのテストが成功 | 12 件が成功 | `traceability.json`(code-generation) | build-and-test | Met |
| BT-06 | requirements FR1.2〜FR1.7(実際のバイナリ) | 0 / 64 / 68 の終了コード、`.state` に触れない | そのとおり | `test-results.md` の実際のバイナリでの確認 | build-and-test | Met |
| BT-07 | requirements FR2 | `PipelineDefinition` に `jobs` が残らず、動作が変わらない | 残りなし。改名だけの段階で 585 件が成功 | `pub jobs` の検索と `cargo test` | build-and-test | Met |
| BT-08 | requirements FR3.1・FR3.2 | README のタイトルと `validate` の記述、`docs-check` の語句 | タイトル変更済み、`validate` あり、7 語句すべてあり | README の 1 行目と検索 | build-and-test | Met |
| BT-09 | requirements NFR1 | 既存のサブコマンドの動作が変わらない | 既存のテストが成功のまま(期待値を更新したのは、使い方の文言と文法の 2 件だけ) | `cargo test --workspace --locked` | build-and-test | Met |
| BT-10 | requirements NFR4 | 版が 0.4.0 | `conductor 0.4.0` | `target/release/conductor --version` | build-and-test | Met |
| BT-11 | team.md(Linux・Windows のテストを CI で実行) | CI でテストが成功 | — | — | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が実行する) |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS。Linux・Windows は既存の CI)。
- deployment-ready: 次のステージで、版 0.4.0 のリリースを、人が決める。

## 既知の制限と残りの項目

- 英語の `-v` の 1 行は、ジョブが 1 個のとき `1 job`、それ以外は `N jobs`(計画の例に少し足した)。
- BT-11 は、push 後の CI の結果で確かめる。
