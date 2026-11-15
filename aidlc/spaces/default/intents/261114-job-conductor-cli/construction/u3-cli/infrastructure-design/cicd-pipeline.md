# CI/CD Pipeline(u3-cli)

上流: nfr-design(各設計、logical-components)、nfr-requirements(tech-stack-decisions、security-requirements の NFR4.8)、infrastructure-specification、monitoring-design、team.md の実践(GitHub Actions、3つの OS、rustfmt と clippy、行カバレッジ 80%、タグでリリース、`main` に履歴を残して取り込む)、質問への回答(Q1〜Q3)。他の Unit(u1-core、u2-runtime、u4-converter)の cicd-pipeline.md が定めた、段階と基準を、**ワークスペース全体の、1つの CI ワークフロー**に合わせ、この Unit が、リリースのワークフローを足す。ワークフローのファイルそのものは、CI Pipeline・Deployment Pipeline のステージで作る。

## 方針

- 実行の場所は GitHub Actions。ローカルでは主な開発環境の OS で確かめ、他の OS は CI で確かめる(team.md)。
- ワークフローは2つ: `ci.yml`(プッシュとプルリクエスト。全 Unit で共有する)、`release.yml`(タグ `v*` のプッシュ。この Unit が所有する)。
- 秘密情報は、`GITHUB_TOKEN` だけ。権限は、`contents: read` を既定とし、リリースのジョブだけ `contents: write`。

## CI のジョブの構成(必須のチェックにする単位)

