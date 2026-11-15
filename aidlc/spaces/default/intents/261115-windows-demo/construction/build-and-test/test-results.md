# Test Results

実行日: 2026-11-15。環境: Windows 11(x86_64)、Rust 安定版。

## ビルドと静的検査

| 命令 | 終了コード | 結果 |
|------|-----------|------|
| `cargo build --release --locked --bin conductor` | 0 | 成功。`conductor --version` は `conductor 0.3.0` |
| `cargo fmt --all -- --check` | 0 | 差分なし |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 警告 0 |

## テスト

命令: `cargo test --workspace --locked`。終了コード 0。合計 566 件成功、失敗 0、無視 1(converter の以前からのもの)。

- この作業のテスト: `cargo test -p conductor-cli --test windows_demo --locked` は 5 件成功(定義の内容、場面 1、場面 2、場面 3・4、場面 5・6)。
- 前回(macOS で 585 件)より少ないのは、OS 限定のテスト(UNIX のシグナルなど)が、Windows では 0 件になるため。この作業のテスト 5 件は Windows 専用で、UNIX では 0 件。

## カバレッジ

命令: `cargo llvm-cov --workspace --locked --ignore-filename-regex "bin[\\/]conductor-(cli-)?test-helper" --summary-only`。終了コード 0。行 96.43%(6,154 行のうち未実行 220)、関数 94.92%、リージョン 96.26%。下限 80% を満たす。測った環境は Windows で、CI の Linux の値とは、OS 限定のコードの分だけ違う。

## 実機でのデモ(Code Generation の中で実施)

`demo\windows\run-demo.cmd` を最後まで実行。場面 1〜6 の終了コードは、0、0、2(`watch` に load のエラー)、完了(`watch` は空)、0、0。`DEMO_PAUSE=1`(Enter を 6 回)と `DEMO_LANG=en_US` でも完了。リポジトリには、意図したファイル以外は残らなかった。

## 実施しなかったこと

- CI(GitHub Actions)の実行。この作業は、まだ push していない。
- Linux・macOS での実行(この作業の変更は `demo/windows/` と Windows 専用のテストだけで、UNIX のデモは変えていない)。
