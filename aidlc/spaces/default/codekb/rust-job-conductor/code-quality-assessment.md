# Code Quality Assessment

## テスト

- `crates/{core,runtime,converter,cli}/tests/` に統合テスト(約 11,600 行)、`#[test]` は全体で約 716
- 内容: 流れ、終了コード、シグナル(実プロセス)、NFR(性能、堅牢性、セキュリティ、ソースの検査)、往復変換、バイナリの実行
- Windows の Ctrl+C は手動の確認表(`crates/cli/tests/WINDOWS_CTRL_C_CHECKLIST.md`)
- カバレッジ: CI の `cargo llvm-cov --fail-under-lines 80`(test-helper を除外)。実測値は未取得(走査は実行しない)

## 静的解析

- `cargo fmt --check`、`cargo clippy -D warnings`(`ci.yml` lint)。設定ファイルなし
- `unsafe`: runtime の `platform/unix.rs:72` の 1 か所。cli は `forbid(unsafe_code)`
- `allow(dead_code)` はテストの共通モジュールのみ。TODO / FIXME なし

## CI/CD

- `ci.yml`(7 ジョブ)、`audit.yml`、`release.yml`(詳細は technology-stack.md)
- リリースはタグ `v*` から、CI 再利用 → 4 ターゲットのビルド → ドラフトのリリース

## ドキュメント

README(日本語、詳細)、各 `lib.rs` の文書コメント、`demo/README.md`。`docs-check` が README の必須語を検査する。

## 版に関する事実の観測(判断はしない)

1. 最小の Rust 版 1.85 が 4 か所に別々に書かれている: `Cargo.toml` の `rust-version`、`ci.yml` の `@1.85.0`(build-check、msrv-test の 2 か所)、README.md:84
2. 多くの CI ジョブは `@stable`(浮動)。build-check と msrv-test のみ固定
3. ランナーは `ubuntu-24.04`、`windows-2022` が固定で、`macos-latest` が浮動(ci.yml、release.yml)
4. アクションは `checkout@v6`、`upload-artifact@v7`、`download-artifact@v8`、`install-action@v2`(メジャー版のタグ指定)
5. `rust-toolchain` ファイルなし。ローカルのツールチェーンは 1.100.0(観測)で、MSRV 1.85 と差がある
6. `Cargo.lock` は version 4(Cargo 1.78 以降の形式)。edition 2024 は 1.85 で安定化されたもの
7. 依存の自動更新の設定なし
8. 「LTS」を名乗る設定・記述は、リポジトリ内に見当たらない
