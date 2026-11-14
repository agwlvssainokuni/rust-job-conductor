# Security Design(u2-runtime)

上流: security-requirements(NFR4.1〜NFR4.9、既存の成果物との差分)、tech-stack-decisions(T1〜T3、NFR1.1、NFR2.1、NFR3.1、NFR5.1)、functional-spec(W1〜W6)、rules(BR1.1〜BR2.10)、contract-summary(C3、E3)。u2-runtime は library であり、認証・認可・暗号化・ネットワークはない。入力は、`command` の文字列、ステートファイル、OS が返すプロセスの結果である。コードは、短い疑似コード(15行以下)に限る。

**この設計は、functional-design と nfr-requirements の関連する記述より優先する**(下の「既存の成果物との差分」を参照)。

## 設計の要点

### 1. ジョブの起動と待ち(JobRunner。NFR4.1、NFR4.6、NFR4.7、NFR4.8、NFR4.9)

1. **command の分け方(Split):** BR1.1 の規則を、文字を1つずつ読む小さな状態機械(通常、一重引用符の中、二重引用符の中、二重引用符の中でバックスラッシュの直後)で実装する。再帰は使わない。結果は、コマンド名と引数の並び、または CommandError。入力の値は、エラーに含めない。`check_command` は、同じ関数を、起動せずに呼ぶ。(NFR4.2)
2. **起動の前の合図の確認:** 合図が立っていれば、起動せず、`started = false`、`interrupted = true` の JobExecution を返す。(BR1.6)
3. **起動(Spawn):** `std::process::Command` で、コマンド名と引数を、1つずつ渡す(シェルを介さない)。UNIX 系では、`process_group(0)` で、新しいプロセスグループにする。作業ディレクトリ、環境変数、標準入出力は、引き継ぐ。起動に失敗したら、`startFailure`。(NFR4.1、NFR4.6)
4. **待ちのくり返し(WaitLoop。Q1: A):** 1つのスレッドで、次をくり返す。
   ```
   loop {
     if let Some(status) = child.try_wait()? { observe(status); break; }
     if signal_raised && !forwarded { platform::forward(child.id(), signal_number); forwarded = true; }
     sleep(backoff);   // 1ms, 2ms, 4ms, 8ms, 10ms, 10ms, ...
   }
   ```
   **ジョブを回収する(`try_wait` が終了を返す)のは、このくり返しの中だけ**であり、シグナルを送るのは、直前の `try_wait` が「まだ終わっていない」を返した回に限る。終了したが、まだ回収していないプロセスは、pid が使い回されないため、送信と終了の競合があっても、無関係のプロセスには届かない。(NFR4.8 の条件 3)
5. **終了の観測:** 終了コード(`exitCode`)、UNIX 系のシグナル終了(`terminatedBySignal`)を、JobExecution に入れる。観測の直後に、合図が立っていれば、`interrupted = true`、`signalNumber` を足す。(BR1.7)
6. **シグナルの送信(`platform` の UNIX 系):** 唯一の `unsafe`。次の条件を、関数の中で確かめてから、`libc::kill` を1回だけ呼ぶ: pid が `i32` に収まり、0 より大きい。シグナル番号が 1〜64 の範囲。失敗(特に、プロセスが存在しない)は、無視する(エラーにしない)。プロセスグループ全体(負の pid)には、送らない。(NFR4.8)Windows の `platform` は、何もしない(Ctrl+C は、OS が配る)。

### 2. ステートの読み込み(StateReader。NFR4.3、NFR4.5、NFR4.7)

1. ファイルを開く。開けなければ、存在しないなら「空」、それ以外は `unreadable`。**開いたハンドルの**メタデータで大きさを見て、上限(1 MiB)を超えれば `tooLarge`。読み取りは「上限 + 1 バイト」までに制限する。
2. 長さ 0 のファイルは、**空のステート**として返す(下の差分)。UTF-8 でなければ `corrupt`(不正な UTF-8)。
3. TOML として読む(再帰の制限は、ライブラリに頼る。u1-core・u4-converter と同じ確認を、Code Generation で行う)。読めなければ `corrupt`。
4. `format_version` を、**ほかの検査より先に**確認する。無ければ `corrupt`、1 でなければ `versionMismatch`。
5. 型付きの構造へ、自分で走査して写し、BR2.3 の一覧(未知のキー、型の誤り、必須のキーの欠落、status の不正、skip_reason と exit_code の矛盾、id の重複、job_order との不一致、resume_job_id が job_order にない、completed と resume_job_id の両立)を、すべて検査する。違反は `corrupt`。
6. `completed` が true のステートは、**空として返す**(BR2.3)。
7. エラーの表示は、kind ごとの固定の文と、ステートファイルのパスだけ。ライブラリのエラー文、入力の値は含めない。(NFR4.5)

