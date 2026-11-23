# 06. ステートファイル(形式・場所・読み書き・整合性・reset)

対象: `conductor` v0.9.0 の機能を含む現在の実装(Rust 実装)。ステートファイルは、あるパイプライン定義の、実行の途中経過(どのジョブがどう終わったか、どこから再開するか)を保存する TOML ファイル。実行の意味論(いつ何を記録するか)は [05-execution-flow.md](05-execution-flow.md) にある。この文書は、ファイルの形式、読み書きの手順、異常系の挙動を定める。

実装の所在(参考): `crates/runtime/src/state/{reader,writer,record,consistency,reset_all}.rs`、`crates/runtime/src/types.rs`(`StateLocation`)、`crates/runtime/src/error.rs`(理由文)。

---

## 1. 場所と名前

ステートファイルは、**定義ファイルと同じディレクトリ**に置く。

| run ID | ファイル名 |
|---|---|
| なし | `<定義ファイル名>.state` |
| あり | `<定義ファイル名>.<run-id>.state` |

- `<定義ファイル名>` は、定義ファイルのパスの最後の要素(拡張子を含む)。例: `conductor.toml` → `conductor.toml.state`、`--config ci/nightly.toml --run-id 12345` → `ci/nightly.toml.12345.state`。
- 定義ファイルのパスは、`--config` で与えられた文字列のまま(正規化・絶対化しない)。ディレクトリ部分が空(`conductor.toml` のように親が無い)なら、実行ディレクトリ。
- run ID は検査済み(`A-Za-z0-9._-`、1〜64 文字、先頭が `.` でない。05 §6)なので、パス区切りや `..` は入らない。
- 一時ファイルの名前: `.<ステートファイル名>.tmp.<プロセス番号>.<時刻(ナノ秒)>.<連番>`(同じディレクトリ。§4)。
- 大文字小文字を区別しないファイルシステム(macOS の既定、Windows)では、`A` と `a` の run ID は同じファイルになる。食い違いは、中身の `run_id` との照合で壊れたステート(65)として検出される(§5)。

---

## 2. 形式(`format_version = 1`)

### 2.1 エンコード

- UTF-8 の TOML(`toml` クレート 1.x の書式。TOML 1.0)。
- 書き出し: キーは、各テーブル内で**アルファベット順**(辞書順)。最上位のスカラーと配列を先に、続いて `[[job]]`(テーブルの配列)を、`job_order` と同じ順で書く。各 `[[job]]` の前に空行。配列はインライン(`["a", "b", "c"]`)。文字列は `"` で囲む(エスケープの細部はライブラリ任せ。読み込みは、TOML として正しければ、どの文字列表記でも受け付ける)。
- 読み込みはキーの順序に依存しない。

### 2.2 最上位のキー

| キー | 型 | 必須 | 意味・値 |
|---|---|---|---|
| `format_version` | 整数 | 必須 | 常に `1`。読み込みでは最初に検査される(§5) |
| `definition_key` | 文字列 | 必須 | 定義ファイルのパス(`definition_path` と**同じ文字列**を書く。読むときは別々に保持するが、どちらも照合に使わない) |
| `definition_path` | 文字列 | 必須 | 定義ファイルのパス(`--config` の文字列。UTF-8 に変換できない部分は置換される)。空でもよい |
| `pipeline_name` | 文字列 | 必須 | 記録時の `[pipeline].name`。照合に使う(§6) |
| `job_order` | 文字列の配列 | 必須 | 記録時のジョブIDを、定義の記述順に並べたもの |
| `job` | テーブルの配列(`[[job]]`) | 必須 | ジョブごとの状態。`job_order` と同じ順・同じ数(§2.3) |
| `resume_job_id` | 文字列 | 任意 | 次に実行する、または再開するジョブのID。無いときはキー自体を書かない。`job_order` に含まれる値 |
| `end_reached` | 真偽値 | 必須 | `@end` に到達したか。一度 true になったら、その run のあいだ false に戻らない |
| `completed` | 真偽値 | 必須 | 完了したか(§8) |
| `run_id` | 文字列 | 任意 | run ID。run ID ありのステートだけが書く。無いステートにはキーを書かない |

上の表にないキーは、最上位にも `[[job]]` の中にも許さない(§5 の `corrupt`)。旧形式のキー(`jobs`、`[[jobs]]` など)も未知のキーとして拒否される。

### 2.3 `[[job]]` のキー

