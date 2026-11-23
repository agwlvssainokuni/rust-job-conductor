# テストの手順(Unit Test Instructions): 残りの整理と読み込みの順序のそろえ

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。

## 実行のコマンド

```bash
# 全体(基準: 1,157 通過、1 無視、0 失敗。HEAD 025067a)
cargo test --locked --workspace

# テスト名の一覧(段の前後で比べる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# クレートごと
cargo test --locked -p conductor-core
cargo test --locked -p conductor-runtime
cargo test --locked -p conductor-converter
cargo test --locked -p conductor-cli
```

(性能のテストと名前付きパイプのテストのファイル名は、`construction/functional-design/functional-spec.md` §5 の N 表に従う。)

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 段 1(動きを変えない整理) | 既存のテスト、T1〜T21・T24、消すテスト 4 つ(§4.6) | 期待する出力の文字列が変わらず通る。テスト名の差は、消した 4 つと足した N、置き場所の変わったテストだけ。件数 1,156 |
| 計算量 | N1〜N3 | 上限(P1 350 ms、P2 1,600 ms、P3 40 ms。5 回の中央値)の中で、結果の中身も正しい。直す前と後の時間を記録する |
| 段 2(利用者から見える変更) | N4〜N8、★ T23 | converter とステートの名前付きパイプは止まらずに「読めない」(N4・N5、直す前の木で落ちる)。権限なしで大きなステートは「読めない」。終わりのない入力でも上限 + 1 で読み止める(N8)。空白だけのコマンド名は 64(N6・N7) |
| 版 | ★ T22 | `conductor 0.9.8` |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,161 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.lock` の差分は版の行だけ |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる)。到達しない分岐と重複したテストが消えるが、下限は十分上回る見込み。

## モックとスタブ

- cli の流れのテストは、既存の偽物(`crates/cli/tests/common/mod.rs`)を使う。偽物は `run_job` の引数の変更に合わせ、環境変数 `CONDUCTOR_JOB_ID` からジョブ ID を取る。

## テストデータ

- 性能のテストは、テストの中で入力を組み立てる。名前付きパイプは `mkfifo` コマンドで、権限のテストは mode 000 のファイルで、一時ディレクトリに作る。ファイルを足さない。
