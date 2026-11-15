# Logical Components(u3-cli)

上流: performance-design、security-design、scalability-design、reliability-design、observability-design、functional-spec、unit-of-work、components、contract-summary。この Unit は、`conductor` の実行ファイルそのもの(`service`)で、サーバーや常駐の仕組み、データベース、ネットワークの通信を持たない。ここでは、NFR のパターンが、この Unit のどこに当てはまるかを、論理的な部品の見取り図として示す。

## クレートの作り(Q1: A)

- パッケージ `conductor-cli`(`crates/u3-cli/`)。`lib.rs` に、全部の処理(引数の解釈、流れ、表示)を置き、`main.rs`(実行ファイル名 `conductor`)は、ライブラリの入口関数を呼んで、返った終了コードで終わるだけの、薄い入口にする。パニックのフックの設定も、`main.rs` の最初で行う(フックの本体は、ライブラリにあり、単体テストできる)。

## 論理的な部品

| 部品 | 責務 | 関わる NFR |
|------|------|------------|
| 入口(main) | パニックのフックを設定し、ライブラリの `run_cli(args, env, io)` を呼び、返った終了コードで、1回だけ終了する | NFR4.7、NFR2.2 |
| 引数の解釈(ArgParser) | `lexopt` で字句に分け、コマンドごとの文法の表で検査する。`UsageError` を返す | NFR4.1 |
| 流れ(Orchestrator) | W2(run)、W3(pipeline)、W4〜W6 の手順。u1-core・u2-runtime・u4-converter の関数を、決まった順に呼ぶ。**依存は、小さな trait(`Runtime`、`Sink`、`Env`、`Signals`)で受け取る**(Q2: A) | NFR2.2〜NFR2.7 |
| シグナルの監視(SignalWatcher) | run・pipeline の起動の経路でだけ、専用のスレッドを起動し、受け取った最初の番号を、合図に書く(`cfg` で、UNIX 系は `signal-hook`、Windows は `ctrlc`) | NFR4.3 |
| 終了コードの決定(ExitMapper) | 純粋な関数。`reportedExitCode`、conductor 自身のエラーの種類から、0〜255 の終了コードを決める(BR5.1、BR5.2) | NFR2.2 |
| 表示(Reporter) | DisplayEvent から、文言(英語・日本語)を組み立てる。Locale・Color は遅延。Escape を通す | NFR6.1〜NFR6.4、NFR4.2、NFR1.4 |
| watch の表示(WatchView) | ステートから、3列の表を組み立てる(純粋) | NFR6.1 |
| convert の呼び出し(ConvertCmd) | u4-converter の `convert` を呼び、結果を、全部作ってから、標準出力に1回で書く | NFR4.4 |
| パニックのフック(PanicHook) | 固定の英語の文を出し、70 で終了(終了の呼び出しは、差し替えられる) | NFR4.7 |

## trait(差し替え口。Q2: A)

| trait | 本番の実装 | テストの実装 |
|-------|------------|--------------|
| `Runtime`(`check_command`、`run_job`、`read_state`、`write_state`、`reset_state`、`load_definition`) | u2-runtime・u1-core の実際の関数 | 失敗を起こす偽(例: ジョブのあとの書き込みの失敗、初期化の失敗、読み込みの失敗) |
| `Sink`(標準出力、標準エラーへの書き込み) | 実際の標準出力・標準エラー | 内容を集める偽。書き込みの失敗を起こす偽(閉じたパイプ) |
| `Env`(環境変数、端末かどうか、`NO_COLOR`) | 実際の環境 | 固定の値 |
| `Signals`(`start(&InterruptSignal) -> Result<(), SignalError>`) | 実際のシグナルの監視(`signal-hook` / `ctrlc`。プロセスで1回だけ登録できる。2回目の呼び出しは、すでに登録済みとして成功とする) | 何もしない偽、または、指定したタイミングで、合図に `raise` する偽(起動前・ジョブのあいだ・検査の途中の中断を起こす)。登録に失敗する偽(70 の確認) |

- 純粋な部品(ArgParser、ExitMapper、Reporter の文言の組み立て、WatchView、Escape)は、trait なしで、直接テストする。

## 失敗の範囲(failure domain)と影響の及ぶ範囲(blast radius)

| 失敗 | 影響の範囲 | 備考 |
|------|------------|------|
| 引数・定義・ステートの検査の失敗 | 1回の実行(ジョブは1つも起動されない) | 終了コード 64・65・66・68(reliability-design の表) |
| ジョブの失敗 | 1回の実行(その後のジョブは起動されない。再開位置をステートに残す) | ジョブの終了コード |
| 内部の誤り(パニック) | 1回の実行。実行中のジョブが、孤児として残りうる | 終了コード 70 |
| ステートの書き込み・初期化の失敗 | 1回の記録(ジョブは、再実行されうる) | 終了コード 65 |

- この Unit は、状態を、ステートファイル(u2-runtime)にしか持たず、常駐もしないため、他の実行や他のジョブに影響を及ぼさない(排他制御はしない)。

## 共有する資源

- ステートファイル(u2-runtime が管理。同じ定義に対して1つ)。排他制御はせず、原子的な置き換えで壊れた内容を残さない(後に書いた方が勝つ)。標準出力・標準エラー(スケジューラが集める)。

## 他の Unit との関係

- u1-core(`load_definition`、`decide_before_run`、`classify_and_decide`、`resume_point_on_interrupt`)、u2-runtime(`check_command`、`run_job`、`read_state`、`write_state`、`reset_state`、`check_consistency`、`initial_state`、`apply_*`)、u4-converter(`convert`)を、この Unit が、呼び出す(ADR-002。唯一の調整役)。

## Assumptions & Open Questions

- 部品の名前(ArgParser など)は、設計上の呼び名であり、Rust のモジュールやファイルの分け方は、Code Generation で決める。 [assumption]
- `Runtime` trait は、u2-runtime・u1-core の関数の薄い包みで、関数の数が多い(約 10)。分け方(複数の小さな trait にするか)は、Code Generation で決める。 [assumption]
