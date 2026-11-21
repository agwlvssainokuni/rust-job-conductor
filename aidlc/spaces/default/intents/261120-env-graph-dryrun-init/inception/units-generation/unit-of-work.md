# ユニットの定義(Unit of Work): v0.9.0 の機能拡張

出典: コンポーネントの一覧(`domain-design/components.md`)、設計の決定(`domain-design/decisions.md`)、要件(`requirements-analysis/requirements.md`)、ユニットの質問(`units-generation-questions.md`。以下 Q1〜Q3)。

## 1. ユニットの一覧

分け方(Q1): 機能ごとに 4 つ。第 2 段階の拡張は、基本の機能と同じユニットに入れる(Q2)。README、デモ、日本語と英語のメッセージは、各ユニットが自分の機能の分を作る(Q3)。

| Unit ID | Directory | 内容 | 種類(kind) | 大きさ |
|---------|-----------|------|-------------|--------|
| U1 | u1-env-injection | 環境変数の注入と `.env` | service | L |
| U2 | u2-graph | `conductor graph` | service | M |
| U3 | u3-dry-run | `--dry-run` と `--assume` | service | M |
| U4 | u4-init | `conductor init` | service | S |

`kind` を `service` にした理由: 4 つとも、単一の実行ファイル(`conductor`)への変更で、設計の成果物(機能の設計、テストなど)を、フルセットで作る対象であるため。独立にデプロイするサービスではない(デプロイの形は、各ユニットに記す)。

## 2. U1 u1-env-injection: 環境変数の注入と `.env`

- 説明: ジョブに環境変数を渡す。定義に `[pipeline.env]`、`[job.env]`、`env_file` を足し、`.env` ファイルを読み、層を優先順位で統合し、組み込みの `CONDUCTOR_*` を渡す。
- 境界(コンポーネント): `CoreTypes`(env と env_file の項目)、`CoreDefinition`(新しいキーの検査、予約名と大文字小文字だけが違う名前の拒否)、`CoreEnvFile`(新規。`.env` の書式の検査、層の統合)、`RuntimeEnvLoader`(新規。ファイルの読み取り)、`RuntimeRunner`(環境変数つきの起動)、`CliFlows`(実行の前に `.env` をすべて読み、誤りなら何も実行しない。`validate` の重複キーの警告)、`CliReport`(新しい誤りと警告のメッセージ)。
- 責務(要件): FR1.1 から FR1.13、FR5.1、FR5.2(環境変数の分)。
- デプロイの形: `conductor` の単一の実行ファイルに含まれる(共有。独立のデプロイはない)。
- 大きさ: L。新しい誤りの種類(`DefinitionErrorKind` の追加)、メッセージの日本語と英語、書式の検査、層の統合、定義の型の変更(多くのテストに影響する)を含む。
- 実装の注意:
  - 新しい外部ライブラリを足さない(project.md)。
  - エラーの表示とステートに、環境変数の値を含めない(project.md)。
  - `format_version` は 1 のまま(FR1.13)。
  - `.env` の誤りの終了コードは、機能の設計で決める(要件の未解決事項)。

## 3. U2 u2-graph: `conductor graph`

- 説明: 定義から、フローの図を標準出力に出す。`git log --graph` 風の text(既定)と Mermaid。`-v` で詳細。`--state` で、ステートを重ねる。
- 境界(コンポーネント): `CoreGraph`(新規。辺の取り出しとレーンの配置)、`CoreFlow`(辺の取り出しの入口を公開)、`CliArgs`(`graph`、`--format`、`--state`、`-v`)、`CliReport`(text と Mermaid の整形、状態の記号)、`CliFlows`(`graph` の流れ。定義の誤りは `validate` と同じ)。
- 責務(要件): FR2.1 から FR2.7、FR5.1、FR5.2(graph の分)。
- デプロイの形: 単一の実行ファイルに含まれる(共有)。
- 大きさ: M。レーンの配置の処理が中心。
- 実装の注意: 辺は、実行のときに次のジョブを決める判断と同じ関数から作る(FR2.4)。ファイルとプロセスに触れない(`--state` の読み取りを除く)。

## 4. U3 u3-dry-run: `--dry-run` と `--assume`

- 説明: `pipeline` と `run` に `--dry-run` を足す。ステートを読み、再開位置とジョブごとの扱いを示す。何も実行せず、ステートも書かない。`--assume` で、終了コードを仮定した流れを模擬する。
- 境界(コンポーネント): `CorePlan`(新規。計画の生成と模擬)、`CliArgs`(`--dry-run`、`--assume`)、`CliReport`(計画の整形)、`CliFlows`(`--dry-run` の流れ。ステートは読み取りだけ)。
- 責務(要件): FR3.1 から FR3.4、FR5.1、FR5.2(dry-run の分)。
- デプロイの形: 単一の実行ファイルに含まれる(共有)。
- 大きさ: M。
- 実装の注意:
  - コマンドの分割は `runtime::split` にあり、`core` から呼べない。分割の置き場所と誤りの出し方は、機能の設計で決める(ドメイン設計のレビューの指摘 R-01)。
  - U1 の統合の結果(渡される環境変数の名前)と、U2 の辺の判断の公開を使う(Q3 の依存の方針)。

## 5. U4 u4-init: `conductor init`

- 説明: 定義のひな形を作る。既にあるときは終了コード 69。`--force` で上書き。`-` で標準出力。ひな形は、ロケールに従い、`[pipeline.env]` と `[job.retry]` の例を含み、必ず `validate` に通る。
- 境界(コンポーネント): `RuntimeFileCreate`(新規。排他的な作成)、`CliArgs`(`init`、`--force`)、`CliReport`(ひな形の文字列)、`CliFlows`(`init` の流れ)。
- 責務(要件): FR4.1 から FR4.5、FR5.1、FR5.2(init の分)。
- デプロイの形: 単一の実行ファイルに含まれる(共有)。
- 大きさ: S。
- 実装の注意: ひな形に `[pipeline.env]` の例を入れるため、U1(新しいキー)に依存する(依存は、ひな形の内容とテストにだけある)。ひな形が `validate` に通ることを、テストで確かめる。

## 6. 全体の注意

- 新しいクレートと、新しい外部の依存は、足さない(ドメイン設計の ADR-005、project.md)。
- `unsafe` は `platform/unix.rs` の外に書かない。`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない(project.md)。
- 各ユニットは、核となる処理を `core` に、ファイルの操作を `runtime` に、表示を `cli` に置く(ドメイン設計の決定)。