| ジョブ名 | 実行の場所 | 内容 | 失敗時 |
|----------|------------|------|--------|
| `lint` | Linux | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` | マージをブロック |
| `build-check` | Linux、Windows | `cargo build --workspace --locked`、最小の Rust のバージョン(1.85)でのビルド(各パッケージ、開発用の依存を除く)、`cargo tree -e normal,build` に、`cc`・`cmake`・`pkg-config`・`bindgen` が現れないこと。検索の検査(Linux): この Unit の `src/` に、`unsafe` と `low_level`、`std::net`、ネットワークのライブラリがないこと。他の Unit の検索の検査も、同じジョブで行う | マージをブロック |
| `test` | Linux、macOS、Windows(OS のマトリクス) | `cargo test --workspace --locked`(1回の実行。OS 固有のテストも同じ実行の中)。プロセスを起動するテストは、各 30 秒の時間の上限。`timeout-minutes: 15`。バイナリを起動する少数のテスト(`conductor --version`、`run` の終了コード、`convert` の標準出力の分け方)を含む | マージをブロック |
| `startup-bench` | Linux、macOS、Windows | `conductor` 自身のオーバーヘッド(起動前 + ジョブのあと)を、20 回測って p95 を出す(リリースのビルド)。`pipeline` のジョブ1つあたりと、ジョブ 1,000 個の定義の時間も記録する | 配分の確定までは警告(ジョブの要約欄)。確定のあと、`run` の合計が 100ms を超えたら失敗。必須のチェックにしない(確定までの間) |
| `coverage` | Linux | `cargo llvm-cov` で、各 Unit のパッケージの行カバレッジ(Unit ごと)を測り、80% 未満なら失敗。補助の実行ファイルのソースは除外。`cfg(windows)` のコードは、Windows の `test` の成功で確かめる。`timeout-minutes: 20`(仮) | マージをブロック |
| `docs-check` | Linux | 利用者向けの文書(README)に、保証の範囲(ジョブの終了と記録のあいだに落ちると、ジョブが再実行されうること)、終了コードの表、macOS での警告の回避の手順、SHA-256 の確認の手順が含まれていること(見出しの検索)。文書の作成は、Code Generation のステージ | マージをブロック |
| `deps`(候補) | Linux | `cargo-deny`(ライセンス)と `cargo-audit`(脆弱性)。導入は CI Pipeline のステージで決める | — |

- **必須のチェック:** `lint`、`build-check`、`test`(OS ごとの3つ)、`coverage`、`docs-check`。パスの絞り込みはしない(文書だけの変更でも起動する)。

## リリースのワークフロー(`release.yml`。タグ `v*`。NFR4.8)

| 段階 | 内容 |
|------|------|
| 1. 検査 | リリースの前に、`ci.yml` と同じ検査(`lint`、`test`)を、タグのコミットで実行する(失敗したら、リリースしない) |
| 2. ビルド(マトリクス: 4つの対象) | `cargo build --release --locked -p conductor-cli --target <対象>`。対象: `x86_64-unknown-linux-musl`(Linux。`musl-tools` を導入)、`aarch64-apple-darwin`、`x86_64-apple-darwin`(Apple シリコンのランナーで、ターゲットを追加してビルド)、`x86_64-pc-windows-msvc`。補助の実行ファイル(テスト用の bin)は、`-p conductor-cli` のビルドに含まれない |
| 3. 動作の確認(スモークテスト) | ビルドした実行ファイルで、`conductor --version`、`conductor convert`(小さな定義)を実行し、終了コード 0 と、期待する出力を確かめる(実行できるランナー上の対象のみ。クロスビルドの `x86_64-apple-darwin` は、Apple シリコンで、Rosetta がある場合に実行。なければ、確認を省き、記録する) |
| 4. まとめ | UNIX 系は `.tar.gz`、Windows は `.zip`。中に `conductor`(`conductor.exe`)と `LICENSE`。名前は `conductor-<版>-<対象>.<拡張子>` |
| 5. チェックサム | まとめたファイルごとに SHA-256 を求め、1つのファイル `SHA256SUMS` に書く |
| 6. 公開 | `gh release create`(`contents: write` のジョブ)で、タグに対応する GitHub のリリースを作り、4つのまとめたファイルと `SHA256SUMS` を添付する。リリースの本文は、タグの注釈と、変更の一覧(`git log` から) |

- **版の管理:** 版は、`Cargo.toml` の `version` と、タグ(`v0.1.0`)を揃える。タグの版と、`conductor-cli` の `version` が一致しなければ、リリースを失敗にする。
- **署名:** この版では、コードサイニングと公証を行わない。チェックサムは、転送の破損の検知まで(配布元の改ざんは防げない)。

## 起動の条件

- `ci.yml`: `main` への push と、`main` に向けたプルリクエスト。
- `release.yml`: タグ `v*` の push だけ。

## ツールチェインとキャッシュ

- Rust は、edition 2024 を使える最新の安定版(ワークフローで固定する)。依存のビルド結果は、`Cargo.lock` のハッシュをキーにしてキャッシュする(キャッシュがなくても、結果は変わらない)。リリースのビルドは、キャッシュなしでも再現できる(`--locked`)。

## ロールバック

- **CI の失敗した変更:** `main` で `git revert` して戻す。履歴を残して取り込んだ(squash しない)取り込みは、マージコミットがあれば `git revert -m 1 <マージコミット>`、なければ、コミットの範囲を、新しい順に revert する。取り込みの形は、CI Pipeline のステージで決める。
- **リリースの取り下げ:** 問題のあるリリースは、GitHub のリリースを削除(または、下書きに戻す)し、タグを削除する。修正後に、新しい版のタグを付けて、再リリースする(同じタグの付け直しは、利用者のダウンロードを混乱させるため、行わない)。利用者は、直前の版のリリースから、実行ファイルを取り直す。
- **配布物は、状態を持たない**(ステートファイルは、利用者の環境で、版の入った形式で持つ)。データの巻き戻しは要らない。

## 秘密情報(secrets)

- `GITHUB_TOKEN` だけ(自動で用意される。リリースのジョブに限り `contents: write`)。署名の鍵、レジストリのトークンは、持たない(署名もレジストリへの公開もしないため)。

## 環境の昇格

- なし。開発(ローカル)と CI の2つで、ステージングや本番は持たない。リリースは、タグで区切る(team.md)。

## Assumptions & Open Questions

- ワークフローのファイルの作成は、CI Pipeline・Deployment Pipeline のステージで行う。他の Unit の cicd-pipeline.md の、段階の番号と、ここのジョブ名の対応は、その作成の中で、整理する。 [assumption]
- Intel の macOS 向けのクロスビルドの実行確認(Rosetta)は、ランナーの提供状況による。確認できない場合は、記録して、リリースを続ける。 [assumption]
- 起動時間の基準(`startup-bench`)は、配分の確定のあと、失敗にする(それまでは、必須のチェックにしない)。 [assumption]
- README(利用者向けの文書)の作成は、Code Generation のステージで行い、`docs-check` は、その内容を確かめる。 [assumption]
