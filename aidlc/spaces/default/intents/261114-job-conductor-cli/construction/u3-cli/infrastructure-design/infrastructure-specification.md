# Infrastructure Specification(u3-cli)

上流: nfr-design(performance、security、scalability、reliability、observability の各設計、logical-components)、nfr-requirements(tech-stack-decisions、security-requirements の NFR4.8)、team.md の実践(GitHub Actions、3つの OS、タグでリリース)、質問への回答(Q1〜Q3)。u3-cli は、`conductor` の実行ファイルそのもので、**サーバー、コンテナ、クラウドの資源、データベース、ネットワークの構成を持たない**。利用者の手元(または、スケジューラのホスト)で動く、単一のバイナリである。この文書でいう「インフラ」は、**配布(リリース)の仕組み**と、それを動かす **CI(GitHub Actions)**である。

## Deployment

| Facet | Choice | Rationale |
|-------|--------|-----------|
| 計算の形(compute) | なし。利用者の環境で、1回で終わるプロセスとして動く(サーバー、コンテナ、サーバーレス、VM は使わない) | `conductor` は、スケジューラが呼ぶ、常駐しないコマンド(requirements、NFR2) |
| ネットワーク(networking) | なし。`conductor` は、ネットワークに接続しない(NFR4.5)。配布のためのダウンロードは、利用者が、GitHub から行う | セキュリティ(NFR4.5) |
| 保存先(storage) | 利用者のローカルのファイルシステム: 定義ファイル、ステートファイル(定義ファイルと同じディレクトリ)。配布物は、GitHub のリリース(Releases)に置く(Q1: A) | u1-core・u2-runtime の設計 |
| 環境(environments) | 開発(ローカル)と CI の2つ。ステージング・本番は持たない。リリースは、タグで区切る | team.md |
| 構成管理(IaC) | `.github/workflows/` の、ワークフローのファイル(YAML)だけ。クラウドの IaC(CDK・Terraform など)はない | 配布と CI だけ |
| 配布の対象(Q2: A) | `x86_64-unknown-linux-musl`(Linux x86_64、静的リンク)、`aarch64-apple-darwin`(macOS の Apple シリコン)、`x86_64-apple-darwin`(macOS の Intel)、`x86_64-pc-windows-msvc`(Windows x86_64)の4つ | 外部のランタイムに依存しない単一のバイナリ(NFR2)。Linux は、静的リンクで、OS の共有ライブラリに依存しない |
| 配布の形式(Q3: A) | UNIX 系は `.tar.gz`、Windows は `.zip`。中に、実行ファイル `conductor`(Windows は `conductor.exe`)と `LICENSE`。チェックサム(SHA-256)は、まとめたファイルごとに、1つのファイル `SHA256SUMS` にまとめて、添付する | 利用者が、転送の破損を確かめられる(NFR4.8) |
| 配布先(Q1: A) | GitHub のリリース。crates.io へは、公開しない(`publish = false`) | この版の範囲 |
| 資源の大きさ(sizing) | CI のランナーは、GitHub が提供する標準のもの(`ubuntu-latest`、`macos-latest`、`windows-latest`)。特別な大きさの指定はしない | ビルドとテストは、小さい |

## Infrastructure Services

| Service | Role | Configuration | Notes |
|---------|------|---------------|-------|
| GitHub Actions | CI と、リリースのビルドの実行 | `.github/workflows/ci.yml`(プッシュとプルリクエスト)、`.github/workflows/release.yml`(タグ `v*` のプッシュ)。権限は、`contents: read` を既定とし、リリースのジョブだけ `contents: write` | 秘密情報は、`GITHUB_TOKEN`(自動で用意される)だけ |
| GitHub Releases | 配布物(アーカイブと `SHA256SUMS`)の保管と配布 | タグごとに1つのリリース。添付: 4つのアーカイブと `SHA256SUMS` | 取り下げは、リリースの削除と、タグの削除・付け直し(cicd-pipeline のロールバック) |

- データベース、キャッシュ、メッセージキュー、検索、CDN、DNS、ロードバランサーは、使わない。

## Shared Infrastructure

| Shared Resource | Owner Unit | Consumer Units | Access Boundary |
|-----------------|------------|----------------|-----------------|
| ワークフロー `ci.yml`(整形、静的解析、ビルド、テスト、カバレッジ、測定) | u3-cli(CI Pipeline のステージで作成。u1-core・u2-runtime・u4-converter の cicd-pipeline.md の設計を、1つのワークフローに合わせる) | u1-core、u2-runtime、u4-converter | リポジトリの `.github/workflows/`。変更は、プルリクエストで、CI を通す |
| ワークフロー `release.yml`(タグでのリリース) | u3-cli | (配布するのは u3-cli のみ) | タグのプッシュと、`contents: write` の権限を持つジョブだけ |
| `Cargo.lock`(依存の固定) | ワークスペース | 全 Unit | `--locked` でのビルド |

## Assumptions & Open Questions

- ワークフローのファイルの作成は、CI Pipeline・Deployment Pipeline のステージで行う。この文書は、その入力となる設計である。 [assumption]
- Intel の macOS 向けの実行ファイル(`x86_64-apple-darwin`)は、Apple シリコンの `macos-latest` のランナーで、ターゲットを追加して、クロスビルドする想定である(ランナーの提供状況は、CI Pipeline のステージで確かめる)。 [assumption]
- コードサイニング(署名)と、Apple の公証(notarization)は、この版では行わない。macOS で、ダウンロードした実行ファイルを動かすとき、OS が警告する場合がある(利用者向けの文書に、回避の手順を書く)。 [assumption]
