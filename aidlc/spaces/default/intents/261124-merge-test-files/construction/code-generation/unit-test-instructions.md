# テストの手順(Unit Test Instructions): テストのファイルの統合

Minimal の戦略。既存の Rust のテストの実行環境(`cargo test`)を使う。新しい設定は要らない。テスト名は日本語。この intent では新しいテストを足さず、消さず、名前を変えない(移すだけ)。

## 実行のコマンド

```bash
# 全体(基準: 1,149 通過、1 無視、0 失敗。HEAD 5c386ef)
cargo test --locked --workspace

# クレートごと(各コミットで件数を見る)
cargo test --locked -p conductor-converter
cargo test --locked -p conductor-runtime
cargo test --locked -p conductor-cli

# テスト名の一覧(前後で比べる。名前の最後の部分で比べて差 0。パスは移したテストだけが変わる)
cargo test --locked --workspace -- --list 2>/dev/null | sort

# 本番の形
cargo clippy -p conductor-cli --lib --bins --locked -- -D warnings

# Windows の対象の clippy(macOS から)
cargo clippy --workspace --all-targets --locked --target x86_64-pc-windows-msvc -- -D warnings

# ドキュメントのリンクの直し漏れ(警告は基準の 5 件のまま)
cargo doc --workspace --no-deps --locked
```

## テストの内容

| 指標 | テスト | 確かめ方 |
|------|--------|----------|
| 小さなファイルの統合 | T1〜T9 | 移したテストが、名前・確かめの行・期待の値を変えずにまとめ先で通る。まとめ元のファイルは無い |
| README のテストの統合 | T10、T11 | `src/tests/docs.rs` で、元の 18 件(単体 8、統合 10)が通る。補助の `readme` は 1 つ |
| 補助の写しの解消 | T12 | `tests/common/fakes.rs` を統合テストと単体テストの両方が読み、両方のテストが通る |
| ひな形の原本 | T13 | 両方のテストが `tests/fixtures/init_{ja,en}.toml` を読み、本番のひな形と一致する。Windows の CI で確かめる(D5) |
| 製品のコードの検査 | `nfr_source`(cli、runtime)、`nfr_security`(converter) | 数と中身を変えずに通る |
| 合格の条件 | 全体 | fmt、clippy、全テスト 1,149 通過、行カバレッジ 80%、MSRV 1.100、`Cargo.toml`・`Cargo.lock` の差分なし |

## 期待するカバレッジ

- プロジェクト全体の行カバレッジ 80% 以上(Build and Test が確かめる。写しで 98.06%)。

## モックとスタブ

- cli の偽の `Env`・`Sink` とステートの誤りの補助は、`tests/common/fakes.rs` の 1 つを使う。

## テストデータ

- 新しく `crates/cli/tests/fixtures/init_ja.toml`・`init_en.toml`(今の原本と同じ中身)を置く。
