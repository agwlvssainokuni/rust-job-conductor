# 機能の仕様(Functional Spec): 実行の結果の型、run_job の入力、定義ファイルの読み込みの順序

意図: `ideation/intent-capture/intent-statement.md`(Q1 A、Q2 A、Q3 A、Q4 B)。§9 の D1・D2・D4・D5 はユーザーが決めた(2026-11-23)。D3 は設計の承認で認められた(2026-11-23)。**変える場所の正確な一覧(ファイルと行、今の形、目標の形)は `inventory.md`**(HEAD d4e6b82 で作った)。この文書は、方針、型と順序の設計、**承認が要るテストの変更(§4)**、足すテスト、コミットの並び、確かめ方、範囲の外、**決めてほしい点(§9)**を書く。

## 1. 方針

- 3 つの段を、この順に進める。段 1(`JobExecution` の型)、段 2(`run_job` に分割済みのコマンドを渡す。F)、段 3(定義ファイルの読み込みの順序。B3)[desc]。
- 段 1・2 は、利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` の出力)を変えない [desc]。段 3 だけが利用者から見える変更で、変わるのは「読む権限のない 1 MiB 超の定義ファイル」の誤りの種類(`tooLarge` → `unreadable`)だけ [Q3]。
- 各段は小さなコミットの積み重ねで、どのコミットでもビルドとテストが通る [Q1] [memory:M1]。承認は、この設計とコードの計画の 1 回。各段の終わりにテストの件数を記録し、結果は最後にまとめて見せる [Q1]。
- 新しい依存を足さない [memory:M5]。`unsafe` は runtime の `platform/unix.rs` だけ、`cfg(unix)`/`cfg(windows)` は `platform` の外に置かない、誤りの表示に入力の値を含めない、ステートに環境変数を書かない、converter は core に依存しない、MSRV 1.100 [memory:M3]。テスト名とドキュメントコメントは日本語。
- 最後のコミットで版を 0.9.6 に上げる。タグとリリースはしない [Q4] [memory:M6]。

## 2. 段ごとの設計

### 2.1 段 1: `JobExecution` を、あり得る場合だけを表す型にする

#### 2.1.1 新しい型(`crates/core/src/types.rs`)

```rust
/// 1 回のジョブの起動の結果(契約 C1。生成は conductor-runtime と、cli のリトライの待機)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecution {
    pub job_id: String,
    pub command: Command,
    /// 起動の結果(あり得る 3 つの場合だけを表す)。
    pub result: ExecutionResult,
}