| キー | 型 | 必須 | 意味・値 |
|---|---|---|---|
| `id` | 文字列 | 必須 | ジョブID |
| `status` | 文字列 | 必須 | `"pending"` / `"success"` / `"warning"` / `"error"` / `"skipped"`(これ以外は `corrupt`。語と型の対応は `JobStatus::as_str`・`JobStatus::parse` の 1 か所) |
| `exit_code` | 整数(`i32` の範囲) | 任意 | ジョブの終了コード。`status` が success / warning / error のときだけ持てる。終了コードが無い結果(起動失敗・シグナル終了)の error には書かない。負数も許す(例 `-1`)。`i32` を超える整数は `corrupt` |
| `skip_reason` | 文字列 | 条件付き | `"byTransition"`(遷移によるスキップ)/ `"byEnd"`(`@end` によるスキップ)。**`status = "skipped"` のときだけ必須、そのほかの `status` では書いてはならない**(語と型の対応は `SkipReason::as_str`・`SkipReason::parse`) |

### 2.4 内容の矛盾(読み込み時に `corrupt` になる条件)

- `job` のジョブID に重複がある。
- `job` の並び(`id` の列)が `job_order` と、数も順序も完全には一致しない。
- `resume_job_id` が `job_order` にない。
- `resume_job_id` があるのに `completed = true`(完了と再開位置は両立しない)。
- `status` が `skipped` なのに `skip_reason` が無い、または `skipped` 以外なのに `skip_reason` がある。
- `status` が pending / skipped なのに `exit_code` がある。

### 2.5 実ファイルの例(実装の出力。全フィールド)

(1)エラーで止まった run ID なしのステート(`conductor.toml` の代わりに `c2.toml`、3 ジョブ。`b` が exit 3)。ファイル名 `c2.toml.state`。

```toml
completed = false
definition_key = "c2.toml"
definition_path = "c2.toml"
end_reached = false
format_version = 1
job_order = ["a", "b", "c"]
pipeline_name = "n2"
resume_job_id = "b"

[[job]]
exit_code = 0
id = "a"
status = "success"

[[job]]
exit_code = 3
id = "b"
status = "error"

[[job]]
id = "c"
status = "pending"
```

(2)`@end` で完了した、run ID ありのステート。ファイル名 `conductor.toml.r1.state`(a が成功し、遷移 `@end`。b・c が `byEnd`)。

```toml
completed = true
definition_key = "conductor.toml"
definition_path = "conductor.toml"
end_reached = true
format_version = 1
job_order = ["a", "b", "c"]
pipeline_name = "nightly"
run_id = "r1"

[[job]]
exit_code = 0
id = "a"
status = "success"

[[job]]
id = "b"
skip_reason = "byEnd"
status = "skipped"

[[job]]
id = "c"
skip_reason = "byEnd"
status = "skipped"
```

(3)全種類の状態を含む例(4 ジョブ。run ID なし、`end_reached = true`、エラーで停止)。

```toml
completed = false
definition_key = "dir/conductor.toml"
definition_path = "dir/conductor.toml"
end_reached = true
format_version = 1
job_order = ["a", "b", "c", "d"]
pipeline_name = "nightly"
resume_job_id = "d"

[[job]]
exit_code = 0
id = "a"
status = "success"

[[job]]
exit_code = 2
id = "b"
status = "warning"

[[job]]
id = "c"
skip_reason = "byEnd"
status = "skipped"

[[job]]
exit_code = -1
id = "d"
status = "error"
```

(この例は、書き込みテストの試料(`status` ごとの値、`skip_reason`、負の `exit_code`)に基づく。形式は上の実出力と同じ。)

### 2.6 初期ステート(`initial_state`)

定義から作る、記録前のステート: 全ジョブ `pending`、`resume_job_id` 無し、`end_reached = false`、`completed = false`、`definition_key` = `definition_path` = 定義ファイルのパス文字列、`pipeline_name` = 定義の名前、`job_order` = 定義のジョブIDの記述順、`run_id` = 決定した run ID(あれば)。

---

## 3. いつ書くか(書き込みの契機)

| 契機 | 内容 | 耐久性 |
|---|---|---|
| ステートが空で、最初のジョブが `Run` と決まった直後(ジョブを起動する前) | 初期ステート。完了のまま残ったステートがあっても、同じ名前のファイルを初期ステートで**上書きする** | `Fast`(同期を省く) |
| ジョブの最終の結果が出た後(リトライを含めて 1 回だけ) | `apply_outcome` の結果。起動前の中断・待機中の中断も、この契機で書く(ジョブの状態は変わらず、`resume_job_id` が付く) | `Durable` |
| スキップが決まったとき(`apply_skip` の結果が元と異なるときだけ) | スキップの記録 | `Durable` |
| `pipeline` のジョブのあいだの中断 | `apply_interrupt` の結果 | `Durable` |
| 完了(`completed = true`)になったとき | 上の契機の書き込みに含まれる(完了のステートも同じ手順) | `Durable` |

