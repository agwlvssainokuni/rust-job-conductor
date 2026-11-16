# Dependencies

## 内部(クレートのあいだ)

```text
conductor-cli       → conductor-core, conductor-runtime, conductor-converter
conductor-runtime   → conductor-core
conductor-core      → (内部の依存なし)
conductor-converter → (内部の依存なし。conductor-core に依存しない)
```

循環はない。

## 外部(`Cargo.toml`)

| ライブラリ | バージョン | 使うクレート | 用途 |
|-----------|-----------|--------------|------|
| lexopt | 0.3.2 | cli | 引数の字句分け |
| toml | 1.2.1 | core(parse, std)、runtime・converter(+ display, serde) | 定義・ステートの読み書き |
| thiserror | 2.0.21 | core、runtime、converter | エラー型 |
| saphyr-parser | 0.3.4 | converter | YAML の解析 |
| signal-hook | 0.4.6(`iterator`のみ) | cli(UNIX) | シグナルの監視 |
| ctrlc | 3.5.2 | cli(Windows) | Ctrl-C の監視 |
| libc | 0.2 | runtime(UNIX) | `kill` |
| tempfile | 3.27.0 | 4 クレートの dev | テスト |

CI は「C のビルドを要する依存がないこと」を確かめ、週次の `cargo audit` を走らせる。
