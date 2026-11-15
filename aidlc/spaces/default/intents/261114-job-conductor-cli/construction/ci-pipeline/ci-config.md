# CI Config

## Sources

- Initial description: 単一バイナリの CLI `conductor`(Rust)。 [desc]
- Workflow-selected scope: mvp [scope]
- チームの実践: GitHub Actions、統合ブランチ `main`(squash しない)、rustfmt と clippy、行カバレッジ 80%、3 OS のテスト、タグでリリース [memory:M1]
- 各 Unit の cicd-pipeline.md(u1〜u4)と、u3-cli の cicd-pipeline.md が定めた段階と基準 [Q1]
- 質問への回答: Q1=A(5 つのジョブをすべてマージのブロックにする)、Q2=B(`cargo audit` を警告として)、Q3=A(`v` タグ、版の一致、下書きのリリース) [Q1] [Q2] [Q3]
- Build and Test の結果(`test-results.md`、`build-and-test-summary.md`): 実行するコマンドと、満たす基準の根拠 [scope]

## Assumptions & Open Questions

None.

## ワークフロー(`.github/workflows/`)

| ファイル | 起動 | 役割 |
|----------|------|------|
| `ci.yml` | `main` への push、`main` へのプルリクエスト、`workflow_call`(リリースから呼ぶ) | マージの関門(下の表) |
| `audit.yml` | プルリクエスト、毎週月曜(03:00 UTC) | `cargo audit`。警告のみ(`continue-on-error`) |
| `release.yml` | タグ `v*` の push | タグの確認 → `ci.yml` の全ジョブ → 4 つのターゲットのビルド → 下書きのリリース |

権限は `contents: read` を既定とし、下書きのリリースを作るジョブだけ `contents: write`。秘密情報は `GITHUB_TOKEN` だけ。

## `ci.yml` のジョブ

| ジョブ | 実行の場所 | コマンド | 失敗時 |
|--------|------------|----------|--------|
| `lint` | Linux | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | マージをブロック |
| `build-check` | Linux、Windows | Rust 1.85 で `cargo build --workspace --locked --all-targets`。Linux では、`cc`・`cmake`・`pkg-config`・`bindgen` と、実行時の `-sys` 依存(許可: `linux-raw-sys`、`windows-sys`)がないことの検査 | マージをブロック |
| `test` | Linux、macOS、Windows | `cargo test --workspace --locked`(安定版) | マージをブロック(OS ごと) |
| `msrv-test` | Linux | Rust 1.85 で `cargo test --workspace --locked` | マージをブロック |
| `coverage` | Linux | `cargo llvm-cov --workspace --locked --ignore-filename-regex 'bin/conductor-(cli-)?test-helper' --fail-under-lines 80 --summary-only` | マージをブロック(要件 NFR5) |
| `docs-check` | Linux | README に必要な節(SHA-256、Gatekeeper、終了コードの表、ステートファイルの場所、at-least-once、シグナル、convert)があること | マージをブロック |
| `startup-bench` | Linux、macOS、Windows | `cargo test -p conductor-cli --release --test nfr_perf`。基準を超えても警告(`STARTUP_BENCH_ENFORCE=false`) | ブロックしない(`continue-on-error`) |

- Build and Test が記録した検査(`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace --locked`、Rust 1.85 のビルドとテスト、`cargo llvm-cov` の 80%)は、すべて、上のジョブに入っている。
- 同じ内容が、コメントの付いた YAML として `.github/workflows/ci.yml` にある。同じ起動で、新しい実行が古い実行を取り消す(`concurrency`)。

## `release.yml` の流れ

1. `verify-tag`: タグのコミットが `main` の履歴にあること。タグの版が `crates/u3-cli/Cargo.toml` の `version` と一致すること。版に `-`(例 `-rc1`)があれば先行版(prerelease)。
2. `checks`: `ci.yml` を呼び、すべてのジョブを、タグのコミットで実行する。
3. `build`(4 つのターゲット): `x86_64-unknown-linux-musl`(Linux)、`aarch64-apple-darwin`、`x86_64-apple-darwin`(macOS ランナーでクロスビルド)、`x86_64-pc-windows-msvc`。`cargo build --release --locked --bin conductor` だけ(補助の実行ファイルは配布しない)。UNIX 系は `.tar.gz`、Windows は `.zip`(中に実行ファイルと `LICENSE`)。まとめたファイルを展開して `conductor --version` を実行(Intel macOS は、実行できないため、省く)。Linux は静的リンクも確かめる。
4. `draft-release`: `SHA256SUMS` を作り、**下書き(draft)**のリリースを作る。**公開は人が行う**(GitHub の画面で内容を確認してから)。Intel macOS の動作の確認なしは、本文に書く。

## 設計との差(cicd-pipeline.md との違い)

- **カバレッジを Unit ごとに測らず、ワークスペース全体で 80% 以上を要求する:** 1 つの Unit だけのテストで測ると、ほかの Unit のコードが、別の Unit のテストで実行されるため、Unit ごとの割合が揺れる。全体で判定する。現状は、すべてのファイルが 91% 以上(`test-results.md`)。
- **Windows の Ctrl+C の自動テストを CI に置かない:** CI に対話的なコンソールがない(Build and Test で、リリース前の手動確認として受け入れ済み)。
- **`startup-bench` は、いまは警告のみ:** 3 つの OS の実測(特に Linux と Windows)が、まだない。CI で実測を重ね、配分を確定したら、`STARTUP_BENCH_ENFORCE` を `true` にして、必須に加える(担当: 開発者)。
- **`cargo deny` は入れない:** Q2 の回答は `cargo audit` の警告まで。ライセンスの検査は、必要になったときに足す。
- **アクションの版は、タグで固定している:** 完全なコミットの SHA で固定する強化は、将来の課題(サプライチェーン)。

## この環境で確かめたことと、確かめていないこと

- 確かめた: 3 つの YAML が構文として正しいこと(`yaml.safe_load`)。ワークフローの中のシェルの検査(`cc` と `-sys` の検索、README の節の検索)を、手元で実行して、期待する結果になること。`cargo build --release --locked --bin conductor` が成功すること。
- **確かめていない:** GitHub Actions 上での実行(リポジトリにまだ載せていないため)。初回の実行で、ワークフローの修正が要る場合がある。リリースのワークフローの実際の実行(タグを付けるまで)。リポジトリの設定(必須のチェックの指定、`v*` タグの保護)は、手動で行う(`quality-gates.md`)。