- `Reject`(66)では何も書かない(初期ステートも)。
- 書き込みの失敗(65)の表示の末尾に、ジョブの起動前なら `(ジョブは起動していません / the job was not started)`、起動後なら `(ジョブは実行されました / the job has already been executed)` が付く(ロケールに従う)。
- 結果を書けなかったときは、以降のジョブを起動せず、65 で終わる。

---

## 4. 書き込み(`write_state`)の手順と原子性

入力: 定義ファイルのパス、run ID、ステート、耐久性。

```text
1. text = serialize(state)            # §2 の TOML。失敗(通常は起きない)なら WriteFailed(理由=「内容を TOML にできません」)
2. path = StateLocation::of(定義パス, run ID)
3. 一時ファイルを、同じディレクトリに、新規作成で作る(`create_new`: 既存なら失敗)
     名前 = ".<ステートファイル名>.tmp.<pid>.<ナノ秒>.<連番>"
     名前が既存なら、連番を進めて作り直す。最大 16 回。それでも作れなければ WriteFailed
     連番 = プロセス内の通し番号(原子的に増やす) + 試行回数(ラップアラウンド加算)
4. 全内容を書き、flush する
5. Durable のときだけ、ファイルの内容を同期する(`sync_all`)
6. 一時ファイルを閉じる(Windows は、開いたままだと移せない・消せない)
7. rename(一時 → ステートファイル)で置き換える
8. 3〜7 のどこかで失敗したら: 自分が作った一時ファイルを消し、WriteFailed(理由=「書き込みに失敗しました」)
     (3〜8 は `temp_file.rs` の `replace_via_temp`。`init --force` の置き換え(§13)と共有する)
9. 成功し、かつ Durable のとき:
     UNIX 系では、ディレクトリを同期(失敗は無視。Windows では何もしない)
     同じ名前の型の一時ファイルのうち、更新から 24 時間以上たった通常のファイルを消す(失敗は無視)
```

性質:

- **原子的**: 読み手は、直前の完全な内容か、新しい完全な内容のどちらかだけを見る。途中の内容は見えない。どの段階で失敗しても、ステートファイルは直前の完全な内容のまま。
- 置き換え先がディレクトリなど、`rename` が失敗するときは `WriteFailed`。ステートは変わらず、一時ファイルも残らない。書けないディレクトリも `WriteFailed`。
- 残った一時ファイル(プロセスが落ちたとき)は、読み込みに影響しない。`Durable` の書き込みの成功後に、24 時間以上古いものだけ消える。`Fast` では消さない。
- ファイルの権限(パーミッション): 明示的には設定しない。一時ファイルは標準の新規作成の権限(UNIX 系では 0666 から umask を引いた値。実測は `-rw-r--r--`)で作られ、`rename` でそのままステートファイルになる。既存のステートファイルを置き換えるときも、権限は引き継がず、新しい一時ファイルの権限になる。所有者の変更・ACL の操作はしない。
- 排他制御(ファイルロック)はしない。§10。

---

## 5. 読み込み(`read_state` / `read_state_raw`)の手順と異常系

### 5.1 手順

入力: 定義ファイルのパス、run ID(ファイル名側の期待値)。上から順に、最初に当てはまる結果で終わる。1〜4 の上限つきの読み取りは、`crates/runtime/src/bounded_read.rs` の `read_regular_file`(`.env` の読み込みと共有する 1 か所)で、`reset --all` の `inspect_run_state`(8 章)も同じ関数で読む。

