# セキュリティのテストの手順(Security Test Instructions)

- 誤りの表示に入力の値を含めない: 定義ファイルの読み込みの誤りは、ファイルの表示名と固定の文だけ(N3〜N5 と既存の `definition_errors.rs`、`error::tests`)。
- ファイルの差し替え: 大きさを開いたハンドルから見るので、確かめてから読むまでの差し替えで上限を越えて読むことはない(上限 + 1 バイトで止まる)。種別を見てから開くまでに名前付きパイプへ差し替えられた場合は止まりうる。これは仕様書 03 に明記した。
- 読む権限のないファイル: 大きさによらず「読めない」になる(N3)。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`(`Command::new(&line.program)` の検査を含む)。足した `cfg(unix)` はテストの 2 か所だけ。
- 新しい依存なし: `Cargo.toml`、`Cargo.lock` の差分は版の行だけ。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`(SM3、SM5)。
