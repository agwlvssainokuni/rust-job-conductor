# User Stories

上流: requirements(FR1〜FR7、NFR1〜NFR6)、team-practices、personas。ストーリーは機能・コマンドごとにまとめ、要件の主なまとまりごとに1〜数個にした。[Q2、Q3] 非機能要件は独立したストーリーにせず、関係する機能ストーリーの受け入れ基準に取り込んだ。[Q4] 優先度は scope-document と requirements の優先度を引き継ぐ。MVP の境界は、Delivery Planning で決める。

ペルソナ: P1 運用担当者・開発者(ジョブの運用者)、P2 ローカル開発者(デバッグ・テスト担当)。

## 1. 定義ファイル

### US1.1 定義に従って、ジョブの実行順と遷移を決める【Must】

As a P1 運用担当者・開発者, I want to ジョブ・終了コードの対応・遷移ルールを `conductor.toml` に書いて、その定義どおりに実行されるようにしたい, so that 警告のときの分岐を、スケジューラを変えずに定義で決められる。

対応する要件: FR1.1、FR1.2、FR1.3、FR1.5

受け入れ基準:
- AC1.1.1: Given パイプラインの名前と説明、ジョブID、コマンド、`warning_exit_codes` を書いた正しい定義ファイルがある、When `conductor run <job-id>` を実行する、Then 定義に従ってジョブが実行される。
- AC1.1.2: Given 成功時の遷移先を書かないジョブがある、When そのジョブが成功する、Then 記述順で次のジョブが次の実行対象になる。
- AC1.1.3: Given 警告時の遷移先に、記述順で2つ後ろのジョブを書いた、When そのジョブが警告で終わる、Then 間のジョブはスキップ対象になり、2つ後ろのジョブが次の実行対象になる。
- AC1.1.4: Given 定義ファイルに複数のジョブがある、When `conductor pipeline <pipeline-name>` を実行する、Then ジョブは定義ファイルに書いた順に実行される。

INVEST: Independent、Valuable(分岐の定義が価値の中心)、Small、Testable。

依存: なし。

### US1.2 定義の誤りを、実行前に検知する【Must】

As a P1 運用担当者・開発者, I want to 定義に誤りがあれば、ジョブを動かす前に拒否されてほしい, so that 間違った定義で本番のジョブを動かしてしまうことを防げる。

対応する要件: FR1.4、FR1.5、NFR4

受け入れ基準:
- AC1.2.1: Given 存在しないジョブIDを遷移先に書いた定義ファイルがある、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、ファイル名・行・原因が表示され、非0で終了する。
- AC1.2.2: Given エラー結果に遷移先を書いた、またはジョブ自身か記述順で前のジョブを遷移先にした定義ファイルがある、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、原因が表示され、非0で終了する。
- AC1.2.3: Given 同じ終了コードを `warning_exit_codes` と成功コードの両方に書いた、または 0 を警告に指定した定義ファイルがある、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、原因が表示され、非0で終了する。
- AC1.2.4: Given 終了コードを文字列で書くなど、型の誤りがある定義ファイル、When `conductor run <job-id>` を実行する、Then ジョブが実行される前に、誤りが検知されて表示される。(NFR4)
- AC1.2.5: Given 定義の `name` が `nightly` である、When `conductor pipeline other` を実行する、Then 実行されず、名前の不一致が表示され、非0で終了する。

INVEST: Independent(検証だけで完結する)、Valuable、Small、Testable。

依存: US1.1。

### US1.3 定義ファイルの場所を指定する【Must】

As a P1 運用担当者・開発者, I want to 定義ファイルを、実行ディレクトリの既定の場所に置くか、パスを指定して使いたい, so that ジョブごと・環境ごとに定義ファイルを使い分けられる。

対応する要件: FR1.6

受け入れ基準:
- AC1.3.1: Given 実行ディレクトリに `conductor.toml` がある、When オプションなしで `conductor run <job-id>` を実行する、Then その `conductor.toml` が使われる。
- AC1.3.2: Given 別のパスに定義ファイルがある、When オプションでそのパスを指定して実行する、Then 指定したファイルが使われる。
- AC1.3.3: Given 定義ファイルが存在しない、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、存在しないファイルのパスが表示され、非0で終了する。

