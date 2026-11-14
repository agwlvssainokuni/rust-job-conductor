# Unit Test Instructions(u1-core)

## 実行の環境

- 言語とツール: Rust(edition 2024)、`cargo`。外部のサービスやネットワークは要らない(依存は、最初の取得だけ公開のレジストリから)。
- テストの枠組み: 標準の `cargo test`(単体は `#[cfg(test)]`、統合は `crates/u1-core/tests/`)。開発用の依存は `tempfile` のみ。
- 設定ファイル: ワークスペースの根の `Cargo.toml` と、`crates/u1-core/Cargo.toml`(Step 1 で作る)。

## この Unit のテストの実行(この Unit に限る)

```bash
cargo test -p conductor-core --locked
```

- 最初のテストを書く前に、この命令が、テスト 0 件で成功することを確かめる(Step 1)。
- 特定のテストだけを動かすときは、フィルタを足す(例: `cargo test -p conductor-core --locked flow::`、`cargo test -p conductor-core --locked --test definition_load`)。
- 全体の命令(`cargo test` を、`-p` なしで)は、この Unit では使わない。

## 整形と静的検査(Unit の確認)

```bash
cargo fmt --all -- --check
cargo clippy -p conductor-core --all-targets --locked -- -D warnings
```

## 期待する網羅(Standard の方針)

| 範囲 | 件数の目安 | 内容 |
|------|------------|------|
| 型とエラー(Types、Describe、Locate) | 6〜8 | 既定値、表示の文に入力の値が入らないこと、キー名のエスケープと切り詰め、行番号(LF・CRLF) |
| 取り込み(Intake) | 5〜6 | 存在しない、1 MiB ちょうど、1 MiB + 1、UTF-8 でない、ディレクトリ、既定の場所 |
| 読み取り・深さ・版 | 6〜8 | 文法の誤り、深さ 32/33/1000、版なし、版 2、版の誤りのとき他の検査をしない |
| 構造の検査(Walk) | 14〜18 | BR1.1〜BR1.13 の各規則の違反と受理、複数の誤りを 1 回で返すこと、行番号 |
| 起動前の判断(decide) | 7〜8 | BR3.5 の 7 つの順序 |
| 分類と終了コード(classify) | 8〜10 | BR2.1〜BR2.4。優先順位、成功コードを書いた場合 |
| 次の動き・完了(next_step、resume) | 8〜10 | BR3.1〜BR3.4、BR3.6、BR3.7。`individual`・`pipeline` の `@end` |
| 横断(NFR) | 3 | 起動時間の測定(失敗にしない)、検証の純粋さ、エラー表示 |

合計: 約 55 件。

## カバレッジ

- 目標: 行カバレッジ **80% 以上**(チームの下限。しきい値は下げない)。
- 全体の下限は、Build and Test が、組み立てたプロジェクトの全テストで判定する。この Unit だけのテストで 80% を下回ったときは、**この Unit のテストの失敗としては扱わず**、結果に記録する。測定は、`cargo llvm-cov -p conductor-core --locked`(導入されていれば)を、**ほかの命令とは別の命令として**実行する。
- この Unit の範囲(`crates/u1-core/src/`)で、行カバレッジ 80% 以上を、Build and Test が確かめる。

## モックとスタブ

- Flow(判断)は、入出力のない関数のため、モックは使わない。引数の値を直接作って渡す(表形式のテスト)。
- ファイルを使うテストは、`tempfile` の一時ディレクトリに作り、テストの終わりに自動で消える。
- 時間の測定(NFR1.1)は、標準のライブラリの時計を使う。基準を超えても失敗にしない(Q1: A)。

## テストデータ

- 定義ファイルの例は、`crates/u1-core/tests/fixtures/*.toml`(正しい例、誤りの例)に置く。短い誤りの例は、テストの中の文字列で直接書く。
- 大きなファイル(1 MiB 前後)と、深い入れ子(33 段、1000 段)は、テストの中で生成する(ファイルをコミットしない)。
- 実際の機密や、利用者の環境のパスは、テストデータに含めない。