| 順 | 処理 | 結果 |
|---|---|---|
| 1 | パスの種別(メタデータ。シンボリックリンクは辿る)を確認 | 存在しない → **空**(`Ok(None)`)。取得失敗 → `unreadable`。通常のファイルでない(ディレクトリ、名前付きパイプ、デバイスへのリンクなど)→ `unreadable`(**開かない**) |
| 2 | ファイルを開く | 存在しない(1 のあとに消えた)→ **空**。その他の開く失敗(権限など)→ `unreadable` |
| 3 | 開いたハンドルの種別と大きさ(メタデータ)を確認 | 取得失敗・通常のファイルでない → `unreadable`。1 MiB(1,048,576 バイト)**超** → `tooLarge` |
| 4 | 上限 + 1 バイトまで読む(終わりのない入力でも止まる) | 読み取り失敗 → `unreadable`。読めたバイト数が上限超 → `tooLarge`(ちょうど上限は受理) |
| 5 | 長さ 0 | **空** |
| 6 | UTF-8 として解釈 | 失敗 → `corrupt`(理由 `UTF-8 の文字列ではありません`) |
| 7 | TOML として解析(最上位はテーブル) | 失敗 → `corrupt`(`TOML として読めません`) |
| 8 | `format_version` の確認(**ほかの検査より先**。将来の版が、未知のキーで corrupt にならないように) | キー無し → `corrupt`(`format_version がありません`)。整数以外 → `corrupt`(`format_version が整数ではありません`)。整数で 1 以外(0 や 2 を含む)→ `versionMismatch`(`対応していない版です(reset が必要です)`) |
| 9 | 最上位に未知のキーがある | `corrupt`(`知らないキーがあります`) |
| 10 | `run_id` の取り出しと照合 | 型が文字列でない → `corrupt`(`値の型が誤っています`)。中身の `run_id`(無ければ「なし」)がファイル名の run ID(無ければ「なし」)と一致しない → `corrupt`(`run_id がファイル名の run ID と一致しません`) |
| 11 | 必須キーの取り出し: `definition_key`、`definition_path`、`pipeline_name`、`end_reached`、`completed`、`job_order`、`job`(この順) | 無い → `corrupt`(`必須のキーがありません`)。型が違う → `corrupt`(`値の型が誤っています`) |
| 12 | `job` の各テーブルを順に検査(§2.3、§2.4 の最後の 2 項) | 未知のキー、`id`/`status` 欠落、`status`/`skip_reason` の不正値、`exit_code` の型・範囲、`skip_reason` と `status` の不整合、`exit_code` と `status` の不整合 → `corrupt` |
| 13 | `resume_job_id` の型 | 文字列でない → `corrupt` |
| 14 | 内容の矛盾(この順): ジョブIDの重複 → `job` の並びと `job_order` の不一致 → `resume_job_id` が `job_order` に無い → `completed` かつ `resume_job_id` あり | いずれも `corrupt`(それぞれ `ジョブIDが重複しています`、`job の並びが job_order と一致しません`、`resume_job_id が job_order にありません`、`completed と resume_job_id が両立しません`) |
| 15 | `read_state` のみ: `completed = true` | **空**(完了のまま残ったステート。ファイルは消さない) |

- `read_state_raw`(`watch` と `graph --state` が使う)は、15 を行わない(完了のステートもそのまま返す)。1〜14 は同じ。
- 検査の `run_id` 照合(10)は、`completed` の判定(15)より前なので、完了のステートでも run ID の食い違いは `corrupt`。
- ステートのパスが通常のファイルでない(書き手のいない名前付きパイプ、ディレクトリ、`/dev/zero` などのデバイスへのシンボリックリンク)とき、開かずに `unreadable`(理由は `開く、または読み取ることができません`)。名前付きパイプでも止まらない。v0.9.7 までは、先に開いたため、書き手のいない名前付きパイプのステートで止まり、`/dev/zero` へのリンクは開いて読み、上限 + 1 バイトで止めて `tooLarge` だった。
- 読む権限のないステートは、1 MiB を超えていても `unreadable`(開く操作が大きさの確認より先に失敗する)。
- 読み取りでは、ファイルを**消さない・書き換えない**。
- 「空」= ステートが無いのと同じ扱い。呼び出し側は、すべて pending のステートとして扱い、最初の `Run` の前に初期ステートを書く。

### 5.2 失敗の表示と終了コード

すべて終了コード 65。標準エラーの 1 行(固定の文 + パス + 固定の理由。入力の値・ライブラリのエラー文は含めない):

```text
conductor: error: <ステートファイルのパス>: <kind の文>: <理由>[; <案内>]
```

| kind | 英語の文 | 日本語の文 | 理由の例(常に日本語) | `reset` の案内を付けるか |
|---|---|---|---|---|
| `unreadable` | `cannot read state file` | `ステートファイルを読めません` | `開く、または読み取ることができません` | 付ける |
| `tooLarge` | `state file is too large` | `ステートファイルが大きすぎます` | `大きさが上限(1 MiB)を超えています` | 付ける |
| `corrupt` | `state file is corrupt` | `ステートファイルが壊れています` | §5.1 の各理由 | 付ける |
| `versionMismatch` | `state file version is not supported` | `ステートファイルの版が合いません` | `対応していない版です(reset が必要です)` | 付ける |
| `writeFailed` | `cannot write state file` | `ステートファイルを書き込めません` | `書き込みに失敗しました` / `内容を TOML にできません` | 付けない(§3 の `(ジョブは…)` が付く) |
| `removeFailed` | `cannot remove state file` | `ステートファイルを削除できません` | `削除に失敗しました` | 付けない |

