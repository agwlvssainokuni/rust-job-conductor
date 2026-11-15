# Integration Test Instructions

## 範囲

Unit 間の結合と、バイナリを実際に起動する統合テストは、`conductor-cli`(u3-cli)の `tests/` にあり、`conductor-core`・`conductor-runtime`・`conductor-converter` を実際に結合する。偽の部品(`Runtime`・`Sink`・`Env`・`Signals`)を使う流れのテストと、実際の `conductor` と補助の実行ファイルを使うテストの 2 層がある。

## 実行

```bash
cargo test -p conductor-cli --locked                       # この Unit の全テスト
cargo test -p conductor-cli --locked --test binary_basic   # バイナリの起動(--version、使い方の誤り、終了コード、標準出力と標準エラー)
cargo test -p conductor-cli --locked --test binary_signal  # 実際のシグナル(UNIX 系)
cargo test -p conductor-cli --locked --test flow_run --test flow_pipeline --test flow_aux
cargo test -p conductor-runtime --locked                   # 実際の子プロセス、ステートファイル
cargo test -p conductor-converter --locked                 # YAML と TOML の往復
```

## 環境とデータ

- 一時ファイルは `tempfile` の一時ディレクトリ。定義ファイルの例は `[[job]]` の形で、テスト内の文字列と `tests/fixtures/`。
- プロセスを起動するテストは、各 30 秒の上限を持ち、超えたら子プロセスを強制終了して失敗にする。
- 性能の測定(`tests/nfr_perf.rs`)は、基準を超えても警告だけで、`STARTUP_BENCH_ENFORCE=true` のときに失敗にする。

## OS ごとの実行

- 主な開発環境(macOS)では、ローカルで全テストを実行する。Linux と Windows は GitHub Actions の CI で実行する(チームの規則)。CI は CI Pipeline のステージで作る。
- Windows の Ctrl+C は、自動テストを作れなかったため、`crates/u3-cli/tests/WINDOWS_CTRL_C_CHECKLIST.md` の手動の確認をリリース前に行う。

## カバレッジの目標

- 全体の行カバレッジ 80% 以上(チームの下限。下げない)。各 Unit の範囲でも 80% 以上。測定の命令は `test-results.md` のとおり。