/// 起動の結果の場合分け(BR1.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionResult {
    /// 起動しなかった。合図が起動の前に立っていた(`run_job`)か、リトライの待機中に立った(cli)。
    /// `signal` は、最初に受け取ったシグナルの番号。
    NotStarted { signal: i32 },
    /// 起動に失敗した(`spawn` の失敗、終了の状態を取得できない、空のプログラム名)。
    StartFailed,
    /// 起動して、終了を観測した。
    Exited {
        /// 終了コード(UNIX でシグナルで終わったときは `None`)。
        exit_code: Option<i32>,
        /// シグナルで終わったときの番号(UNIX のみ。Windows は常に `None`)。
        terminated_by_signal: Option<i32>,
        /// 終了の観測の直後に中断の合図が立っていたら、最初に受け取ったシグナルの番号。
        interrupted_by: Option<i32>,
    },
}
```

- 組み立ての関数(`impl JobExecution`): `new(job, result)`、`exited(job, exit_code)`(今のまま。`Exited { Some(c), None, None }`)、`not_started(job, signal)`、`start_failed(job)`。`job_id` と `command` は `job` から写す(今と同じ)。
- 公開: `ExecutionResult` は `JobExecution` と同じく `crates/core/src/types.rs` に置き、`conductor_core` のルートから公開する(`crates/core/src/lib.rs:116` の `pub use types::*;` に含まれる。`use conductor_core::ExecutionResult;` で使える)。runtime・cli・テストはこの道で使う。
- 今の項目と同じ意味の読み出しの関数(`started()`、`interrupted()` など)は**置かない**(Q2 で B を採らなかった理由と同じ。読む側は `match` で場合を見る)[Q2]。
- 名前は D5 で決定(`ExecutionResult`、項目名 `result`。ユーザー、2026-11-23)。
- 消えるもの: 3 つの真偽値(`started`、`start_failure`、`interrupted`)と、`interrupted` に付随する `signal_number`。`exit_code` と `terminated_by_signal` は `Exited` の中に移る(起動していないときに値を持てなくなる)。

#### 2.1.2 今の値から新しい型への対応

今のコードが作る値は 13 の場所(`inventory.md` 1b の P1〜P13)で、すべて次の 3 つの形のどれか 1 つに当たる。

| 新しい型 | 今の値(started / start_failure / interrupted / signal_number) | 作る場所(inventory 1b) |
|----------|------------------------------------------------------------------|---------------------------|
| `NotStarted { signal: n }` | 偽 / 偽 / 真 / `Some(n)`、`exit_code`・`terminated_by_signal` は `None` | P2、P8、P9、P13(起動前の合図) |
| `StartFailed` | 真 / 真 / 偽 / `None`、`exit_code`・`terminated_by_signal` は `None` | P1(段 2 で消える)、P3、P6、P12 |
| `Exited { exit_code, terminated_by_signal, interrupted_by }` | 真 / 偽 / 偽 / `None` なら `interrupted_by: None`。真 / 偽 / 真 / `Some(n)` なら `interrupted_by: Some(n)` | P4、P5、P7、P10、P11、P13(終了コード) |

ほかの組み合わせ(たとえば `started = false` で中断でもない、`start_failure` と `interrupted` が同時、`interrupted` なのに `signal_number` が `None`)は、どの場所も作らない(テストの中の組み立てだけにある。§4 の T3、T9)。

#### 2.1.3 読む側の結果が同じであること(同値の表)

左の列の値ごとに、今の読み方の結果と新しい読み方の結果が一致する。「今」の列は、`classify.rs:62-101` の判定の順(07 章 4.4 の順 1〜10)で、その値が最初に当てはまる行。

| 新しい値 | `classify`(今 → 新。分類、返す終了コード、`conductor_error`) | `result_detail`(cli の表示) | 中断の表示 | リトライの対象 | `apply_outcome` が写す `exit_code` | `run_step` の `after` |
|----------|------------------------------------------------------------------|-------------------------------|------------|----------------|-------------------------------------|------------------------|
| `NotStarted { signal: n }` | 順 1: 分類なし、`128 + n`、偽 → 同じ | `Interrupted { signal: n, started: false }` | `n` | 否 | `None` | `JobNotStarted` |
| `StartFailed` | 順 4(中断でないので順 3 は通らない): エラー、127、偽 → 同じ | `StartFailure` | なし | 否 | `None` | `JobExecuted` |
| `Exited { interrupted_by: Some(n), .. }` | 順 3: エラー、`128 + n`、偽 → 同じ(終了コードとシグナルは見ない) | `Interrupted { signal: n, started: true }` | `n` | 否 | 観測した `exit_code` | `JobExecuted` |
| `Exited { terminated_by_signal: Some(s), interrupted_by: None, .. }` | 順 5: エラー、`128 + s`、偽 → 同じ | `Signal(s)` | なし | 否 | 観測した `exit_code`(UNIX では `None`) | `JobExecuted` |
| `Exited { exit_code: None, terminated_by_signal: None, interrupted_by: None }` | 順 6: エラー、1、真 → 同じ | `ExitCode(0)` | なし | 否 | `None` | `JobExecuted` |
| `Exited { exit_code: Some(c), terminated_by_signal: None, interrupted_by: None }` | 順 7〜10: 終了コードで分類 → 同じ | `ExitCode(c)` | なし | `c` が `exit_codes` にあれば対象 | `Some(c)` | `JobExecuted` |

- 新しい `classify` は `match` で、上の表の上から順に判定する。今の順 2(「起動していないのに中断でもない → 127」)は、型の上で作れないので分岐ごと消える(`classify.rs:58-60` の説明も)。
- `retry_decision` が `Retry` になるのは最後の行だけなので、cli の `RetryScheduled.last_exit_code` は常にその `c`(今の `exit_code.unwrap_or_default()` と同じ値)。
- この表の 6 行を、そのまま 1 つの単体テストにする(§5 の N1)。

#### 2.1.4 ステートファイル

`JobExecution` はステートファイルに書かれない(`serde` の derive が無い。書くのは `PipelineState` と `JobState`)。ステートに届くのは `apply_outcome` が写す `exit_code` の値だけで、§2.1.3 の表のとおり今と同じ値になる。よって書式も値も変わらない(`inventory.md` 1e)。既存のステートのテスト(`runtime/tests/state_*.rs`、`flow_with_state.rs`、cli の `flow_*.rs` の `status_of`・`state()`)がそのまま通ることで確かめる。

#### 2.1.5 仕様書

rust の 02・05・07・11・12 の該当の行(`inventory.md` 1f)を、3 つの場合の言葉に直す。05 §2.3 の順 1'、07 §4.4 の順 2(「起こらない」)を消す。language-agnostic は段 1 では変えない(D2 の決定。真偽値の観測モデルは、言語に依らない説明として正しいまま)。

### 2.2 段 2: `run_job` が分割済みのコマンドを受け取る(F)

#### 2.2.1 署名

- runtime: `pub fn run_job(job: &JobDefinition, line: &CommandLine, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution`。
- cli: `Runtime::run_job(&self, job: &JobDefinition, line: &CommandLine, env: &[(&str, &str)], signal: &InterruptSignal) -> JobExecution`。`SystemRuntime` はそのまま渡す。
- cli の流れ: `run_job_with_retry(ctx, session, job, line)`。`run_step` が持つ `command: &CommandLine`(`session.checked.commands` の同じ位置。今は表示にだけ使う)を渡す。リトライの何回目でも、同じ `line` を使う。
- 引数の名前は `line` にし、`Command::new(&line.program)` の書き方を保つ(`nfr_source.rs` の検査)。
- `run_job` の中に、分ける呼び出し(`command_line`、`split`、`check_command`)は残らない。分割は、実行の前の検査(`Ctx::check_definition`)の 1 回だけになる。

#### 2.2.2 分けられないときの防御の行き先

| 今 | 今後 |
|----|------|
| `run_job` の中で分け直し、失敗したら `start_failure`(`runner.rs:119-125`。cli が先に検査するので通常は来ない) | **型で不要になる**: `run_job` は分割済みの `CommandLine` を受け取るので、「分けられない」は `run_job` に届かない。分けられない `command` は、今どおり実行の前の検査(`check_definition`)が定義の誤りとして報告して 64 で終わる |
| (同じ場所で)`Command::Argv` の先頭が空・空白だけなら `Empty`(`split.rs:139-146`。定義の検査で防いだうえでの二重の検査) | **1 行の防御が残る**: `CommandLine` のフィールドは公開なので、呼び出し側が空のプログラム名を直接作れる。`run_job` は、`line.program` が空または空白だけなら `StartFailed` を返す(起動しない。分割はしない)。OS の `spawn` に空の名前を渡したときの挙動(Windows で確かめていない)に頼らないため(D4 の決定)。**この確認は、起動前の合図の確認より前に置く**(今の `run_job` の順「分割の失敗 → 合図」に合わせる。後ろに置くと、空の `program` で合図が立っていたときの結果が `StartFailed` から `NotStarted` に変わるため) |

- 起動の失敗の表示(`report_execution` のコマンド名)は今どおり `line.program` を使う。
- `CommandLine` のフィールドを非公開にして、`check_command` だけが作れるようにする案は、`dry_run.rs`、`step.rs` とテスト(`nfr_robust.rs` の `line.args` など)の書き方を変えるので採らない(D4 の代案)。

#### 2.2.3 仕様書(「4 つの章」)

rust の **02**(署名、流れの図、`Runtime` トレイト)、**05**(起動の手順、§7.5 の擬似コード、§14.4 の見出し)、**07**(§3.6 の検査の時点と防御、§4.1 の署名と手順 2、§4.3 の `start_failure` の中身、§13 相当の `:445`)、**12**(作り直しの手順の `run_job`)の 4 つ。行は `inventory.md` 2d。あわせて 11 章のテストの説明(`nfr_robust.rs`、`runner_basic.rs`)を直す。署名の変更には「v0.9.5 までは `run_job(job, env, signal)` で、中で分け直した」の注記を付ける(02・07 の既存の「v0.8.0 以前は…」と同じ書き方)。

language-agnostic は、D2 の決定に従い最小限だけ直す: 05 §1.2 の起動の手順 1(「command を引数に分ける。失敗なら `start_failure`」)と §7.5 の擬似コードの `run_job` の引数、07 §3.6 の防御の行、§4.1 の入力(「ジョブ、分割したコマンド、足す環境変数の並び」)と手順 2、§4.3 の `start_failure` の中身(「分割の失敗」を外す)。行は `inventory.md` 2d。真偽値の観測モデルそのものは変えない(段 1 と同じ)。

### 2.3 段 3: 定義ファイルの読み込みを「開いてから、開いたものの大きさを見る」順にする(B3)

#### 2.3.1 新しい順序(D1 で形 B に決定。ユーザー、2026-11-23)

**形 B(採る形)**: 種別だけ先にパスで見てから開き、大きさは開いたハンドルで見る。

1. `std::fs::metadata(path)`: `NotFound` → `FileMissing`、ほかの失敗 → `Unreadable`(権限 / 失敗)。通常のファイルでない → `Unreadable`「通常のファイルではありません」。**大きさは見ない**。
2. `File::open(path)`: `NotFound`(1 のあとに消えた)→ `FileMissing`、ほか → `Unreadable`(権限 / 失敗)。
3. 開いたハンドルの `metadata()`: 取れない → `Unreadable`。通常のファイルでない(1 のあとに差し替わった)→ `Unreadable`「通常のファイルではありません」。大きさが 1 MiB を超える → `TooLarge`。
4. 上限 + 1 バイトまで読む(超えれば `TooLarge`、失敗は `Unreadable`)。UTF-8 でない → `Unreadable`。

**形 A(採らない形。記録のため残す)**: `crates/converter/src/intake.rs:48-98` と同じ。先に `File::open`(`NotFound` → `FileMissing`。ほかの失敗は、パスのメタデータで通常のファイルでないと分かれば「通常のファイルではありません」、そうでなければ権限 / 失敗)→ 開いたハンドルの `metadata()` で種別と大きさ → 読み取りと UTF-8。

**Q3 の「先に開く」を字のとおりには守らない理由**: Q3(A)の狙いは、(1) 読む権限のない大きなファイルを「読めない」と報告すること、(2) 大きさを開いたものから見て、確かめてから開くまでの差し替えの心配をなくすこと、の 2 つ。形 B は、大きさを見る前に開き、大きさは開いたハンドルから取るので、2 つとも満たす。開く前にパスで見るのは種別(通常のファイルか)だけで、大きさは見ない。種別を先に見るのは、書き手のいない名前付きパイプ(FIFO)を開くと、書き手が来るまで待ち続けるため(形 A と converter はそうなる)。今の core は FIFO で止まらないので、形 A にすると、告げていない「止まる」動きが `load_definition` と `run_id_env` に入ってしまう。誤りの種類と理由は、FIFO を除くすべての場合で converter と同じになる(§2.3.2)。確かめてから開くまでに差し替わっても、開いたハンドルで種別と大きさを見直すので、上限を超えて読むことはない。ただし、種別を見たあと開く前に、通常のファイルが書き手のいない FIFO に差し替わる競合では、形 B でも開く時点で止まりうる(この競合を除いて止まらない。今の core にも、確かめてから開くまでの同じ隙間がある)。

#### 2.3.2 誤りの種類(どちらの形でも同じものと、違うもの)

| 場合 | 今(v0.9.5) | 形 B | 形 A | converter |
|------|-------------|------|------|-----------|
| ファイルが無い | `fileMissing` | `fileMissing` | `fileMissing` | `fileMissing` |
| ディレクトリ(UNIX) | `unreadable`(通常でない) | 同じ(1 で止まる) | 同じ(開けて、ハンドルが通常でない) | 同じ |
| ディレクトリ(Windows。開くと権限の誤り) | `unreadable`(通常でない) | 同じ(1 で止まる。開かない) | 同じ(converter と同じ補いで「通常でない」) | 同じ |
| 読む権限のない小さなファイル | `unreadable`(権限) | 同じ | 同じ | 同じ |
| **読む権限のない 1 MiB 超のファイル** | **`tooLarge`** | **`unreadable`(権限)** | **`unreadable`(権限)** | `unreadable`(権限) |
| 読めて 1 MiB 超 | `tooLarge` | `tooLarge` | `tooLarge` | `tooLarge` |
| 1 MiB 超で UTF-8 でない | `tooLarge` | `tooLarge` | `tooLarge` | `tooLarge` |
| UTF-8 でない | `unreadable`(UTF-8) | 同じ | 同じ | 同じ(文は converter のもの) |
| 書き手のいない名前付きパイプ(FIFO。UNIX) | `unreadable`(通常でない。開かない) | 同じ(1 で止まる。種別を見たあと開く前に FIFO に差し替わる競合を除く) | **開く時点で、書き手が来るまで待ち続ける** | 待ち続ける |
| 親のディレクトリに入れない | `unreadable`(権限) | 同じ | 同じ | 同じ |

- Windows のディレクトリ(converter の `not_regular` の扱い): **core にも揃える**。形 B では 1 で止まり、開かないので補いは要らない。形 A では converter と同じ補いを写す。どちらでも、今の `ディレクトリは_unreadable`(理由に「通常のファイル」)が Windows の CI でも通る。
- 形 A は、書き手のいない FIFO で止まる(今の core には無い動き。`load_definition` と `run_id_env` の両方)。converter にはすでにある。形 B は、この点だけ converter と違い、ほかの場合の種類と理由は converter と同じ。どちらも、表示に入力の値を含めない(理由の文は今の固定の文のまま)。
- 1 のあとに消えたファイルを `File::open` が `NotFound` で返したときは `fileMissing` にする(converter と同じ。今の core は `unreadable`「読み取りに失敗しました」。確かめてから開くまでのあいだに消えたときだけの違いで、テストでは作らない)。
- `SystemRuntime::run_id_env`(`deps.rs:225-229`)も `read_definition` を使うが、誤りはすべて `None` に丸めるので、見える変化は無い。
- converter は変えない [Q3]。

#### 2.3.3 仕様書

rust の 03 §2.2 と 02 の `definition/intake.rs` の行、language-agnostic の 03 §3.2 を新しい順の表にする(行は `inventory.md` 3e)。表の後ろに、次の注記を付ける(`docs/spec/rust/04-cli.md:73` の `--to` と同じ書き方):

> 読む権限のない(mode 000 の)ファイルは、大きさにかかわらず `unreadable` である(開けるかの確認が、大きさの確認より先)。v0.9.5 までは、パスのメタデータで大きさを先に見たため、1 MiB を超える読む権限のないファイルは `tooLarge` だった。

さらに「通常のファイルでない(名前付きパイプを含む)ものは、開かずに `unreadable`」を書く。

#### 2.3.4 版の更新(0.9.6)

段 3 の最後のコミットで、`Cargo.toml` × 4、`Cargo.lock`、`crates/cli/tests/binary_basic.rs:246`(T18)、仕様書の `0.9.5` の表記(16 ファイル)を 0.9.6 にする。段 2・3 で足す「v0.9.5 までは…」の注記は昔の版を指すので上げない(`inventory.md` 3f)。前回の版の更新 `ea09b46` と同じ範囲。

## 3. 実物で確かめて分かった点

- `JobExecution` は、どこでも直列化されない(ステートの書式に影響しない)。
- 今の偽物(`ScriptedRuntime`)は、`Exec::StartFailure` のあとでも実行中の合図があれば `interrupted` を立てる(本物は立てない)。この組み合わせを使うテストは無い。新しい偽物は本物に合わせる(T1)。
- `retry_flow.rs` の `起動の失敗は_停止する` は、起動の失敗に終了コード 127(リトライの一覧にある値)を持たせて「起動の失敗が一覧より優先」を確かめている。新しい型では起動の失敗は終了コードを持てず、この優先は型で決まる(T3)。
- `runtime/tests/nfr_source.rs:153-156` は、runtime の src で `Command::new(&line.program)` 以外の `Command::new` を拒む。段 2 の引数の名前は `line` にする。
- `nfr_source.rs` が見るのは runtime の `src` だけ(`CARGO_MANIFEST_DIR`)。core の単体テストの `#[cfg(unix)]`(`intake.rs:244` に前例)は対象外。
- 形 A は、書き手のいない FIFO で開く時点で止まる(converter も同じ)。今の core は止まらない。このため D1 で形 B に決めた。
- 段 3 で期待が変わる既存のテストは無い(読む権限のない大きなファイルを確かめるテストが無い)。期待の文字列が変わるのは、版の更新の T18 だけ。

## 4. テストの変更の一覧(ユーザーの承認が要る)

Q2: A に従い、変えるのは**組み立て、import、項目の読み方(同じ意味の新しい書き方への置き換え)、偽物の `run_job` の引数**だけで、**確かめる中身と期待する出力の文字列は変えない** [Q2]。そのうえで、型の上で作れなくなる組み合わせを組み立てていた箇所と、分割が `run_job` から外れることで道が変わる箇所は、**★**を付けて中身への影響を書く。T18 は版の更新による期待の文字列の変更 [Q4]。

### 4.1 読み方の置き換えの規則(段 1。T6〜T8、T12 で使う)

確かめる項目を増やさず、減らさない(型で決まるものを除く)。

| 今の読み方 | 新しい書き方 |
|------------|--------------|
| `e.started && e.start_failure`(と `(e.exit_code, e.terminated_by_signal) == (None, None)`、`!e.interrupted`) | `assert_eq!(e.result, ExecutionResult::StartFailed)`(終了コード・シグナル・中断が無いことは型で決まる) |
| `e.start_failure` だけ | 同上 |
| `!e.started && e.interrupted && !e.start_failure`、`e.signal_number == Some(n)`、`(None, None)` | `assert_eq!(e.result, ExecutionResult::NotStarted { signal: n })` |
| `!e.started && e.interrupted`、`e.signal_number.is_some()` | `assert!(matches!(e.result, ExecutionResult::NotStarted { .. }))` |
| `e.started && !e.start_failure`(と、確かめていた `exit_code`・`terminated_by_signal`・`interrupted`・`signal_number`) | `Exited { .. }` に、確かめていた項目だけを書いた `matches!`(全部を確かめていたなら `assert_eq!` で全体) |
| `!e.interrupted`、`e.signal_number == None` | `matches!(e.result, ExecutionResult::Exited { interrupted_by: None, .. })` |
| `e.interrupted`、`e.signal_number == Some(n)` | `matches!(e.result, ExecutionResult::Exited { interrupted_by: Some(n), .. })`(`n` は確かめていた値) |
| `e.started`(Windows の 2 件。起動を試みたか) | `!matches!(e.result, ExecutionResult::NotStarted { .. })` |
| `started: execution.started`(`flow_with_state.rs`) | `started: !matches!(execution.result, ExecutionResult::NotStarted { .. })` |

### 4.2 一覧

| ID | 段 | ファイル:行 | 箇所 | 変える内容 | 理由 | 中身への影響 |
|----|----|-------------|------|------------|------|--------------|
| T1 | 1 | `crates/cli/tests/common/mod.rs:28`(import)、`:348-377`(`ScriptedRuntime::run_job`)、`:739-748`(`MemRuntime::run_job`) | 3 | 「土台を作って項目を書き換える」を、「台本と合図から `ExecutionResult` を決めて `JobExecution::new`・`not_started` で作る」に。実行中の合図は `Exited` のときだけ `interrupted_by` に入れる(本物と同じ) | `JobExecution` の項目が変わる | 無し。`Exec::StartFailure` と実行中の合図を組み合わせるテストは無い(`inventory.md` 1b の P12 の注) |
| T2 | 1 | `crates/core/tests/flow_sequences.rs:21`(import)、`:46-56`(補助 `exited`)、`:291-300` | 3 | 補助は `result: ExecutionResult::Exited { exit_code: Some(code), .. None }`。`:291` は `NotStarted { signal: 15 }` | 同上 | 無し |
| T3 ★ | 1 | `crates/core/tests/retry_flow.rs:18`(import)、`:38-49`(補助 `exec`)、`:51-56`(補助 `exited`)、`:101-109`、`:112-119`、`:122-131`、`:134-143` | 7 | 補助 `exec` は `Exited { None, None, None }`。`起動の失敗は_停止する`(`:101-109`)は `StartFailed`、`シグナルでの終了は_停止する` は `Exited { None, Some(9), None }`、`中断された実行は_停止する` は `Exited { Some(75), None, Some(2) }`、`起動前に中断された実行は_停止する` は `NotStarted { signal: 15 }`。期待の `Stop` は変えない | 同上 | ★ `起動の失敗は_停止する` は、起動の失敗に終了コード 127(一覧にある値)を持たせられなくなる。「起動の失敗は一覧の 127 より優先」は型で決まる(同じ `Stop` を確かめる) |
| T4 | 1 | `crates/core/tests/flow_perf.rs:49-58`、`plan_perf.rs:91-100`、`graph_edges.rs:158-169`(と import) | 3 | `flow_perf`・`plan_perf` は `JobExecution::exited(&def.job[0], 0)`(同じ値)。`graph_edges` の補助は `result: Exited { Some(code), None, None }` | 同上 | 無し |
| T5 | 1 | `crates/runtime/tests/nfr_content.rs:138-147` | 1 | `result: Exited { exit_code: Some(3), .. None }` | 同上 | 無し |
| T6 | 1 | `crates/runtime/tests/runner_basic.rs:39-41`、`:73`、`:87-88`、`:98-99`、`:129-131`、`:140`、`:151`、`:172-174`、`:183-184`、`:215-216`、`:234`、`:254`(と import) | 12 | §4.1 の規則で読み方を置き換える | 同上 | 無し |
| T7 | 1 | `crates/runtime/tests/runner_signal.rs:67-71`、`:78-80`、`:89-92`、`:163`、`:182-185` | 5 | 同上 | 同上 | 無し |
| T8 | 1 | `crates/runtime/tests/runner_env.rs:29`、`nfr_robust.rs:179`、`:185-186`、`flow_with_state.rs:234` | 4 | 同上 | 同上 | 無し |
| T9 ★ | 1 | `crates/core/src/flow/classify.rs`(単体): `:120-136`(補助 `exec`・`code`)、`:185-189`、`:197-201`、`:211-219`、`:226-231`、`:237-243`、`:253-270` | 9 | 補助は `Exited`。`起動失敗は_127` は `StartFailed`、`シグナル終了は_128_足す番号` は `Exited { None, Some(9), None }`、`起動しなかった中断は分類しない` は `NotStarted { signal: 2 }`。期待の値は変えない | 同上 | ★ (1) `優先順位_中断は終了コードとシグナルより先`: 入力から「起動失敗」を外し `Exited { Some(0), Some(9), Some(15) }` に(143 は不変。起動失敗と中断は同時に作れない)。(2) `優先順位_起動失敗はシグナル終了より先`: `StartFailed` に(シグナルを持てない。127 は不変。優先は型で決まる。テストは残す)。(3) `想定外の組み合わせでもパニックしない`: 「起動せず中断でもない」の 2 行(`:253-258`)を消す(作れない)。`no_code` と `huge` は残る |
| T10 | 1 | `crates/core/src/flow/next_step.rs`(単体): `:226-236`(補助)、`:355-376`(4 つの場合) | 5 | `StartFailed`、`Exited { None, Some(9), None }`、`Exited { Some(0), None, Some(2) }`、`NotStarted { signal: 2 }` | 同上 | 無し(4 つとも作れる) |
| T11 ★ | 1 | `crates/core/src/flow/record.rs`(単体): `:223-233`(補助)、`:389-392` | 2 | 補助は `Exited { code, None, None }`。`起動しなかった中断は_status_を変えない` は `NotStarted { signal: 2 }` | 同上 | ★小: 今は番号が `None` の中断を組み立てている。新しい型は番号が必須なので 2 を入れる(確かめているのは状態が変わらないことで、番号は関わらない) |
| T12 | 1 | `crates/runtime/src/runner.rs`(単体): `:277`、`:279`、`:287-289`、`:299-300` | 4 | §4.1 の規則で読み方を置き換える(`:277`・`:279` は段 2 の T17 でさらに変わる) | 同上 | 無し |
| T13 | 2 | `crates/cli/tests/common/mod.rs:334-339`、`:734-739` | 2 | 偽物の `run_job` に `_line: &CommandLine` を足す(使わない) | `Runtime::run_job` の引数が変わる | 無し |
| T14 | 2 | `crates/runtime/tests/runner_basic.rs:26`、`:169`、`:208`、`runner_signal.rs:38`、`:105`、`:140`、`:175`、`runner_env.rs:28`、`flow_with_state.rs:223`、`nfr_robust.rs:184`(と import) | 10 | 呼ぶ前に `check_command(&job.command).expect("分けられる")` で `CommandLine` を作って渡す | `run_job` の引数が変わる | 無し(どれも分けられる `command`) |
| T15 ★ | 2 | `crates/runtime/tests/runner_basic.rs:84-90`(`配列の_command_の先頭が空なら起動失敗`) | 1 | `check_command` は誤りを返すので、`CommandLine` を直接作る(先頭の要素を `program`、無ければ空、残りを `args`)。期待(`StartFailed`)と名前は変えない | 同上 | ★ 確かめる道が「`run_job` の中の分割の誤り」から「`run_job` の空の `program` の防御」(§2.2.2)に変わる。結果は同じ |
| T16 ★ | 2 | `crates/runtime/tests/nfr_robust.rs:166-181`(`起動できないコマンドや範囲外の番号でパニックしない` の前半) | 1 | 6 つの場合のうち、分けられない 3 つ(空文字列、`'`、`''`)は「`check_command` が誤りを返す」を確かめ、残りの 3 つ(無い名前、ディレクトリ、`/`)は今どおり `run_job` が `StartFailed` を返すことを確かめる。後半(範囲外の番号)は T14 の引数だけ | 同上 | ★ 分けられない 3 つは、もう `run_job` に渡せない。「パニックせず誤りを値で返す」ことは、その場所(`check_command`)で確かめる |
| T17 ★ | 2 | `crates/runtime/src/runner.rs`(単体)`:274-280`(`分けられない_command_は起動失敗`)、`:283-301`(残り 2 つの呼び出し) | 3 | `:274-280` は名前を `空の_program_は起動失敗` にし、`CommandLine { program: "" }` と `"  "` を渡して `StartFailed` を確かめる。残り 2 つは T14 と同じく引数だけ | 同上 | ★ テスト名の変更。`run_job` は分けないので「分けられない」が意味を失う。代わりに、残る防御(§2.2.2)を確かめる |
| T18 ★ | 3 | `crates/cli/tests/binary_basic.rs:246` | 1 | `"conductor 0.9.5\n"` → `"conductor 0.9.6\n"` | 版の更新(Q4: B) | ★ 期待の文字列の変更(前回 `ea09b46` と同じ) |
| T19 ★ | 1 | `crates/core/tests/flow_sequences.rs` の `優先順位_中断と起動失敗とシグナル`(元の :255-275) | 1 | 起動失敗・シグナル・中断を同時に立てた値を、作れる 3 つの値(`Exited { Some(0), Some(9), Some(2) }`、`StartFailed`、`Exited { Some(0), Some(9), None }`)に置き換える。期待の 130 / 127 / 137 と assert のメッセージは変えない | 同上 | ★ T9 と同じ種類(作れなくなる組み合わせ)。実装の Step 2 で一覧の漏れとして見つかり、ユーザーが 2026-11-23 に追加を認めた |
| T20 | 1 | `flow_sequences.rs:115`(補助 `Sim`)、`runner_basic.rs:110`、`:150`、`:156`、`:214`、`:268`(Windows)、`runner_signal.rs:164`、`:180`、`runner_env.rs:30` | 9 | §4.1 の規則で読み方を置き換える(主に `exit_code`) | 同上 | 無し。実装の Step 2 で一覧の漏れとして見つかり、ユーザーが 2026-11-23 に追加を認めた |

- 合計: 段 1 で 58 か所(T1〜T12)、段 2 で 17 か所(T13〜T17)、段 3 で 1 か所(T18)。実装中に見つかった漏れとして、段 1 に T19(★、1 テスト)と T20(9 か所)を追加(ユーザーが 2026-11-23 に認めた)。
- **テストの件数は変わらない**(消すテストは無い。T9 の (3) は 1 つのテストの中の 2 行で、件数に影響しない)。
- 段 3 の順序の変更で期待が変わる既存のテストは無い(`inventory.md` 3d)。

### 4.3 テストを変えずに通るもの(参考)

- cli の流れのテスト(`flow_run.rs`、`flow_pipeline.rs`、`flow_retry.rs`、`flow_dry_run.rs`、`signals_*.rs` など)は、偽物(T1、T13)だけを直せば、1 行も変えずに通る。表示・終了コード・ステートの期待はそのまま。
- `plan/simulate.rs` は `JobExecution::exited` を使うので変えない。`--dry-run` のテストは変えずに通る。
- 段 3 の既存のテスト(`inventory.md` 3d)は変えずに通る。

### 4.4 ★の代案(採らない場合)

| ID | 代案 |
|----|------|
| T3、T9 (2) | テストを消す(件数 -1 ずつ)。優先は型で決まるので確かめる意味が薄い。ただし件数の記録が複雑になるので、残す方を勧める |
| T9 (3)、T11 | 無い(作れない値は組み立てられない) |
| T15、T16、T17 | `run_job` に、`&Command` を受けて中で分ける古い形の包みを残す(既存のテストは変えずに済むが、段 2 の目的「分け直さない」を弱め、テストしか使わない入口が残る。勧めない) |
| T18 | 無い(版を上げると決めた。Q4: B) |

## 5. 足すテスト(test-after。各段の実装のコミットのあと)

| ID | 段 | 場所 | テスト名(案) | 確かめること |
|----|----|------|----------------|--------------|
| N1 | 1 | `crates/core/src/flow/classify.rs`(単体) | `三つの場合の分類は表のとおり` | §2.1.3 の表の 6 行(`NotStarted`、`StartFailed`、`Exited` の中断・シグナル・終了コードなし・終了コード)の分類、返す終了コード、`conductor_error` |
| N2 | 2 | `crates/runtime/tests/runner_basic.rs` | `渡した分割済みのコマンドで起動し_command_を分け直さない` | `job.command` を分けられない文字列(`'`)にし、`line` に補助の実行ファイル(`exit 3`)を渡すと、`Exited { Some(3), None, None }` になる(今の形なら `StartFailed`) |
| N3 | 3 | `crates/core/src/definition/intake.rs`(単体。UNIX。**mode 000 にしたファイルを開けたとき(root など)は確かめない**。既存の `権限がないと_unreadable` は `if let Err` の形なので、そのまま写すと root では 1 MiB 超が `TooLarge` になって落ちる。先に `File::open` で開けるかを見て、開けたら確認を飛ばす) | `権限のない_1_mib_超のファイルは_unreadable` | mode 000 の 1 MiB + 1 バイトのファイルが `Unreadable`(理由に「権限」)。今の形なら `TooLarge` で落ちる |
| N4 | 3 | 同上(単体。全 OS) | `上限超過は_utf8_でないより先` | 1 MiB 超で UTF-8 でないファイルは `TooLarge`(順序の不変。converter の `大きすぎるは_utf8_でないより優先する` と同じ) |
| N5 | 3 | 同上(単体。UNIX。D1 の決定で足す) | `名前付きパイプは開かずに_unreadable` | `mkfifo` で作った FIFO が、待たずに `Unreadable`(理由に「通常のファイル」)。止まったときに落ちるよう、別のスレッドで呼んで 5 秒で打ち切る |

- 合計の見込み: **+5 件**。macOS での件数: 1,137 → 段 1 の後 1,138 → 段 2 の後 1,139 → 段 3 の後 **1,142**。無視は 1 のまま。
- ファイルが無い・通常のファイルでない・読めるが上限超過・UTF-8 でない場合の種類が変わらないことは、既存のテスト(`inventory.md` 3d)がそのまま通ることで確かめる。

## 6. コミットの並び(小さな単位。各コミットでビルドとテストが通る)

段 1(4 コミット):
1. core・runtime・cli: `JobExecution::not_started` と `start_failed` を足し、runtime の `run_job` と cli の `retry.rs` が使う(まだ今の項目の形。動きは同じ。テストは変えない)
2. core・runtime・cli: `JobExecution` を `ExecutionResult` に切り替える(`classify`、`retry_decision`、`apply_outcome`、`run_job`、`step.rs`、`retry.rs` と、T1〜T12)。項目を変えると、すべての組み立てが同時に直らないとビルドが通らないため、1 つのコミットにする。差分はファイルごとに読める並び(製品のコード → 単体テスト → 統合テスト → 偽物)で見せる
3. core のテスト: N1
4. 仕様書(rust): 段 1(02、05、07、11、12)

段 2(3 コミット):
5. runtime・cli: `run_job` が `CommandLine` を受け取り、分け直さない。空の `program` の防御。`Runtime` トレイト、`run_job_with_retry`、`run_step`(T13〜T17)
6. runtime のテスト: N2
7. 仕様書: 段 2(rust の 02、05、07、12、11 と、D2 の決定による language-agnostic の 05、07 の最小限)

段 3(4 コミット):
8. core: `read_definition` を新しい順に(形 B)
9. core のテスト: N3、N4、N5
10. 仕様書: 段 3(rust の 03、02、language-agnostic の 03。「v0.9.5 までは…」の注記)
11. 版を 0.9.6 にする(`Cargo.toml` × 4、`Cargo.lock`、T18、仕様書の表記)

合計 11 コミット。各段の終わり(4、7、11 のあと)にテストの件数を記録する。

## 7. 確かめ方

- 段の前と後で、`cargo test --locked --workspace` の件数を記録する(基準: **1,137 通過、1 無視、0 失敗**。`inventory.md` §0)。合格は、失敗 0 で、件数が「基準 + 足したテスト」(§5)に一致すること [Q1]。
- テストのファイルの変更は §4 の一覧と §5 の新しいテストだけ。段ごとに `git diff --stat -- 'crates/*/tests'` と、`src` の中の単体テスト(`#[cfg(test)] mod tests`)の差分で確かめる。期待する出力の文字列の差分が T18 の 1 行だけであることを、`git diff` で確かめる。
- 段 1: `grep -rn "start_failure\|\.started\b\|\.interrupted\b\|signal_number:" crates/*/src crates/*/tests` で、`JobExecution` の古い項目が残っていないこと(`InterruptSignal::signal_number()`、`FakeSignals.started`、`flow_with_state.rs` の自前の `Step.started` は別のもの)。`classify` の分岐が §2.1.3 の表の 6 つであること。
- 段 2: `grep -n "command_line\|split(\|check_command" crates/runtime/src/runner.rs` が 0 件(単体テストを除く)。`Command::new(&line.program)` が残ること(`nfr_source.rs` が通る)。
- 段 3: N3 が直す前の木で落ち、直した後で通ることを記録する(test-after の順で、直すコミット → テストのコミット)。N5 は今の core でも通る(形 B が今の「FIFO で止まらない」を保つことの確かめ)ので、直す前と後の両方で通ることを記録する。既存の取り込みのテスト(`inventory.md` 3d)が変えずに通ること。Windows の `ディレクトリは_unreadable` は CI で確かめる。
- ステートファイル: 既存のステートのテストが変えずに通ること(§2.1.4)。
- 合格の条件: `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、全テスト、行カバレッジ 80% 以上(`cargo llvm-cov --fail-under-lines 80`)、MSRV 1.100 でのテスト [memory:M2] [memory:M3] [memory:M4]。新しい依存を足さない(`Cargo.lock` の差分は版の 4 行だけ)[memory:M5]。`unsafe` と `cfg` を足さない(runtime と cli の `nfr_source.rs`)。converter の差分は 0(版の行を除く)。
- README と `docs-check` に触れる変更は無い。

## 8. 範囲の外

- converter の読み込み(FIFO で止まる点も含めて、変えない)[Q3]。
- `exit_code` と `terminated_by_signal` を 1 つの列挙型(「終了コード」か「シグナル」)にすること。UNIX では片方だけに値があるが、OS の性質を型で約束することになり、`classify` の「終了コードが無い」の分岐の扱いも変わる。段 1 の 3 つの真偽値の整理とは別の話。
- `CommandLine` のフィールドを非公開にすること(D4 の代案)。
- `EdgeList` の削除、凍結した 5 つの仕様書(`docs/spec/{go,python,typescript,csharp,powershell}`)[desc]。
- タグとリリース [Q4]。
- `.env` の読み込み(runtime の `envfile.rs`)の順序。

## 9. 決めてほしい点(と決定)

| ID | 問い | 推奨 | 代案 | 決定 |
|----|------|------|------|------|
| D1 | 段 3 の順序は、形 B(種別だけ先にパスで見て、開いてから大きさ)か、形 A(converter とまったく同じ書き方。先に開く)か | **形 B**。告げた変更(読む権限のない大きなファイル → `unreadable`)は同じで、ほかの場合の種類と理由も converter と同じ。違いは、書き手のいない FIFO で止まらないこと(今の core と同じ)だけ。形 A は、今の core に無い「止まる」動きを、告げずに足してしまう | 形 A。2 つの読み込みの書き方が完全に同じになる(読みやすさ)。FIFO で止まる点は仕様書に書く | 決定: 形 B。FIFO のテスト N5 を足し、件数の見込みは 1,142。Q3 の「先に開く」を字のとおりには守らない理由は §2.3.1 に記録(ユーザー、2026-11-23) |
| D2 | language-agnostic の仕様書を、段 1・2 でも直すか | 段 1 は**直さない**(真偽値の観測モデルは、言語に依らない説明として正しいまま)。段 2 は**最小限を直す**(05 §1.2 の手順 1、07 §3.6 の防御、§4.1 の入力と手順 2、§4.3 の `start_failure` の中身。参照の実装が分け直さなくなるので、その記述だけ) | 段 1・2 とも直さない(段 3 だけ直す) | 決定: language-agnostic は段 2(最小限)と段 3 だけ直す。段 1 は直さない(ユーザー、2026-11-23) |
| D3 | §4 の ★(T3、T9、T11、T15、T16、T17、T18)の、中身に触れるテストの変更を認めるか | 認める(どれも、型で作れなくなった組み合わせか、防御の場所の移動か、版の更新) | §4.4 | 決定: 認める(ユーザーが設計の承認で認めた、2026-11-23) |
| D4 | `run_job` に残す防御 | `line.program` が空・空白だけなら `StartFailed`(1 行。今の `Argv` の検査と同じ条件) | (a) 防御を置かず `spawn` の誤りに任せる(Windows での挙動を確かめていない)。(b) `CommandLine` のフィールドを非公開にし、`check_command` だけが作れるようにする(型で完全に防げるが、`dry_run.rs`・`step.rs`・テストの書き方が変わる) | 決定: `run_job` に「`program` が空・空白だけなら `StartFailed`」の 1 行を残す(ユーザー、2026-11-23) |
| D5 | 新しい型の名前 | `ExecutionResult`(項目名 `result`)、場合は `NotStarted { signal }`、`StartFailed`、`Exited { exit_code, terminated_by_signal, interrupted_by }` | `Launch`(項目名 `launch`)など | 決定: `ExecutionResult`(`NotStarted`、`StartFailed`、`Exited`)と項目名 `result`(ユーザー、2026-11-23) |

## 10. 成功の指標との対応(traceability.json)

| ID | 成功の指標(`intent-statement.md`) | 対応する規則 |
|----|------------------------------------|--------------|
| SM1 | 段 1(`JobExecution` の型。書式・出力・終了コードは不変) | BR1.1〜BR1.5 |
| SM2 | 段 2(`run_job` が分けたコマンドを受け取り、分け直さない。防御の行き先。トレイトと仕様書の章) | BR2.1〜BR2.4 |
| SM3 | 段 3(読み込みの順序。読む権限のない大きなファイルは `unreadable`。ほかは不変。converter は変えない。仕様書) | BR3.1〜BR3.5 |
| SM4 | 版(最後のコミットで 0.9.6。タグとリリースはしない) | BR3.6 |
| SM5 | テストの扱いと合格の条件(変えるテストの一覧と承認、件数の記録、fmt・clippy・カバレッジ・MSRV・依存) | BR4.1〜BR4.4 |
