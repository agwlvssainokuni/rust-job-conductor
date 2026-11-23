# セキュリティのテストの手順(Security Test Instructions)

- 誤りの表示に入力の値を含めない: 変換の文法の誤りの理由は固定の文(「TOML として読めません」「YAML として読めません」「入力の構造を解釈できません」)。`.env` の読み込みの誤りも固定の文と表示名だけ。`report_messages.rs`、`convert_ac.rs`、`env_loader.rs` で確かめた。
- `.env` の読み込み: 大きさは開いたハンドルから見て、上限 + 1 バイトで読み止める。読む権限のないファイルは大きさによらず「読めない」(N11)。名前付きパイプは開かずに「読めない」(N12)。種類を見てから開くまでの差し替えの競合は残る(対象は利用者自身の `.env`。定義ファイルと同じ扱い)。
- runtime の製品のコードに、パニックしうる書き方(`unwrap`、`expect`、`panic!`、`unreachable!` など)を足していない: `nfr_source.rs`。
- `unsafe` と `cfg(unix|windows)` を `platform` の外に置かない、ネットワークのライブラリを使わない: `nfr_source.rs`。
- 新しい依存なし: `Cargo.lock` の差分は版の行だけ。`cargo audit` は CI(警告のみ)。

いずれも `cargo test --locked --workspace` で通った。上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