INVEST: Independent、Small、Testable。

依存: US1.1。

## 2. 個別実行

### US2.1 ジョブを1つ起動して、結果を分類する【Must】

As a P1 運用担当者・開発者, I want to 既存のスケジューラから `conductor run <job-id>` を呼んで、ジョブの終了コードを成功・警告・エラーに分類したい, so that 警告のときに別の処理へ分岐できる。

対応する要件: FR2.1、FR2.2

受け入れ基準:
- AC2.1.1: Given ジョブのコマンドがシェルのメタ文字(`;`、`&&`、`|`)を含む引数を持つ、When `conductor run` で実行する、Then メタ文字はシェルに解釈されず、そのまま引数として渡される。
- AC2.1.2: Given 作業ディレクトリと環境変数を設定した状態で `conductor` を起動する、When ジョブを実行する、Then ジョブは同じ作業ディレクトリと環境変数で起動される。
- AC2.1.3: Given 終了コード 0、`warning_exit_codes` の値、それ以外の非0を返すジョブ、When それぞれ実行する、Then 順に成功、警告、エラーとして分類される。
- AC2.1.4: Given 存在しないコマンド、またはシグナルで終了するジョブ、When `conductor run` で実行する、Then エラーとして扱われ、原因が表示され、非0で終了する。

INVEST: Independent、Valuable(分類が分岐の前提)、Small、Testable。

依存: US1.1。

### US2.2 警告や条件に応じて、後続をスキップ・ジャンプする【Must】

As a P1 運用担当者・開発者, I want to 警告のときに後続の特定のジョブをスキップしたり、代替のジョブへ進んだりしたい, so that 直列にしか流せないスケジューラでも、警告時の別処理を実現できる。

対応する要件: FR2.3、FR2.4

受け入れ基準:
- AC2.2.1: Given 警告時の遷移先が、記述順で2つ後ろのジョブである、When そのジョブが警告で終わる、Then 間のジョブはスキップ対象としてステートに記録される。
- AC2.2.2: Given スキップ対象になったジョブがある、When そのジョブを `conductor run` で呼び出す、Then コマンドは実行されず、終了コード 0 で終了する。
- AC2.2.3: Given 遷移先が「パイプラインをここで終える」である、When そのジョブが警告で終わる、Then 以降のジョブはすべてスキップ対象になる。
- AC2.2.4: Given 遷移先を書かない成功のジョブ、When 成功する、Then 記述順で次のジョブが実行され、スキップは起きない。

INVEST: Independent(US2.1 の分類があれば成立)、Valuable、Small、Testable。

依存: US2.1、US4.1。

### US2.3 スケジューラに期待どおりの終了コードを返す【Must】

As a P1 運用担当者・開発者, I want to 警告、エラー、スキップのときに `conductor` が返す終了コードを決めておきたい, so that スケジューラが 0 / 非0 の2値しか受け付けなくても、止めるべきときだけ止められる。

対応する要件: FR2.5

受け入れ基準:
- AC2.3.1: Given 警告で終わるジョブ、When `conductor run` で実行する、Then 終了コード 0 で終了する。
- AC2.3.2: Given エラーで終わるジョブ、When `conductor run` で実行する、Then ジョブの終了コードがそのまま(非0)返る。
- AC2.3.3: Given スキップ対象のジョブ、When `conductor run` で実行する、Then 終了コード 0 で終了する。

INVEST: Independent、Small、Testable。

依存: US2.1、US2.2。

### US2.4 `conductor` 自身の表示を制御する【Must】

As a P1 運用担当者・開発者, I want to 既定ではジョブ本体の出力だけが流れ、`conductor` 自身のエラーは必ず表示されるようにしたい, so that 既存のスケジューラのログを汚さずに、失敗の原因はいつでも分かる。

対応する要件: FR2.6

受け入れ基準:
- AC2.4.1: Given オプションを付けない、When 成功するジョブを `conductor run` で実行する、Then `conductor` 自身の表示は出ず、ジョブ本体の出力だけが流れる。
- AC2.4.2: Given オプションを付けない、When 定義ファイルの誤りなど `conductor` 自身のエラーが起きる、Then 原因が表示される。
- AC2.4.3: Given スキップ対象のジョブ、When オプションを付けずに `conductor run` で実行する、Then 何も表示されず、終了コード 0 で終了する。
- AC2.4.4: Given `conductor` 自身の表示を有効にするオプションを付ける、When ジョブを `conductor run` で実行する、Then オプションなしでは出なかった `conductor` 自身の表示が出る。オプションの名称と、表示する内容の粒度は設計で定める。

