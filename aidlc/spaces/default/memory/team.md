# Team-Level Rules

> This team's affirmed practices and corrections. Loaded after `org.md` as
> strict-additive guidance.
> Populated by the practices-discovery affirmation gate. Edit at the gate,
> not directly.

## Way of Working

- **Integration branch**: main
- `main` を統合ブランチとし、Bolt のブランチは squash せず、履歴をそのまま残して `main` に取り込む(インタビュー Q1: B)。
- 複数環境を持たないため、リリースはタグで区切り、長命のリリースブランチは作らない。
- 開発者は Rust に不慣れなため、変更を小さく保ち、小さな単位でレビューする。

## Walking Skeleton

- 薄いエンドツーエンドの縦切りを最初に作ることはしない。機能を 1 つずつ順に作る(インタビュー Q2: B)。

## Testing Posture

- **Methodology**: test-after
- **Methodology evidence**: インタビュー Q3 で、チームは test-after(実装してからテストを書く)を選んだ。
- **Ordering**: 各テスト可能な層を実装してから、その層のテストを書いて実行する。
- `cargo test` による単体テストと、バイナリを実行する統合テストを含める。
- 行カバレッジ 80% を下限とする(インタビュー Q4: A)。
- 主な開発環境の OS では、ローカルでテストを実行する。それ以外の OS(Linux・macOS・Windows のうち該当するもの。制約 C3)のテストは、GitHub Actions の CI で実行する。マージ前に CI を通す。

## Guard Policy

<!-- Affirmed by the team. Mode: strict, relaxed, or off. Strict here holds for every intent, whatever one person asks in chat; relaxed or off here applies to every intent whose policy came from its scope, unless project.md or the person sets another; changing this line changes it. A section under the retired Change Control heading, written by an earlier release, is still read. -->

## Deployment

- 単一バイナリ CLI のため、ステージング環境への自動デプロイは該当しない。
- タグを付けると、CI(GitHub Actions)が 3 OS(Linux・macOS・Windows)向けのリリースバイナリを自動でビルドする(インタビュー Q5: A)。

## Code Style

- フォーマッタは `rustfmt`(`cargo fmt`)、リンタは `clippy`(`cargo clippy`)を使う。CI で失敗した場合はマージをブロックする(インタビュー Q6: A)。
- 命名は Rust 標準(関数・変数は snake_case、型は PascalCase、定数は SCREAMING_SNAKE_CASE)とし、プロジェクト独自の命名規則は設けない。
- 設定ファイル(`rustfmt.toml`、`clippy.toml` 等)が必要な場合はリポジトリのルートに置く。
## Forbidden

<!-- Team-specific forbidden patterns -->

## Mandated

<!-- Team-specific mandates -->

## Corrections

<!-- Self-learning loop appends here. -->