### 3. ステートの書き込み(StateWriter。NFR4.4)

`write_state(path, state, durability)`。`durability` は、`Durable`(既定。結果の書き込み)と `Fast`(起動前の初期ステート)の2つ。

1. 同じディレクトリに、一時ファイル `.<ステートファイル名>.tmp.<プロセス番号>.<連番>` を、**新規作成**(既存ならエラー。他のファイルを上書きしない)で作る。
2. 全内容(TOML、版つき)を書く。`Durable` のときは、ファイルの内容を同期する(`sync_all`)。`Fast` のときは、同期しない。
3. ステートファイルへ、置き換えで移す(`rename`)。UNIX 系で、`Durable` のときは、ディレクトリを同期する(失敗しても、エラーにしない)。
4. どの段階で失敗しても、自分が作った一時ファイルを消し(消せなければ、そのまま)、`writeFailed` を返す。ステートファイルは、変わらない。

### 4. 照合と記録(Consistency、Recorder。NFR4.7)

- `check_consistency` は、パイプライン名、ジョブIDの集合、並びを、この順に比べる(BR2.5)。純粋な関数。
- 記録(`initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`)は、純粋な関数で、入出力を持たない。`end_reached` は、一度 true になったら false に戻さない。

### 5. パニックしない、桁あふれしない(NFR4.7)

- 外部の入力(command、ステート、OS の結果)に対する処理は、すべて戻り値のエラーで失敗を返す。添字は、長さを確かめてから使う。
- `128 + signalNumber`、終了コードの変換は、`i32` の範囲に飽和させる。
- 補助の関数 `unsafe` は、`platform` の UNIX 系の1か所だけ。クレートに `#![deny(unsafe_code)]` を置き、そのモジュールだけに `#[allow(unsafe_code)]` を付ける。

## 既存の成果物との差分(この設計を優先する)

| 対象 | 変更の内容 |
|------|------------|
| rules の BR2.3、security-requirements の NFR4.3 | 長さ 0 のファイルは、空のステートとして扱う(NFR Requirements の差分表と同じ)。StateReader の 2 |
| rules の BR2.4、functional-spec の W3、contract C3 | `write_state` に `durability`(`Durable` / `Fast`)を足す。一時ファイルは、新規作成で作る。ディレクトリの同期は、`Durable` のときだけ |
| rules の BR1.7、functional-spec の W1 | 待ちは、1つのスレッドの、短い間隔(1ms から 10ms)の `try_wait` のくり返しで行う。シグナルの送信は、回収の前(直前の `try_wait` が「まだ」を返した回)にだけ、宛先は子プロセスの pid だけ、1回だけ |
| rules の BR2.9 | 起動前の初期ステートの書き込みは、`Fast`。結果の書き込みは、`Durable` |

## 他の NFR の設計(NFR1.1、NFR2.1、NFR3.1、NFR5.1)

- **NFR1.1:** `check_command`(全ジョブ)、ステートの読み込み、照合、初期ステートの書き込み(`Fast`)が、起動経路に入る。いずれも、入出力は、小さなファイル1つの読み書きだけ。静的な初期化(起動時に動く処理)を持たない。待ちのくり返しの最初の間隔は 1ms で、短いジョブの終了の観測の遅れは、最大でも数 ms。ジョブ10個で、p95 20ms 以内を、仮の目標として、Code Generation で実測する(超えても、CI を失敗にしない)。
- **NFR2.1:** 依存は、Rust だけ(`libc` は宣言だけ)。`cargo tree` に `-sys` がない。
- **NFR3.1:** OS 固有のコードは、`platform` モジュールの中だけ。ほかの部品は、OS を知らない。
- **NFR5.1:** テスト専用の補助の実行ファイル(Q2: A。この Unit のクレートの `bin`。引数・環境変数・作業ディレクトリ・終了コード・シグナルの受信を出力する)で、実際のプロセスの起動を、3つの OS で試す。書き込みの失敗は、置き換え先をディレクトリにする方法で起こす(権限に頼らない)。行カバレッジ 80% 以上(Linux)。`cfg(windows)` のコードは、Windows の CI のテストの成功で確かめる。

