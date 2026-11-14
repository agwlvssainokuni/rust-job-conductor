# Unit Test Instructions(u4-converter)

## 実行の環境

- 言語とツール: Rust(edition 2024)、`cargo`。外部のサービスやネットワークは要らない(依存は、最初の取得だけ公開のレジストリから)。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は `crates/u4-converter/tests/`)。開発用の依存は `tempfile` のみ。
- 設定ファイル: ワークスペースの根の `Cargo.toml`(`crates/*` を含む)と、`crates/u4-converter/Cargo.toml`(Step 1 で作る)。

## この Unit のテストの実行(この Unit に限る)

```bash
cargo test -p conductor-converter --locked
```

- 最初のテストを書く前に、この命令が、テスト 0 件で成功することを確かめる(Step 1)。
- 特定のテストだけを動かすときは、フィルタを足す(例: `cargo test -p conductor-converter --locked resolve::`、`cargo test -p conductor-converter --locked --test convert_roundtrip`)。
- 全体の命令(`cargo test` を、`-p` なしで)は、この Unit では使わない。

## 整形と静的検査(Unit の確認)

```bash
cargo fmt --all -- --check
cargo clippy -p conductor-converter --all-targets --locked -- -D warnings
```

## 期待する網羅(Standard の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| 型とエラー(Value、Describe) | 5〜7 | 値の等しさ、表示の文に入力の値が入らないこと、キー名のエスケープ・切り詰め、`key_path` の書式 |
| 取り込み(Intake) | 5〜6 | 存在しない、1 MiB ちょうど、1 MiB + 1、終わりのない入力、UTF-8 でない、ディレクトリ |
| YAML の読み取り | 8〜10 | 文法の誤り、重複するキー、再帰するアンカー、深さ 32/33/100,000、スカラーの解決、タグ |
| TOML の読み取り | 4〜5 | 文法の誤り、深さ、日時の印、空の TOML |
| 文書の確認と展開(Resolve) | 8〜10 | 複数の文書、rootNotMap、アンカー・エイリアス、マージキー、要素数の上限、深さ、表せない値、失敗の順序(BR1.12) |
| 書き出し(Write) | 4〜5 | 引用符(値・キー・`<<`)、浮動小数、`i64` の端 |
| 往復・AC | 6〜8 | BR1.11 の往復(両方向)、AC7.1.1〜AC7.1.3 |
| 横断(NFR) | 4 | ソースの検索(`std::process`、`std::net`、静的な初期化)、副作用なし、パニックしない、全 kind の表示 |

合計: 約 50 件。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。
- 全体の下限は、Build and Test が、組み立てたプロジェクトの全テストで判定する。この Unit だけのテストで 80% を下回ったときは、**この Unit のテストの失敗としては扱わず**、結果に記録する。測定は、`cargo llvm-cov -p conductor-converter --locked`(導入されていれば)を、**ほかの命令とは別の命令として**実行する。
- この Unit の範囲(`crates/u4-converter/src/`)で、行カバレッジ 80% 以上を、Build and Test が確かめる。

## モックとスタブ

- 変換の本体は、文字列だけで試せる関数(`convert_text`)のため、モックは使わない。入力の文字列を直接作って渡す。
- ファイルを使うテストは、`tempfile` の一時ディレクトリに作り、テストの終わりに自動で消える。
- 暴走の入力のテスト(深さ 100,000、展開で膨らむ入力)は、各 30 秒の時間の上限(別のスレッドで動かし、`recv_timeout` で待つ)を持つ。上限が検知するのは、ハングだけである。

## テストデータ

- 入力の例は、`crates/u4-converter/tests/fixtures/*.yaml`、`*.toml`(正しい例、誤りの例)に置く。短い例は、テストの中の文字列で直接書く。
- 大きなファイル(1 MiB 前後)、深い入れ子(33 段、1000 段、100,000 段)、展開で膨らむ YAML は、テストの中で生成する(ファイルをコミットしない)。
- 実際の機密や、利用者の環境のパスは、テストデータに含めない。
