# Team-Level Rules

> This team's affirmed practices and corrections. Loaded after `org.md` as
> strict-additive guidance.
> Populated by the practices-discovery affirmation gate. Edit at the gate,
> not directly.

## Way of Working

- **Integration branch**: main
- `main` を統合ブランチとする。履歴は線形で、マージコミットは無い(すべてのコミットが非マージで、作者は 1 人)。Bolt のブランチは squash せず、履歴をそのまま残して `main` に取り込む(インタビュー Q1)。
- 変更は PR を使わず、直接 `main` にコミットする(インタビュー Q1)。CI は `push`(main)でも `pull_request` でも動き、関門は `push` の CI で事後に効く。
- 長命のブランチは作らない。リモートのブランチは `origin/main` のみ。
- リリースはタグ `v*` で区切る。タグは `main` の上にあり、かつ `crates/cli/Cargo.toml` の版と一致しなければならない(`release.yml` の `verify-tag` が検査)。タグは `v0.1.0` から付いている。
- 機能を足す intent は版の 2 桁目を上げ(v0.9.0、v0.10.0 など)、整理・修正の intent は 3 桁目を上げる(v0.9.1、v0.10.1 など)。機能を足す intent は、すべての機能を作り終えてから版を上げる(インタビュー Q2)。
- 変更は小さく保つ。1 つのコミットは 1 つの小さな単位(型の追加、読み取りと検証、表示、テスト、版の更新など)とする。開発者は Rust に不慣れなため、小さな単位でレビューする。
- コミットの件名は日本語で、対象のクレート(`core:`、`runtime:`、`cli:`、`converter:`)、`docs:`、`aidlc:` を先頭に付ける。版の更新は「版を X にする」の 1 コミットとする。
- MSRV(最小の Rust の版)は 1.100 に固定する。引き上げは明示のコミットで行い、README の記載との一致をテスト(`msrv_consistency.rs`)で保つ。

## Walking Skeleton

- 薄いエンドツーエンドの縦切りを最初に作ることはしない。機能を 1 つずつ順に作る(インタビュー Q2)。
- 履歴もこれに合う。v0.9.0 は、機能(U1 環境変数、U2 graph、U3 `--dry-run`、U4 init)を 1 つずつ、core、runtime、cli の層の順に積み、各機能を README・デモの更新で締め、最後に版を上げている。
- 機能を足す intent では、機能を 1 つずつ作る。順序は関わる部分ごとにまとめる(v0.10.0 の例: 定義の書き方 B1 → B2 → B3、グラフ B4、出力とステート B5 → B6 → B7)(スコープ定義 Q4、インタビュー Q2)。
- 各機能の最後に README とデモを更新し、すべてを作り終えてから版を上げる(スコープ定義 Q6、インタビュー Q2)。

## Testing Posture

- **Methodology**: test-after
- **Methodology evidence**: チームは test-after を続けると決めた(インタビュー Q3)。履歴も整合する。v0.9.0 の U4 では、`create_file` の追加(602d15c)の後にそのテスト(90a27fa)、ひな形の生成(0fbd466)の後にひな形のテスト(5b750f8)が続く。U3 も流れ(a9eb237)の後に流れのテスト(365f45c)と実バイナリのテスト(062c627)が続く。`code-quality-assessment.md` も「チームの方針(test-after、行カバレッジ 80%)と一致する」と記す。
- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。
- テストの種類: 各クレートの `tests/` に統合テスト(core 36、runtime 16、cli 27、converter 2 のファイル。`tests/` の直下の `.rs` の数)。単体テストは各モジュールの `mod tests` と、`crates/cli/src/tests/`(12 ファイル)、`crates/converter/src/tests/`(4 ファイル)(どちらも `mod.rs` と補助を含むすべての `.rs` の数)。`#[test]` は合計 1,472(`grep -rE '^\s*#\[test\]' crates --include='*.rs'` の行の数)。
- 流れのテストは偽の `Runtime`/`Sink`/`Env`/`Signals` を注入する。実バイナリのテストは `conductor` とテスト専用の補助バイナリを動かす。非機能のテスト(`nfr_*.rs`、`*_perf.rs`)、網羅のテスト(`ALL` 配列)、参照出力との一致(`graph_layout_oracle.rs`)、文書との一致(`src/tests/docs.rs`、`msrv_consistency.rs`)、デモのテスト(`unix_demo.rs`、`windows_demo.rs`)を含む。テスト名は日本語。
- テストの都合で本番のコードを曲げない。テスト専用の引数・`pub`・関数を本番に残さず、流れのテストは公開の入口と差し替え口で書く(インタビュー Q6)。
- 行カバレッジ 80% を下限とする。CI の `coverage` ジョブが `cargo llvm-cov --fail-under-lines 80` で強制する(テスト専用の補助バイナリは除外)。
- 主な開発環境の OS は macOS で、テストはローカルの macOS で実行する。他の OS のテストは GitHub Actions の CI(`test`: ubuntu、macos、windows)で確認する。マージ前に CI を通す(インタビュー Q3)。
- 最小の Rust(1.100.0)でもテストを実行する(`msrv-test`)。
- 起動の性能(`nfr_perf.rs`)は `startup-bench` で測るが、警告のみで失敗にしない(`STARTUP_BENCH_ENFORCE=false`、`continue-on-error`)。
- 機能を足す intent でも、test-after と 80% の下限、単体テストと実バイナリを動かす統合テストを続ける(インタビュー Q3)。
- ステートの中身を調べるテスト(`nfr_content.rs`)は、開始と終了の時刻だけを許す(v0.10.0 の B6 で直した)。それ以外(コマンド・環境変数・出力)は検査する(インタビュー Q7)。

