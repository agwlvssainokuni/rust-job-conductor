# Practices Discovery — インタビュー

初稿(team-practices.md)は、フレームワークの既定値を出発点にしたドラフトです。以下で、実際の進め方を確認します。

## Q1. [開発] ブランチとマージの進め方

コードをどのように取り込みますか(初稿の既定: `main` を唯一の統合先とし、短命ブランチを squash マージして履歴を一直線に保つ)。

- A. 初稿の既定どおり(`main` を統合先とし、squash マージする)
- B. `main` を統合先とするが、squash せずに履歴をそのまま残す
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q2. [開発] 最初に薄い一通りの流れを作るか

最初に薄いエンドツーエンドの縦切りを作りますか? ウォーキングスケルトンとは、全体を端から端まで一通り動かせる最小限のバージョンのことで、本格的な機能を足す前に、各部がつながることを確かめるために最初に作ります(初稿の既定: 作る。定義ファイルを読む → 1ジョブを実行 → ステートに記録 → 終了コードを返す)。

- A. はい、最初に作る(初稿の既定)
- B. いいえ、機能ごとに順に作る
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: B

## Q3. [開発] テストの進め方

テストをいつ書きますか(初稿の既定: 実装したあとにテストを書く)。Rust に不慣れな開発者が進めるため、選び方が品質に影響します。

- A. 実装のあとにテストを書く(test-after。初稿の既定)
- B. テストを先に書いてから実装する(TDD)
- C. 振る舞いの例を先に書き、細かい部分は実装後にテストを書く(組み合わせ)
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q4. [開発] テストの基準と実行環境

カバレッジの下限と、テストを実行する環境を確認します(初稿の既定: 行カバレッジ 80%、マージ前に Linux・macOS・Windows の3つで実行)。

- A. 初稿の既定どおり(80%、3つのOSで実行)
- B. カバレッジの下限は設けず、3つのOSでテストを実行する
- C. カバレッジの下限は設けず、テストは主な開発環境のOSだけで実行する
- D. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A. 主な開発環境のOS以外はCIで(GitHub Actions)。

## Q5. [運用] リリースと配布の進め方

完成したバイナリをどのようにリリース・配布しますか(初稿の既定: マージ時にCIでビルドと検査を行い、タグ付けで3OS向けのバイナリを作成する。配布先は未確認)。

- A. タグ付けで、CI が3OS向けのバイナリを自動で作成する(初稿の既定)
- B. 自動化はせず、必要なときに手元でビルドして配布する
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q6. [開発] コードの書き方の基準

コードの整形と静的チェックをどうしますか(初稿の既定: `rustfmt` で整形、`clippy` でチェックし、CI で失敗したらマージできない)。

- A. 初稿の既定どおり(`rustfmt` と `clippy` を使い、失敗したらマージできない)
- B. 使うが、警告が出てもマージは止めない
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Q7. [プロダクト] 守るべき・禁止すべき決まり

チームとして必ず守る決まり(ALWAYS)や、禁止する決まり(NEVER)はありますか。

- A. 特になし
- B. ある(内容は Other に記入)
- C. Not yet defined(まだ定まっていない)
- X. Other (please specify)

[Answer]: A

## Consolidated Summary Confirmation

Does this all look correct before I generate the artifact?

- Looks correct
- Request changes

[Answer]: Looks correct
