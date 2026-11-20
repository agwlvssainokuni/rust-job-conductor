# Team-Level Rules

> This team's affirmed practices and corrections. Loaded after `org.md` as
> strict-additive guidance.
> Populated by the practices-discovery affirmation gate. Edit at the gate,
> not directly.

## Way of Working

- **Integration branch**: main
- `main` を統合ブランチとする。履歴は線形で、マージコミットは無い(297 コミットすべてが非マージ)。Bolt のブランチは squash せず、履歴をそのまま残して `main` に取り込む(基準: 前回のインタビュー Q1: B)。
- 長命のブランチは作らない。リモートのブランチは `origin/main` のみ。
- リリースはタグ `v*` で区切る。タグは `main` の上にあり、かつ `crates/cli/Cargo.toml` の版と一致しなければならない(`release.yml` の `verify-tag` が検査)。タグは `v0.1.0` から `v0.8.0` まで付いている。
- 変更は小さく保つ。1 つのコミットは 1 つの小さな単位(型の追加、読み取りと検証、表示、テスト、版の更新など)とする。開発者は Rust に不慣れなため、小さな単位でレビューする(基準)。
- MSRV(最小の Rust の版)は 1.100 に固定する。引き上げは明示のコミットで行い、README の記載との一致をテスト(`msrv_consistency.rs`)で保つ。
- 変更は PR を使わず、直接 `main` にコミットする(インタビュー Q1)。CI は `push`(main)でも `pull_request` でも動く。

## Walking Skeleton

- 薄いエンドツーエンドの縦切りを最初に作ることはしない。機能を 1 つずつ順に作る(基準: 前回のインタビュー Q2: B)。
- 証拠は基準と矛盾しない。v0.5.0 から v0.8.0 の履歴は、1 つの機能(run-id、完了後のステート、リトライ)を、core、runtime、cli の層の順に積み、最後に README・デモ・版の更新で締めている。
- 今回の intent(v0.9.0 の拡張)も、この方針を続け、機能を 1 つずつ順に作る(インタビュー Q2)。

## Testing Posture

- **Methodology**: test-after
- **Methodology evidence**: 基準(前回のインタビュー Q3)でチームは test-after を選んだ。履歴も整合する。たとえば `[job.retry]` では、型・読み取りの実装(d554ece、445d478)の後に、テストを追加するコミット(8ec98b5)が続き、実行の流れ(19d7b1a)の後に README のテスト(de4e3ad)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。
- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。
- テストの種類: 各クレートの `tests/` に統合テスト(core 10、runtime 13、cli 21、converter 5 のファイル)。流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor-cli-test-helper` を使う。非機能のテスト(`nfr_*.rs`)、網羅のテスト(`ALL` 配列)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。
- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。
- ローカルでは、主な開発環境の OS でテストを実行する。それ以外の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で実行する。マージ前に CI を通す。
- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。
- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。
- 今回の機能にも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。
- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS は CI で確認する(インタビュー Q6)。

## Guard Policy

<!-- Affirmed by the team. Mode: strict, relaxed, or off. Strict here holds for every intent, whatever one person asks in chat; relaxed or off here applies to every intent whose policy came from its scope, unless project.md or the person sets another; changing this line changes it. A section under the retired Change Control heading, written by an earlier release, is still read. -->

## Deployment

- 単一バイナリ CLI のため、ステージング環境への自動デプロイは該当しない。環境は 1 つ(利用者の手元)で、リリースはタグで区切る。
- タグ `v*` を付けると、`release.yml` が動く。流れは `verify-tag`(main 上かつ版の一致)、`checks`(`ci.yml` を再利用して、マージの関門と同じ検査をすべて通す)、`build`(4 対象)、`draft-release` の順。
- 4 対象は、`x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`。配布するのは `conductor` のみで、テスト専用の補助バイナリは含めない。成果物は tar.gz(Windows は zip)と `SHA256SUMS`。
- **リリースは下書き(draft)として作られ、人が確認して公開する。** 自動で公開はしない。バージョンに `-` を含むタグは prerelease になる。
- 展開したバイナリの `--version` で smoke test を行う(Intel macOS を除く。クロスビルドで動作の確認なし。リリースの注記に明記)。Linux は静的リンクも確認する。
- 署名・公証はしない。
- `cargo audit`(`audit.yml`)は PR と毎週月曜に走るが、警告のみでマージを止めない。
- 署名・公証、Intel macOS の動作確認、`audit` の強制、`startup-bench` の強制は、今回も見送る(インタビュー Q5)。

## Code Style

- フォーマッタは `rustfmt`(`cargo fmt --all -- --check`)、リンタは `clippy`(`cargo clippy --workspace --all-targets --locked -- -D warnings`)を使う。CI の `lint` で失敗したらマージをブロックする(基準: 前回のインタビュー Q6: A)。
- 設定ファイル(`rustfmt.toml`、`clippy.toml`)は無く、既定を使う。必要になったらリポジトリのルートに置く。
- 命名は Rust 標準(関数・変数は snake_case、型は PascalCase、定数は SCREAMING_SNAKE_CASE)。プロジェクト独自の命名規則は設けない。
- ドキュメントコメントとテスト名は日本語で書く。ドキュメントコメントは規則の番号(BR/NFR/FR)に対応させる。
- 依存は `Cargo.lock` を追跡し、CI は常に `--locked`。C のコンパイルを要する依存と `-sys` クレートを持たない(CI の `build-check` が検査)。
- `unsafe` は runtime の `platform/unix.rs` の 1 か所だけ。cli は `#![forbid(unsafe_code)]`。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない。ネットワークのライブラリを使わない(`nfr_source.rs` が検査)。
- README は必須語句の検査(`docs-check`)つきで保つ。
- 今回の機能では、新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。
## Forbidden

<!-- Team-specific forbidden patterns -->

## Mandated

<!-- Team-specific mandates -->

## Corrections

<!-- Self-learning loop appends here. -->
