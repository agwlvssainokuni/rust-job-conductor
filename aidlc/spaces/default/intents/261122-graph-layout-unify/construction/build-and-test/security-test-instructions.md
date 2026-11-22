# セキュリティのテストの手順(Security Test Instructions): graph の配置のリファクタリング

この変更は、図の配置(純粋な関数)と、その整形だけで、入力の読み取り、ファイル、ネットワーク、権限には触れない。

## 確かめること(既存のテストで続けて確かめる)

```bash
cargo test --locked -p conductor-cli --test nfr_source
cargo test --locked -p conductor-cli --test binary_graph
```

- `nfr_source.rs`: `unsafe` が runtime の `platform/unix.rs` の外にないこと、`cfg(unix)`/`cfg(windows)` が `platform` の外にないこと、ネットワークのライブラリを使わないこと。
- `binary_graph.rs`: 秘密の値(`SECRET`)が、graph の出力に出ないこと。
- 依存: `Cargo.toml`、`Cargo.lock` に差分がないこと(`git diff`)。`cargo audit`(`audit.yml`)は既存のとおり。

## ライセンス

- git のソース(GPL v2)は読んでいない。規則は、`git log --graph` の出力の観察と、ユーザーが承認した図だけから立てた(BR5.1)。
