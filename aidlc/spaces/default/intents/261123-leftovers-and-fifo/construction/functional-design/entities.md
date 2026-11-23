# エンティティ(Entities): 残りの整理と読み込みの順序のそろえ

永続のデータ(ステートファイル)の形は変えない。利用者に見える出力は、段 2 の 3 つ(converter に書き手のいない名前付きパイプを渡したとき、ステートのパスが名前付きパイプや文字デバイスへのリンクのとき、文字列の `command` の先頭の語が空白だけのとき)と版の表示だけが変わる。公開の型の変更は段 1 のもの(`EffectiveEnv`、cli の `Runtime` トレイトと表示の型、converter の誤りの型、公開範囲)。詳細は `inventory.md`。

```yaml
entities:
  # ---- core ----
  - name: EffectiveEnv
    description: 1 つのジョブの 1 回の起動に足す環境(core の envfile/mod.rs。公開)。読まれない job_id とテストだけが読む names() を消す(inventory.md 1b-2)。
    attributes:
      - name: entries
        logical_type: Vec<EffectiveEntry>(今のまま。層の順。完全に同じ名前は 1 つ)
      - name: additions()
        logical_type: Vec<(&str, &str)>(今のまま)
  - name: ExitCodeClass
    description: 終了コードの分類の結果(core の flow/classify.rs)。pub(crate) から classify.rs の中だけの非公開に。classify_exit_code も同じ(inventory.md 1b-4)。
  - name: 公開モジュールと定数(core)
    description: flow の下位モジュール(classify、decide、next_step、resume、retry)、plan::simulate、envfile::{merge, parse}、graph::{edges, layout, overlay} を非公開に。関数と型は今の再公開の名前のまま。定数 EXIT_START_FAILURE・EXIT_SIGNAL_BASE・EXIT_UNLISTED_ZERO・ASSUME_EXIT_CODE_MIN・ASSUME_EXIT_CODE_MAX・RESERVED_ENV_PREFIX・MAX_REPORTED_ERRORS は pub(crate)。MAX_ENV_FILE_BYTES・MAX_ENV_ENTRIES は公開のまま(inventory.md 1e)。
  - name: test_support
    description: core の src の単体テストだけが使う補助のモジュール(新規。#[cfg(test)]。crate の中)。job(id) と definition(name, jobs)(inventory.md 1i-1)。

  # ---- runtime ----
  - name: ReplaceError
    description: 一時ファイルに書いて置き換える処理の失敗(runtime の temp_file.rs。新規。crate の中)。replace_via_temp が返す(inventory.md 1c-1)。
    attributes:
      - name: CreateTemp
        logical_type: (io::ErrorKind)。一時ファイルを作れない
      - name: WriteOrRename
        logical_type: (io::ErrorKind)。書き込み・同期・置き換えの失敗(一時ファイルは消した)
  - name: ReadFailure
    description: 通常のファイルを形 B で上限つきで読む read_regular_file の失敗(runtime の bounded_read.rs。新規。crate の中)。.env(段 1)とステート(段 2)が共有する(inventory.md 1c-2、2b)。上限 + 1 バイトまで読む段は read_opened(読み手と上限を受け取る)に分け、ファイルを使わない単体テスト N8 で確かめる。
    attributes:
      - name: Missing
        logical_type: 無い(.env は fileMissing、ステートは空)
      - name: Unreadable
        logical_type: 通常のファイルでない(開かない)、開けない、種別と大きさを確かめられない、読めない
      - name: TooLarge
        logical_type: 大きさが上限を超える(開いたハンドルの大きさ、または読めたバイト数)
  - name: 非公開にする pub(crate)(runtime)
    description: write_state_file、persist、remove_stale_temps、serialize、temp_name、create_temp、split、read_limited、Waitable、Waited、wait_loop(inventory.md 1f)。

  # ---- converter ----
  - name: RawNode
    description: 読み取りの木の節(converter の raw.rs。crate の中)。行番号の 0 の番兵をやめる(inventory.md 1g-3)。
    attributes:
      - name: kind
        logical_type: RawKind(今のまま)
      - name: line
        logical_type: Option<usize>(YAML は Some(1 始まり)、TOML は None。v0.9.7 までは usize で、TOML は 0)
  - name: ConversionError
    description: 変換の失敗の報告(converter の error.rs。公開)。理由は固定の文だけなので &'static str に(inventory.md 1g-4)。
    attributes:
      - name: file / line / key_path / kind
        logical_type: 今のまま
      - name: reason
        logical_type: '&''static str(v0.9.7 までは String)'

  # ---- cli ----
  - name: Runtime(トレイト)
    description: cli の差し替え口(公開)。run_job から job を外し、runtime の run_job と同じ形にする(Q3: A。inventory.md 1d)。retry.rs の中の job の使い方(環境変数の合成、リトライの判断)は変わらない。
    attributes:
      - name: run_job
        logical_type: (&self, line, env, signal) -> JobExecution(v0.9.7 までは (&self, job, line, env, signal))
  - name: StateContext
    description: ステートの誤りを表示するときの文脈(公開。今のまま 5 種)。for_read は Option<&RunId> を受け取る。run ID の有無を bool で渡す所をこの型にまとめる(inventory.md 1g-2)。
    attributes:
      - name: Read / ReadRunId / JobNotStarted / JobExecuted / Plain
        logical_type: 今のまま
  - name: DisplayEvent::ConsistencyMismatch
    description: ステートと定義の食い違いの表示(公開)。with_run_id: bool を context: StateContext(Read か ReadRunId)に。表示は変わらない(inventory.md 1g-2)。
    attributes:
      - name: state_path / kind
        logical_type: 今のまま
      - name: context
        logical_type: StateContext
  - name: escape_full
    description: 64 文字で切らずにエスケープだけする関数(cli の report/escape.rs。公開)。v0.9.7 までは escape_path(パス以外にも使うため改名。D6。決定(ユーザー、2026-11-23))。

  # ---- テストの偽物(cli の tests/common) ----
  - name: ScriptedRuntime / MemRuntime
    description: テストの偽物の Runtime。run_job の署名をトレイトに合わせる。ScriptedRuntime は env の CONDUCTOR_JOB_ID からジョブ ID を取り(テストの側の事情。無ければパニック。D1。決定(ユーザー、2026-11-23))、今と同じ記録と台本の引き当てを行う。
```

## 利用者に見える変わること(段 2)

| 場面 | v0.9.7 まで | v0.9.8 から |
|------|-------------|-------------|
| `convert` に書き手のいない名前付きパイプを渡す | 開こうとして止まる | 開かずに `unreadable`(`通常のファイルではありません`)、終了コード 67 |
| ステートのパスが書き手のいない名前付きパイプ(`run`・`pipeline`・`watch`・`graph --state`) | 開こうとして止まる | 開かずに `unreadable`(`開く、または読み取ることができません`)、終了コード 65 |
| ステートのパスが `/dev/zero` などの文字デバイスへのシンボリックリンク | 開いて、終わりのない読み取りを上限 + 1 バイトで止め `tooLarge` | 開かずに `unreadable`(`開く、または読み取ることができません`)、終了コード 65(★ T23) |
| `command = "'  '"`(文字列の先頭の語が空白だけ) | 分けられ、起動の失敗(127) | 定義の誤り `command is empty`(64)。配列の `["  "]` と同じ |
| `conductor --version` | `conductor 0.9.7` | `conductor 0.9.8` |
