# 証拠

調べた者: 再実行のリード(pipeline-deploy)。協力者は無効(単独の作業)。リードは上記のファイルと履歴を調べ、解釈を推論した。決定の部分は人の回答。ビルドとテストは実行していない。AI-DLC 自身のファイルは調べていない。

## 調べたもの

- 基準: `aidlc/spaces/default/memory/team.md`(5 つの節)。
- git の履歴(297 コミット)、タグ、ブランチ。
- `.github/workflows/ci.yml`、`audit.yml`、`release.yml`。
- `Cargo.toml`(ワークスペースと 4 クレート)、`rustfmt.toml`/`clippy.toml` の有無。
- `crates/*/tests/` の配置。
- コード知識ベース: `aidlc/spaces/default/codekb/rust-job-conductor/` の `code-quality-assessment.md`、`technology-stack.md`(他の artifact は、上の事実に必要な範囲で参照)。

## 事実と、それが支える実践

| 実践 | 事実 | 出所 |
|------|------|------|
| `main` が統合ブランチ、長命のブランチ無し | ブランチは `main` と `origin/main` のみ。マージコミット 0 件。作者は 1 人で 297 コミット | `git branch -a`、`git log --merges`、`git log` |
| リリースはタグで区切る | タグ `v0.1.0` から `v0.8.0` | `git tag` |
| 変更は小さい | `[job.retry]` は型、エラー種類、読み取り、テスト、判断、待機、表示、流れ、README テスト、版、デモの 11 前後のコミットに分かれる | `git log -- crates .github`(d554ece から 6e882de) |
| MSRV の方針 | `rust-version = "1.100"`。f724dec で 1.85 から 1.100 に引き上げ、記載の一致を確かめるテストを追加。CI は 1.100.0 で `build-check` と `msrv-test` | `Cargo.toml`、`ci.yml`、`msrv_consistency.rs` |
| 引き上げの後に clippy 指摘への追従 | ネストした `if` を let チェーンに書き換えるコミットが 4 クレートに続く | c6f82aa、eaa868b、01aee48、01b80a6 |
| test-after | 実装のコミットの後にテストのコミットが続く(445d478 の次に 8ec98b5、19d7b1a の次に de4e3ad)。ただし同じコミットにテストを含むものもある | `git log` |
| 行カバレッジ 80% | `coverage` ジョブ(`cargo llvm-cov --fail-under-lines 80`、補助バイナリを除外) | `ci.yml` |
| 3 OS でテスト | `test`: ubuntu-24.04、macos-latest、windows-2022 | `ci.yml` |
| CI のジョブ | lint、build-check(ubuntu・windows)、test(3 OS)、msrv-test、coverage、docs-check、startup-bench(3 OS、警告のみ) | `ci.yml` |
| 起動の性能は警告のみ | `STARTUP_BENCH_ENFORCE: "false"`、`continue-on-error: true` | `ci.yml` |
| `audit` は警告のみ | `continue-on-error: true`。PR と毎週月曜 | `audit.yml` |
| リリースは下書き、人が公開 | `gh release create --draft --generate-notes`。prerelease は `-` を含む版 | `release.yml` |
| タグの検査 | `git merge-base --is-ancestor $GITHUB_SHA origin/main` と、cli クレートの版との一致 | `release.yml` の `verify-tag` |
| リリース前に CI と同じ検査 | `checks` が `ci.yml` を `workflow_call` で再利用し、`build` が `needs` で依存する | `release.yml` |
| 4 対象のビルドと smoke test | musl・aarch64 mac・x86_64 mac(クロス、smoke 無し)・windows msvc。SHA256SUMS | `release.yml` |
| lint と fmt の設定は既定 | `rustfmt.toml`、`clippy.toml`、`.rustfmt.toml` が無い。CI は `cargo fmt --check` と `clippy -D warnings` | ファイルの有無、`ci.yml` |
| 依存の制約 | `cargo tree` で C ビルドの依存と `-sys` クレートを検査 | `ci.yml` の `build-check` |
| 命名と言語 | Rust 標準。ドキュメントコメントとテスト名は日本語 | `code-quality-assessment.md` |
| `unsafe` の制約 | runtime の 1 か所のみ。cli は `forbid(unsafe_code)` | `technology-stack.md`、`nfr_source.rs` |
| 既知の負債 | `startup-bench` と `audit` が警告のみ、Intel mac が未確認、`.idea/` が追跡されている | `code-quality-assessment.md` |

## 基準からの変化(前回の確認以降に見えるもの)

- リリース: 下書きのリリースを人が公開する流れと、タグの検査(main 上、版の一致)。基準の Deployment は「タグを付けると CI が 3 OS 向けのバイナリを自動でビルドする」としか書いておらず、下書きと人の公開、4 対象(Intel mac を含む)、`SHA256SUMS`、リリース前の CI 再利用、smoke test が抜けている。
- MSRV の方針: 基準に無い。1.100 に固定し、記載の一致をテストで保つ。
- CI のジョブ: 基準は「3 OS のテスト」と 80% 下限のみ。実際は msrv-test、build-check、startup-bench、lint、coverage、docs-check、audit が揃っている。
- 小さな変更の慣行: 基準の Way of Working に既にある。履歴が裏づける。

## 証拠では決められなかったこと(インタビューの点)

1. コミットを直接 `main` に積むのか、PR を経由するのか(マージコミット無し、単独作者)。
2. 今回の intent(v0.9.0)の作り方: 機能を 1 つずつ順に作る方針を続けるか(履歴は整合するが、意向は人の判断)。
3. 今回の機能で、テストの下限(80%)と非機能のテストをどこまで求めるか。
4. 新しい依存(`.env` の読み込み、グラフの描画)の追加の可否と、既存の依存の制約(C ビルド無し、`unsafe` の位置)を規則として確定するか。
5. 署名・公証、Intel mac の動作確認、`audit` と `startup-bench` の強制を、今後も見送るか。
6. 基準の「主な開発環境の OS」がどの OS か(CI からは分からない)。
7. 「discovered-rules.md」の候補のうち、人が強い制約として確定するもの。

## インタビューの決定

人が全 7 問に答え、確認した。

1. Q1: PR は使わず、直接 `main` にコミットする。統合ブランチは `main`。
2. Q2: 機能を 1 つずつ順に作る。最初に薄い縦切り(walking skeleton)は作らない。
3. Q3: test-after を続ける。行カバレッジ 80% を下限とし、単体テストと実バイナリを動かす統合テストを含める。
4. Q4: 新しい外部ライブラリは追加しない。`.env` の読み込みとグラフの描画は自前で書く。NEVER 規則として確定した。
5. Q5: 署名・公証、Intel Mac の動作確認、依存監査の強制、起動の性能テストの強制は、今回も見送る。
6. Q6: 主な開発環境の OS は macOS。テストはローカルで実行し、他の OS は CI で確認する。
7. Q7: `discovered-rules.md` の候補をすべて強い制約として確定した(ALWAYS 5 件、NEVER 5 件)。Q4 の規則を NEVER として 1 件加えた(計 NEVER 6 件)。

`discovered-rules.md` の規則は、人がインタビューの Q7 と Q4 で確認したもの。

## 未解決の不確実性

なし。