## 設計の判断(代替案との比較)

### D1 待ちは、1つのスレッドの、短い間隔の確認のくり返し [Q1: A]

- **Context:** ジョブの終了を待ちながら、合図を見る。シグナルを、終了後の(使い回された)pid に送る事故を避けたい。
- **Decision:** 1つのスレッドが、`try_wait` で、ジョブの終了を確かめるくり返し(1ms から 10ms まで間隔を広げる)にし、同じくり返しの中で合図を見る。回収は、このくり返しだけが行う。
- **Consequences:** 良い点は、競合の隙間がない(終了後・未回収のプロセスは、pid が使い回されない)。ロックが要らない。悪い点は、ジョブの終了の観測が、最大 10ms 遅れる(長いジョブでは、CPU を、ほとんど使わない)。
- **Alternatives Rejected:** (B) 専用のスレッドが、ブロックして待つ案: 終了の観測は即時だが、回収と送信のあいだに、ロックで守りきれない隙間が残り、pid の使い回しの事故の危険があるため退けた。(C) `waitid` で、回収せずに待つ案: 隙間は無くせるが、`unsafe` が増え、Windows で別の実装が要るため退けた。

### D2 テスト専用の補助の実行ファイルは、この Unit のクレートの bin [Q2: A]

- **Context:** 実際のプロセスの起動、引数の受け渡し、シグナルの受信を、3つの OS で試す。
- **Decision:** クレートの中に、テスト専用の `bin` を置く。`cargo test` が自動でビルドし、統合テストが、そのパスを使う。
- **Consequences:** 良い点は、ワークスペースの構成が増えず、ビルドが単純。悪い点は、配布のビルドに、補助の実行ファイルが含まれうる(u3-cli の配布は、自分の実行ファイルだけを使う)。
- **Alternatives Rejected:** (B) 別のクレート(`crates/test-helper`): 配布のビルドに混ざらないが、クレートが増え、テストから、パスを探す手間が増えるため退けた。

### D3 シグナルは、子プロセスの pid だけに、回収の前に、1回だけ送る

- **Context:** 送る相手を誤ると、無関係のプロセスに届く。
- **Decision:** 宛先は、`Child::id()`(pid > 0、`i32` に収まる)だけ。プロセスグループ全体には送らない。回収の前の1回だけ。
- **Consequences:** 良い点は、事故の範囲が最小。悪い点は、ジョブが起動した子孫のプロセスには、届かない(ジョブ自身の責務。BR1.7)。
- **Alternatives Rejected:** プロセスグループ全体に送る案: 子孫にも届くが、負の pid の誤りが、広い範囲に影響し、ジョブの責務の範囲を超えるため退けた。

### D4 起動前の初期ステートの書き込みは、同期を省く(`Fast`)

- **Context:** 起動の予算(100ms)に、初期ステートの書き込みが含まれる。同期は、環境によって、数十ミリ秒かかりうる。
- **Decision:** 初期ステートは `Fast`、結果は `Durable`。電源断で、長さ 0 のファイルが残りうるが、長さ 0 は、空のステートとして読む(StateReader の 2)ため、再作成できる。
- **Consequences:** 良い点は、起動の予算を守れる。悪い点は、`write_state` の引数と、読み込みの規則が増える。
- **Alternatives Rejected:** 常に同期する案: 単純だが、起動の予算を超えうる。初期ステートを書かない案: 書けないディレクトリを、ジョブの実行の前に検知できず(BR2.9)、ジョブのあとで失敗するため退けた。

## Assumptions & Open Questions

- 待ちのくり返しの間隔(1ms から 10ms)は、仮の値である。Code Generation で、短いジョブの観測の遅れと、CPU の使用を、実測して見直す。 [assumption]
- 長さ 0 のファイルを空として扱う規則(D4)は、電源断以外で、長さ 0 のファイルが、誤って作られた場合も、空として扱う(エラーにならない)。この寛容さを許す。 [assumption]
- `toml` の再帰の制限と、`libc` の最小のバージョンでのビルドは、Code Generation の最初に確かめる(u1-core・u4-converter と同じ手順)。確かめられなければ、止めて人に確認する。 [assumption]
- 認証・認可・暗号化は、この Unit に該当しない。 [assumption]
