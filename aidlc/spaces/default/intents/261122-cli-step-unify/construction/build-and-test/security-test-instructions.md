# セキュリティのテストの手順(Security Test Instructions): run と pipeline の重複をなくすリファクタリング

この変更は、cli の内部の手順の置き場所だけで、入力の読み取り、ステートの中身、権限、ネットワークには触れない。

```bash
cargo test --locked -p conductor-cli --test nfr_source
```

- `nfr_source.rs`: `unsafe` が runtime の `platform/unix.rs` の外にないこと、`cfg(unix)`/`cfg(windows)` が `platform` の外にないこと、ネットワークのライブラリを使わないこと(新しい `step.rs` も対象)。
- ステートに環境変数を書かないこと、誤りの表示に入力の値を含めないことは、既存のテストで続けて確かめる(書く内容は変えていない)。
- 依存: `Cargo.toml`、`Cargo.lock` に差分がない。
