# CI/CD Pipeline(u1-core)

上流: nfr-design(security-design、logical-components)、nfr-requirements(tech-stack-decisions)、team.md の実践(GitHub Actions、3つの OS、rustfmt と clippy、行カバレッジ 80%、タグでリリース)。u1-core は library であり、サーバー、コンテナ、クラウドの資源、監視、デプロイ先の環境を持たない。このため、この Unit が負うのは `cicd-pipeline.md` だけである(infrastructure-specification と monitoring-design は、種類(kind)の対象外)。この文書は、CI で何を確かめるかの設計であり、ワークフローのファイルそのものは、後の CI Pipeline のステージで作る。

## 方針

- 実行の場所は GitHub Actions。ローカルでは主な開発環境の OS で確かめ、他の OS は CI で確かめる(team.md)。
- この Unit は単独では配布しない。リリースのバイナリは、u3-cli を含むリリース用のワークフロー(タグで起動)が作る。したがって、この Unit に、デプロイ、環境の昇格、ブルー/グリーン、カナリアはない。

## 段階と確認の対応(stage → gate)

| 段階 | 内容 | 失敗時 | 関わる NFR |
|------|------|--------|------------|
| 1. 整形の確認 | `cargo fmt --all -- --check` | マージをブロック | team.md(Code Style) |
| 2. 静的解析 | `cargo clippy --all-targets -- -D warnings` | マージをブロック | team.md(Code Style) |
| 3. ビルド | `cargo build --locked`(`Cargo.lock` に従う)。あわせて `cargo tree` で依存の一覧を出力し、C などの他言語のビルドを要する依存(`-sys` で終わるクレートや、ビルドスクリプトで C を使うもの)が含まれないことを確かめる | マージをブロック | NFR1.1、NFR2.1 |
| 4. テスト | `cargo test --locked` を、Linux・macOS・Windows の3つで実行する | マージをブロック | NFR3.1、NFR5.1、NFR4.1〜NFR4.7(定義の検証のテスト) |
| 5. カバレッジ | Linux だけで `cargo llvm-cov` により行カバレッジを測り、80% 未満なら失敗にする(他の OS ではテストの成功だけを確かめる。Q2: A) | マージをブロック | NFR5.1(80% の下限) |
| 6. 起動時間の確認 | この Unit の処理(取り込みから結果まで)を、ジョブ10個程度の定義で 100 回繰り返し、時間の分布から p95(昇順の 95 番目)を求める。測定は、テストの中で標準のライブラリの時計を使って行い、追加の道具は使わない。3つの OS で実行する | 基準(20ms)を超えたら、ジョブの要約欄(GitHub Actions の Job Summary)に測定値と基準を書き出して警告とし、失敗にはしない(Q1: A)。Code Generation の実測で、失敗にするかを決める | NFR1.1 |
| 7. 依存の確認(候補) | `cargo-deny`(ライセンス)と `cargo-audit`(脆弱性)。導入は CI Pipeline のステージで決める | — | NFR2.1、依存の方針(security-design) |

## 起動の条件

- `main` への push と、`main` に向けたプルリクエスト。
- パスの絞り込みはしない。「マージ前に CI を通す」(team.md)ために、必須のチェックは、すべての変更で実行する。文書だけの変更でも起動する(パスの絞り込みで起動しないと、必須のチェックが保留のままになり、マージできなくなるため)。
- 単一クレートの配置を前提にせず、ワークスペースの構成(Unit ごとのクレート)は、CI Pipeline のステージで確定する。`rustfmt.toml`、`clippy.toml`、`benches/` も対象に含める。

## ツールチェインとキャッシュ

- Rust は、edition 2024 を使える最新の安定版(D5)。ワークフローで、バージョンを明示して固定する。
- 依存のビルド結果は、`Cargo.lock` のハッシュをキーにしてキャッシュする。キャッシュがなくても、結果は変わらない(再現性を優先する)。

## ロールバック

- 失敗した変更は、`main` で `git revert` して戻す。履歴をそのまま取り込んだ(squash しない)マージコミットを戻すときは、親を指定する(`git revert -m 1 <マージコミット>`)。Bolt のブランチは、履歴をそのまま残して `main` に取り込むため(team.md)、取り込みの単位で戻せる。
- この Unit は、状態を持たず、配布もしないため、データの巻き戻しや、環境の戻しは要らない。

## 秘密情報(secrets)

- この Unit の CI は、秘密情報を必要としない。読み取りだけの権限(`contents: read`)で動かす。
- 依存の取得は、公開のレジストリだけを使う。

## 環境の昇格

- なし。環境は、開発(ローカル)と CI の2つだけで、ステージングや本番は持たない。リリースは、タグで区切る(team.md)。

## 3つの OS 固有の注意

- Windows の主スレッドのスタックは、Linux・macOS より小さい(既定で 1 MiB)。深さ 1000 の入力のテスト(NFR4.7)は、この違いで結果が変わりうるため、3つの OS すべてで実行し、異常終了しないことを確かめる。
- 改行が CRLF の定義ファイルでも、行番号が一致することを、テストに含める(Windows での既定の改行)。
- パスの区切りは、テストで固定の文字列にせず、標準のライブラリの組み立てを使う。

## Assumptions & Open Questions

- ワークフローのファイル(`.github/workflows/`)の作成は、CI Pipeline のステージで行う。この文書は、その入力となる設計である。 [assumption]
- 起動時間の確認(段階 6)の基準 20ms は仮の値であり、Code Generation で実測してから、失敗にするかを決める。 [assumption]
- 行カバレッジの測定に使う `cargo-llvm-cov` が、3つの OS で動くかは、CI Pipeline のステージで確かめる。動かない OS では、Linux だけで測る。 [assumption]
