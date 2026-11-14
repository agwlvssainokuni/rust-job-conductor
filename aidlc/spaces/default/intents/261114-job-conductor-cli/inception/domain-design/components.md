# Components(コンポーネント一覧)

上流: requirements(FR1〜FR7、NFR1〜NFR6)、stories(US1.1〜US7.1)、team-practices。ここでの部品は「作るコード」の論理的なまとまりであり、デプロイの形(`conductor` は単一バイナリ)や技術スタックは決めない。ファイルシステムなど、部品が使うものは外部依存として扱う。部品の分け方は中間の粒度とし、7つの部品にした。[Q1]、ADR-001

## Part A: 機械可読な一覧

```yaml
components:
  - name: Cli
    summary: コマンドとオプションを受け付け、各部品を呼び出して1回の実行を組み立てる調整役
    behaviour: >
      run / pipeline / watch / reset / convert の各コマンドと、--config、-v / --verbose
      を受け付ける。定義を読み、ステートを読み、Flow に次の動きを判断させ、JobRunner
      にジョブを起動させ、結果をステートに書き、Reporter に表示させる流れを組み立てる。
      スケジューラへ返す終了コードを最終的に決める(警告は 0、エラーはジョブの終了コード、
      成功コードに含まれない 0 は 1、起動失敗は 127、シグナルは 128 + シグナル番号)。
      他の部品どうしは直接呼び合わない(ADR-002)。
    responsibilities:
      - コマンドとオプションの解釈
      - 1回の実行の流れの組み立て(読み込み、判断、起動、記録、表示)
      - スケジューラへ返す終了コードの決定
      - シグナルの受け取りと JobRunner への伝達
    depends_on:
      - component: DefinitionLoader
        interaction: 定義ファイルを読み込み、検証してもらう
        style: sync
      - component: Flow
        interaction: 結果の分類、次に実行・スキップするジョブ、完了の判定を決めてもらう
        style: sync
      - component: JobRunner
        interaction: ジョブのコマンドを起動し、終了コードやシグナルを受け取る
        style: sync
      - component: StateStore
        interaction: ステートの読み込み、書き込み、初期化、定義との照合を頼む
        style: sync
      - component: Converter
        interaction: convert コマンドで定義ファイルを変換してもらう
        style: sync
      - component: Reporter
        interaction: 表示する内容(種類とデータ)を渡し、文言の組み立てと出力を頼む
        style: sync
    dependents: []
    external_dependencies:
      - name: OS のプロセス・シグナル
        kind: other
        purpose: シグナルを受け取り、JobRunner に伝える
    entities: []

  - name: DefinitionLoader
    summary: 定義ファイル(conductor.toml)を読み、構造と内容を検証する
    behaviour: >
      定義ファイルを読み込み、パイプラインの名前と説明、各ジョブ(ID、コマンド、
      警告・成功の終了コード、遷移ルール)を取り出す。起動時に検証する: 存在しない
      ジョブID、エラー結果への遷移先、自分自身や記述順で前のジョブへの遷移先、
      警告と成功の重複、0 の警告指定、型の誤り、パイプライン名の不一致。
      誤りがあれば、ファイル名・行・原因を返す。定義ファイルは、既定で実行ディレクトリの
      conductor.toml とし、--config で別のパスを指定できる。
    responsibilities:
      - 定義ファイルの読み込み
      - 定義の検証と、誤りの位置・原因の報告
      - ジョブの記述順の保持
    depends_on: []
    dependents:
      - component: Cli
        interaction: 定義ファイルを読み込み、検証してもらう
    external_dependencies:
      - name: ローカルのファイルシステム
        kind: other
        purpose: 定義ファイルの読み取り
    entities:
      - name: PipelineDefinition
        identifier: name
        attributes: [name, description, jobs]
        references: []
      - name: JobDefinition
        identifier: id
        attributes: [id, command, warningExitCodes, successExitCodes, transitions]
        references: []

  - name: Flow
    summary: 結果の分類と、次に実行・スキップするジョブ、完了の判定を決める(入出力を持たない判断ロジック)
    behaviour: >
      終了コードを、成功(成功コードを書かなければ 0 のみ、書けばその値だけ)・警告・エラーに
      分類する。成功コードに含まれない 0 はエラーとする。分類結果と遷移ルールから、
      次に実行するジョブと、スキップするジョブ(現在と遷移先のあいだ)を決める。
      @end 以降のジョブは、すべてスキップにする。記述順で最後のジョブが実行またはスキップで
      終わったとき、または一連の実行で @end に到達したときを、完了と判定する。
      エラーで終わったときは停止とし、再開の位置(エラーになったジョブ)を決める。
      ファイルやプロセスには触れず、必要な情報はすべて引数で受け取る。
    responsibilities:
      - 終了コードの分類
      - 遷移・スキップ・ジャンプの判断
      - 完了と再開位置の判定
    depends_on: []
    dependents:
      - component: Cli
        interaction: 結果の分類、次の動き、完了の判定を決めてもらう
    external_dependencies: []
    entities: []

  - name: JobRunner
    summary: ジョブのコマンドを、シェルを介さず直接起動し、終了コードとシグナルを扱う
    behaviour: >
      コマンド文字列を、コマンド名と引数に分けて、シェルを介さずに起動する。作業ディレクトリと
      環境変数は conductor から引き継ぐ。ジョブの標準出力・標準エラーは、そのまま通す。
      起動できないとき、シグナルで終了したとき、終了コードがあるときを区別して返す。
      時間制限は設けない。conductor がシグナルを受け取ったときは、ジョブにも転送し、
      終了を待つ(Windows では終了の要求に読み替える)。
    responsibilities:
      - コマンドの分割と直接起動
      - 終了コード・起動失敗・シグナル終了の区別
      - シグナルのジョブへの転送
    depends_on: []
    dependents:
      - component: Cli
        interaction: ジョブを起動し、結果を受け取る
    external_dependencies:
      - name: OS のプロセス管理
        kind: other
        purpose: ジョブのコマンドの起動、シグナルの送信、終了の待機
    entities:
      - name: JobExecution
        identifier: jobId
        attributes: [jobId, command, exitCode, terminatedBySignal, startFailure]
        references:
          - entity: JobDefinition
            owned_by: DefinitionLoader
            relationship: 各 JobExecution は、1つの JobDefinition に基づいて起動される

  - name: StateStore
    summary: ステートファイルを安全に保持し、破損と定義との食い違いを検知する
    behaviour: >
      実行ディレクトリの近傍のステートファイルに、ジョブごとの結果、スキップ・ジャンプ、
      エラーの位置、定義のジョブ構成(ID と記述順)を保持する。書き込みは、中断されても
      壊れた内容が残らないようにする。読み込み時に、読めない・壊れているときは、パスと原因を
      返す。定義のジョブ構成を受け取り、ステートに記録されたものと食い違うときは、食い違いを
      返す(ADR-004)。初期化(パイプライン完了時の自動初期化、reset)を行う。ステートは
      定義ファイル1つにつき1つ。
    responsibilities:
      - ステートの読み込みと、破損の検知
      - ステートの安全な書き込み
      - 定義とステートの食い違いの検知
      - ステートの初期化
    depends_on: []
    dependents:
      - component: Cli
        interaction: ステートの読み込み、書き込み、初期化、定義との照合を頼む
    external_dependencies:
      - name: ローカルのファイルシステム
        kind: other
        purpose: ステートファイルの読み書き(原子的な更新)
    entities:
      - name: PipelineState
        identifier: pipelineName
        attributes: [pipelineName, jobStates, jobOrder, completed]
        references:
          - entity: PipelineDefinition
            owned_by: DefinitionLoader
            relationship: 各 PipelineState は、1つの PipelineDefinition に対応する
      - name: JobState
        identifier: jobId
        attributes: [jobId, status, exitCode, skipReason]
        references:
          - entity: JobDefinition
            owned_by: DefinitionLoader
            relationship: 各 JobState は、1つの JobDefinition に対応する

  - name: Converter
    summary: 定義ファイルを YAML と TOML のあいだで変換する
    behaviour: >
      入力ファイルを読み、キーと値の意味が同じになるように、指定された形式で出力する。
      コメントと記述順は保持しない。入力が読めない、または文法が誤っているときは、変換せずに
      原因を返す。
    responsibilities:
      - YAML と TOML の相互変換
      - 変換できないときの原因の報告
    depends_on: []
    dependents:
      - component: Cli
        interaction: convert コマンドで定義ファイルを変換してもらう
    external_dependencies:
      - name: ローカルのファイルシステム
        kind: other
        purpose: 入力の読み取りと、出力の書き出し
    entities: []

  - name: Reporter
    summary: conductor 自身の表示(-v、watch、エラー)を組み立て、環境のロケールに合わせた文言で出力する
    behaviour: >
      Cli から、表示の種類(ジョブの結果、スキップ、遷移、状態の一覧、エラー)とデータを受け取り、
      文言を組み立てて出力する。既定では conductor 自身の表示は出さず、-v / --verbose で有効に
      する(エラーは常に表示する)。文言の言語は、環境のロケールに従う(英語と日本語)。
      状態は色だけでなく文字でも表し、端末以外への出力では色を無効にする。
      watch は、ジョブID・状態・補足の3列の一覧を、1回表示して終了する。
    responsibilities:
      - 表示の種類ごとの文言の組み立て
      - ロケールに応じた言語の選択
      - 色の有効・無効の切り替え
      - watch の一覧の整形
    depends_on: []
    dependents:
      - component: Cli
        interaction: 表示する内容(種類とデータ)を渡し、出力を頼む
    external_dependencies:
      - name: 環境のロケール・標準出力
        kind: other
        purpose: 言語の判定と、端末かどうかの判定、出力
    entities: []
```

