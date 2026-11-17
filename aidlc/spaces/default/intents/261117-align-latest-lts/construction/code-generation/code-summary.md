# Code Summary — 最新の Rust に合わせたリファクタリング

## 確定した版

- 確定した MSRV: **1.100**(`rust-version = "1.100"`、CI は `@1.100.0`)。根拠: 2026-11-17 に `rustup check` が示した最新の安定版は rustc 1.100.0(2026-11-09)。(要件 FR1.1)
- edition は 2024 のまま。`Cargo.lock` は変更していない。(FR1.4、C3)

## 作成・変更したファイル

コミットは 5 件で、`main` 上にある(push はしていない)。

| コミット | 内容 | ファイル |
|---|---|---|
| f724dec | MSRV の引き上げと、記載の一致を確かめるテストの追加 | `Cargo.toml`、`README.md`(84 行目)、`.github/workflows/ci.yml`(コメント 2 か所、`@1.100.0` 2 か所)、`crates/cli/tests/msrv_consistency.rs`(新規、3 テスト) |
| 01b80a6 | core | `crates/core/src/definition/walk.rs`、`crates/core/tests/nfr_performance.rs` |
| 01aee48 | runtime | `crates/runtime/src/runner.rs`、`crates/runtime/tests/nfr_performance.rs`、`crates/runtime/tests/flow_with_state.rs` |
| eaa868b | converter | `crates/converter/src/resolve.rs`、`crates/converter/src/yaml_read.rs` |
| c6f82aa | cli | `crates/cli/src/{args,ctx,pipeline,run}.rs`、`crates/cli/tests/nfr_perf.rs` |

## 主な実装上の判断

- MSRV を 1.100 に上げると、clippy の `collapsible_if` が let チェーンを提案するようになる。新たに出た指摘は 19 か所(src 15、tests 4)で、いずれも入れ子の `if` を let チェーン 1 つにまとめる機械的な変更で直した。評価の順序と結果は変わらない。(FR2.1、FR2.3)
- MSRV を上げる前の警告は 0 件で、非推奨 API の警告もなかった。(FR2.2)
- 新規テスト `msrv_consistency.rs` は、`Cargo.toml`、`README.md`、`ci.yml` の MSRV の記載が一致することを読み取りだけで確かめる。(FR1.2)

## 書き換えた箇所

- core: `definition/walk.rs:256`、`:496`、`:497`
- runtime: `runner.rs:76`
- converter: `resolve.rs:515`、`yaml_read.rs:130`、`:226`、`:267`
- cli: `ctx.rs:101`、`:127`、`:270`、`pipeline.rs:82`、`run.rs:77`、`:129`、`args.rs:307`

通る既存のテストの例: `crates/core/tests/definition_errors.rs` の `fr2_run_id_env_の型と形の誤りは定義の誤り`(`walk.rs:256`)、`crates/runtime` の `合図が立てば_1回だけ転送し_間隔は_10ms_まで広がる`(`runner.rs:76`)。ほかの箇所は、fixtures の往復変換、重複キー、アンカー、cli のテスト 268 件が通る。個々の箇所と個々のテスト名の対応は確かめていない。

## 計画の表にない箇所を直したもの(11 か所)

clippy が新しく指摘し、CI の `lint`(`-D warnings`)を通すために、最小限で機械的に直した。

- src: `walk.rs:496`、`runner.rs:76`、`yaml_read.rs:226`、`args.rs:307`、`pipeline.rs:82`、`run.rs:77`、`run.rs:129`
- テストコード(`GITHUB_STEP_SUMMARY` に書く補助)4 か所: `crates/core/tests/nfr_performance.rs:75`、`crates/runtime/tests/nfr_performance.rs:73`、`crates/cli/tests/nfr_perf.rs:58`、`crates/runtime/tests/flow_with_state.rs:140`。アサーションには触れていない。(FR3.3: FR2.1 に必要な最小限の修正)

## 書き換えなかった箇所(clippy が指摘せず、構造上そのままでは書き換えられない)

- `runtime/state/record.rs:85`: 内側の `if` のあとに、ほかの文が続く
- `runtime/state/reader.rs:304`: 内側に `if` が 2 つ並ぶ
- `converter/resolve.rs:352`: 内側の `if` に `else` がある
- `converter/resolve.rs:520`: 内側の `if` のあとに `self.deliver` が続く
- `converter/scalar.rs:64`、`:70`: 内側の `if` のあとに `return None` が続く

## テストの結果(ローカル、macOS、rustc 1.100.0)

- 変更前(基準): `cargo test --workspace --locked` は 701 件が成功、0 件が失敗、1 件が無視(計画が書いていた約 716 件とは異なる。`#[test]` の数と、実行される数の差とみられる)
- 変更後: 704 件が成功、0 件が失敗、1 件が無視(基準に新規の 3 件を加えた数)
- `cargo fmt --all -- --check`: 問題なし
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告 0 件
- `cargo build --workspace --all-targets --locked`: 成功
- `cargo tree`: `-sys` クレート、`cc`、`cmake`、`pkg-config`、`bindgen` はない(NFR3)
- テストのアサーション: 基準(266f105)から現在までの差分に、`assert` と `expect` の増減はない(FR3.3)

## 実行していない(書いたが実行していない)テストと検査

- カバレッジ(`cargo llvm-cov --fail-under-lines 80`、NFR2): ローカルでは実行していない。`cargo-llvm-cov` の有無も確かめていない。CI の `coverage` ジョブで測る。
- Linux・Windows 上のテスト: ローカル(macOS)では実行できない。CI で実行する。
- CI での `@1.100.0` のジョブ(`build-check`、`msrv-test`、NFR4): プッシュ後の CI でしか確かめられない。この段階の時点では、CI は実行されていない。

## 計画からのずれ

- 計画の表にない 11 か所を直した(上記)。clippy の指摘への対応として、計画の規則(最小限で FR2.1 に必要な修正は直し、記録する)の範囲内。
- 計画の「書き換える箇所」の規則では、通るテストを個別に確かめることになっていたが、個々のテスト名までは確かめていない。全テストが通ることで検証した。
