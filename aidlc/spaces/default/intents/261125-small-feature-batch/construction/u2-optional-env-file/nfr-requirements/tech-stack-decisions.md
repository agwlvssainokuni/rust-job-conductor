# 技術の選択: u2-optional-env-file

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR2.1)、契約(`contract-summary.md` の C1)、team.md の Code Style。

新しい技術は選ばない。

| 項目 | 選択 | 理由 |
|---|---|---|
| 定義の読み取り | 既存の `toml` の位置つきの木(`walk/env.rs`) | 表の項目も今の走査で読める |
| 型 | core の `types.rs` に EnvFileRef を足す(契約 C1)。読む対象の一覧も EnvFileRef の並びで返す(同じ形の別の型は作らない) | 契約 C1 の `load_env_files` の入力と揃え、型を増やさない |
| 読み取り | runtime の `envfile.rs` と既存の `bounded_read::read_regular_file` | 「無い」と「読めない」をすでに見分けている |
| cli | `Runtime::load_env_files` の引数を EnvFileRef の並びに変える(本番と偽の実装)。流れの順番は変えない | 差し替え口の形が変わるだけ |
| テスト | 各クレートの `tests/` と `mod tests`、`cargo-llvm-cov` | team.md の Testing Posture |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
