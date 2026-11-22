# テストの結果(Test Results)

macOS(主な開発環境)、rustc 1.100.0、`7089a03` の時点。

| 検査 | 結果 |
|------|------|
| `cargo fmt --all -- --check` | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 警告なし |
| `cargo test --locked --workspace` | 1,142 通過、0 失敗、1 無視 |
| `cargo llvm-cov`(CI と同じ引数) | 行 98.23%、関数 98.29%、領域 97.88%(下限 80%) |
| `conductor --version` | `conductor 0.9.6` |

## 件数の移り変わり

| 時点 | 通過 | 基準からの増分 |
|------|------|----------------|
| 基準(`d3e7fac`) | 1,137 | — |
| Step 1・2 のあと | 1,137 | +0 |
| 段 1 のあと | 1,138 | +1(N1) |
| 段 2 のあと | 1,139 | +2(N2) |
| 段 3 のあと | 1,142 | +5(N3、N4、N5) |

どの段も見込みと一致した。

## 段 3 の新しいテストの前後

| テスト | 直す前(`c49ff0b`) | 直した後 |
|--------|----------------------|----------|
| N3(読む権限のない 1 MiB 超のファイル) | 落ちた(`TooLarge`) | 通った(`Unreadable`) |
| N4(上限超過は UTF-8 でないより先) | 通った | 通った |
| N5(名前付きパイプ) | 通った(止まらない) | 通った(止まらない) |

## テストの変更

- 既存のテストの変更は T1〜T20 の範囲だけ。T19・T20 は実装中に見つかった設計の一覧の漏れで、ユーザーが 2026-11-23 に追加を認めた。
- 期待する出力の文字列の変更は T18 の 1 行(`conductor 0.9.6`)だけ。
- 新しいテストは N1〜N5 の 5 件。

## 未確認

- Linux、Windows、MSRV のジョブは push の後の CI で確かめる。特に Windows のディレクトリの扱いと、Linux での N3・N5。

上流: `construction/code-generation/unit-test-instructions.md`(SM1〜SM5 の確かめ方)、`construction/code-generation/code-generation-plan.md`。
