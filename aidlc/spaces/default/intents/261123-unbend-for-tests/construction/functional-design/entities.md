# エンティティ(Entities): テストの都合で曲げられた本番のコードと、残りの整理

永続のデータ(ステートファイル)の形は変えない。利用者に見える出力も変えない。変わるのは公開の型と関数の形(テストのためだけの項目・関数・差し替え口を消す)と、テストの補助。詳細は `inventory.md`。

```yaml
entities:
  # ---- core ----
  - name: DefinitionErrorKind / EnvFileErrorKind
    description: 定義と .env の誤りの種類(core。公開)。テストだけが使う code() と ALL を消す(inventory.md 1f、1g)。仕様書の識別子(kind)は、変種の名前の先頭を小文字にしたもの(例 FileMissing → fileMissing)と、型の説明に書く。
    attributes:
      - name: label()
        logical_type: '&''static str(今のまま)'
      - name: reason()
        logical_type: '&''static str(EnvFileErrorKind だけ。今のまま)'
  - name: LoadedEnvFiles
    description: 読み取った .env の内容の集まり(core の envfile。公開)。テストの便宜の add を消す。組み立ては push_file と link だけ(inventory.md 1d)。
  - name: PipelineState
    description: 実行状態の全体(core。公開)。製品で読まれない definition_key と、常に job_states の ID の並びと同じ job_order を消す(inventory.md 2b)。ステートファイルの形(両キーの書き出しと検査)は変えない。
    attributes:
      - name: pipeline_name / definition_path / job_states / resume_job_id / end_reached / completed / run_id
        logical_type: 今のまま
  - name: graph の公開の関数
    description: is_lane_drawable を消す。flow_edges は pub(crate)、node_state_of は overlay.rs の中だけ。公開は build_graph、layout_graph、overlay_from_state、pending_overlay(inventory.md 1c、1j)。
  - name: ExitCodeClass
    description: 終了コードの分類の中間の型(core の flow/classify.rs。非公開)。消す。classify_exit_code は JobOutcome を返す(inventory.md 2d)。
  - name: decide_from_state
    description: ステート(無いこともある)から起動前の判断を決める関数(core の flow。新規。公開)。無ければ、記録は空、再開位置なし、end_reached は偽として decide_before_run を呼ぶ(inventory.md 2f。D7。決定(ユーザー、2026-11-23))。
    attributes:
      - name: 入力
        logical_type: job_id &str、definition &PipelineDefinition、state Option<&PipelineState>
      - name: 出力
        logical_type: Decision
  - name: EXIT_START_FAILURE / EXIT_SIGNAL_BASE / EXIT_UNLISTED_ZERO
    description: 終了コードの定数(core の flow/classify.rs)。pub(crate) から、classify.rs の中だけの非公開に(inventory.md 2e-1)。

  # ---- runtime ----
  - name: ResetAllReport
    description: reset --all の結果(runtime。公開)。removed を消し、kept だけにする(構造体のまま。D9。決定(ユーザー、2026-11-23)。inventory.md 1b)。
    attributes:
      - name: kept
        logical_type: Vec<String>(今のまま。読めない・壊れているために残したファイルの名前、並べ替え済み)
  - name: CommandError / CommandErrorKind
    description: command を分けられないときの報告(runtime。公開)。kind から決まる写しの reason の項目を消し、CommandErrorKind::reason() で引く。Display の文は同じ(inventory.md 2c。D6。決定(ユーザー、2026-11-23))。
    attributes:
      - name: CommandError.kind
        logical_type: CommandErrorKind(今のまま)
      - name: CommandErrorKind::reason()
        logical_type: '&''static str(新規。UnclosedQuote と Empty の固定の文)'
  - name: CreateError
    description: ファイルを作れなかった理由(runtime。公開)。テストだけが使う ALL を消す(inventory.md 1g)。
  - name: remove_state_file / remove_stale_temps
    description: ステートの削除と、古い一時ファイルの後始末(runtime の state/writer.rs。crate の中と非公開)。remove_state_file は Result<(), StateError>(消したかの真偽値を返さない)、remove_stale_temps は今の時刻を中で取る(inventory.md 1b、1e)。

  # ---- converter ----
  - name: ConversionErrorKind
    description: 変換の失敗の種類(converter。公開)。#[doc(hidden)] の code() と ALL を消す(inventory.md 1f、1g)。

  # ---- cli ----
  - name: Reporter
    description: 表示の組み立てと出力(cli。公開)。#[doc(hidden)] の language_resolved と color_resolved を消す(inventory.md 1a)。
  - name: SignalError
    description: シグナルの準備の失敗(cli。公開)。AlreadyRegistered を消し、Failed だけの列挙型にする(D5。決定(ユーザー、2026-11-23)。inventory.md 1i)。Signals::start はプロセスで 1 回だけ呼ぶ。
    attributes:
      - name: Failed
        logical_type: 登録できなかった(終了コード 70)
  - name: panic_hook
    description: パニックのフック(cli。公開のモジュール)。公開は install_panic_hook だけ。本体は非公開の write_internal_error(&mut dyn Write) -> i32(inventory.md 1h)。
  - name: UsageErrorKind
    description: 使い方の誤りの種類(cli。公開)。テストだけが使う ALL を消す(inventory.md 1g)。CommandKind::ALL は本番の from_name が使うので残す。

  # ---- テストの補助 ----
  - name: tests/common/kinds.rs(core・converter・cli)
    description: 誤りの種類の一覧の写し(テストだけ)。一覧の定数、変種ごとの位置を返す網羅の match(const fn)、並びと重複のコンパイル時の確かめ(functional-spec.md §2.3。D1・D2。決定(ユーザー、2026-11-23))。
  - name: FakeEnv::color_read(cli の tests/common)
    description: 偽の Env が、NO_COLOR を読んだか、端末を問い合わせたかを返す(新規)。既存の locale_read と並べて、表示の言語と色の遅延の判定を確かめる(inventory.md 1a)。
  - name: add_env_file(core の tests/common)
    description: LoadedEnvFiles に内容を足し、同じ文字列から引けるようにする(新規。push_file のあと link)(inventory.md 1d)。
```

## 利用者に見える変わること

なし。標準出力と標準エラーの文、終了コード、ステートファイル(バイト列)、`--dry-run` と `graph` の出力、`conductor --version`(`conductor 0.9.8`)は変わらない。
