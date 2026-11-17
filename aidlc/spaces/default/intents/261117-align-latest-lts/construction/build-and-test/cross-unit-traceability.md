# Cross-Unit Traceability

出力元: `aidlc engine sensor-traceability --stage build-and-test`。この作業は Unit に分けていない(Unit なしの 1 回の Code Generation)。

## 判定

**未カバー 1 件(NFR2)。** ツールの判定は `pass: false`。ただし NFR2 は、`code-generation` の `traceability.json` で `Deferred`(Build and Test で確認する)としていた項目で、この段階で測って満たした(下表)。承認の場で、この点を確認してください。

## 要件ごとの対応

| ID | 判定 | 対応する成果物 | 備考 |
|---|---|---|---|
| FR1.1 | 対応済み | `Cargo.toml` | `rust-version = "1.100"` |
| FR1.2 | 対応済み | `crates/cli/tests/msrv_consistency.rs` | 3 件のテストが通る |
| FR1.3 | 対応済み | `.github/workflows/ci.yml` | `@1.100.0` の 2 か所 |
| FR1.4 | 対象外(N/A) | — | `Cargo.lock` は変更不要だった(`--locked` のビルドが通る) |
| FR2.1 | 対応済み | `crates/cli/src/ctx.rs` ほか | clippy の警告 0 件 |
| FR2.2 | 対象外(N/A) | — | 非推奨 API の警告は 0 件 |
| FR2.3 | 対応済み | `crates/core/src/definition/walk.rs` ほか | let チェーンへの書き換え |
| FR2.4 | 対象外(N/A) | — | crate ごとの 5 件の小さなコミット |
| FR3.1 | 対応済み | `crates/cli/tests/exit_codes.rs` | 既存のテストが変更なしで通る |
| FR3.2 | 対応済み | `crates/converter/tests/convert_roundtrip.rs` | 既存のテストが変更なしで通る |
| FR3.3 | 対象外(N/A) | — | 基準のコミットから、アサーションの増減がない |
| NFR1 | 対応済み(ローカル)/ CI で確認 | `.github/workflows/ci.yml` | `fmt`、`clippy`、`test` はローカル(macOS)で通る。Linux・Windows は CI |
| NFR2 | **未カバー(`Deferred`)** | — | この段階で測定: 行 91.81%(`test-results.md`)。下限 80% を満たす |
| NFR3 | 対応済み | `.github/workflows/ci.yml` | `cargo tree` で `-sys` と C のビルド依存がないことを確認 |
| NFR4 | 対応済み(ローカル)/ CI で確認 | `.github/workflows/ci.yml` | ローカルは 1.100.0 でビルドとテストが通る。`@1.100.0` のジョブは CI |