案内(読み込みのときだけ): run ID なし → `; run \`conductor reset\` to discard the state`(日本語 `; \`conductor reset\` でステートを初期化できます`)、run ID あり → `; run \`conductor reset --run-id <ID>\` to discard the state`(`<ID>` は文字どおりの `<ID>`)。

実例(英語ロケール):

```text
conductor: error: c2.toml.state: state file is corrupt: TOML として読めません; run `conductor reset` to discard the state
conductor: error: c2.toml.state: state file version is not supported: 対応していない版です(reset が必要です); run `conductor reset` to discard the state
conductor: error: c2.toml.r3.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state
```

### 5.3 異常系の挙動の一覧(`run` / `pipeline`)

| 状況 | 結果 | ジョブの起動 | ステート |
|---|---|---|---|
| ファイルが無い | 空として続行 | する | 最初の `Run` の前に初期ステートを書く |
| 長さ 0 | 空として続行 | する | 同上(上書き) |
| `completed = true` | 空として続行(定義との照合もしない) | する | 初期ステートで上書き |
| 読めない(権限等) | 65 | しない | 変更なし |
| 1 MiB 超 | 65 | しない | 変更なし |
| UTF-8 でない・TOML でない・未知のキー・必須キー欠落・型の誤り・内容の矛盾 | 65(`corrupt`) | しない | 変更なし |
| `format_version` が 1 以外の整数 | 65(`versionMismatch`) | しない | 変更なし |
| 中身の `run_id` とファイル名の run ID が不一致、または片方だけ有る | 65(`corrupt`) | しない | 変更なし |
| 定義と食い違い(§6) | 65 | しない | 変更なし |
| 書き込み・削除の失敗 | 65 | §3 の表のとおり | 直前の内容のまま(書き込み)/ 残る(削除) |

どの異常でも、回復は `conductor reset`(または `--run-id`、`--all`)。`reset` は読まず検査もしないため、どんなファイルでも消せる(§7)。

---

## 6. 定義との整合性検査(`check_consistency`)

空でない(完了していない)ステートを読んだ後、`run`・`pipeline` の前処理(`Ctx::read_checked_state`)で、定義と照合する。純粋な関数(`crates/core/src/consistency.rs`。`conductor-runtime` が同じ名前で再公開する)。

```text
if state.pipeline_name != definition.name:              Mismatch(PipelineName)
defined  = 定義のジョブIDの記述順の配列
recorded = state.job_order
if defined == recorded:                                  Ok
if sorted(defined) == sorted(recorded):                  Mismatch(JobOrder)    # 並びだけが違う
else:                                                    Mismatch(JobIds)      # 追加・削除・改名
```

| 状況 | 結果 |
|---|---|
| パイプライン名が違う | `PipelineName` |
| ジョブID の集合が違う(追加・削除・改名。重複の有無も含め、並べ替えた列が違う) | `JobIds` |
| 集合は同じで記述順だけが違う | `JobOrder` |
| すべて同じ | `Ok` |

- `definition_key` と `definition_path` は照合に使わない(定義ファイルの置き場所を動かしても食い違いにしないため)。
- ジョブのコマンド、遷移、成功コード、リトライ設定などの違いは検知しない(IDと並びだけを見る)。
- `completed = true` のステートは「空」なので、照合しない。
- 食い違いのとき、ジョブは起動せず、ステートも変えず、65。標準エラー:

```text
conductor: error: <ステートファイルのパス>: state does not match the definition: <種類>; <案内>
```

  種類: `pipeline name` / `job ids` / `job order`(日本語: `パイプライン名` / `ジョブID` / `ジョブの並び`)。日本語の文は `ステートが定義と食い違っています`。案内は §5.2 と同じ(run ID あり/なしで切り替わる)。

---

## 7. 初期化(`reset`)

### 7.1 `conductor reset [--config <パス>] [--run-id <ID>]`