INVEST: Independent、Small、Testable。

依存: US2.1。

### US2.5 実行時の取り決め(時間制限・同時実行・起動の速さ・配布形態)【Must】

As a P1 運用担当者・開発者, I want to `conductor` が時間制限や排他制御をせず、毎回すばやく起動し、追加の準備なしで3つのOSで動くようにしたい, so that 既存のスケジューラの運用を変えず、毎ステップの呼び出しにも負担をかけない。

対応する要件: FR2.7、FR2.8、NFR1、NFR2、NFR3

受け入れ基準:
- AC2.5.1: Given 長時間かかるジョブ、When `conductor run` で実行する、Then `conductor` は時間切れにせず、ジョブの終了を待つ。(FR2.7)
- AC2.5.2: Given 同じ定義に対して `conductor run` を2つ同時に起動する、When 実行する、Then `conductor` は排他制御をせず、それぞれが実行される。(FR2.8)
- AC2.5.3: Given ジョブ10個程度の定義ファイル、When `conductor run <job-id>` を同じ環境で20回測定する、Then `conductor` 自身のオーバーヘッドの p95 が 100ms 以内で、Linux・macOS・Windows のそれぞれで満たす。(NFR1、NFR3)
- AC2.5.4: Given 外部ランタイムをインストールしていない環境、When `conductor` を実行する、Then 追加のインストールなしで動作する。(NFR2)
- AC2.5.5: Given Linux・macOS・Windows のそれぞれ、When 同じ定義で `conductor run <job-id>` を実行する、Then 同じ結果になる。(NFR3)

INVEST: Independent、Valuable、Small、Testable。

依存: US2.1。

## 3. 一連の実行

### US3.1 パイプライン全体を一括実行する【Must】

As a P2 ローカル開発者, I want to `conductor pipeline <pipeline-name>` でパイプライン全体を一気に実行したい, so that スケジューラなしで、分岐やスキップが定義どおりに動くかを確かめられる。

対応する要件: FR3.1、NFR3

受け入れ基準:
- AC3.1.1: Given 3つのジョブを定義し、すべて成功する、When `conductor pipeline` を実行する、Then 3つが順に実行され、終了コード 0 で終了する。
- AC3.1.2: Given 2つ目のジョブが警告で、警告時の遷移先が4つ目のジョブである、When `conductor pipeline` を実行する、Then 3つ目のジョブはスキップされ、4つ目が実行される。
- AC3.1.3: Given 2つ目のジョブの遷移先が「パイプラインをここで終える」である、When `conductor pipeline` を実行する、Then 以降のジョブは実行されず、パイプラインは完了とみなされ、ステートが初期化される。
- AC3.1.4: Given Linux・macOS・Windows のそれぞれ、When 同じ定義で `conductor pipeline` を実行する、Then 同じ結果になる。(NFR3)

INVEST: Independent(個別実行とは別に提供できる)、Valuable、Small、Testable。

依存: US1.1、US2.1、US4.1。

### US3.2 一括実行中にエラーが起きたら停止する【Must】

As a P2 ローカル開発者, I want to 一括実行中にジョブがエラーで終わったら、そこで止まってほしい, so that 後続のジョブが誤って実行されず、原因を確かめられる。

対応する要件: FR3.2

受け入れ基準:
- AC3.2.1: Given 2つ目のジョブがエラーになる、When `conductor pipeline` を実行する、Then 3つ目以降は実行されず、ジョブの終了コード(非0)で終了する。
- AC3.2.2: Given 2つ目のジョブがエラーになる、When 終了する、Then 自動での再試行は行われない。
- AC3.2.3: Given エラーで止まったあと、When ステートを確認する、Then エラーの位置が保持されている。

INVEST: Independent、Small、Testable。

依存: US3.1、US4.1。

## 4. ステート

### US4.1 実行状態をローカルに保持し、壊れたときは実行しない【Must】

