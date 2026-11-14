# CI/CD Pipeline(u4-converter)

上流: nfr-design(security-design、logical-components)、nfr-requirements(security-requirements、tech-stack-decisions)、team.md の実践(GitHub Actions、3つの OS、rustfmt と clippy、行カバレッジ 80%、タグでリリース)、質問への回答(Q1、Q2)。u4-converter は library であり、サーバー、コンテナ、クラウドの資源、監視、デプロイ先の環境を持たない。この Unit が負うのは `cicd-pipeline.md` だけである(infrastructure-specification と monitoring-design は、種類(kind)の対象外)。この文書は、CI で何を確かめるかの設計であり、ワークフローのファイルは、後の CI Pipeline のステージで、ワークスペースに1つだけ作る。

## 方針

- 実行の場所は GitHub Actions。ローカルでは主な開発環境の OS で確かめ、他の OS は CI で確かめる(team.md)。
- **ワークスペース全体で、1つのワークフロー**にする(Q1: A)。整形・静的解析・ビルド・テストは `--workspace` で、全 Unit をまとめて行う。この Unit 専用の段階は、カバレッジの対象と、暴走のテストの時間の上限だけである。u1-core の cicd-pipeline.md と、同じワークフローを共有する。
- この Unit は、単独では配布しない。リリースのバイナリは、u3-cli を含むリリース用のワークフロー(タグで起動)が作る。したがって、この Unit に、デプロイ、環境の昇格、ブルー/グリーン、カナリアはない。

## 段階と確認の対応(stage → gate)

| 段階 | 内容 | 失敗時 | 関わる NFR |
|------|------|--------|------------|
| 1. 整形の確認 | `cargo fmt --all -- --check` | マージをブロック | team.md(Code Style) |
| 2. 静的解析 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | マージをブロック | team.md(Code Style) |
| 3. ビルド | `cargo build --workspace --locked`。あわせて `cargo tree` で依存の一覧を出力し、C などの他言語のビルドを要する依存(`-sys` で終わるクレートや、ビルドスクリプトで C を使うもの)が含まれないことを確かめる。最小の Rust のバージョン(1.85)でも、この Unit がビルドできることを、1つの OS で確かめる | マージをブロック | NFR2.1 |
| 4. テスト | `cargo test --workspace --locked` を、Linux・macOS・Windows の3つで実行する。往復のテスト(BR1.11)、各 kind のテスト、上限と深さの境界のテスト、キー名のエスケープのテストを含む | マージをブロック | NFR3.1、NFR5.1、NFR4.1〜NFR4.7 |
| 5. 暴走のテストの時間の上限 | 深さ 100,000 の入力、展開で膨らむ YAML の入力のテストは、**各 30 秒**の時間の上限を、テストの中で持つ。上限は、テストを別のスレッドで動かし、結果を待つ時間で測り、超えたら、そのテストを失敗にする(ハングしたスレッドは、テストのプロセスが終わるとき、一緒に終わる)。CI のジョブ全体は、15 分で打ち切る(`timeout-minutes: 15`)(Q2: A) | マージをブロック | NFR4.3、NFR4.4 |
| 6. カバレッジ | Linux だけで `cargo llvm-cov` により、`crates/u4-converter/src/` を含むワークスペースの行カバレッジを測り、80% 未満なら失敗にする(他の OS ではテストの成功だけを確かめる) | マージをブロック | NFR5.1 |
| 7. 依存の確認(候補) | `cargo-deny`(ライセンス)と `cargo-audit`(脆弱性)。導入は CI Pipeline のステージで決める(u1-core の依存の方針と同じ) | — | NFR2.1 |

## 起動の条件

- `main` への push と、`main` に向けたプルリクエスト。
- パスの絞り込みはしない。「マージ前に CI を通す」(team.md)ために、必須のチェックは、すべての変更で実行する。文書だけの変更でも起動する(パスの絞り込みで起動しないと、必須のチェックが保留のままになり、マージできなくなるため)。

## ツールチェインとキャッシュ

- Rust は、edition 2024 を使える最新の安定版。ワークフローで、バージョンを明示して固定する。
- 依存のビルド結果は、`Cargo.lock` のハッシュをキーにしてキャッシュする。キャッシュがなくても、結果は変わらない。

## 3つの OS 固有の注意

- 深い入力のテスト(深さ 100,000)は、Windows のスタックの小さい主スレッドでも、異常終了しないことを確かめる。YAML は、再帰を使わない読み取り(NFR4.4)のため、OS による違いが出ない設計だが、TOML のライブラリの再帰の制限は、3つの OS で確かめる。
- 改行が CRLF の入力でも、変換の結果(往復の等しさ)が変わらないことを、テストに含める。
- パスの区切りは、テストで固定の文字列にせず、標準のライブラリの組み立てを使う。

## ロールバック

- 失敗した変更は、`main` で `git revert` して戻す。履歴をそのまま取り込んだ(squash しない)マージコミットを戻すときは、親を指定する(`git revert -m 1 <マージコミット>`)。
- この Unit は、状態を持たず、配布もしないため、データの巻き戻しや、環境の戻しは要らない。

## 秘密情報(secrets)

- この Unit の CI は、秘密情報を必要としない。読み取りだけの権限(`contents: read`)で動かす。依存の取得は、公開のレジストリだけを使う。

## 環境の昇格

- なし。環境は、開発(ローカル)と CI の2つだけで、ステージングや本番は持たない。リリースは、タグで区切る(team.md)。

## Assumptions & Open Questions

- ワークフローのファイル(`.github/workflows/`)の作成は、CI Pipeline のステージで、ワークスペースに1つだけ行う。この文書は、u1-core の cicd-pipeline.md とあわせて、その入力となる設計である。 [assumption]
- 暴走のテストの時間の上限 30 秒は、仮の値である。Code Generation で実測してから、見直す。 [assumption]
- 行カバレッジの測定に使う `cargo-llvm-cov` が、CI の Linux で動くことは、CI Pipeline のステージで確かめる。 [assumption]