- 読まず、検査せず(破損・食い違い・版違い・大きさに関係なく)ステートファイルを削除する。定義ファイルは検証しない。
- 対象のファイル: run ID の決定(05 §6.2。`--run-id` があれば定義を読まない。なければ定義の `run_id_env` だけをゆるく読み、その環境変数を見る)で決まった 1 つ。
- 結果: ファイルが無ければ**成功**(0、何も表示しない)。`remove_file` が成功 → 0。失敗(ディレクトリ・権限等。「存在しない」以外)→ `removeFailed`、65。
- 完了のステートも消す。標準出力・標準エラーには何も出さない。
- `remove_file` の挙動のとおり、パスがシンボリックリンクなら、リンク自体を削除する。
- run ID の値が規則に合わなければ 68。

### 7.2 `conductor reset --all [--config <パス>]`

定義の全ステートを、run ID の有無によらず消す。環境変数は見ない(値の誤りも 68 にならない)。`--run-id` との併用は 68。

対象の判定(厳密):

1. 一覧にするのは、定義ファイルと同じディレクトリだけ(再帰しない)。
2. `<定義ファイル名>.state`(run ID なし): **通常のファイルのときだけ**(`symlink_metadata` で判定。シンボリックリンクは辿らず、消さない)、中身を読まずに削除。
3. `<定義ファイル名>.<run-id>.state`: 名前の `<run-id>` が有効な run ID(§1・05 §6.1)であるものだけが候補。名前は、定義ファイル名で始まり、続けて `.`、run ID、`.state`。候補をファイル名の昇順に処理する。
   - 通常のファイルでない(ディレクトリ、シンボリックリンク)→ 黙って除外。取得中に消えた → 除外。メタデータの取得が失敗 → `kept` に入れる。
   - 中身を見分ける(`inspect_run_state`):

| 中身 | 結果 |
|---|---|
| 開けない、1 MiB 超、読み取り失敗、UTF-8・TOML・形式の誤り(`run_id` の不一致以外の `corrupt`)、`versionMismatch` | **残して**、名前を `kept` に入れる |
| 長さ 0 | 残す。`kept` に**入れない**(誰のものか分からない) |
| 読めて、中身の `run_id` が名前の run ID と一致(完了のままのものを含む) | **削除** |
| 読めるが、中身の `run_id` が無い、または値が違う(別の定義のステートでありうる) | 残す。`kept` に入れない(黙って残す) |

4. 一時ファイル(`.<ステート名>.tmp.*`)、ほかの名前のファイル、ほかのディレクトリは対象外。
5. 削除の失敗(「存在しない」以外)→ `removeFailed`、65(処理は途中で打ち切る)。ディレクトリ一覧の取得に失敗(ディレクトリが無い場合を除く)→ `unreadable`、65。ディレクトリが無ければ成功。

結果の出力: `kept` が空でなければ、標準エラーに警告(終了コードは 0)。消すものがなければ何も出さず 0。

```text
conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):
  c2.toml.r2.state
```

日本語: `conductor: warning: 読めなかったため、次のステートファイルは残しました(`conductor reset --run-id <ID>` で個別に消せます):`、続けて 2 スペースで字下げしたファイル名を 1 行ずつ。

実測(`c2.toml` に対して `reset --all`): run ID なしの `c2.toml.state` は削除、順序が壊れた `c2.toml.r2.state` は `kept`(警告)、`run_id = "r2"` を持つが名前が `r3` の `c2.toml.r3.state` は黙って残る、ほかの定義のステート `conductor.toml.r1.state` は対象外で残る。

---

## 8. 完了と `--remove-on-complete`

### 8.1 完了のステートの扱い

- 完了(`completed = true`)の条件は 05 §10。完了のステートは、`completed = true` のまま**残る**(自動では消えない)。書式は変わらない(`format_version = 1`)。
- 読み込み(`read_state`)は、完了のステートを「空」として返す(§5.1 の 15)。したがって次の `pipeline` は先頭から始まり、`run` は最初のジョブ以外が順序外(66)になる。
- `watch` は `read_state_raw` で完了のステートを読み、表の下に完了を示す行(英語 `completed`、日本語 `完了`)を足す。`no state (nothing has run yet)` を出すのは、ファイルが無いか長さ 0 のときだけ。壊れた完了のステートは 65。
- `reset`・`reset --all` は完了のステートも消す。

### 8.2 削除の指定

| 項目 | 値 |
|---|---|
| オプション | `--remove-on-complete`(`run`・`pipeline` だけ。値を取らない。ほかのコマンドに付けると 68) |
| 定義 | `[pipeline]` の `remove_on_complete`(真偽値。省略時 false。真偽値でなければ定義の誤り 64) |
| 実効値 | `--remove-on-complete` があれば true、なければ定義の値。環境変数では切り替えない。オプションで false にする手段はない |