As a P1 運用担当者・開発者, I want to 実行状態がローカルのステートファイルに安全に保持され、壊れていたら実行されないでほしい, so that 壊れた状態のまま誤ったジョブを動かさずに済む。

対応する要件: FR4.1、FR4.4、FR4.5

受け入れ基準:
- AC4.1.1: Given ジョブを実行する、When 実行が終わる、Then ステートが実行ディレクトリの近傍に作られ、スキップ・ジャンプと各ジョブの結果が保持される。
- AC4.1.2: Given ステートの書き込み中にプロセスが中断された、When 次に `conductor run <job-id>` を実行する、Then 書き込み途中の内容は読まれず、直前の完全なステートか、破損の検知になる。
- AC4.1.3: Given ステートファイルが読めない、または壊れている、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、ステートファイルのパスと原因が表示され、非0で終了する。

INVEST: Independent、Valuable、Testable。

依存: なし。

### US4.2 パイプライン完了時に、ステートを自動で初期化する【Must】

As a P1 運用担当者・開発者, I want to パイプラインが最後まで終わったら、ステートが自動で初期化されてほしい, so that 夜間バッチのように同じ流れを繰り返しても、前回の状態が残らない。

対応する要件: FR4.2

受け入れ基準:
- AC4.2.1: Given 3つのジョブを個別実行で最後まで実行した、When 最後のジョブが終わる、Then ステートは初期化され、次に最初のジョブを実行できる。
- AC4.2.2: Given 最後のジョブがスキップ対象である、When 最後のジョブを `conductor run` で呼び出す、Then スキップされ、その時点で完了とみなされ、ステートが初期化される。
- AC4.2.3: Given 2つ目のジョブの遷移先が「パイプラインをここで終える」で、3つ目が最後のジョブである、When 個別実行で2つ目から順に呼び出す、Then 3つ目は実行されずにスキップされ、その時点でステートが初期化される。
- AC4.2.4: Given 一連の実行で「パイプラインをここで終える」に到達した、When 実行が終わる、Then その時点でステートが初期化される。

INVEST: Independent、Valuable、Small、Testable。

依存: US4.1、US2.2。

### US4.3 エラーから再開する【Must】

As a P1 運用担当者・開発者, I want to ジョブがエラーで終わったあと、同じパイプラインを流し直すと、エラーになったジョブから再開してほしい, so that 成功済みのジョブをやり直さずに済む。

対応する要件: FR4.3

受け入れ基準:
- AC4.3.1: Given 個別実行で、2つ目のジョブがエラーで終わった、When スケジューラが同じパイプラインを1つ目から呼び直す、Then 1つ目はスキップされ、2つ目から再開する。
- AC4.3.2: Given 一連の実行で、2つ目のジョブがエラーで終わった、When `conductor pipeline` をもう一度実行する、Then 1つ目はスキップされ、2つ目から再開する。
- AC4.3.3: Given エラーで終わったあと、When `conductor reset` を実行して流し直す、Then 先頭から実行される。
- AC4.3.4: Given エラーで終わったあと、When 何も操作しない、Then 自動での再実行は行われない。

INVEST: Independent、Valuable、Small、Testable。

依存: US4.1、US6.1。

### US4.4 定義ファイルが変わったことを検知する【Must】

As a P1 運用担当者・開発者, I want to ステートが残った状態で定義ファイルのジョブ構成を変えたとき、食い違いが検知されてほしい, so that 古いステートで誤ったスキップや再開が起きない。

対応する要件: FR4.6

受け入れ基準:
- AC4.4.1: Given ステートが残った状態で、定義ファイルのジョブIDを改名した、When `conductor run <job-id>` を実行する、Then ジョブは実行されず、食い違いの原因が表示され、非0で終了する。
- AC4.4.2: Given 食い違いが検知された、When `conductor reset` を実行してから流し直す、Then 正常に実行できる。
- AC4.4.3: Given ジョブの記述順だけを変えた、When `conductor run <job-id>` を実行する、Then 食い違いとして検知される。

INVEST: Independent、Valuable、Small、Testable。

依存: US4.1、US6.1。

## 5. 補助コマンド

### US5.1 現在の状態を確認する(`watch`)【Should】

