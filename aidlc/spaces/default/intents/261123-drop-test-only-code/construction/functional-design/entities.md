# エンティティ(Entities): テストだけが使う本番のコードの削除

永続のデータ(ステートファイル)の形は変えない。利用者に見える出力も変えない。変わるのは公開の型と関数の形(テストだけが使う変換・問い合わせ・既定の値・トレイトの実装を消し、テストだけが外から使う関数と再公開を狭める)と、テストの補助の引数。詳細は `inventory.md`。

```yaml
entities:
  # ---- core ----
  - name: Command
    description: ジョブのコマンド(core。公開)。テストだけが使う From<&str> と From<String> を消す。作るときは Command::Line(..) か Command::Argv(..) を書く(inventory.md 1a)。
    attributes:
      - name: Line(String) / Argv(Vec<String>)
        logical_type: 今のまま
  - name: JobStatus
    description: ジョブの状態(core。公開)。本番で使われない Default を消す(inventory.md 1d-1)。as_str と parse は今のまま。
  - name: RunIdError
    description: run ID の検査に通らなかったこと(core。公開)。4 つの種類(Empty、TooLong、InvalidChar、LeadingDot)を、種類を持たない単位構造体 1 つにする(D2。決定: 推奨のとおり。ユーザー、2026-11-24。inventory.md 3a)。値を持たない。
    attributes:
      - name: (なし)
        logical_type: 'struct RunIdError;(Debug、Clone、Copy、PartialEq、Eq)'
  - name: DefinitionErrorKind / EnvFileErrorKind
    description: 定義と .env の誤りの種類(core。公開)。Display、std::error::Error、Hash を消す。種類の文は label()、理由は EnvFileErrorKind::reason()(inventory.md 3c、3d)。
    attributes:
      - name: label()
        logical_type: '&''static str(今のまま)'
      - name: reason()
        logical_type: '&''static str(EnvFileErrorKind だけ。今のまま)'
  - name: EnvFileError
    description: .env の読み取りの誤り(core。公開)。kind.reason() を返すだけの reason() を消す。Display と Error は残す(inventory.md 1f。Q5)。
    attributes:
      - name: file / line / kind
        logical_type: 今のまま
  - name: graph::StateOverlay
    description: 図に重ねるステートの内容(core。公開)。本番で使われない Default を消す。空の値は項目を書いて作る(inventory.md 1d-3)。
    attributes:
      - name: states / resume_job_id
        logical_type: 今のまま
  - name: core の公開の関数と定数
    description: sanitize を消す(escape_display に一本化。inventory.md 1e)。decide_before_run は flow/decide.rs の中だけ(公開の入口は decide_from_state)、flow::classify は flow の中だけ、MAX_RUN_ID_LEN は run_id.rs の中だけ、is_valid_env_name・is_reserved_env_name・JobDefinition::effective_success_codes は crate の中(D6。決定: 推奨のとおり。ユーザー、2026-11-24。inventory.md 2a〜2c)。

  # ---- runtime ----
  - name: InterruptSignal
    description: 中断の合図(runtime。公開)。is_raised、PartialEq、Eq を消す。読むのは signal_number() だけ(None は立っていない)。Debug の raised の項目は signal_number().is_some() で書く(D8)。Default は残す(D1)。D1・D8 とも決定: 推奨のとおり(ユーザー、2026-11-24)(inventory.md 1c)。
    attributes:
      - name: new() / raise(n) / signal_number()
        logical_type: 今のまま
  - name: Durability
    description: ステートの書き込みの耐久性(runtime。公開)。本番で使われない Default を消す。呼ぶ所は Durable か Fast を常に明示する(inventory.md 1d-2)。
  - name: runtime の再公開
    description: core の check_consistency、apply_interrupt、apply_outcome、apply_skip、initial_state、ConsistencyResult、MismatchKind の再公開をやめる。使う所は conductor_core から使う(inventory.md 3b。Q3)。

  # ---- converter ----
  - name: ConversionErrorKind
    description: 変換の失敗の種類(converter。公開)。std::error::Error と Hash を消す。Display は本番の表示が使うので残す(inventory.md 3c-3、3d-3)。
  - name: TargetFormat
    description: 変換後の形式(converter。公開)。本番で使われない Hash を消す(inventory.md 3d-4)。

  # ---- cli ----
  - name: RunIdArg
    description: --run-id の指定(cli。公開)。テストだけが使う as_str を消す。値は Empty か Id(RunId) で比べる(inventory.md 1b)。
  - name: Language
    description: 表示の言語(cli。公開)。本番で使われない Default を消す(inventory.md 1d-4)。
  - name: cli のライブラリの公開の形
    description: report::escape の escape_display の再公開をやめる(escape_name と escape_full は公開のまま)。args、deps、panic_hook、types のモジュールを非公開にする(公開の項目は根で再公開済み)。根の GraphOptions と DryRunOptions の再公開をやめる(CliCommand の項目の型としては残る)。公開のモジュールは exit、report、run_id(inventory.md 2d〜2f)。

  # ---- テストの補助 ----
  - name: runtime の tests/common の job と command_line
    description: 引数を impl Into<Command> から impl Into<String> にし、中で Command::Line を作る(呼ぶ所はすべて文字列。D5。決定: 推奨のとおり。ユーザー、2026-11-24。inventory.md 1a)。
  - name: runtime の tests/runner_basic.rs の run
    description: 引数を impl Into<Command> から Command にし、中で check_command で分ける(文字列と配列の両方で呼ばれる。D5)。
  - name: cli の tests/args_grammar.rs の run_id(新規。ファイルの中の補助)
    description: 規則どおりの文字列から Some(RunIdArg::Id(..)) を作る(inventory.md 1b)。
```

## 利用者に見える変わること

なし。標準出力と標準エラーの文、終了コード、ステートファイル(バイト列)、`--dry-run` と `graph` の出力、`conductor --version`(`conductor 0.9.8`)は変わらない。ライブラリとしての公開の形(上の型と関数)だけが変わる。
