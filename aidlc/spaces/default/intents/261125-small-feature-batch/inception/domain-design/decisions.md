# 設計の決定(ADR): v0.10.0 の小さな機能のまとまり

出典: 要件(`requirements.md`)、コンポーネントの一覧(`components.md`)、ドメイン設計の質問(`domain-design-questions.md`)、チームの実践(`team-practices.md`)。

## ADR-001: 新しい部品は既存のクレートの中のモジュールにする

- **Context**: 7 つの機能は、どれも既存の core・runtime・cli の流れの中で動く。クレートを増やすと、ビルド・CI・依存の検査の対象が増える。
- **Decision**: 新しい部品(CoreTime、RuntimeClock、RuntimeLog)は、既存のクレートの中の新しいモジュールとして置く。
- **Consequences**: 依存の向き(cli → runtime → core)と、`unsafe`・`cfg` の置き場所の規則がそのまま使える。各クレートが少し大きくなる。
- **Alternatives Rejected**: 時刻やログのための新しいクレートを作る案。部品が小さく、クレートを分けても得るものが無いため。

## ADR-002: 時刻の整形は core、現在の時刻は runtime から差し替え口を通して取る

- **Context**: ステートに UTC の RFC 3339 で時刻を書く(FR6.2)。新しい依存は使えない(NFR4)。core は入出力を持たない。流れのテストは偽の `Runtime` を注入して書く(team-practices.md)。
- **Decision**: 整形・読み取り・所要時間の計算は core の CoreTime(純粋)に置く。現在の時刻は runtime の RuntimeClock(std の `SystemTime`)が取り、cli の `Runtime` に足すメソッドを通して流れに渡す(Q1 の A)。
- **Consequences**: 日付の計算を入出力なしでテストできる。流れのテストは偽の時計で時刻を決められる。`Runtime` のメソッドが 1 つ増え、偽の実装もそれに合わせる。`unsafe` も `cfg` も要らない。
- **Alternatives Rejected**: すべてを runtime に置く案(Q1 の B)。純粋な計算が入出力の部品に入り、テストに本物の時計が混ざるため。

## ADR-003: リトライの既定は、定義の検証の中で各ジョブに当てはめる

- **Context**: `[pipeline.retry]` を既定にする(FR1.2)。ジョブの `retry` があれば丸ごと使う(FR1.3)。既定の `exit_codes` と成功・警告の重なりは誤り(FR1.5)。今のリトライの判断はジョブの `retry` だけを見る。
- **Decision**: CoreDefinition が検証の終わりに、各ジョブの実効の `retry` を決める。以後の判断と流れは今のまま(Q3 の A)。
- **Consequences**: 重なりの検査が検証の中で自然に行える。リトライの判断、`graph -v`、`--dry-run` が同じ実効の値を使う。定義の型から「既定だったか」の区別は消える(要るなら機能の設計で残す)。
- **Alternatives Rejected**: パイプラインとジョブの `retry` を別々に持ち、判断のたびに両方を見る案(Q3 の B)。合わせる処理が複数の場所に散り、食い違いが起きやすいため。

## ADR-004: 試行の時刻は cli の流れが測り、開始をステートに書いてからジョブを動かす

- **Context**: 試行ごとに開始と終了の時刻を残す(FR6.1)。`watch` は動いている最中のジョブに経過時間を出す(FR6.5)。試行の記録が 1 つも無い瞬間を作らない(要件のレビューの指摘 R-06)。
- **Decision**: CliFlows が、試行の前に差し替え口から現在の時刻を取り、開始をステートに書いてから `run_job` を呼ぶ。終わったら終了の時刻を書く(Q2 の B)。
- **Consequences**: `watch` は動いている最中のジョブの開始を読める。ステートの書き込みが試行ごとに 1 回増える(書き込みは原子的で、大きさは小さい)。`run_job` の結果の形は時刻のために変えずに済む。
- **Alternatives Rejected**: `run_job` が開始と終了を返す案(Q2 の A)。開始の時刻がジョブの終わるまで分からず、実行中の表示ができないため。

## ADR-005: ジョブの出力の複製は runtime が行い、ログのファイルの用意は別の部品にする

- **Context**: `log_dir` があるとき、出力を端末にも出しつつ、試行ごとに 1 つのファイルへまとめる(FR5.2、FR5.3)。今は子の出力を親から引き継ぐだけ。
- **Decision**: RuntimeRunner が、ログのファイルを渡されたときだけ子の標準出力と標準エラーをパイプで受け、端末とファイルの両方へ書く。ディレクトリとファイルの用意は RuntimeLog に分ける。`log_dir` が無い定義は今のまま引き継ぐ(FR5.5)。
- **Consequences**: 子の出力は端末に直結しなくなる(`log_dir` のときだけ)。そのため、子から見た標準出力は端末でなくなる。順番の保ち方、読み取りの並行、権限、名前の衝突は機能の設計で決める。
- **Alternatives Rejected**: ファイルだけに書いて後で端末に出す案(実行中に見えないため)。外部の `tee` を使う案(Windows に無く、依存が増えるため)。この 2 つは現実的でなく、質問しなかった。

## ADR-006: DOT の描画は cli の表示に、text・mermaid と同じ構造から足す

- **Context**: `graph --format dot` は `-v` と `--state` に対応する(FR4.2)。text と mermaid は、core の図の構造と状態の重ね合わせから描いている。
- **Decision**: CliReport に DOT の描画を足し、既存の構造と重ね合わせを入力にする。値は DOT の文字列として escape する(FR4.3)。
- **Consequences**: 3 つの形式の情報が揃う。状態をどの属性で表すかは機能の設計で決める(要件のレビューの指摘 R-04)。
- **Alternatives Rejected**: core に DOT の描画を置く案。表示の形は cli の責務という、今の分け方に反するため。

## ADR-007: `unset_env` は core の統合に足し、runtime がジョブの環境から消す

- **Context**: `unset_env` は親から引き継いだ変数だけを消す(FR3.2)。`env`・`env_file` との重なりは誤り(FR3.3)。今の統合の結果は「足す変数」だけ。
- **Decision**: CoreEnvFile の統合の結果を「足す変数」と「消す名前」にする。RuntimeRunner は親の環境から消す名前を除いてから足す。
- **Consequences**: 重なりの検査が統合の中で行える(`.env` の中身との重なりは読んだ後に分かる)。`run_job` に消す名前を渡す形に変わる。
- **Alternatives Rejected**: cli の流れで親の環境を作り直して渡す案。親の環境の取得が流れに入り、入出力が core・runtime の外に漏れるため。

## ADR-008: 打ち消しは `retry = false` と書く

- **Context**: 特定のジョブで既定のリトライを打ち消す(FR1.4)。
- **Decision**: `[[job]]` の `retry` に `false` を許す。`retry` は表か `false` のどちらか(Q4 の A)。`true` は誤り。
- **Consequences**: 書き方が短く、`retry` の 1 つのキーで意味が閉じる。`retry` の型の検査が「表か false」に広がる。
- **Alternatives Rejected**: 別のキー `no_retry = true` を足す案(Q4 の B。`retry` と両方書けて矛盾がありうるため)。`max = 0` と書く案(Q4 の C。`max` の範囲の意味が変わり、他の項目が要るか曖昧なため)。
