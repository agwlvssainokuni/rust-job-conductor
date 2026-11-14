# Logical Components(u2-runtime)

上流: security-design(この Unit の security-design.md)、functional-spec、unit-of-work、components、contract-summary。この Unit は library であり、サーバーや常駐の仕組み、データベース、ネットワークの通信を持たない。ここでは、NFR のパターンが、この Unit のどこに当てはまるかを、論理的な部品の見取り図として示す。

## 論理的な部品

| 部品 | 責務 | 入出力 | 関わる NFR |
|------|------|--------|------------|
| 分け方(Split) | command の文字列を、コマンド名と引数に分ける(BR1.1)。起動せずに検査する `check_command` も、これを使う | 文字列 → CommandLine(または CommandError) | NFR4.2 |
| 起動(Spawn) | コマンド名と引数を、シェルを介さずに起動する。UNIX 系では、新しいプロセスグループにする。作業ディレクトリ・環境変数・標準入出力を引き継ぐ | CommandLine → 子プロセス(または startFailure) | NFR4.1、NFR4.6、NFR4.9 |
| 待ちのくり返し(WaitLoop) | 1つのスレッドで、子プロセスの終了を短い間隔で確かめ、同じくり返しで合図を見る。回収は、ここだけ | 子プロセス、InterruptSignal → JobExecution | NFR4.7、NFR4.8 |
| 送信(platform::signal) | UNIX 系だけ。pid と番号を確かめて、`kill` を1回呼ぶ(唯一の `unsafe`)。Windows は、何もしない | pid、シグナル番号 → なし | NFR4.8、NFR3.1 |
| ステートの読み込み(StateReader) | 開いたハンドルの大きさ確認、上限 + 1 バイトの読み取り、版の確認(先)、型付きの構造への走査、内容の矛盾の検査。長さ 0 と completed を空として返す | ファイル → PipelineState / 空 / StateError | NFR4.3、NFR4.5、NFR4.7 |
| ステートの書き込み(StateWriter) | 一時ファイルの新規作成、全内容の書き込み、`Durable` のときの同期、置き換え、失敗時の後始末 | PipelineState、durability → 成功 / StateError | NFR4.4 |
| 照合(Consistency) | パイプライン名、ジョブIDの集合、並びを比べる(純粋) | PipelineState、定義 → ConsistencyResult | NFR4.3(BR2.5) |
| 記録(Recorder) | `initial_state`、`apply_outcome`、`apply_skip`、`apply_interrupt`(純粋)。`end_reached` は、一度 true になったら戻さない | ステート、結果 → ステート | NFR4.7(BR2.8) |
| 初期化(Reset) | ステートファイルを、読まずに削除する。存在しなければ成功 | パス → 成功 / StateError | NFR4.4(BR2.6) |
| 表示用の組み立て(Describe) | kind ごとの固定の文と、ステートファイルのパスから、エラーの内容を組み立てる | 誤り → StateError / CommandError | NFR4.5 |
| テスト専用の補助の実行ファイル(bin) | 引数・環境変数・作業ディレクトリ・終了コード・シグナルの受信を出力する。配布には使わない | — | NFR5.1(D2) |

- ファイルシステムに触れるのは、StateReader、StateWriter、Reset だけ。プロセスに触れるのは、Spawn と WaitLoop(と platform::signal)だけ。Split、Consistency、Recorder、Describe は、入出力のない純粋な処理で、単体テストで、ファイルとプロセスなしに確かめられる。(NFR5.1)
- OS ごとの違いは、`platform` モジュール(シグナルの送信、プロセスグループの指定)の中に閉じ込める。(NFR3.1)

## 失敗の範囲(failure domain)と影響の及ぶ範囲(blast radius)

| 失敗 | 影響の範囲 | 備考 |
|------|------------|------|
| ジョブの起動の失敗 | 1回の呼び出し(JobExecution の startFailure) | 常に値で返し、パニックしない(NFR4.7) |
| ステートの読み込みの失敗 | 1回の実行(ジョブは起動されない) | StateError で返し、reset で回復できる |
| ステートの書き込みの失敗 | 1回の記録(ステートは直前の完全な内容のまま) | StateError(writeFailed)。at-least-once により、ジョブが再実行されうる(BR2.9) |
| シグナルの送信の失敗 | なし(無視する) | プロセスが存在しない場合など |

- この Unit は、状態をファイル1つ(ステートファイル)にしか持たず、常駐もしないため、他の実行や他のジョブに影響を及ぼさない(排他制御はしない。BR2.10)。

## 共有する資源

- ステートファイル1つ(同じ定義に対して)。排他制御はせず、原子的な置き換えで、壊れた内容を残さない(後に書いた方が勝つ)。他の共有の資源は、持たない。

## 他の Unit との関係

- u1-core: 共有する型(JobExecution、JobState、PipelineState、JobOutcome、NextStep、ResumePoint)を使う(依存)。
- u3-cli(Cli): この Unit の関数を、ジョブごとの流れ(`decide_before_run` → `run_job` → `classify_and_decide` → 記録 → `write_state`)で呼ぶ。シグナルの受け取りと、合図の用意は、u3-cli が行う。 ADR-002

## Assumptions & Open Questions

- 論理的な部品の名前(Split など)は、設計上の呼び名であり、Rust のモジュールやファイルの分け方は、Code Generation で決める。 [assumption]
