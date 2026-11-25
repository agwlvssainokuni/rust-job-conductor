# 証拠

調べた者: 再実行のリード(pipeline-deploy)。協力者は無効(単独の作業)。リードは下記のファイルと履歴を調べ、解釈を推論した。ビルド・テスト・プログラムは実行していない。AI-DLC 自身のファイルは、指定された記憶のファイルとコードの知識ベース以外は調べていない。

## 調べたもの

- 基準: `aidlc/spaces/default/memory/team.md`(5 つの節)と `project.md`(`## Mandated` 5 件、`## Forbidden` 6 件)。
- 前回の intent の実践の artifact: `aidlc/spaces/default/intents/261120-env-graph-dryrun-init/inception/practices-discovery/evidence.md`。
- git: `git log`(867 コミット)、`git log --merges`、`git tag`、`git branch -a`、作者の一覧、v0.8.0..v0.9.0 の `crates` の履歴、v0.9.0 以降の版の更新のコミット。
- `.github/workflows/ci.yml`、`release.yml`、`audit.yml`。
- 根の `Cargo.toml`(`rust-version`)と各クレートの `Cargo.toml`(版、依存)。
- `crates/*/tests/` と `crates/*/src/tests/` のファイルの数。
- コードの知識ベース(`aidlc/spaces/default/codekb/rust-job-conductor/`): `code-structure.md`、`technology-stack.md`、`dependencies.md`、`code-quality-assessment.md`、`architecture.md`、`business-overview.md`。
- 今回の intent のスコープ: `ideation/scope-definition/intent-backlog.md`、`scope-definition-questions.md`。

## 事実と、それが支える実践