## Guard Policy

<!-- Affirmed by the team. Mode: strict, relaxed, or off. Strict here holds for every intent, whatever one person asks in chat; relaxed or off here applies to every intent whose policy came from its scope, unless project.md or the person sets another; changing this line changes it. A section under the retired Change Control heading, written by an earlier release, is still read. -->

## Deployment

- 単一バイナリ CLI のため、ステージング環境への自動デプロイは該当しない。環境は 1 つ(利用者の手元)で、リリースはタグで区切る。
- タグ `v*` を付けると、`release.yml` が動く。流れは `verify-tag`(main 上かつ版の一致)、`checks`(`ci.yml` を再利用して、マージの関門と同じ検査をすべて通す)、`build`(4 対象)、`draft-release` の順。
- 4 対象は、`x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`。配布するのは `conductor` のみで、テスト専用の補助バイナリは含めない。成果物は tar.gz(Windows は zip)と `SHA256SUMS`。
- **リリースは下書き(draft)として作られ、人が確認して公開する。** 自動で公開はしない。バージョンに `-` を含むタグは prerelease になる。
- 展開したバイナリの `--version` で smoke test を行う(Intel macOS を除く。クロスビルドで動作の確認なし。リリースの注記に明記)。Linux は静的リンクも確認する。
- 戻し方(推論。人の確認を経ていない): 公開前は下書きを消してタグを付け直す。公開後は前の版のリリースが残るので、利用者は前の版のバイナリに戻せる。
- `cargo audit`(`audit.yml`)は PR と毎週月曜に走るが、警告のみでマージを止めない。
- 署名・公証、Intel macOS の動作確認、`audit` の強制、`startup-bench` の強制は見送る(インタビュー Q4。署名の見送りは 2026-11-28 にも確かめた)。

## Code Style

- フォーマッタは `rustfmt`(`cargo fmt --all -- --check`)、リンタは `clippy`(`cargo clippy --workspace --all-targets --locked -- -D warnings`)を使う。CI の `lint` で失敗したらマージをブロックする。
- 設定ファイル(`rustfmt.toml`、`clippy.toml`)は無く、既定を使う。lint の抑止(`#[allow]`)は `platform/unix.rs` の `unsafe_code` の 1 つだけ。必要になったら設定をリポジトリのルートに置く。
- 命名は Rust 標準(関数・変数は snake_case、型は PascalCase、定数は SCREAMING_SNAKE_CASE)。プロジェクト独自の命名規則は設けない。
- ドキュメントコメントとテスト名は日本語で書く。ドキュメントコメントは規則の番号(BR/NFR/FR)に対応させる。
- 依存は `Cargo.lock` を追跡し、CI は常に `--locked`。C のコンパイルを要する依存と `-sys` クレート(`linux-raw-sys`・`windows-sys` を除く)を持たない(CI の `build-check` が検査)。
- `unsafe` は runtime の `platform/unix.rs` の 1 か所だけ。core・converter・cli は `#![forbid(unsafe_code)]`。`cfg(unix)`/`cfg(windows)` は runtime と cli の `platform` の外に置かない(テストの中を除く)。ネットワークのライブラリを使わない(`nfr_source.rs` が検査)。
- 入出力の副作用は runtime と cli の `deps`・`platform` に閉じ込め、core は純粋に保つ。
- エラーの表示に入力の値を含めない。ステートには開始と終了の時刻だけを書くことを認め、コマンド・環境変数・ジョブの出力は書かない(インタビュー Q7、`nfr_content.rs` が検査)。
- README は必須語句の検査(`docs-check`)つきで保つ。使い方の表は `args/`(`mod.rs` の文法の表と `options.rs` のオプションの表)、`report/messages.rs`、README の 3 か所を揃える。
- 機能を足す intent でも、新しい外部ライブラリ(依存)を追加しない。必要な処理(時刻の書式、DOT の整形、`watch` の表示し直しなど)は自前で書く。既存の規則(`project.md` の NEVER)をそのまま使う(インタビュー Q5)。

## Forbidden

<!-- Team-specific forbidden patterns -->

## Mandated

<!-- Team-specific mandates -->

## Corrections

<!-- Self-learning loop appends here. -->
