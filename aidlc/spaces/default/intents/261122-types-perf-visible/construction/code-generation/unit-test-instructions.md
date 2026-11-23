# テストの手順(Unit Test Instructions): 型と構造の整理・計算量・利用者から見える変更

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,146 通過、1 無視、0 失敗。HEAD f1c9871)
cargo test --locked --workspace

# テスト名の一覧(段の前後で比べる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# 性能のテスト(debug。CI と同じ)
cargo test --locked -p conductor-core --test plan_simulate_perf --test graph_lookup_perf --test env_parse_perf

# 段 3
cargo test --locked -p conductor-converter --test convert_ac
cargo test --locked -p conductor-runtime --test env_loader
cargo test --locked -p conductor-cli --test binary_basic
```

(ファイル名が違うときは、実際のテストのファイル名に合わせる。)

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 段 1(型と構造) | N1、N2、N3、T1〜T23・T26 | 期待する出力の文字列が変わらず通る。`label()` の日本語の文が変更前と 1 文字も同じ(N2)。cli の表示が各クレートの `label()` を使う(N3)。件数 1,149 |
| 段 2(計算量) | N4〜N8 | 上限(P1・P2 150 ms、P3 350 ms、P4 600 ms、P5 100 ms。5 回の中央値)の中で、結果の中身も正しい。直す前と後の時間を記録する。件数 1,154 |
| 段 3(利用者から見える変更) | N10、N11、N12、★ T24 | 文法の誤りの理由が形式ごとの文。読む権限のない大きな `.env` は `Unreadable`(N11 は直す前に落ちる)。名前付きパイプで止まらない(N12)。ほかの場合の種類は既存のテストが無変更で通る |
| 版 | ★ T25 | `conductor 0.9.7` |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,157 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.lock` の差分は版の行だけ |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。到達しない分岐が消えるので、下がらない見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う(`run_job` の引数の変更に合わせる)。

## テストデータ

- 性能のテストは、テストの中で入力を組み立てる(ジョブ 1,000 個、`.env` 10,000 行)。N11 は 1 MiB 超のファイル、N12 は `mkfifo` の名前付きパイプを一時ディレクトリに作る。ファイルを足さない。