完了時の手順: 完了のステートを `Durable` で書く → 実効値が真なら `reset_state`(run ID に応じたファイルを削除。存在しなくても成功)→ 成功なら 0(`run` ではジョブの終了コード)。削除に失敗したら 65(完了のステートは `completed = true` のまま残る。`watch` は完了として表示する)。

`run` では、完了の契機が「スキップの記録」のときも同様に、書き込みの後に削除する。

---

## 9. 整合した状態の不変条件(実装が保つもの)

`apply_*` が作るステートは、次を満たす(読み込みの検査と対応する)。

- `job` と `job_order` は同じ順・同じID。
- `skipped` のジョブにだけ `skip_reason` がある。`exit_code` は結果のある状態(success / warning / error)にだけある。
- `end_reached` は単調(true になったら戻らない)。
- `completed = true` のとき `resume_job_id` は無い(`NextStep` が完了のとき再開位置を持たない。`apply_skip` は `resume_job_id = first_pending` で、完了の条件は `first_pending` が無いこと)。
- ただし、`apply_outcome` が、エラーのジョブの状態を保つ「起動前の中断」のとき、`status = error` のまま `exit_code` が消える(05 §12 の 2)。
- (v0.9.0 以降)ステートに、環境変数(名前も値も)、`.env` の内容、`env` の設定を**書かない**。ステートの書式(§2)は v0.8.0 と同じで、`PipelineState` の型に環境変数の項目はない。再開のときも、`.env` は、呼び出しごとに読み直す(05 の 14.5)。
- (v0.9.0 以降)`graph --state` は、ステートを `read_state_raw` で、完了したものも含めて**読むだけ**で、書かず・消さない。`--dry-run` は、`read_state`(完了したものは「空」)で読むだけで、書かず・消さない。どちらも、定義との照合(§6)を行い、食い違いは 65。

---

## 10. 並行実行・排他・持続性

- **排他制御はしない**(ファイルロック、PID ファイルなし)。同じ定義(同じ run ID)で複数の conductor を同時に動かすと、それぞれが読んで書き、後に書いた方が勝つ。書き込み自体は原子的なので、ファイルが壊れることはないが、記録は失われうる。同時の実行は避けること。
- 別の実行単位(run)で同じ定義を並行して使うときは、run ID でステートを分ける(§1)。run ID ごとに独立したファイルと再開位置を持つ。
- **at-least-once**: ジョブの終了と、結果の書き込みのあいだに conductor が落ちると、ジョブは未実行として扱われ、再実行されうる。
- **リトライの回数はステートに書かない**(書式に項目がない)。再開は、そのジョブを最初の実行から数え直す。
- ステートは、定義ファイルのあるディレクトリに作られる。書き込めないディレクトリでは、最初のジョブの前に 65 で止まる(初期ステートが書けない)。
- 古い run のステートは自動では消えない。`reset --all` か、`--remove-on-complete` で片づける。

---

## 11. 実装と README の差

README とステートの実装のあいだに、矛盾は見つからなかった。README が詳しく書いていない、実装の挙動(この文書は実装に従う)を挙げる。

| # | 項目 | 実装の挙動 |
|---|---|---|
| 1 | `reset --all` と長さ 0 のファイル | 長さ 0 の `<定義>.<run-id>.state` は、消さず、警告にも出さない(README は「読めない・壊れている」ものを警告する、としか書かない) |
| 2 | `reset`(個別)とシンボリックリンク | 個別の `reset` は `remove_file` で、リンク自体を削除する。README の「シンボリックリンクは消しません」は `reset --all` の説明 |
| 3 | 書き込みの手順の細部 | 一時ファイルの名前、新規作成、同期の有無(`Fast` / `Durable`)、24 時間以上古い一時ファイルの後始末は、README に記述がない(README は「一時ファイルに書いてから置き換える」まで) |
| 4 | 読み込みの検査の順序と上限 | 1 MiB の上限、未知のキーの拒否、`format_version` の先行検査は、README に記述がない(README は「読めない・壊れている・版の不一致」とだけ書く) |
| 5 | 理由文の言語 | 理由(例: `TOML として読めません`)は、ロケールに関わらず日本語の固定文。種類の文と案内はロケールに従う |
| 6 | `definition_key` | 書式に残っているが、`definition_path` と同じ値で、照合に使わない |

## 12. 未確認

