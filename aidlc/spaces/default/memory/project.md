# Project-Level Rules

> Project-specific specialisation and corrections. Loaded after `org.md` and
> `team.md` as strict-additive guidance.
> Populated by practices-discovery and the self-learning loop.
>
> Use sparingly: most teams don't need a project layer. Reach for it
> only when this specific project needs stable, durable guidance beyond the
> team practice (for example, package-specific release checks or an additional
> regression suite for a legacy component).

## Way of Working

<!-- Project-specific specialisation. Example: -->
<!-- This monorepo requires package-scoped branch names and a package owner -->
<!-- review in addition to the team's normal merge policy. -->
<!-- A project that merges into another branch than team.md names says so on its own line, read before team.md's: -->
<!-- - **Integration branch**: develop -->

## Walking Skeleton

<!-- Project-specific specialisation. Example: -->
<!-- The walking skeleton must exercise the legacy service adapter as well -->
<!-- as the new service boundary. -->

## Testing Posture

<!-- Project-specific specialisation. -->

## Guard Policy

<!-- Project-specific. Mode: strict, relaxed, or off. Strict here holds for every intent, whatever one person asks in chat; relaxed or off here applies to every intent whose policy came from its scope, unless the person sets another; changing this line changes it. A section under the retired Change Control heading, written by an earlier release, is still read. -->

## Deployment

<!-- Project-specific specialisation. -->

## Code Style

<!-- Project-specific specialisation. -->

## Tech Stack

<!-- Technology choices locked for this project. -->

## Decided

<!-- Decisions made in earlier stages that should not be re-asked. -->
<!-- Format: DECIDED: [decision] (Stage [slug], [date]) -->

## Scope Overrides

<!-- Custom scope rules for this project. -->

## Forbidden

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: - NEVER [behavior] (affirmed [date]) -->
<!-- Example: - NEVER throw exceptions across service layer boundaries (affirmed 2026-05-17) -->

- NEVER C のコンパイルを要する依存や `-sys` クレートを実行時の依存に加えない(CI の `build-check`)。 (affirmed 2026-11-20)

- NEVER `unsafe` を runtime の `platform/unix.rs` の外に書かない。cli では `unsafe` を禁止する(`nfr_source.rs`)。 (affirmed 2026-11-20)

- NEVER `cfg(unix)`/`cfg(windows)` を `platform` の外に置かない(`nfr_source.rs`)。 (affirmed 2026-11-20)

- NEVER エラーの表示に入力の値を含めない。ステートに環境変数を書かない(`code-quality-assessment.md` の既存の性質)。 (affirmed 2026-11-20)

- NEVER テスト専用の補助バイナリをリリースの成果物に含めない(`release.yml`)。 (affirmed 2026-11-20)

- NEVER 今回の作業で新しい外部ライブラリ(依存)を追加しない。`.env` の読み込みとグラフの描画は自前で書く(インタビュー Q4)。 (affirmed 2026-11-20)

- NEVER テストの都合で本番のコードを曲げない(テスト専用の引数・`pub`・関数を本番に残さない)。 (affirmed 2026-11-25)

## Mandated

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: - ALWAYS [behavior] (affirmed [date]) -->
<!-- Example: - ALWAYS use Result<T,E> for fallible operations in service layer (affirmed 2026-05-17) -->

- ALWAYS 行カバレッジ 80% 以上を保つ(CI の `coverage` ジョブ。`--fail-under-lines 80`)。 (affirmed 2026-11-20)

- ALWAYS 最小の Rust(MSRV 1.100)でビルドとテストが通ることを保ち、引き上げるときは README の記載と一致させる(`build-check`、`msrv-test`、`msrv_consistency.rs`)。 (affirmed 2026-11-20)

- ALWAYS リリースのタグは `main` の上に付け、`crates/cli/Cargo.toml` の版と一致させ、リリースは下書きとして作って人が確認してから公開する(`release.yml`)。 (affirmed 2026-11-20)

- ALWAYS `cargo fmt` と `cargo clippy -D warnings` を通してからマージする(CI の `lint`)。 (affirmed 2026-11-20)

- ALWAYS 変更は小さな単位に保ち、小さな単位でレビューする(チームの慣行)。 (affirmed 2026-11-20)

## Corrections

<!-- Project-specific corrections from human feedback. -->
<!-- Format: NEVER/ALWAYS [behavior] (learned [date]) -->
