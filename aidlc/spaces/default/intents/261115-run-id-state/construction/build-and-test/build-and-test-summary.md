# Build and Test Summary

## 状態

- ビルド(安定版 1.97 と最小の 1.85)、リリースビルド、fmt、clippy、Windows 向けの型検査: すべて成功。
- テスト: 663 件成功、失敗 0(前回 597 件 + 新しい 66 件)。行カバレッジ 97.72%(下限 80%)。
- 実際のバイナリ: run ID ごとに別のステートができ、並行しても互いの再開位置が変わらず、`reset --all` が対象だけを消す。デモは完走。README の必須の語句はすべてある。
- 未実行: Linux・Windows でのテスト実行(既存の CI で行う)。`reset --all` のシンボリックリンクのテストは、UNIX 専用。

## 作成したテストの種類

| 種類 | 内容 |
|------|------|
| 単体(core) | `RunId` の境界の入力、環境変数名の形、`peek_run_id_env`(壊れた入力でパニックしない)、定義の `run_id_env`(11 件) |
| 単体・統合(runtime) | パス、書いて読み戻す、run ID なしの形が不変、照合(完了のステートでも)、run ID ごとの独立、`reset --all` の対象判定(別の定義、壊れたもの、一時ファイル、無効な名前、シンボリックリンク、ディレクトリ、別ディレクトリ、大きすぎるもの、版違い、空ファイル、削除の失敗)(20 件) |
| 単体・流れ・統合(cli) | 引数と解決(7)、流れ(偽の `Runtime`。23)、実際のバイナリ(5) |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|-----------|--------|----------|--------|----------|--------------|---------|
| BT-01 | team.md Testing Posture | 行カバレッジ 80% 以上(全体) | 97.72% | `test-results.md` の `cargo llvm-cov` | build-and-test | Met |
| BT-02 | unit-test-instructions | 全テストが成功(既存を含む) | 663 件成功、失敗 0 | `cargo test --workspace --locked` | build-and-test | Met |
| BT-03 | team.md Code Style | `cargo fmt --check` と `cargo clippy -D warnings` が成功 | 両方 終了コード 0 | `test-results.md` | build-and-test | Met |
| BT-04 | rust-version 1.85 | 1.85 でビルドとテストが成功 | 成功(663 件) | `cargo +1.85 build/test` | build-and-test | Met |
| BT-05 | requirements FR1〜FR7 | 要件ごとのテストが成功 | core 11、runtime 20、cli 35 が成功 | `traceability.json`(code-generation) | build-and-test | Met |
| BT-06 | requirements FR1・FR3・FR5(実際のバイナリ) | run ID ごとの分離と再開、`reset --all` の対象判定、68・64・65・0 の終了コード | そのとおり | `test-results.md` の実際のバイナリでの確認 | build-and-test | Met |
| BT-07 | requirements NFR1 | run ID を使わないときの動作・ステートの形・メッセージが変わらない | 既存のテストが、追従以外の修正なしで成功(意味を変えたのは 2 か所のみ) | `cargo test --workspace --locked` | build-and-test | Met |
| BT-08 | requirements NFR2 | 検査済みの run ID だけがパスに使われ、`reset --all` が対象外を消さない | 境界のテストが成功 | `run_id.rs`・`run_id_state.rs` のテスト | build-and-test | Met |
| BT-09 | requirements FR7 | README と `--help` に記述があり、`docs-check` の語句が残る | あり。7 語句すべてあり | README の検索 | build-and-test | Met |
| BT-10 | requirements NFR4 | 版が 0.5.0 | `conductor 0.5.0` | `target/release/conductor --version` | build-and-test | Met |
| BT-11 | requirements NFR2(エラーの文に入力の値を含めない) | run ID の値が、固定の文に入らない | 使い方の誤りと照合の失敗の文には入らない。ステートの誤りの表示のパスには、run ID が含まれる(従来どおりパスを表示するため) | `run_id_state.rs`(`照合の失敗の文に run ID の値が入らない`)と `code-summary.md` | build-and-test | Met(例外を記録。run ID は検査済みの安全な文字だけ) |
| BT-12 | team.md(Linux・Windows のテストを CI で実行) | CI でテストが成功 | — | — | なし(この計画は CI Pipeline を持たない。既存の CI が push のたびに実行する) | N/A(この計画が CI Pipeline を実行しないため。既存の `.github/workflows/ci.yml` が実行する) |

## 準備の評価

- build-ready: 可。test-ready: 可(macOS。Linux・Windows は既存の CI)。
- deployment-ready: 次のステージで、版 0.5.0 のリリースを、人が決める。

## 既知の制限と残りの項目

- `reset --all` は、「通常のファイルか」の確認(`symlink_metadata`)と、ファイルを開く動作の間の入れ替え(TOCTOU)を、防いでいない(標準ライブラリだけでは、開くときのシンボリックリンク拒否ができない)。同じディレクトリに書ける人がいる前提の攻撃で、記録しておく。
- 中身に `run_id` がない、または値が違うファイルは、壊れていなくても消さず、警告にも出さない(別の定義のステートでありうるため)。長さ 0 のファイルも同様。
- 別の定義 `x.toml.a`(run ID `b`)と、定義 `x.toml`(run ID `a.b`)は、同じファイル名 `x.toml.a.b.state` になりうる。中身の `run_id` の照合で、読むときに食い違いが検出される(65)。
- 大文字小文字を区別しないファイルシステムでは、`A` と `a` が同じファイルになる。中身の照合で検出される(README に、区別しない運用を書いた)。
- BT-12 は、push 後の CI の結果で確かめる。