| 実践 | 事実 | 出所 |
|------|------|------|
| `main` が統合ブランチ、長命のブランチ無し | ブランチは `main` と `origin/main` のみ。マージコミット 0 件。作者は 1 人で 867 コミット | `git branch -a`、`git rev-list --merges`、`git log --format=%an` |
| PR を使わず直接 `main` | マージコミットが無く、単独作者。`code-quality-assessment.md` も「関門は `push` の CI で事後に効く」と記す | `git log`、`code-quality-assessment.md` |
| リリースはタグで区切る | タグ `v0.1.0` から `v0.9.9`(18 個)。v0.9.0 の後に v0.9.1 から v0.9.9 の 9 個が増えた | `git tag` |
| 版の上げ方 | 機能の intent は 2 桁目(v0.8.0 リトライ、v0.9.0 環境変数・graph・dry-run・init)、graph の改善と整理の intent は 3 桁目(v0.9.1 から v0.9.9) | 版の更新のコミット(5fc537c、9949237、dfdc038 から 1f88e59) |
| 変更は小さい | v0.9.0 の U4(init)は空の枠、型、`create_file`、そのテスト、ひな形、ひな形のテスト、引数・流れ、テストの合わせの 8 コミット | `git log v0.8.0..v0.9.0 -- crates` |
| コミットの件名の慣行 | v0.9.0 は `U1:` から `U4:` の単位の接頭辞。以降は `core:`、`runtime:`、`cli:`、`converter:`、`docs:`、`aidlc:` の接頭辞。版は「版を X にする」 | `git log` |
| 機能を 1 つずつ順に作る | v0.9.0 は U1 → U2 → U3 → U4 の順で、各単位を README・デモで締め、最後に版 | `git log v0.8.0..v0.9.0` |
| test-after | 実装のコミットの後にテストのコミットが続く(602d15c の次に 90a27fa、0fbd466 の次に 5b750f8、a9eb237 の次に 365f45c と 062c627)。ただし同じコミットにテストを含むものもある | `git log` |
| テストの配置 | `tests/*.rs`: core 33、runtime 14、cli 25、converter 2。`src/tests/`: cli 11、converter 4。`#[test]` 合計 1,170 | `ls`、`find`、`code-quality-assessment.md` |
| テストの都合で本番を曲げない | v0.9.x の整理の intent(`261123-unbend-for-tests`、`261123-drop-test-only-code`、`261124-cli-public-surface`)が、外から使われない `pub` を狭め、テストを公開の入口だけで書き直した(e827a63、ca924c1、dd15174、059bdb0) | `git log`、intent の一覧 |
| 行カバレッジ 80% | `coverage` ジョブ(`cargo llvm-cov --fail-under-lines 80`、補助バイナリを除外) | `ci.yml` |
| 3 OS でテスト、MSRV でもテスト | `test`: ubuntu-24.04、macos-latest、windows-2022。`msrv-test` と `build-check` は 1.100.0 | `ci.yml` |
| MSRV 1.100 | `rust-version = "1.100"` | 根の `Cargo.toml` |
| 起動の性能と `audit` は警告のみ | `STARTUP_BENCH_ENFORCE: "false"` と `continue-on-error: true`。`audit.yml` は `cron: "0 3 * * 1"` と `continue-on-error: true` | `ci.yml`、`audit.yml` |
| リリースは下書き、人が公開 | `--draft --generate-notes`。版に `-` を含めば `--prerelease` | `release.yml` |
| 4 対象のビルドと smoke test | musl・aarch64 mac・x86_64 mac(クロス、smoke 無し)・windows msvc。`SHA256SUMS` | `release.yml`、`technology-stack.md` |
| lint と fmt の設定は既定 | 設定ファイルが無い。`#[allow]` は `platform/unix.rs` の 1 つ | `ci.yml`、`code-quality-assessment.md` |
| 依存の制約 | `cargo tree` で C ビルドの依存と `-sys` クレート(`linux-raw-sys`・`windows-sys` を除く)を検査。直接の実行時の依存は 7 個、開発時は `tempfile` 1 個 | `ci.yml`、`dependencies.md` |
| 副作用の閉じ込め、OS 差の閉じ込め | 副作用は core の `intake`、runtime、cli の `deps`・`platform` に。`cfg` は runtime と cli の `platform` の中だけ | `architecture.md`、`code-structure.md` |
| `unsafe` と内容の性質 | `unsafe` は `libc::kill` 1 か所。エラーの表示に入力の値を含めず、ステートに command・環境変数・出力・時刻を書かない | `code-quality-assessment.md`、`nfr_source.rs`、`nfr_content.rs` |
| 今回の機能の範囲 | B1〜B7 の 7 つ、すべて Must。新しい外部ライブラリの追加は範囲外。旧いステートの読み込みは不要(スコープ定義 Q5: B) | `intent-backlog.md`、`scope-definition-questions.md`、`business-overview.md` |

## 基準からの更新(事実の差し替え)

- コミットの数: 297 → 867。
- タグ: `v0.1.0` から `v0.8.0` → `v0.1.0` から `v0.9.9`。
- テストのファイルの数: core 10、runtime 13、cli 21、converter 5 → `tests/` は core 33、runtime 14、cli 25、converter 2。加えて `src/tests/` に cli 11、converter 4(単体テストを `src/tests/` へ移した、v0.9.x の整理の結果)。
- Walking Skeleton の証拠: v0.5.0〜v0.8.0 の例 → v0.9.0 の U1〜U4 の例に差し替え。今回の順序(B1〜B7)と、機能ごとの README・デモの更新(スコープ定義 Q4・Q6)を足した。
- Testing Posture の Methodology evidence: v0.8.0 の例 → v0.9.0 の例に差し替え。単体テストの `src/tests/`、参照出力・文書との一致のテストを足した。
- Way of Working: 版の上げ方(2 桁目と 3 桁目)、コミットの件名の慣行を足した。
- Code Style: `cfg` の場所を「runtime と cli の `platform`」に正確にし、副作用の閉じ込め、内容の性質、使い方の表の 3 か所を足した。「今回」を v0.10.0 の内容(時刻、DOT、`watch`)に合わせた。
- Deployment: 戻し方の 1 行を足した(リードの推論。`release.yml` の下書きの流れと、過去のリリースが残ることから)。
- 前回のインタビューの回答で「今回」を指していた箇所は、v0.10.0 のインタビュー(下記)で確かめ、v0.10.0 の決定として書き直した。v0.9.0 だけを指す記述は残していない。

