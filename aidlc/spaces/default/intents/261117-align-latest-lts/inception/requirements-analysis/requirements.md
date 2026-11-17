# Requirements — 最新の LTS に合わせたリファクタリング

## Sources

- [desc] Initial description: 「最新のLTSに合わせてリファクタリング」
- [scope] Workflow-selected scope: refactor
- [Q1]〜[Q4]: `requirements-analysis-questions.md` の回答(Q1: A、Q2: B、Q3: A、Q4: A)
- 既存コードの調査: `aidlc/spaces/default/codekb/rust-job-conductor/` の `business-overview.md`、`architecture.md`、`code-structure.md`(技術的な版情報は `technology-stack.md`、`dependencies.md`)

## Intent analysis

- 目的: このプロジェクト(4 クレートの Cargo ワークスペースで、`conductor` バイナリを作る Rust 製 CLI)を、最新の Rust の安定版に合わせる。[desc]
- 「LTS」の解釈: Rust には LTS チャンネルがない(`technology-stack.md` に LTS の記述はない)ため、Rust ツールチェーンの最新の安定版を基準とする。[Q1]
- 現状: edition 2024、`rust-version = "1.85"`(`Cargo.toml`、`ci.yml` の 2 か所、`README.md` の 4 か所に記載)。(`technology-stack.md`)
- 方針: 版を上げ、それに合わせてコードを現代的な書き方に整理する。利用者から見える動作は変えない。[Q2][Q4]

## Functional requirements

### FR1 Rust の版の引き上げ [Q1][Q3]

- FR1.1 `Cargo.toml` の `[workspace.package]` の `rust-version`(MSRV)を、実装時点の最新の安定版に引き上げる。合格基準: 引き上げ後の版でビルドとテストが通る。
- FR1.2 MSRV を書いている 4 か所(`Cargo.toml`、`ci.yml` の `build-check` と `msrv-test`、`README.md` のビルド要件)を同じ版に揃える。合格基準: 4 か所の版が一致する。
- FR1.3 CI で版を固定しているジョブ(`build-check`、`msrv-test`)は新しい MSRV を使う。`@stable` を使うジョブはそのままにする。合格基準: 固定ジョブの Rust の版が `rust-version` と一致する。
- FR1.4 版の引き上げに伴い `Cargo.lock` の更新が必要になった場合は、版の更新に必要な範囲に限って更新する。合格基準: `cargo build --locked` が通る。

### FR2 コードの現代化 [Q2]

- FR2.1 新しい rustc と clippy が出す警告をすべて解消する。合格基準: `cargo clippy -D warnings` が警告なしで通る。
- FR2.2 非推奨になった API を置き換える。合格基準: 非推奨の警告が出ない。
- FR2.3 新しい MSRV で使える言語機能と標準ライブラリの API に、コードを書き換えてよい。ただし、読みやすくなる、または記述が短くなる場合に限る。合格基準: 書き換えた箇所で、元の挙動と同じことをテストが確認している。
- FR2.4 変更は小さな単位に分け、各単位でテストが通る状態を保つ。(`team.md`: 開発者は Rust に不慣れなため、変更を小さく保つ)

### FR3 利用者から見える動作の維持 [Q4]

- FR3.1 `conductor` コマンドの出力と終了コードを変えない。合格基準: 既存のバイナリ実行の統合テストが変更なしで通る。
- FR3.2 設定ファイルの形式を変えない。合格基準: 既存の設定ファイルを読み込むテストが変更なしで通る。

## Non-functional requirements

- NFR1 `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test` が、CI の 3 OS(Linux・macOS・Windows)で通る。(`team.md` のコードスタイルとテストの方針)
- NFR2 行カバレッジ 80% の下限を維持する。(`team.md`、`ci.yml` の `coverage` ジョブ)
- NFR3 `-sys` クレートと C のビルド依存を増やさない。合格基準: `build-check` の検査が通る。(`dependencies.md`)
- NFR4 新しい MSRV でのビルド(`build-check`)とテスト(`msrv-test`)が通る。

## Constraints

- C1 利用者から見える動作は変えない。[Q4]
- C2 リリースはタグで区切り、長命のリリースブランチは作らない。(`team.md`)
- C3 エディションは現状の 2024 のまま。(`technology-stack.md`)

## Assumptions

- [assumption] 「最新の安定版」は、実装時点で公開されている最新の安定版とする。スキャン時のローカルは rustc/cargo 1.100.0 だが、これは一度きりの観測で、確定した値ではない。
- [assumption] OS イメージ(ubuntu-24.04、windows-2022、macos-latest)の更新は、Q1 で選ばれていないため対象外。
- [assumption] 依存クレートの更新も、Q1 で選ばれていないため対象外。新しい MSRV やコード修正のために必要になった場合だけ、最小限に更新する。

## Out of scope

- CI・リリースの OS イメージの更新(Q1: B・D は選ばれていない)
- 依存クレートの一括更新、Dependabot などの導入
- 新機能の追加、利用者から見える動作の変更
- `rust-toolchain` ファイルや `rustfmt.toml` などの新しい設定ファイルの導入

## Open questions

- MSRV の具体的な版(実装時点の最新の安定版)は、Functional Design 以降で確定する。
- 書き換える範囲(FR2.3)は、新しい MSRV で使える機能を洗い出したうえで、コードの設計の段階で絞る。