- `toml` クレートが、特殊な文字(バックスラッシュ、引用符、制御文字)を含む文字列を、どの表記(基本文字列 / リテラル文字列)で書き出すか。読み込みは TOML として正しければ受け付けるので、再実装は、TOML として正しい出力ならよい。
- Windows での `rename` による置き換えの細部(既存ファイルが他のプロセスに開かれているとき)と、ディレクトリを `File::open` したときの `unreadable` の判定(UNIX 系では、ディレクトリを開いて読むのに失敗して `unreadable` になる想定だが、テストで確認していない)。
- ファイルの権限の挙動は、UNIX 系の実測(`-rw-r--r--`)と、コードに権限の設定が無いことによる。Windows の ACL は未確認。
- 一時ファイルの `<時刻>` は、`UNIX_EPOCH` からのナノ秒(取得に失敗すれば 0)。クロックが戻った場合の挙動は、名前の衝突の再試行(最大 16 回)に任せている。

## 13. ファイルの排他的な作成(`create_file`。`init` が使う。v0.9.0 以降)

ステートの書き込みとは別の部品だが、同じ「一時ファイルから置き換える」作り方(§4 の 3、6、7)を共有するので、ここに置く。実装は `crates/runtime/src/file_create.rs`。

`create_file(path, bytes, force) -> Result<CreateOutcome, CreateError>`:

| 型 | 内容 |
|----|------|
| `CreateOutcome` | `Created`(新規に作った)、`Replaced`(`--force` で、既にあったファイルを置き換えた) |
| `CreateError` | `AlreadyExists`(既にある。`force` なし)、`ParentMissing`(親のディレクトリが無い。作らない)、`IsDirectory`(作る先がディレクトリ)、`PermissionDenied`(書く権限がない)、`OtherIo`(そのほかの入出力の誤り) |

**`force = false`(新規)**:

1. `OpenOptions::new().write(true).create_new(true).open(path)`(ファイルがなければ作る、1 回の操作)。失敗したら、パスの今の姿(`symlink_metadata`)で分類する: ディレクトリなら `IsDirectory`、ファイル、またはシンボリックリンク(宙に浮いたリンクを含む)なら `AlreadyExists`、メタデータも取れなければ、元の誤りの種類(`NotFound` → `ParentMissing`、`PermissionDenied` → `PermissionDenied`、そのほか → `OtherIo`)。OS によって、ディレクトリへの `create_new` の誤りの種類が違うため、この分類にする。
2. 全内容を書き、flush する。閉じてから(Windows は、開いたままのファイルを消せない)、書き込みに失敗していれば、作ったファイルを消して `OtherIo`。成功は `Created`。

**`force = true`(置き換え)**:

1. 親のディレクトリ(`path.parent()`。空なら `.`)のメタデータを取る。取れなければ種類で分類(`NotFound` → `ParentMissing`、`PermissionDenied` → `PermissionDenied`、そのほか → `OtherIo`)。ディレクトリでなければ `OtherIo`。
2. `path` の `symlink_metadata`: ディレクトリなら `IsDirectory`。ファイルまたはシンボリックリンクなら `existed = true`(シンボリックリンクは、リンクそのものを置き換える)。無ければ `existed = false`。
3. 同じディレクトリに、一時ファイルを新規作成で作る(3・4 は `temp_file.rs` の `replace_via_temp`。中で `create_temp_beside` → `create_temp`。ステートの書き込みと同じ関数・同じ連番。名前 = `.<ファイル名>.tmp.<pid>.<ナノ秒>.<連番>`。§4 と同じ作り方。最大 16 回)。作れなければ、種類で分類する。
4. 全内容を書き、flush し、閉じて、`rename(一時 → path)` で置き換える。書き込みか `rename` に失敗したら、一時ファイルを消す(`PermissionDenied` は `PermissionDenied`、そのほかは `OtherIo`)。成功は、`existed` なら `Replaced`、でなければ `Created`。

- **親のディレクトリは作らない**。内容は、そのまま書く(改行を変換しない)。同期(`sync_all`)はしない。
- 表示は、`CreateError` の種類ごとの固定の文とパスだけで、OS のエラーの文を含めない(08 の 4.13)。
- 置き換えたファイルは、新しい権限と所有者になる(UNIX 系では、新規作成の権限。§4 の権限の項と同じ)。Windows では、読み取り専用のファイルは置き換えられず、UNIX 系では置き換えられる。
- **既知の限界**: 新規の作成は、ファイルを作ってから書くので、その短い間と、強制終了のあとには、空のファイルが見えることがある(その場合、2 回目の `init` は `AlreadyExists` になる。`--force` で直せる)。