## Part B: 人が読むための表示

### Component Diagram

```mermaid
graph TD
  Cli --> DefinitionLoader
  Cli --> Flow
  Cli --> JobRunner
  Cli --> StateStore
  Cli --> Converter
  Cli --> Reporter
```

Text fallback: `Cli` が、`DefinitionLoader`、`Flow`、`JobRunner`、`StateStore`、`Converter`、`Reporter` の6つを呼び出す。6つの部品どうしは、直接呼び合わない。

### Component Summary

| Component | Purpose | Depends On | Dependents | Entities Owned |
|-----------|---------|------------|------------|----------------|
| Cli | コマンドの受け付けと、1回の実行の組み立て | DefinitionLoader、Flow、JobRunner、StateStore、Converter、Reporter | なし | なし |
| DefinitionLoader | 定義ファイルの読み込みと検証 | なし | Cli | PipelineDefinition、JobDefinition |
| Flow | 分類・遷移・完了の判断(入出力なし) | なし | Cli | なし |
| JobRunner | ジョブの直接起動、終了コード・シグナルの扱い | なし | Cli | JobExecution |
| StateStore | ステートの保持、破損・食い違いの検知、初期化 | なし | Cli | PipelineState、JobState |
| Converter | YAML と TOML の相互変換 | なし | Cli | なし |
| Reporter | 表示の組み立てとロケール対応 | なし | Cli | なし |

