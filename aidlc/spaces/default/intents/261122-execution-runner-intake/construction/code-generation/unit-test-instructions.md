# テストの手順(Unit Test Instructions): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,137 通過、1 無視、0 失敗)
cargo test --locked --workspace

# 段 1
cargo test --locked -p conductor-core
cargo test --locked -p conductor-runtime --test runner_basic --test runner_signal --test runner_env --test nfr_robust --test nfr_content

# 段 2
cargo test --locked -p conductor-runtime
cargo test --locked -p conductor-cli

# 段 3
cargo test --locked -p conductor-core --lib definition::intake
cargo test --locked -p conductor-cli --test binary_basic
```

(ファイル名やモジュール名が違うときは、実際の名前に合わせる。)

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| SM1(段 1) | N1、T1〜T12 | 分類・リトライ・終了コード・表示が同じ。`JobExecution` の古い項目が残らない。件数 1,138 |
| SM2(段 2) | N2、T13〜T17 | `run_job` の中に分割が残らない。分けられないコマンドは今どおり実行前の検査で 64。件数 1,139 |
| SM3(段 3) | N3、N4、N5、既存の取り込みのテスト(無変更) | 権限のない大きなファイルは Unreadable。ほかの場合の種類は不変。FIFO で止まらない |
| SM4(版) | T18 | `--version` が `conductor 0.9.6` |
| SM5(テストと合格の条件) | 全体 | テストのファイルの差分は T1〜T18 と N1〜N5 だけ。fmt、clippy、全テスト、カバレッジ 80%。件数 1,142 |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。到達しない分岐が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う(T1、T13 で新しい型と引数に合わせる)。

## テストデータ

- N3 は 1 MiB + 1 バイトのファイル、N5 は `mkfifo` の FIFO を、テストの中で一時ディレクトリに作る。ファイルを足さない。
