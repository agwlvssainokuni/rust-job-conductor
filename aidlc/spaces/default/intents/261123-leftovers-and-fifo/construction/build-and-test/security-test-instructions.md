# セキュリティのテストの手順(Security Test Instructions)

- 読み込みの順序(形 B): converter の入力とステートも、定義ファイルや `.env` と同じく、パスで種別を見てから開き、開いたハンドルで大きさを見て、上限 + 1 バイトで読み止める。書き手のいない名前付きパイプや文字デバイスへのリンクでも止まらずに「読めない」(N4、N5、★ T23)。終わりのない入力でも上限で止まる(N8)。種別を見てから開くまでの差し替えの競合は、定義ファイル・`.env` と同じく残る。
- 読む権限のない大きなステートは「読めない」(N5)。
- 誤りの表示に入力の値を含めない: 理由はすべて固定の文(converter の `reason` は `&'static str`)。`report_messages.rs`、`nfr_security.rs` で確かめた。
- runtime の製品のコードに、パニックしうる書き方(`unwrap`、`expect`、`panic!` など)を足していない: `nfr_source.rs`。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に足していない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.lock` の差分は版の行だけ。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