## 既存の規則の確認(`project.md`)

- ALWAYS 5 件(カバレッジ 80%、MSRV、タグとリリース、fmt と clippy、小さな単位)は、CI と履歴が今も裏づける。新たに書き直す必要は無い。
- NEVER 6 件(C ビルドと `-sys`、`unsafe` の位置、`cfg` の位置、エラーとステートの内容、補助バイナリの配布、新しい依存)も、CI・テスト・コードが今も裏づける。新しい依存の禁止は「今回の作業で」と書かれているが、今回の intent のスコープも新しい外部ライブラリを範囲外とする(意図の表明 Q8)ため、同じ規則で足りる。
- 注意: B6(時刻の記録)はステートに開始と終了の時刻を足す。これは `project.md` の規則(ステートに環境変数を書かない)には触れないが、`code-quality-assessment.md` の「ステートに時刻を書かない」という既存の性質と `nfr_content.rs` の検査を、意図して変える。

## 証拠では決められなかったこと(インタビューで確かめた点)

`practices-discovery-questions.md` の 8 問で確かめた。人はすべてに答え、まとめの確認に「Looks correct」と答えた。

## インタビューの決定

| 問 | 決定 | 反映先 |
|----|------|--------|
| Q1 取り込み方 | A: これまでどおり、PR を使わず直接 `main` にコミットし、作業の履歴を squash せずに残す | `team-practices.md` の Way of Working |
| Q2 作る順序 | A: 薄い縦切りは先に作らない。B1〜B7 を 1 つずつ作り、各機能の最後に README とデモを更新し、最後に版を 0.10.0 にする | Walking Skeleton、Way of Working(版) |
| Q3 テスト | A: これまでどおり test-after、行カバレッジ 80% 以上、単体テストと実バイナリの統合テスト、ローカルは macOS で他の OS は CI | Testing Posture |
| Q4 見送り | A: 署名・公証、Intel macOS の動作確認、`cargo audit` の強制、`startup-bench` の強制は、v0.10.0 でも見送る | Deployment |
| Q5 依存 | A: 時刻の書式、DOT の整形、`watch` の表示し直しも自前で書く。既存の NEVER 規則(新しい依存を足さない)をそのまま使う | Code Style |
| Q6 新しい規則 | A: 「NEVER テストの都合で本番のコードを曲げない(テスト専用の引数・`pub`・関数を本番に残さない)」を確定する | `discovered-rules.md` の Forbidden、Testing Posture |
| Q7 ステートの時刻 | A: ステートに開始と終了の時刻だけを書くことを認める。コマンド・環境変数・出力は書かない。`nfr_content.rs` は時刻だけを許すように直す。`project.md` の規則(環境変数を書かない)は変えない | Code Style、Testing Posture |
| Q8 ほかの規則 | A: 無い | `discovered-rules.md` の Mandated は `None.` |

## 未解決の不確実性

- Deployment の戻し方の 1 行(公開前は下書きを消してタグを付け直す、公開後は前の版に戻せる)はリードの推論で、インタビューでは問うておらず、人の確認を経ていない。`team-practices.md` にも推論と明記した。
- コミットの件名の接頭辞の慣行と、版の 2 桁目・3 桁目の上げ方は、履歴から読み取ったもので、インタビューで個別には問うていない。
- Q7 による `nfr_content.rs` の変更と、`code-quality-assessment.md` の「ステートに時刻を書かない」という記述の更新は、B6 の Construction で行う。それまでコードの知識ベースの記述は古いままになる。