### Entity Ownership

| Entity | Owning Component | Identifier | Attributes | References |
|--------|------------------|------------|------------|------------|
| PipelineDefinition | DefinitionLoader | name | name、description、jobs | なし |
| JobDefinition | DefinitionLoader | id | id、command、warningExitCodes、successExitCodes、transitions | なし |
| JobExecution | JobRunner | jobId | jobId、command、exitCode、terminatedBySignal、startFailure | JobDefinition(DefinitionLoader) |
| PipelineState | StateStore | pipelineName | pipelineName、jobStates、jobOrder、completed | PipelineDefinition(DefinitionLoader) |
| JobState | StateStore | jobId | jobId、status、exitCode、skipReason | JobDefinition(DefinitionLoader) |

### External Dependencies

| Component | Dependency | Kind | Purpose |
|-----------|------------|------|---------|
| Cli | OS のプロセス・シグナル | other | シグナルの受け取り |
| DefinitionLoader | ローカルのファイルシステム | other | 定義ファイルの読み取り |
| JobRunner | OS のプロセス管理 | other | ジョブの起動、シグナルの送信、終了の待機 |
| StateStore | ローカルのファイルシステム | other | ステートファイルの原子的な読み書き |
| Converter | ローカルのファイルシステム | other | 入力の読み取りと出力の書き出し |
| Reporter | 環境のロケール・標準出力 | other | 言語と端末かどうかの判定、出力 |

### Rationale

| Component | なぜ独立した部品か |
|-----------|--------------------|
| Cli | 1回の実行の流れを組み立てる、唯一の調整役。他の部品は、互いを知らずに済む(ADR-002) |
| DefinitionLoader | 定義ファイルの形式と検証は、定義の書き方が変わったときだけ変わる。独立した関心事である |
| Flow | 判断ロジックは、入出力を持たないため、単体でテストしやすい。変更の理由(遷移の仕様)が他と異なる(ADR-005) |
| JobRunner | OS のプロセス・シグナルの扱いは、OS ごとの違いが大きく、変更の理由が他と異なる(複数 OS 対応のリスクを閉じ込める) |
| StateStore | ステートの安全な書き込みと整合は、固有のライフサイクルを持つ。ステート形式の変更は、他の部品に波及させたくない(ADR-004) |
| Converter | 他の部品に依存しない独立した機能で、優先度(Should)と提供のタイミングが異なる |
| Reporter | 表示と言語の切り替えは、変更の速度(文言の追加)が他と異なる。他の部品を文言に依存させない(ADR-003) |

**Alternatives Rejected(分け方):** 細かく分ける案(「結果の分類」を Flow から独立させるなど、8部品)と、粗く分ける案(4部品)は、ADR-001 で退けた。

## Assumptions & Open Questions

- Cli を調整役とし、他の6部品は互いを呼ばない構成にした(ADR-002)。要件・ストーリーの「個別実行」と「一連の実行」のどちらも、同じ部品の組み合わせで実現できることを前提にしている。 [assumption]
- 属性の名前(`warningExitCodes` など)は、所有と形を示すための仮の名前であり、型・制約・値の範囲は Functional Design で定める。 [assumption]
- 個別実行で、順序外のジョブが呼ばれた場合と、実行済みのジョブが再び呼ばれた場合の動作は、Flow の判断ロジックに関わるため、Functional Design で決める(requirements の未解決事項)。 [assumption]
- ステートファイルの名前と保存場所の具体、別パスの定義ファイルを指定したときのステートの紐づけは、StateStore の仕様として Functional Design で決める(requirements の未解決事項)。 [assumption]