As a P2 ローカル開発者, I want to `conductor watch` で現在の実行状態を一覧で確認したい, so that どのジョブが成功・警告・スキップ・エラーになったかを、すぐ把握できる。

対応する要件: FR5.1、NFR6

受け入れ基準:
- AC5.1.1: Given 一部のジョブが実行済みでステートがある、When `conductor watch` を実行する、Then ジョブごとの状態が一覧で1回表示され、コマンドが終了する。
- AC5.1.2: Given ステートが空(初期状態)である、When `conductor watch` を実行する、Then ステートが空であることが表示され、エラーにならない。
- AC5.1.3: Given 出力が端末ではなくファイルにリダイレクトされている、When `conductor watch` を実行する、Then 色の制御文字が含まれず、状態は文字でも判別できる。(NFR6)

INVEST: Independent、Small、Testable。

依存: US4.1。

### US6.1 状態を初期化する(`reset`)【Must】

As a P2 ローカル開発者, I want to `conductor reset` で保持している状態を初期化したい, so that いつでも最初からやり直せる。

対応する要件: FR6.1

受け入れ基準:
- AC6.1.1: Given エラー終了で残ったステートがある、When `conductor reset` を実行する、Then ステートが初期化され、次の実行は先頭から始まる。
- AC6.1.2: Given ステートがない、When `conductor reset` を実行する、Then エラーにならず、終了コード 0 で終了する。

INVEST: Independent、Small、Testable。

依存: US4.1。

### US7.1 定義ファイルを YAML と TOML で相互に変換する(`convert`)【Should】

As a P1 運用担当者・開発者, I want to `conductor convert <file> --to <format>` で、定義ファイルを YAML と TOML のあいだで変換したい, so that 既存の YAML の資産を、`conductor.toml` として使える。

対応する要件: FR7.1、FR7.2

受け入れ基準:
- AC7.1.1: Given 正しい TOML の定義ファイル、When `conductor convert <file> --to yaml` を実行する、Then キーと値の意味が同じ YAML が得られる。コメントと記述順は保持されない。
- AC7.1.2: Given 正しい YAML の定義ファイル、When `conductor convert <file> --to toml` を実行する、Then キーと値の意味が同じ TOML が得られる。
- AC7.1.3: Given 文法が壊れた入力ファイル、When `conductor convert` を実行する、Then 変換は行われず、原因が表示され、非0で終了する。

INVEST: Independent(他の機能に依存しない)、Small、Testable。

依存: なし。

## ストーリーの優先度と依存の概観

| ストーリー | 優先度 | 依存するストーリー |
|------------|--------|--------------------|
| US1.1 | Must | なし |
| US1.2 | Must | US1.1 |
| US1.3 | Must | US1.1 |
| US2.1 | Must | US1.1 |
| US2.2 | Must | US2.1、US4.1 |
| US2.3 | Must | US2.1、US2.2 |
| US2.4 | Must | US2.1 |
| US2.5 | Must | US2.1 |
| US3.1 | Must | US1.1、US2.1、US4.1 |
| US3.2 | Must | US3.1、US4.1 |
| US4.1 | Must | なし |
| US4.2 | Must | US4.1、US2.2 |
| US4.3 | Must | US4.1、US6.1 |
| US4.4 | Must | US4.1、US6.1 |
| US5.1 | Should | US4.1 |
| US6.1 | Must | US4.1 |
| US7.1 | Should | なし |

## Assumptions & Open Questions

- 次の項目は、要件(requirements)の未解決事項を引き継ぎ、設計で決める。いずれも、US4.1〜US4.4 と US2.4 の受け入れ基準の前提に関わる。 [assumption]
  - 個別実行で、順序外のジョブ(前のジョブが未実行のまま)が呼ばれた場合と、すでに実行済みのジョブが再び呼ばれた場合の動作。
  - ステートファイルの名前と保存場所の具体、別パスの定義ファイルを指定したときのステートの紐づけ。
  - `conductor` 自身の表示を有効にするオプションの名称と粒度、「パイプラインをここで終える」の名称と書き方、成功コードを指定するキー名、起動失敗・シグナル終了時の具体的な終了コードの値。
- 一連の実行(`conductor pipeline`)の入出力イメージは未作成であり、AC3.1.x は、Requirements と Rough Mockups の記述に基づく。 [assumption]
