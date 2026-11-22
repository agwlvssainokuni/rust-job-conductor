# セキュリティのテストの手順(Security Test Instructions): `--dry-run` の模擬の重複をなくすリファクタリング

この変更は、純粋な関数の置き場所と模擬の内部の持ち方だけで、入力の読み取り、ファイル、権限、ネットワークには触れない。core に入出力は持ち込んでいない。

```bash
cargo test --locked -p conductor-cli --test nfr_source
cargo test --locked -p conductor-runtime --test nfr_source
```

- `unsafe`、`cfg(unix)`/`cfg(windows)`、ネットワークのライブラリを足していないこと。
- ステートに環境変数を書かないこと(書く内容は変えていない)。
- 依存: `Cargo.toml`、`Cargo.lock` に差分がない。
