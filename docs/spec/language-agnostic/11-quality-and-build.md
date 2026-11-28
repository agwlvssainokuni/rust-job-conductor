# 11 品質・受け入れテスト・デモ・リリース

対象: `conductor` v0.10.2 までの振る舞い。この文書は、実装言語やビルドの仕組みを問わず、`conductor` の実装が満たすべき品質の基準を定める。

| 章 | 内容 |
|----|------|
| 1 | この文書の位置づけ |
| 2 | 非機能要件(性能、堅牢性、セキュリティ、保守性)の基準値と、強制か目安かの区別 |
| 3 | 受け入れテストの共通事項(実行環境、テスト用ジョブ、定義とステートの略記、判定の書き方) |
| 4 | 受け入れテスト一覧(機能ごと。ID は `AC-xxx`。4.17〜4.20 は v0.9.0 の機能、4.21〜4.27 は v0.10.0 の機能) |
| 5 | デモの仕様(構成、場面 0〜15、UNIX 系と Windows) |
| 6 | リリース物の要件(対応 OS、配布の形) |
| 7 | 品質ゲートの要件(言語中立) |
| 8 | 未確認の事項 |

他章との関係: 振る舞いの規定は [03](03-definition-format.md)(定義)、[04](04-cli.md)(CLI)、[05](05-execution-flow.md)(流れ)、[06](06-state-file.md)(ステート)、
[07](07-process-and-signals.md)(プロセスとシグナル)、[08](08-output-and-i18n.md)(出力)、[09](09-converter.md)(変換)、[10](10-errors-and-exit-codes.md)(終了コード)にある。
この文書の受け入れテストは、それらを、「前提 → 実行 → 期待」の形で、外から確かめる。再構築の手順は [12-rebuild-guide.md](12-rebuild-guide.md) にある。

---

## 1. この文書の位置づけ

- この文書は、言語・ビルド固有の事項(ビルドツールの名前、静的解析の設定、最小対応版のコマンドなど)を書かない。実装は、自分の言語の道具で、同じ基準を満たせばよい。
- 受け入れテストは、**実行ファイルを外から実行して**確かめる(終了コード、標準出力、標準エラー、作られるファイル)。実行ファイルの外からは確かめにくいものは、区分を「単体」「注入」「点検」「手動」として示す(3.6 節)。
- 受け入れテストの根拠は、参照実装のテストの名前を、各項目の「根拠」の欄に併記した。**テスト名だけ**で、その内容の説明は、この文書の本文が持つ。根拠が「派生」とあるのは、参照実装に同じ形のテストはなく、規定(05〜10 章)から導いた項目である。
- 確認できなかった点は「未確認」と書く(8 章)。

---

## 2. 非機能要件

「強制」は、基準を超えると受け入れテストが不合格になるもの。「目安」は、参照実装の基準値で、超えたら警告として記録する(不合格にしない)。他の実装は、目安を満たすことを目標とするが、
挙動(利用者から見える結果)を損なわない範囲で、別の値を採ってよい。「仮」は、参照実装の作者が、暫定として置いた値。

### 2.1 性能

p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目。測定値は、(標準エラーなどに)行として記録し、CI のサマリがあれば、そこにも記録する。基準を超えたときは、警告として記録する。
「強制」への切り替えは、実行の設定で選べるようにする(参照実装は、環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のとき、起動の測定の基準超えを不合格にする。CI の既定は `false`)。

| 項目 | 基準 | 扱い | 受け入れテスト |
|------|------|------|----------------|
| 定義(ジョブ 10 個)の読み込みと検証(取り込みから結果まで)を 100 回 | p95 20 ms | 目安 | AC-397 |
| 起動前の処理(`command` の検査、ステートの読み、照合、速い書き込み)を 100 回、ジョブ 10 個の定義 | p95 20 ms(仮)。耐久性つきの書き込みは基準なし(記録のみ) | 目安 | AC-398 |
| `run` の `conductor` 自身のオーバーヘッド(ジョブ 10 個の定義、20 回) | p95 100 ms | 目安 | AC-399 |
| `pipeline` のジョブ 1 つあたり(ジョブ 10 個、5 回。ジョブ自身の起動を含む) | p95 50 ms | 目安 | AC-400 |
| ジョブ 1,000 個の定義で `run job0` が終わるまで | 1,000 ms(仮) | 目安 | AC-401 |
| ジョブ 1,000 個の `pipeline` の流れの判断(プロセスの起動とファイルなし) | 10,000 ms | 目安。ただし、完了のステートが残ることは常に検査する | AC-402 |
| ジョブの実行中(約 3 秒)の `conductor` の CPU 時間(UNIX 系) | 100 ms 以下 | **強制**(CPU 時間を読めない環境ではスキップ) | AC-403 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され、どちらもジョブの終了コードで終わり、ステートは読み戻せる(排他制御はしない) | **強制** | AC-404 |
| リトライの待機の中断 | 合図から 1 秒以内に返る。確認の間隔は 50 ms 以下 | **強制**(1 秒)/ 目安(50 ms) | AC-275 |
| 図の生成(v0.9.0 以降)。ジョブ 1,000 個で、構造の組み立てと配置。最悪の 6 つの形(入れ子が最大の形、交差する形、幅の広い交差の形、同じ行き先が離れて重なる形、「両方の飛び越し」と分岐の入れ子、「両方の飛び越し」と分岐の幅広い交差) | 1 秒以内 | **強制** | AC-486 |
| 計画の生成(v0.9.0 以降)。ジョブ 1,000 個がすべて `env` と `env_file` を持ち、すべて実行する計画 | 1 秒以内 | **強制** | AC-501 |
| 計画の模擬(v0.9.7 以降)。ジョブ 1,000 個がすべて実行する計画 / `@end` のあとの 999 個のスキップの計画を、それぞれ 1 回。v0.10.1 以降は、実行と同じ判断を、ジョブの位置と済んだ前の部分の目印を持ち回って、1 つの判断あたり定数の手間で行う(05 の 16.2) | 参照実装の基準(開発用の版、5 回の中央値): 150 ms / 150 ms(v0.10.1 でも変えていない) | 目安 | AC-573 |
| 図の重ね合わせ(v0.9.7 以降)。ジョブ 1,000 個の状態の重ね合わせを 100 回 | 参照実装の基準(同): 350 ms | 目安 | — |
| 図の辺の取り出し(v0.9.7 以降)。ジョブ 1,000 個、遷移 1,996 個を 100 回 | 参照実装の基準(同): 600 ms | 目安 | — |
| `.env` の解析の重複キー(v0.9.7 以降)。10,000 行で後半 5,000 行が逆順の重複を 1 回 | 参照実装の基準(同): 100 ms | 目安 | — |
| 定義の `env` の表の大文字小文字の検査(v0.9.8 以降)。`[pipeline.env]` に別々の名前 10,000 個の定義の読み込みを 1 回 | 参照実装の基準(同): 350 ms | 目安 | — |
| 遷移の重複の確認(v0.9.8 以降)。1 つのジョブに `success` の遷移 20,000 個と `warning` の遷移 20,000 個(重複の誤り 39,998 件)の定義の読み込みを 1 回 | 参照実装の基準(同): 1,600 ms | 目安 | — |
| 読む対象の `env_file` の一覧(v0.9.8 以降)。`[pipeline].env_file` に別々の記述 10,000 個と、ジョブに同じ 10,000 個を 1 回 | 参照実装の基準(同): 40 ms | 目安 | — |
| DOT の描画(v0.10.0 以降)。直線の 1 万ジョブの図の DOT を 1 回 | 1 秒以内。行の数と再開位置の太い枠も確かめる | **強制** | AC-540 |
| ログのファイルへの写し(v0.10.0 以降)。16 MiB を出力するジョブを `log_dir` つきで実行 | 端末に出た内容とファイルの中身が一致する(時間の基準なし) | **強制**(中身) | AC-548 |
| `watch --follow` の繰り返し(v0.10.0 以降)。1,000 回の読み直し | 動き続け、ステートに書き込まない(時間の基準なし) | **強制** | AC-571 |
| ジョブの終了の確認の間隔 | 1・2・4・8・10 ms、以降 10 ms(上限 10 ms) | 目安 | 07 章 5 章 |

測定のしかた:

- 「`conductor` 自身のオーバーヘッド」は、`conductor` を起動してから終了するまでの時間から、ジョブ自身の実行時間(ジョブが、起動の直後と終了の直前に取った時刻の差)を引いた値。
- 「ジョブ 10 個の定義」は、10 個とも同じ形の簡単なジョブ(テスト用ジョブの `exit 0` か `timing`)。
- 測定の対象は、リリースの形(最適化した版)の実行ファイルで行う。参照実装の測定の行には、開発用の版の文言が出るが、CI は最適化した版で実行する(文言と実態の差。基準の判定には影響しない)。
- 性能の測定は、3 OS(Linux・macOS・Windows)で行い、失敗しても全体は不合格にしない(目安)。

### 2.2 堅牢性

外部の入力(コマンドライン、`command` の文字列、ステートファイル、定義ファイル、変換の入力、OS の結果)に対して、**異常終了しない**(失敗は終了コードとメッセージで返す)。【強制】

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ)500 回で、終了コードが 0〜255、標準エラーに制御文字(改行を除く)と U+202E がない | AC-405 |
| UTF-8 でない引数(UNIX 系) | ジョブID、`--config`、オプション、`convert` の引数に、不正なバイト列を渡しても異常終了しない | AC-406 |
| 壊れたステート | 壊れた TOML、ランダムなバイト列、状態・再開位置・フラグを乱したステート、巨大な値で、異常終了しない(終了コード 65) | AC-407 |
| 空・制御文字を含むジョブID | 異常終了しない | AC-408 |
| ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件で、分割が成功したら `program` は空でない。成功か誤り(64)かのどちらか | AC-409 |
| 大きな `command` | 約 30 万引数が分割できる。閉じない引用符は誤り。再帰に依存しない | AC-410 |
| 起動できない `command` | 空文字列、`'`、`''`、存在しない名前、ディレクトリのパス、`/` は、すべて起動失敗(または定義の誤り)。範囲外のシグナル番号の合図でも起動せず中断として扱う | AC-411 |
| 入れ子の深い入力 | 深さ 1,000 の TOML(定義・ステート)、深い YAML で、異常終了せず失敗を返す(既定の小さなスタックでも) | AC-412 |
| 製品のコードの書き方 | 外部の入力の経路に、異常終了を起こす書き方(未検査の取り出し、強制的な中断)がない | AC-421 |
| 内部の誤り | 異常終了を捕まえる仕組みが、固定の英語 `conductor: error: internal error` を標準エラーに書き、終了コード 70。標準エラーが閉じていても 70 | AC-414 |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力で異常終了しない。成功したランダムな構造は往復で等しい | AC-413 |
| 時間の上限 | プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して不合格) | 3.5 節 |

### 2.3 セキュリティ

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| シェルを呼ばない | 起動は、分割(または配列)で得た `program` だけ。`sh`・`cmd`・`bash`・`powershell`・`-c`・`/C` を挟む経路がない | AC-116(点検)と 4.2 節 |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`\|`、`$HOME`、`*`、`>x` などが、そのまま引数として渡る。Windows の `.cmd` への危険な引数で、続くコマンドが実行されない | 4.2 節 |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。定義ファイルのディレクトリは変わらない | AC-415 |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、ライブラリのエラー文)が現れない。キー名は制御文字をエスケープして表示 | AC-416 |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない。(v0.10.0 以降)足してよいのは、試行の開始と終了の時刻(`started_at`・`ended_at` と、その時刻の値)だけ | AC-417、AC-553 |
| ログのファイル(v0.10.0 以降) | 中身はジョブの出力だけ(`conductor` の表示を書かない)。UNIX 系は権限 0600 で作る。ファイル名に、ディレクトリの区切りと Windows の予約名が残らない(安全な ID) | AC-541、AC-543、AC-544 |
| 環境変数の値を出さない(v0.9.0 以降) | `env`・`.env` の値が、標準出力、標準エラー(`-v` を含む)、ステートファイルに現れない。`graph`・`--dry-run` の出力にも現れず、名前だけが出る(`graph` は名前も出さない)。`reservedEnvName`・`envNameCaseConflict`・`invalidEnvValue` の表示にも値が現れない | AC-468、AC-482、AC-492 |
| 表示のエスケープ | 制御文字・方向制御文字を無効化する(4.14 節の表)。定義・ステート・変換の各部が、同じ入力に同じ結果を返す | 4.14 節 |
| ネットワークに接続しない | 実行中に外向きの通信をしない。実装に、HTTP・TLS などの通信の部品を含めない | AC-419(点検) |
| 安全でない呼び出しの隔離 | 型・境界の検査を外す呼び出しは、シグナル送信の 1 か所だけ(それがある言語では)。ほかでは使わない | AC-420(点検) |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可した 4 つのシグナルだけ、1 回だけ | 4.10 節の単体の項目 |
| テスト専用の経路を製品に持たない | 製品の入口に、わざと異常終了するコマンドなどがない。配布物は `conductor` だけ(テスト用ジョブを含めない) | AC-422(点検)、6 章 |
| 変換の副作用 | 変換の部品に、プロセス・ネットワーク・環境変数のアクセスがない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | AC-418 |
| 既知の脆弱性 | 依存の既知の脆弱性を、定期的に(毎週)と、変更のたびに検査する。結果は警告で、マージをブロックしない | 7 章 |
| 配布物の完全性 | チェックサムのファイル(`SHA256SUMS`)を付ける(署名はしない。転送の破損の検知まで)。macOS の署名・公証はしない | 6 章 |

### 2.4 保守性

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| OS 固有コードの隔離 | OS ごとの分岐・OS 固有の呼び出しは、「プラットフォーム層」(OS 固有の処理をまとめた 1 か所)の中だけ。ジョブの起動の部品と、実行ファイルの入口の部品のそれぞれに持つ | AC-420 |
| 外部の入力で異常終了しない書き方 | 2.2 節 | AC-421 |
| 整形と静的検査 | 整形の差分と静的検査の警告が 1 つでもあればマージを止める | 7 章 |
| 命名 | 実装の言語の標準の命名規則に従う(プロジェクト独自の規則は設けない) | (規約) |
| 部品の分け方 | 定義の検証と流れの判断(入出力なし)、ジョブの起動とステート(入出力あり)、変換、入口(引数・流れの組み立て・表示)を分ける。入出力なしの部品は、ファイルとプロセスに触れずに検査できる | [02-architecture.md](02-architecture.md) |
| ソースへの著作権表示 | 各ソースファイルの先頭に、Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く | AC-423(点検) |

---

## 3. 受け入れテストの共通事項

### 3.1 実行環境

- 実行ファイル `conductor`(以下「被試験体」)と、テスト用ジョブ(3.2 節)を用意する。
- 各テストは、空の一時ディレクトリ(以下 `T`)で行う。`T` の中に定義ファイル(既定の名前 `conductor.toml`)を作り、`T` をカレントディレクトリにして被試験体を起動する。
  ステートファイルは、定義ファイルと同じディレクトリにできる(例: `T/conductor.toml.state`)。
- 環境変数は、特に記さない限り、次のとおり。`LANG=C`。`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`・`CONDUCTOR_RUN_ID` は未設定にする(表示を英語に固定し、利用者の環境の影響を除く)。
- 標準入力は空(閉じるか、空のファイル)にする。
- パスに空白や非 ASCII を含めない(特に記す場合を除く)。

### 3.2 テスト用ジョブ(`TJ`)

受け入れテストで使う、小さな実行ファイル。被試験体の配布物には含めない。OS ごとに実行できる形(UNIX 系の実行ファイル、Windows の `.exe`)にする。
定義の中では、`TJ` はその実行ファイルのパスを表す(例: `command = "TJ exit 3"`)。

```text
TJ [--out <ファイル>] <モード> [引数...]
```

- `--out <ファイル>` が先頭にあれば、**記録の行**をそのファイルに追記する(1 行 1 件、末尾に改行)。なければ標準出力に書く。ジョブの標準出力を被試験体越しに読めない場合があるため、結果はファイルで受け取る。
- 記録に出す文字列(値)は、`q(s)` で表す: 二重引用符で囲み、`"` と `\` の前に `\` を置き、改行は `\n`、復帰は `\r`、タブは `\t` と書く。
- 未知のモードは、終了コード 2。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n で終わる(省略・整数でないときは 2) |
| `run <label> <n>` | 記録 `ran=<label>` を出し、終了コード n(省略・整数でないときは 0) |
| `echo <args...>` | 各引数を、記録 `arg[i]=q(引数)`(i は 0 から)として 1 行ずつ出し、終了コード 0 |
| `env <NAME>...` | 各環境変数を、設定されていれば `env[NAME]=set:q(値)`、なければ `env[NAME]=unset` として出す。終了コード 0 |
| `cwd` | カレントディレクトリを `cwd=q(パス)` として出す。終了コード 0 |
| `pid` | 自分のプロセス識別子を `pid=<N>` として出す |
| `say <out> <err> [n]` | `<out>` を標準出力に、`<err>` を標準エラーに、改行を付けずにそのまま書き、終了コード n(省略は 0)。`--out` に従わない |
| `sleep [秒]` | `pid=<N>` と `ready` を記録し、眠る。秒があれば、その秒のあと終了コード 99。なければ眠り続ける。シグナルのハンドラを持たず、既定の動作で終わる(シグナルによる終了を、被試験体が観測できるようにするため) |
| `flaky <counter-file> <fails> <code>` | `<counter-file>` の整数(なければ 0)に 1 を足して書き戻す。足した値が `<fails>` 以下なら終了コード `<code>`、超えたら 0 |
| `timing` | 起動の直後の時刻と終了の直前の時刻を、UNIX 時刻のナノ秒で `start_ns=`・`end_ns=` として標準出力に書く(性能の測定用) |
| `burn <ms>` | 指定のミリ秒(既定 100)、CPU を使い続けて終わる(CPU 時間の測定用) |
| `flood <n>`(v0.10.0 以降) | 英小文字(`a` から順に 26 文字をくり返す)の 63 バイトと改行の 64 バイトを 1 かたまりとして、合わせて n バイト(最後のかたまりは途中まで)を標準出力に書き、終了コード 0。`--out` に従わない(大きな出力の写しの検査用) |
| `streams <n>`(同) | `out <i>`⏎ を標準出力に、`err <i>`⏎ を標準エラーに、i = 0 から n−1 まで交互に、1 行を 1 回の書き込みで書き、終了コード 0。`--out` に従わない |
| `orphan <秒>`(同) | 自分と同じ実行ファイルを `sleep <秒>`(既定 30)で起動し(標準出力・標準エラーを引き継ぐ子の子)、`parent done` を標準出力に書いて、子の子を待たずに終了コード 0 で終わる。起動できなければ終了コード 3(出力を持ち続ける子の子の検査用) |

補足: 参照実装の補助の実行ファイルは、`flaky` の代わりに、デモのスクリプト(`flaky`)が同じ役を担う。`env` と `cwd` の記録の形は、参照実装が言語固有の形で書くが、意味は同じ。
UNIX 系の受け入れテストでは、シグナルで終わるジョブを `["sh", "-c", "kill -9 $$"]`(配列形式)で作る(テスト用ジョブにモードを足さない)。

### 3.3 定義の略記

受け入れテストの前提の「定義」は、次の略記で書く。

```text
P(<パイプライン名> [run_id_env=<ENV>] [remove_on_complete=<true|false>]; <ジョブ>, <ジョブ>, ...)
```

| 略記 | 展開 |
|------|------|
| `P(p; ...)` | `format_version = 1`、`[pipeline]` の `name = "p"`、続いて `[[job]]` を、書いた順に並べる |
| `<id>:<n>` | `[[job]]`、`id = "<id>"`、`command = "TJ exit <n>"` |
| `<id>:<n>{...}` | 同上に、波括弧の中の修飾を足す |
| `w=[…]` | `warning_exit_codes = […]` |
| `s=[…]` | `success_exit_codes = […]` |
| `on=success->X` / `on=warning->X` | `[[job.transitions]]` に `on = "success"`(または `"warning"`)、`next = "X"`(X はジョブID か `@end`) |
| `retry=(exit_codes=[…], max=N, interval="…", multiplier=M, max_interval="…")` | `[job.retry]`(書いたキーだけ) |
| `cmd=<文字列>` | `command` を、その文字列(TOML の文字列。`TJ` は実行ファイルのパス)にする。`id:n` の `n` は使わない |
| `argv=[…]` | `command` を、その文字列の配列にする |
| `O` | 記録の出力ファイル(`T/out.txt` など。`--out` に渡す) |

例: `P(p; a:0, b:7{w=[3]}, c:0{on=success->@end})`。

### 3.4 ステートの略記

ステートの期待は、次の略記で書く。`end` と `completed` は、書かなければ `false`。`resume` は、書かなければ「キーがない」。

```text
S[<id>=<状態>(<終了コード>), ... ; resume=<id>, end=<true|false>, completed=<true|false>, run_id=<id>]
```

状態は `pending`・`success`・`warning`・`error`・`skipped`。スキップは `skipped/byTransition` か `skipped/byEnd`。終了コードがなければ括弧を省く。
例: `S[a=success(0), b=error(7), c=pending ; resume=b]`。ステートファイルの書式は [06-state-file.md](06-state-file.md)。

(v0.10.0 以降)動いたジョブ(`success`・`warning`・`error`)は、試行の時刻(`[[job.attempt]]` の `started_at`・`ended_at`。06 の 2.2a)も持つ。時刻は実行のたびに変わるので、略記には書かず、比べるときは除く。試行の期待は 4.26 節に書く。前提のステートを手で作るときは、動いたジョブに試行を 1 つ以上書く(書かなければ、v0.9.9 までの形として 65 になる。AC-559)。

### 3.5 判定の書き方

| 欄 | 意味 |
|----|------|
| 実行 | 被試験体の引数。`T` をカレントディレクトリにする。`--config` を省くと `T/conductor.toml` |
| 終了 | 終了コード |
| OUT | 標準出力。「空」は 0 バイト。引用は完全一致。「含む」は部分一致 |
| ERR | 標準エラー。同上。複数行は `⏎` で区切って書く(行の終わりの改行は省く) |
| 前提 | 実行の前に作るもの(定義、ステート、環境変数) |
| 区分 | 根拠の欄の末尾に `区分: …` と書く。書かなければ「自動」(外から実行して確かめる)。UNIX 系 / Windows / 非 root は、その環境でだけ行う |

- どのテストも、**30 秒の上限**で実行する(ハングの検知。超えたら、子プロセスを強制終了して不合格にする)。
- 「ジョブが実行された」は、`TJ run <label> <n>` の記録 `ran=<label>` で確かめる(`O` を読む)。「実行されない」は、その記録がないこと。
- 「ステートが変わらない」は、ステートファイルの内容(バイト列)が、実行の前後で同じこと。「ステートがない」は、ファイルがないこと。

### 3.6 区分

| 区分 | 意味 |
|------|------|
| (自動) | 被試験体を外から実行して確かめる |
| 単体 | 実装の内部の関数(分割、終了コードの範囲合わせ、エスケープ、判断の純粋な関数)を直接呼んで確かめる。外から再現しにくい入力のため。実装は、これらを検査できる形で持つ |
| 注入 | 標準出力への書き込みの失敗、シグナル登録の失敗など、外から作れない失敗を、差し替え(偽の出力先など)で起こして確かめる |
| 点検 | ソースや依存を、検索・確認して確かめる(実行ではない) |
| 手動 | 人が実機で確かめて、記録を残す |
| 計測 | 性能の測定(2.1 節) |

---

## 4. 受け入れテスト一覧

ID は、`AC-` と通し番号。以下の表は、ID・機能・前提・実行・期待・根拠の順。根拠は、参照実装のテストの名前(`ファイル名::テスト名`、日本語の名前の `_` は空白に当たる)。
期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま([08](08-output-and-i18n.md))。

### 4.1 定義の検査(`validate`、`run`、`pipeline` 共通)

特に記さない限り、実行は `validate`(`--config` を省く)。「64」の期待は、終了 64、OUT 空、ステートファイルを作らない、ジョブを起動しない、を含む。
ERR の期待は、**最初の行**のうち、`conductor: error: <ファイル>[:<行>]: ` に続く種類の文([08 4.3 節](08-output-and-i18n.md))を示す。ファイル名は、`--config` に渡した表記のまま。
基本の定義 `B0` は、`P(p; a:0)` とする。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-001 | 正しい定義 | `B0` | `validate` | 終了 0。OUT・ERR 空。`T/conductor.toml.state` を作らない | `binary_basic::validate は正しい定義で 0…`、`flow_validate::正しい定義は 0 で何も表示せず…` |
| AC-002 | validate の -v(1 ジョブ) | `B0` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (1 job)`。OUT 空 | `flow_validate::v は標準エラーに 1 行だけ出す` |
| AC-003 | validate の -v(複数) | `P(p; a:0, b:0, c:0)` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (3 jobs)` | 同上 |
| AC-004 | validate の -v(日本語) | `P(p; a:0, b:0, c:0)`、`LANG=ja_JP.UTF-8` | `validate -v` | 終了 0。ERR = `[conductor] 定義は正しいです(ジョブ 3 個)` | `report_messages::v の文言` |
| AC-005 | 定義ファイルがない | ファイルなし | `validate --config missing.toml` | 64。ERR = `conductor: error: missing.toml: definition file does not exist: 指定の場所にファイルがありません` | `core::definition_valid::存在しないファイルは file_missing…` |
| AC-006 | 定義ファイルがない(日本語) | 同上、`LANG=ja_JP.UTF-8` | 同上 | 64。ERR = `conductor: error: missing.toml: 定義ファイルが存在しません: 指定の場所にファイルがありません` | `report_messages::定義の誤りの日本語は…` |
| AC-007 | 通常のファイルでない | `T/d` がディレクトリ | `validate --config d` | 64。ERR の種類の文 = `cannot read definition file`(理由 `通常のファイルではありません`) | 派生(03 章 2.2 節) |
| AC-008 | 空のファイル | 長さ 0 | `validate` | 64。ERR = `conductor: error: conductor.toml: format_version is missing: …`(行なし) | 派生(03 章 2.2 節)、`core::definition_errors::br1_7` |
| AC-009 | 文法の誤り(行つき) | `format_version = 1⏎[pipeline⏎` | `validate` | 64。ERR の最初の行が `conductor: error: conductor.toml:2: syntax error: ` で始まる | `core::definition_errors::文法の誤りは syntax で行つき` |
| AC-010 | キーの重複(文法の誤り) | `format_version = 1⏎format_version = 1⏎` | `validate` | 64。種類の文 = `syntax error`(行 2) | 派生(03 章 3 節) |
| AC-011 | 書式の版が 1 でない | `format_version = 2` に `B0` の他を足す | `validate` | 64。`unsupported format_version` | `core::definition_errors::br1_7 版の確認が先で他の誤りは出ない` |
| AC-012 | 書式の版が整数でない | `format_version = "1"` | `validate` | 64。`unsupported format_version` | 同上 |
| AC-013 | 版の確認が先 | `format_version` なし、未知のキーと型の誤りを含む | `validate` | 64。ERR は 1 行だけ。種類の文 = `format_version is missing` | `core::definition_errors::br1_7` |
| AC-014 | 旧形式のキー | `B0` の `[[job]]` を `[[jobs]]` に | `validate` | 64。`unknown key` | `flow_run::古い jobs のキーは定義の誤り` |
| AC-015 | 未知のキー(ジョブ) | `B0` の job に `colour = "red"` | `validate` | 64。`unknown key`(行はそのキーの行) | `core::definition_errors::br1_6 知らないキー` |
| AC-016 | 未知のキー(パイプライン) | `[pipeline]` に `owner = "x"`(または `remove_on_complet = true`) | `validate` | 64。`unknown key` | `core::definition_errors::fr2_2…未知のキーは今までどおり拒否` |
| AC-017 | `[pipeline]` がない | `format_version = 1` と `[[job]]` だけ | `validate` | 64。`wrong value type`(行 1) | `core::definition_errors::br1_5 パイプラインの型の誤りと欠落` |
| AC-018 | `[[pipeline]]`(配列) | `[[pipeline]]` と書く | `validate` | 64。`wrong value type` | `core::definition_errors::br1_12 パイプラインが 2 つ以上` |
| AC-019 | パイプライン名が空 | `name = "  "` | `validate` | 64。`required value is empty` | `core::definition_errors::br1_13 構造の制約` |
| AC-020 | ジョブがない | `job` を書かない(または `job = []`) | `validate` | 64。`no jobs are defined` | 同上 |
| AC-021 | ジョブIDの重複 | `P(p; a:0, a:0)` | `validate` | 64。`duplicate job id` | `core::definition_errors::br1_13` |
| AC-022 | ジョブIDが不正 | ジョブID が `""`、または `@x` | `validate` | 64。`invalid job id` | 同上 |
| AC-023 | 遷移先がない | `P(p; a:0{on=success->nowhere})` | `validate` | 64。`transition target job is not defined` | `core::definition_errors::br1_1` |
| AC-024 | エラー結果への遷移 | `[[job.transitions]]` の `on = "error"` | `validate` | 64。`no transition can be written for the error result` | `core::definition_errors::br1_2` |
| AC-025 | `on` の値が不正 | `on = "later"`、または `"Success"` | `validate` | 64。`wrong value type` | 派生(03 章 6.4 節) |
| AC-026 | 自分自身・前のジョブへの遷移 | `P(p; a:0{on=success->a}, b:0)`、`P(p; a:0, b:0{on=success->a})` | `validate` | 64。`transition target must be a later job or @end` | `core::definition_errors::br1_3` |
| AC-027 | `@end` への遷移は正しい | `P(p; a:0{on=success->@end}, b:0)` | `validate` | 0 | `core::definition_valid::定義の外の遷移も next が end なら受理される` |
| AC-028 | 同じ結果の遷移の重複 | `on = "success"` の遷移を 2 つ | `validate` | 64。`duplicate transition for the same result` | `core::definition_errors::br1_13` |
| AC-029 | 0 を警告にできない | `P(p; a:0{w=[0]})` | `validate` | 64。`0 cannot be a warning exit code` | `core::definition_errors::br1_4` |
| AC-030 | 成功と警告の重なり | `P(p; a:0{w=[3], s=[0,3]})` | `validate` | 64。`success and warning exit codes overlap` | 同上 |
| AC-031 | 終了コードの型の誤り | `w=["x"]`、`w=[1.5]`、`w=[99999999999999999999]` | `validate` | 64。`wrong value type`(要素ごとに 1 行) | `core::definition_errors::br1_5 型の誤り` |
| AC-032 | 負の終了コード・空の成功コード | `w=[-1]`、`s=[]` | `validate` | 0(受理する) | 派生(03 章 6.2 節) |
| AC-033 | `command` の型・欠落・空 | `command` なし / `""` / `"   "` / `[]` / `[""]` / `["  ", "x"]` / `5` / `["a", 5]` | `validate` | 64。なし・`5`・`["a",5]` は `wrong value type`。ほかは `required value is empty` | `core::definition_errors::fr1_3 / fr1_4 / fr1_5` |
| AC-034 | 配列形式の `command` の受理 | `argv=["my prog", "a b", "$HOME", ""]`、`argv=["echo", ""]` | `validate` | 0 | `core::definition_valid::fr1_2 配列の command は要素のまま保持される` |
| AC-035 | `run_id_env` の名前の形 | `run_id_env` が `1A` / `A-B` / `""` / `A B` / `A=B` / `A.B` | `validate` | 64。`invalid environment variable name`。整数 `1` なら `wrong value type`。`_a1`・`GITHUB_RUN_ID`・`A` は 0 | `core::definition_errors::fr2_run_id_env の型と形の誤りは定義の誤り`、`binary_run_id::無効な値は 68…` |
| AC-036 | `remove_on_complete` の型 | `"true"` / `1` / `[true]` | `validate` | 64。`wrong value type`。`true`・`false`・省略は 0 | `core::definition_errors::fr2_4`、`fr2_2` |
| AC-037 | 誤りは位置の順に 1 行ずつ | `ME`(この節の末尾の定義) | `validate` | 64。ERR は 4 行。順に `…:2: unknown key`、`…:10: wrong value type`、`…:15: unknown key`、`…:19: transition target job is not defined` | `core::definition_errors::複数の誤りが一度にすべて返る`、`flow_run::定義の誤りは 64 ですべて表示し…` |
| AC-038 | 行番号は CRLF でも同じ | `ME` を CRLF 改行で | `validate` | 同上(行番号が同じ) | `core::definition_errors::行番号は crlf でも一致する` |
| AC-039 | BOM つきの定義 | `B0` の先頭に UTF-8 の BOM | `validate` | 0 | 派生(03 章 3 節) |
| AC-040 | 定義の大きさの上限(受理) | `B0` に、コメント行を足して、ちょうど 1,048,576 バイト | `validate` | 0 | 派生(03 章 2.2 節) |
| AC-041 | 定義の大きさの上限(超過) | 1,048,577 バイト(UTF-8 でなくてもよい) | `validate` | 64。`definition file is too large` | 派生(03 章 2.2 節) |
| AC-042 | UTF-8 でない定義 | 上限以内で、不正なバイト列を含む | `validate` | 64。`cannot read definition file` | 派生(03 章 2.2 節) |
| AC-043 | 入れ子の深さ(受理) | `a = [` を 32 段にしただけのファイル(`format_version` なし) | `validate` | 64。`format_version is missing`(深さの検査は通った) | `core::definition_errors::深さ 32 は受け入れ 33 と 1000 は syntax` |
| AC-044 | 入れ子の深さ(超過) | 33 段、1,000 段 | `validate` | 64。`syntax error` | 同上、`core::library_capability` |
| AC-045 | パイプライン名の照合 | `P(p; a:0)` | `validate p` / `validate wrong` / `validate` | 0 / 64(`pipeline name does not match the definition`)/ 0(照合しない) | `flow_validate::パイプライン名は 一致で 0 不一致で 64 省略で照合しない` |
| AC-046 | 名前の不一致の誤りに値を含めない | `name = "SECRET-NAME"` | `validate other-name` | 64。ERR に `SECRET-NAME` も `other-name` も現れない | `core::definition_valid::パイプライン名が違うと pipeline_name_mismatch で値は出ない` |
| AC-047 | 誤りの文に入力の値を含めない | `format_version = 1`、`[pipeline]` の `name = "p"`、`[[job]]` の `id = "a"`、`command = 5`、`warning_exit_codes = ["SENTINEL-VALUE"]` | `validate` | 64。ERR に `SENTINEL-VALUE` が現れない | `core::nfr_security`、`core::definition_errors::nfr2` |
| AC-048 | `command` の引用符が閉じていない | `P(p; a:0{cmd="TJ exit 0 'x"})` | `validate` | 64。ERR = `conductor: error: conductor.toml: job 'a': command has an unclosed quote` | `flow_validate::command の誤りは 64 で run と同じ誤りの文` |
| AC-049 | `command` が空になる | `cmd="''"` | `validate` | 64。`job 'a': command is empty` | 同上 |
| AC-050 | `command` の誤りは全ジョブ報告 | 2 つのジョブがそれぞれ誤り | `validate` | 64。ERR は 2 行(ジョブごと) | `flow_run::command の誤りは 64 でジョブ id を表示する` |
| AC-051 | `command` の誤り(日本語) | 同上、`LANG=ja_JP.UTF-8` | `validate` | ERR = `conductor: error: conductor.toml: job 'a': command の引用符が閉じていません` | `report_messages` |
| AC-052 | `run` でも `command` の誤りは 64 でジョブは起動しない | 1 つ目が正しく、2 つ目が誤り | `run a` | 64。`ran=` の記録なし。ステートなし | `flow_run::command の誤りは 64 でジョブ id を表示する` |
| AC-053 | 起動できないプログラムでも validate は 0 | `cmd="/no/such/program"` | `validate` | 0 | `flow_validate::起動できないプログラムの command でも 0` |
| AC-054 | 壊れたステートでも validate は 0 | `B0`、`T/conductor.toml.state` に `garbage = = =` | `validate p` | 0。ステートは変わらない | `binary_basic::validate は…壊れたステートでも 0`、`flow_validate::ステートが壊れていても…` |
| AC-055 | validate の引数の誤り | `B0` | `validate a b` / `validate --run-id x` / `validate --to yaml` | 68 | `flow_validate::位置引数が 2 個や to は使い方の誤りで 68`、`binary_run_id::無効な値は 68…` |
| AC-056 | `run_id_env` が不正な定義は run でも 64 | `run_id_env = "1A"` | `run a` | 64 | `binary_run_id::無効な値は 68 で validate は run_id を受け付けず run_id_env の誤りは 64` |
| AC-057 | validate は副作用を持たない | `B0` の `command` が「`TJ run a 0`」(`O` に記録) | `validate` | 0。`O` が作られない。`T` のファイル一覧が実行の前後で同じ | `core::nfr_security`(派生) |

#### `[job.retry]` の検査

`R0` は、`P(p; a:75{retry=(exit_codes=[75], max=3, interval="1s")})`。各行は、`R0` の `retry` を変えて `validate` する。誤りは終了 64、種類の文は ERR の最初の行。

| ID | 機能 | 変更後の `retry` | 期待 | 根拠 |
|----|------|------------------|------|------|
| AC-058 | 受理(固定の間隔) | `R0` のまま | 0 | `core::definition_retry::固定の間隔の設定を読める`、`flow_validate::正しい retry を持つ定義は 0…` |
| AC-059 | 受理(倍率と上限) | `multiplier=2, max_interval="1m"` を足す | 0 | `core::definition_retry::倍率と上限つきの設定を読める` |
| AC-060 | 受理(境界) | `exit_codes=[1,255,127]`、`max=1` と `max=100`、`interval` が `1s`・`24h`・`86400s`・`1440m`、`multiplier=2` と `10`、`max_interval` が `interval` と同じ | いずれも 0 | `core::definition_retry::最小値の境界を受理する / 最大値の境界を受理する / br1_13 上限が間隔と同じなら受理する` |
| AC-061 | 受理(インラインの表) | `retry = { exit_codes = [75], max = 3, interval = "1s" }` | 0 | `core::definition_retry::インラインテーブルでも書ける` |
| AC-062 | `retry` が表でない | `retry = 5`、または `[[job.retry]]` | 64。`retry must be a table` | `core::definition_retry::br1_2` |
| AC-063 | 必須のキーがない | `[job.retry]` を空に | 64。`a required key of retry is missing` が 3 行(`exit_codes`、`max`、`interval` の順) | `core::definition_retry::br1_3 空の表は 3 つの必須のキーすべてを報告する` |
| AC-064 | 未知のキー | `retries = 2` を足す | 64。`unknown key` | `core::definition_retry::br1_4`、`flow_validate::retry の許可されていないキーは 64` |
| AC-065 | `exit_codes` が空 | `exit_codes=[]` | 64。`retry exit_codes is empty` | `core::definition_retry::br1_5 終了コードの一覧が空` |
| AC-066 | `exit_codes` の要素の範囲 | `exit_codes=[0, 75, 256]` | 64。`retry exit_codes has an invalid exit code` が 2 行(0 と 256。要素ごと)。`-1`、`1.5`、`"75"`、`true` も同じ | `core::definition_retry::br1_5 終了コードの範囲の内側と外側 / 複数の不正な要素は要素ごとに報告する` |
| AC-067 | 成功・警告の終了コードと重なる | `{s=[0,5]}` で `exit_codes=[5,75]`、`{w=[4]}` で `exit_codes=[4]` | 64。`retry exit_codes overlap the success or warning exit codes` | `core::definition_retry::br1_7` |
| AC-068 | 最大回数の範囲 | `max=0`、`101`、`-1`、`1.5`、`"3"` | 64。`retry max is invalid` | `core::definition_retry::br1_9` |
| AC-069 | 間隔の書式 | `interval` が `10` / `1.5s` / `0s` / `25h` / `1S` / `86401s` / `1ms` / `" 1s"` / `"1 s"` / 整数 | 64。`retry interval is invalid`(`max_interval` も同じ規則) | `core::definition_retry::br1_10 間隔の境界の内側と外側 / 間隔の上限にも同じ書式の規則を使う` |
| AC-070 | 倍率の範囲 | `multiplier=1`、`11`、`0`、`2.0`、`"2"`(`max_interval` を添える) | 64。`retry multiplier is invalid` | `core::definition_retry::br1_12` |
| AC-071 | 倍率と上限は片方だけ不可 | `multiplier=2` だけ、`max_interval="1m"` だけ | 64。`retry multiplier and max_interval must be given together or not at all` | `core::definition_retry::br1_11` |
| AC-072 | 上限が間隔より小さい | `interval="1m", multiplier=2, max_interval="30s"` | 64。`retry max_interval is smaller than interval` | `core::definition_retry::br1_13 上限が間隔より小さい` |
| AC-073 | 誤りは位置の順、ほかの誤りと一緒に | `retry` の誤り 2 つと、別の定義の誤りを混ぜる | 64。ERR は位置の順に 1 行ずつ | `core::definition_retry::br1_14`、`flow_validate::retry の誤りは複数あれば 1 行ずつ位置の順` |
| AC-074 | `run` と同じ誤りの文 | 上のいずれかの誤りの定義で `run a` を実行 | `validate` と同じ文で 64 | `flow_validate::retry の誤りは 64 で run と同じ誤りの文` |
| AC-075 | 誤りの文に入力の値を含めない | `interval = "SENTINEL-VALUE"` で `validate`。64。ERR に `SENTINEL-VALUE` が現れない | 同左 | `core::definition_retry::誤りの表示に入力の値は出ない` |

`ME`(誤りが複数ある定義。行番号は、この並びのとおり):

```toml
format_version = 1
unknown_top = true

[pipeline]
name = "p"

[[job]]
id = "a"
command = "x"
success_exit_codes = ["zero"]

[[job]]
id = "b"
command = "y"
colour = "red"

[[job.transitions]]
on = "success"
next = "nowhere"
```

### 4.2 `command` の分割と起動

「分割」の行は、`P(p; a:0{cmd="TJ --out O echo <X>"})` を作り、`run a` を実行する(`TJ`、`O` は 3.3 節)。`<X>` は、各行の「`command` の引数部」。
期待は、終了 0、OUT・ERR 空、`O` の記録(`arg[0]=…` 以降)。記録は `q(…)` の形(3.2 節)。ここでは、引数の値だけを書く(`arg[i]` の添字は先頭から 0)。

| ID | 機能 | `command` の引数部(`TJ --out O echo` の後ろ) | 期待する引数の並び | 根拠 |
|----|------|------------------------------------------------|---------------------|------|
| AC-076 | 分割: 基本 | `a b` | `a`、`b` | `split` の単体(07 章 3.3 節) |
| AC-077 | 分割: 空白・タブ・改行の混在 | `␣␣a ⇥ b ⏎ c␣␣`(TOML では複数行文字列で書く) | `a`、`b`、`c` | 同上 |
| AC-078 | 分割: 一重引用符 | `'a b' 'c\d' '"x"'` | `a b`、`c\d`、`"x"` | 同上 |
| AC-079 | 分割: 二重引用符と `\"` | `"a \" b"` | `a " b` | 同上 |
| AC-080 | 分割: 二重引用符の中の `\n` `\\` | `"a\nb" "c\\d"` | `a\nb`、`c\\d`(バックスラッシュは、そのまま。`\n` は 2 文字) | 同上 |
| AC-081 | 分割: `\\"` は引用符を閉じない | `"\\"x"` | `\"x` | 同上 |
| AC-082 | 分割: 引用符の外のバックスラッシュ | `a\ b` | `a\`、`b` | 同上 |
| AC-083 | 分割: 末尾が `\` のパスは一重引用符 | `'C:\dir\'` | `C:\dir\` | 同上 |
| AC-084 | 分割: `--msg='a b'` | `--msg='a b'` | `--msg=a b` | 同上 |
| AC-085 | 分割: 引用符のつなぎ | `a"b c"'d e'f` | `ab cd ef`(1 つ) | 同上 |
| AC-086 | 分割: 空の引数 | `'' "" x` | (空)、(空)、`x` | 同上 |
| AC-087 | 分割: 非 ASCII | `'あ い' う` | `あ い`、`う` | 同上 |
| AC-088 | 分割: メタ文字は解釈しない | `a;b && c \| d > e $HOME * `` `x` `` (y)` | `a;b`、`&&`、`c`、`\|`、`d`、`>`、`e`、`$HOME`、`*`、`` `x` ``、`(y)`。`T` に `e` というファイルができない | `runner_basic::引数のメタ文字がそのまま渡る` |
| AC-089 | 分割: 区切りは 4 文字だけ | `a<U+00A0>b`(U+00A0 は空白に見えるが区切りではない) | `a<U+00A0>b`(1 つ) | 同上(07 章 3.2 節) |
| AC-090 | 配列形式: 要素はそのまま | `argv=["TJ", "--out", "O", "echo", "a b", "  lead and trail  ", "$HOME", "a;b", "*", ""]` | `a b`、`  lead and trail  `、`$HOME`、`a;b`、`*`、(空) | `runner_basic::配列の command の要素は分割も展開もされずにそのまま渡る` |
| AC-091 | 文字列と配列は同じ結果 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 7"})` と、同じ内容の `argv` 形式 | それぞれ `run a` → 0、`pipeline p` → 7、OUT・ERR 空。`O` = `ran=a`⏎`ran=b` が同じ | `binary_basic::配列の command は run と pipeline で文字列と同じ結果になる` |

分割の誤り(拒否)。実行は `validate`。終了 64。ERR の最初の行は `conductor: error: conductor.toml: job 'a': <文>`。

| ID | 機能 | `command`(文字列) | 期待する文 | 根拠 |
|----|------|---------------------|------------|------|
| AC-092 | 拒否: 一重引用符が閉じていない | `echo 'a` | `command has an unclosed quote` | `split` の単体(07 章 3.4 節) |
| AC-093 | 拒否: 二重引用符が閉じていない | `echo "a` | 同上 | 同上 |
| AC-094 | 拒否: 末尾が二重引用符の中の `\` | `echo "a\` | 同上 | 同上 |
| AC-095 | 拒否: `\\"` で閉じない | `echo "a\\" b` | 同上 | 同上 |
| AC-096 | 拒否: パスの末尾の `\"` | `prog "C:\dir\"` | 同上 | 同上 |
| AC-097 | 拒否: 空の語だけ | `''`、`'' x` | `command is empty` | 同上 |
| AC-097a | 拒否: 空白だけの先頭の語(v0.9.8 以降) | `'  '`、`'  ' x`、`"⇥"` | `command is empty`(64。ジョブを起動しない。v0.9.7 までは起動の失敗 127) | `split` の単体、`flow_validate::空白だけのコマンド名は 64 で ジョブを起動しない` |
| AC-098 | 拒否: 区切りだけ | `␣⇥⏎`(TOML では `"  \t\n"`)。定義の検査が、空白だけの `command` を先に拒否する | `required value is empty`(定義の検査。64) | `core::definition_errors::fr1_5`(派生) |

起動と実行環境。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-099 | 環境変数の引き継ぎ | `P(p; a:0{cmd="TJ --out O env CT_VAR CT_NO_SUCH"})`。環境に `CT_VAR=hello`、`CT_NO_SUCH` は未設定 | `run a` | 終了 0。`O` = `env[CT_VAR]=set:"hello"`⏎`env[CT_NO_SUCH]=unset` | `runner_basic::環境変数と作業ディレクトリを引き継ぐ` |
| AC-100 | 作業ディレクトリは引き継ぐ | `cmd="TJ --out O cwd"`。定義は `T/sub/x.toml` | `T` で `run a --config sub/x.toml` | `O` = `cwd=q(T のパス)`(`T/sub` ではない) | `runner_basic::環境変数と作業ディレクトリを引き継ぐ`(派生: 定義の場所へ移らない) |
| AC-101 | run ID を環境変数として渡さない | `cmd="TJ --out O env CONDUCTOR_RUN_ID"` | `run a --run-id r1` | `O` = `env[CONDUCTOR_RUN_ID]=unset` | 派生(07 章 4.2 節) |
| AC-102 | 標準出力・標準エラーはそのまま | `cmd="TJ say 'OUT to stdout' 'ERR to stderr' 0"` | `run a` | 終了 0。OUT = `OUT to stdout`(改行なし、完全一致)。ERR = `ERR to stderr`(完全一致) | `binary_basic::ジョブの出力は そのまま通り conductor の表示が混ざらない` |
| AC-103 | 存在しないプログラム | `cmd="/no/such/program-for-conductor-test"` | `run a` | 終了 127。ERR に `conductor: error: job 'a': cannot start command '/no/such/program-for-conductor-test'`。OUT 空。ステート `S[a=error ; resume=a]` | `binary_basic::ジョブを起動できないと 127`、`flow_run::起動できないときは 127 で cannot start command` |
| AC-104 | 配列形式の起動失敗は先頭の要素の名前 | `argv=["no-such-prog-zzz", "--flag"]` | `run a` | 127。ERR に `cannot start command 'no-such-prog-zzz'` | `flow_run::配列の command の起動失敗は先頭の要素の名前を表示する` |
| AC-105 | 空白を含む先頭の要素 | `argv=["my prog", "x"]`(`my prog` というファイルはない) | `run a` | 127。`cannot start command 'my prog'` | `runner_basic`(07 章 3.5 節) |
| AC-106 | 実行できないファイル(UNIX 系) | `cmd` が実行権限のない通常のファイル(`T/data.txt`)のパス | `run a` | 127 | `runner_basic::実行できないファイルは起動失敗` |
| AC-107 | ディレクトリ・`/` を指定(UNIX 系) | `cmd="/"`、`cmd` がディレクトリのパス | `run a` | 127 | `runtime::nfr_robust::起動できない command`、07 章 4.2 節 |
| AC-108 | 起動失敗は表示の言語に従う | `LANG=ja_JP.UTF-8`、存在しないプログラム | `run a` | 127。ERR に `コマンドを起動できません` | `report_messages` |
| AC-109 | 終了コードの返り方 | `exit 0`、`exit 3`、`exit 255` のジョブ(警告の指定なし) | `run a` | 0 / 3 / 255。OUT・ERR 空 | `runner_basic::終了コードの返り方`、`binary_basic::警告は 0 スキップは 0 エラーはジョブの値` |
| AC-110 | 大きな終了コード | `TJ exit 300` | `run a` | 44(UNIX 系も Windows も `300 mod 256`) | `runner_basic::終了コードの返り方`、`flow_run::範囲外の終了コードは どの os でも下位 8 ビット` |
| AC-111 | 256 の終了コード(Windows) | `TJ exit 256`、`-v` | `run -v a` | 1。ERR に `exit code 256, conductor exits with 1` | 同上(Windows) |
| AC-112 | 負の終了コード(Windows) | `TJ exit -1`、`-v` | `run -v a` | 255。ERR に `exit code -1, conductor exits with 255` | 同上(Windows) |
| AC-113 | `.cmd` への危険な引数(文字列形式。Windows) | `T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + ` 'x" & echo injected > T\injected.txt'` | `run a` | ジョブが起動される(`started`)か起動を拒否される(127)のどちらか。`T/injected.txt` ができない | `runner_basic::cmd への危険な引数は注入にならない` |
| AC-114 | `.cmd` への危険な引数(配列形式。Windows) | 同じスクリプト。`argv=[パス, "x\" & echo injected > T\injected.txt"]` | `run a` | 同上 | `runner_basic::cmd への配列の危険な引数は注入にならない` |
| AC-115 | `.cmd` への安全な引数(Windows) | `T/ok.cmd`(`exit /b 7`)。`cmd` = そのパス + ` plain` | `run a` | 7 | `runner_basic::cmd への安全な引数は起動できる` |
| AC-116 | シェルを使わないことの点検 | ソース | (検索) | シェルの名前(`sh`・`cmd`・`bash`・`powershell`・`-c`・`/C`)で起動する経路がない。起動する呼び出しが、1 種類(分割結果の `program` を渡すもの)だけ | `runtime::nfr_source::シェルを呼ばない`。区分: 点検 |

### 4.3 `run`(個別実行)

特に記さない限り、`P(…)` の `<id>:<n>` は、3.3 節のとおり `TJ exit <n>`。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-117 | 成功 | `P(p; a:0)` | `run a` | 終了 0。OUT・ERR 空。ステート: `S[a=success(0) ; completed=true]` が残る(最後のジョブで完了。完了のステートは残る) | `binary_basic::成功は 0 で 標準出力も標準エラーも空`、`flow_run::fr1_1 最後のジョブの完了で完了のステートを残す` |
| AC-118 | 警告 | `P(p; warn:3{w=[3]}, fail:7, last:0)` | `run warn` | 0。OUT・ERR 空。ステート `warn=warning(3)` | `binary_basic::警告は 0 スキップは 0 エラーはジョブの値`、`flow_run::警告は 0 で終わり記録される` |
| AC-119 | 実行済みのスキップ | 上のあと | `run warn` | 0。OUT・ERR 空。ステートは変わらない | 同上、`flow_run::実行済みのスキップは 0 で書き込まない` |
| AC-120 | エラーはジョブの終了コード | 上のあと | `run fail` | 7。OUT・ERR 空。ステート `S[warn=warning(3), fail=error(7), last=pending ; resume=fail]` | 同上、`flow_run::エラーはジョブの終了コードで終わり再開位置が残る` |
| AC-121 | エラーのあとの再実行 | 上のあと、定義の `fail` を `fail:0` に直す | `run fail` | 0。`fail=success(0)`。続けて `run last` → 0、`completed=true` | `flow_run::エラーからの再開は そのジョブを再実行する`、`runtime::flow_with_state::ac4_3_1` |
| AC-122 | 自動の再実行はない | `P(p; a:7)` | `run a` | 7。ジョブは 1 回だけ実行(`TJ run` で確認) | `runtime::flow_with_state::ac4_3_4 自動の再実行はない` |
| AC-123 | 順序外は拒否(66) | `P(p; a:0, b:0, c:0)`、ステートなし | `run b` | 66。ERR = `conductor: error: job 'b': cannot run this job now: earlier jobs have not finished (out of order)`。OUT 空。ステートを作らない | `flow_run::順序外は 66 で ステートを書かない`、`runtime::flow_with_state::順序外の呼び出しは拒否されステートを作らない` |
| AC-124 | 順序外でステートは変わらない | `S[a=success(0), b=error(1), c=pending ; resume=b]` | `run c` | 66。ステートは変わらない | `flow_run::順序外は ステートがあっても変えない` |
| AC-125 | 知らないジョブID | `B0` | `run nojob` | 68。ERR の最初の行 = `conductor: error: job is not defined in the definition file: 'nojob'`。続けて空行と使い方。OUT 空。ステートなし | `flow_run::知らないジョブ id は 68` |
| AC-126 | 定義の誤りは知らないジョブIDより優先 | 定義に誤りあり | `run nojob` | 64 | `flow_run::定義の誤りは 知らないジョブ id より優先する` |
| AC-127 | ジョブIDなし | `B0` | `run` | 68。`missing argument` | `binary_basic::使い方の誤りは 68…` |
| AC-128 | 既定の定義ファイル | `T/conductor.toml` | `run a`(`--config` なし) | 定義を読む。ステートは `T/conductor.toml.state` | `flow_run::config を省くと 実行ディレクトリの conductor.toml を読む` |
| AC-129 | `--config` の場所 | `T/sub/x.toml` | `run a --config sub/x.toml` | ステートは `T/sub/x.toml.state` | `core::definition_valid`、`runtime::state_read::定義ファイルと同じディレクトリのステートを読む` |
| AC-130 | -v: 成功 | `cmd="TJ say hello ''"` | `run -v a` | 0。OUT = `hello`。ERR = `[conductor] job=a result=success (exit code 0)` | `binary_basic::v は標準エラーだけに出し 標準出力はジョブのまま` |
| AC-131 | -v: 警告の遷移 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run -v a` | 0。OUT 空。ERR = `[conductor] job=a result=warning (exit code 1)`⏎`[conductor] transition: warning -> c (skipped: b)` | `flow_run::v はジョブの結果と遷移を標準エラーに表示する` |
| AC-132 | -v: `@end` への遷移 | `P(p; a:0{on=success->@end}, b:0)` | `run -v a` | 0。ERR = `[conductor] job=a result=success (exit code 0)`⏎`[conductor] transition: success -> @end`。ステート `end=true`、`completed=false` | `flow_run::v は end への遷移を表示する` |
| AC-133 | -v: 日本語 | `LANG=ja_JP.UTF-8`、`P(p; a:0, b:0)` | `run -v a`、続けて `run -v a` | ERR = `[conductor] job=a result=success (終了コード 0)`。2 回目に `[conductor] job=a スキップ (実行済み)` | `flow_run::v は日本語のロケールで日本語になる` |
| AC-134 | -v: スキップの行 | `P(p; a:0, b:0)`、`a` を実行済み | `run -v a` | 0。ERR = `[conductor] job=a skipped (already executed)` | `flow_run::スキップの記録は書き込む`(派生) |
| AC-135 | `@end` のあとの個別実行 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `run a`、`run -v b`、`run -v c` | すべて 0。`b`・`c` は実行されない。`run b` の ERR = `[conductor] job=b skipped (skipped after @end)`。ステート: `b=skipped/byEnd`、`c=skipped/byEnd`、`end=true`。最後に `completed=true` | `runtime::flow_with_state::ac4_2_3 end のあとの個別実行は最後のジョブのスキップで初めて完了する` |
| AC-136 | 遷移でスキップされたジョブの個別実行 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run a`、`run -v b` | `run a` のあと `b=skipped/byTransition`。`run b` は 0、`b` は実行されず、ERR = `[conductor] job=b skipped (skipped by a transition)` | `runtime::flow_with_state::遷移によるスキップのあと最後のジョブの実行で完了する`(派生) |
| AC-137 | 成功コードに含まれない 0 | `P(p; a:0{s=[5]})` | `run a` | 1。ERR = `conductor: error: job 'a': exit code 0 is not listed in success_exit_codes`(-v なしでも出る)。ステート `a=error(0)`、`resume=a` | `flow_run::成功コードに含まれない 0 は 1 で原因を常に表示する`、`runtime::flow_with_state::ac2_1_6` |
| AC-138 | 成功コードに含まれない 0(-v) | 同上 | `run -v a` | 1。ERR = `[conductor] job=a result=error (exit code 0, conductor exits with 1)`⏎`conductor: error: job 'a': exit code 0 is not listed in success_exit_codes` | `report_messages`(08 章 2.1 節の例) |
| AC-139 | 成功コードの指定 | `P(p; a:5{s=[5]})`、`P(p; a:0{s=[0,2]})`、`P(p; a:2{s=[2]})` | `run a` | いずれも 0(成功) | `core::flow_sequences::成功コードを書いたジョブの 0 はエラーで conductor のエラー`(派生) |
| AC-140 | シグナルで終了したジョブ(UNIX 系) | `argv=["sh", "-c", "kill -9 $$"]` | `run -v a` | 137。ERR = `[conductor] job=a result=error (terminated by signal 9)`。ステート `a=error`(終了コードなし)、`resume=a` | `flow_run::シグナルで終了したジョブは 128 足す番号`、`flow_aux`(08 章の例) |
| AC-141 | 完了のステートが残っていても先頭から | `P(p; a:0, b:0)`。`run a`、`run b`(完了)のあと | `run a`(`TJ run` で記録) | 0。`a` が再び実行される。`completed` のステートは上書きされる | `flow_run::fr1_3 run の完了のステートが残っていても再実行は先頭から始まる` |
| AC-142 | 完了のあと先頭以外は拒否 | 上の完了のステート | `run b` | 66 | `flow_pipeline::fr1_3 完了のステートが残っていても 先頭以外のジョブの run は 66` |
| AC-143 | `--remove-on-complete`(run) | `P(p; a:0)` | `run a --remove-on-complete` | 0。OUT・ERR 空。`T/conductor.toml.state` がない | `binary_basic::fr2_1 run の remove_on_complete も完了でファイルを消す` |
| AC-144 | 定義の `remove_on_complete` | `P(p remove_on_complete=true; a:0)` | `run a` | 0。ステートがない | `flow_run::fr2_2 定義の remove_on_complete が true なら run でも消える` |
| AC-145 | 完了でなければ消さない | `P(p; a:0, b:0)` | `run a --remove-on-complete` | 0。ステートが残る(`a=success`、`completed=false`) | `flow_run::fr2_1…`(派生) |
| AC-146 | ステートの形(エラーで停止)(v0.10.0 で変更) | 定義 `c2.toml`(`P(n2; a:0, b:3, c:0)`) | `pipeline n2 --config c2.toml` | 3。`T/c2.toml.state` の内容は TOML で、`completed=false`、`definition_key="c2.toml"`、`definition_path="c2.toml"`、`end_reached=false`、`format_version=1`、`job_order=["a","b","c"]`、`pipeline_name="n2"`、`resume_job_id="b"`、`[[job]]` は `a=success exit_code=0`、`b=error exit_code=3`、`c=pending`(`run_id` キーなし)。`a` と `b` は `[[job.attempt]]` を 1 つずつ持ち、どちらも `started_at` と `ended_at` が UTC の `YYYY-MM-DDTHH:MM:SSZ` の形で、実行の前後に取った時刻のあいだにあり、始まり ≦ 終わり。`c` は `attempt` のキーを持たない | `runtime::state_write::ステートファイルの内容は版つきで e3 のキーを持つ`、`binary_basic::u6_fr6_1 ステートに試行の始まりと終わりの時刻が書かれる`、`runtime::state_write::u6 試行が無ければ今と同じ形で attempt を書かない`、06 章 4.2 節 |
| AC-147 | ステートの書式(v0.10.0 で変更) | 上のステート | (ファイルを読む) | キーは各テーブル内でアルファベット順、最上位が先で `[[job]]` が続き、各 `[[job]]` の前に空行。各 `[[job]]` の中は、スカラーのキーのあとに `[[job.attempt]]` を古い順に続け(中のキーは `ended_at`、`started_at` の順)、各 `[[job.attempt]]` の前に空行 | 06 章 4.1 節(派生) |
| AC-148 | ステートに機微な値を書かない | `cmd="TJ say SECRET-OUT '' 3"`(終了コード 3 のエラーになる)。環境変数 `SECRET_ENV=SECRET-ENVVAL` を設定 | `run a` | ステートファイル(終了コード 3、`resume=a`)に、`command` の文字列、`SECRET-OUT`、`SECRET-ENVVAL` が現れない | `runtime::nfr_content` |
| AC-149 | 初期ステートの書き込みに失敗(UNIX 系、非 root) | `T` を書き込み不可にする(定義は読める) | `run a` | 65。ERR に `cannot write state file` と `(the job was not started)`。ジョブは実行されない | `runtime::flow_with_state::書けないディレクトリでは最初のジョブの前に止まる`、`flow_run::初期ステートが書けないときは 65 でジョブを起動しない` |
| AC-150 | ジョブのあとの書き込みの失敗(UNIX 系、非 root) | 定義 `argv=["sh", "-c", "chmod a-w . ; exit 7"]`(ジョブが `T` を書き込み不可にする) | `run a` | 65(ジョブの 7 ではない)。ERR に `cannot write state file` と `(the job has already been executed)` | `flow_run::ジョブのあとの書き込みの失敗は 65 でジョブは実行済み`(派生の再現) |
| AC-151 | 定義の誤り(ファイルなし・誤り) | 定義なし / `format_version = 1` だけ | `run a` | 64。ERR は `conductor: error: ` で始まる。OUT 空 | `binary_basic::定義の誤りは 64`、`flow_run::定義ファイルがないのも 64` |

### 4.4 `pipeline`

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-152 | 全部成功: 記述順に実行 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 0"})`(2 ジョブ) | `pipeline p` | 0。OUT・ERR 空。`O` = `ran=a`⏎`ran=b`。ステート `S[a=success(0), b=success(0) ; completed=true]` が残る | `binary_basic::pipeline は最後まで実行して 完了のステートを残し…`、`flow_pipeline::fr1_1` |
| AC-153 | 完了のステートの形 | 上のあと | (ステートを読む) | `resume_job_id` なし、`completed=true`、`end_reached=false`。形式は変わらない | `flow_pipeline::fr1_2 完了のステートの形式は今までどおり` |
| AC-154 | 再実行は先頭から | 上のあと | `pipeline p` | 0。`a`、`b` が再び実行される(`O` に 2 組) | `flow_pipeline::fr1_3`、`binary_basic` |
| AC-155 | 警告の遷移で間をスキップ | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)` | `pipeline -v nightly --run-id r1` | 0。ERR = `[conductor] job=extract result=warning (exit code 1)`⏎`[conductor] transition: warning -> load (skipped: transform)`⏎`[conductor] job=transform skipped (skipped by a transition)`⏎`[conductor] job=load result=success (exit code 0)`。`transform` は実行されない | `flow_pipeline::警告の遷移で間のジョブをスキップする`、08 章 2.1 節の例 |
| AC-156 | `@end` で完了 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `pipeline p` | 0。`b`・`c` は実行されない。ステート `S[a=success(0), b=skipped/byEnd, c=skipped/byEnd ; end=true, completed=true]` | `flow_pipeline::fr1_1 end で完了して 完了のステートを残す`、`runtime::flow_with_state::ac4_2_4` |
| AC-157 | エラーで停止 | `P(p; a:0, b:7, c:0)`(各ジョブは `TJ run` 形式で記録) | `pipeline p` | 7。`c` は実行されない。ステート `S[a=success(0), b=error(7), c=pending ; resume=b]` | `flow_pipeline::エラーで停止し 再試行せず ジョブの終了コードで終わる` |
| AC-158 | エラーからの再開 | 上のあと、定義の `b` を `b:0` に直す | `pipeline -v p` | 0。`a` は再実行されない(ERR に `[conductor] job=a skipped (already executed)`)。`b`・`c` が実行される。完了のステート | `flow_pipeline::エラーからの再開は 成功済みを飛ばして エラーのジョブから`、`runtime::flow_with_state::ac4_3_1_ac4_3_2` |
| AC-159 | 個別実行の続きから一括実行 | `P(p; a:0, b:0, c:0)`。`run a` のあと | `pipeline p` | 0。`a` は再実行されない。`b`・`c` が実行される | `flow_pipeline::個別実行の続きから一括実行できる` |
| AC-160 | 起動できないジョブで停止 | `P(p; a:0, b:0, c:0)` の `b` を存在しないプログラムに | `pipeline p` | 127。`c` は実行されない。ERR に `cannot start command` | `flow_pipeline::起動できないジョブで止まる` |
| AC-161 | パイプライン名の不一致 | `B0` | `pipeline wrongname` | 64。`pipeline name does not match the definition`。ジョブは実行されない | `flow_pipeline::パイプライン名の不一致は 64` |
| AC-162 | パイプライン名なし | `B0` | `pipeline` | 68。`missing argument` | `flow_pipeline::pipeline は引数のパイプライン名がないと 68` |
| AC-163 | `--remove-on-complete` | `P(p; a:0, b:0)` | `pipeline p --remove-on-complete` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state (nothing has run yet)` | `binary_basic::fr2_1 pipeline の remove_on_complete は完了でファイルを消す` |
| AC-164 | 定義の `remove_on_complete = true` | `P(p remove_on_complete=true; a:0, b:0)` | `pipeline p` | 0。ステートがない | `flow_pipeline::fr2_2` |
| AC-165 | オプションと定義の組み合わせ | `remove_on_complete=false` の定義 | `pipeline p --remove-on-complete` | ステートがない(オプションが優先して消す) | `flow_pipeline::fr2_3` |
| AC-166 | 完了でないとき消さない | `P(p remove_on_complete=true; a:0, b:7)` | `pipeline p` | 7。ステートが残る(エラーで停止) | `flow_pipeline::fr2_1…`(派生) |
| AC-167 | 完了のあと先頭以外の run は 66 | 完了のステート | `run b` | 66 | `flow_pipeline::fr1_3 完了のステートが残っていても 先頭以外のジョブの run は 66` |
| AC-168 | 初期ステートが書けない(UNIX 系、非 root) | `T` を書き込み不可 | `pipeline p` | 65。ジョブは起動されない | `flow_pipeline::初期ステートが書けないときは 65 でジョブを起動しない` |
| AC-169 | 完了のステートが残るとき定義の変更は照合されない | `pipeline p` で完了 → 定義のジョブIDを変える | `pipeline p` | 0(完了のステートは「空」として扱う。65 にならない) | `runtime::flow_with_state::完了のまま残ったステートは先頭から再実行できる` |

### 4.5 `watch`

(v0.10.0 以降)表は、所要時間の列(`DURATION` / `所要時間`)を含む 4 列(08 の 6 章)。下の期待の `<D>` は、そのジョブの所要時間(短いジョブなので `0s` のような `<秒>s` の形)を、右に空白を足して所要時間の列の幅(見出しの 8 文字)にそろえたもの。所要時間の値の規則は 4.26 節。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-170 | 表の形(英語)(v0.10.0 で変更) | `P(p; a:0, b:7, c:0)` で `pipeline p`(7 で終わる)のあと | `watch` | 終了 0。OUT = `Pipeline: p`⏎`JOB  STATUS   DURATION  NOTE`⏎`a    success  <D>  exit code 0`⏎`b    error    <D>  exit code 7`⏎`c    pending`(行末の空白なし。`c` は動いていないので所要時間も補足も空)。ERR 空 | `flow_aux::watch は表を標準出力に 1 回出す`、`report_messages::watch の表は 4 列で 状態は文字で書く`、`binary_basic::u6_fr6_5 watch の表に所要時間の列がある` |
| AC-171 | ステートなし | ステートなし | `watch` | 0。OUT = `no state (nothing has run yet)`(1 行) | `flow_aux::watch はステートが空なら空であることを表示して 0` |
| AC-172 | ステートなし(日本語) | `LANG=ja_JP.UTF-8` | `watch` | 0。OUT = `ステートはありません(まだ実行されていません)` | `report_messages::watch の空の表示` |
| AC-173 | 長さ 0 のファイルも「なし」 | 長さ 0 のステートファイル | `watch` | 同上(`no state …`) | `flow_aux::fr3_3 no state はファイルなしと長さ 0 のときだけ` |
| AC-174 | 完了の行(v0.10.0 で変更) | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)`、`pipeline nightly --run-id r1` のあと | `watch --run-id r1` | OUT = `Pipeline: nightly`⏎`Run ID: r1`⏎`JOB        STATUS   DURATION  NOTE`⏎`extract    warning  <D>  exit code 1`⏎`transform  skipped            skipped by a transition`⏎`load       success  <D>  exit code 0`⏎`completed`(スキップの行は所要時間が空で、状態の語のあとに区切り・列の幅・区切りの 12 空白) | `flow_aux::fr3_1 watch は完了のステートの表の下に完了の行を出して 0`、`flow_aux::fr3_2 watch は run id ありの完了のステートにも完了の行を出す`、08 章 6.3 節 |
| AC-175 | 日本語の表(v0.10.0 で変更) | 同上、`LANG=ja_JP.UTF-8` | `watch --run-id r1` | OUT = `パイプライン: nightly`⏎`Run ID: r1`⏎`ジョブ     状態     所要時間  補足`⏎`extract    warning  <D>  終了コード 1`⏎`transform  skipped            遷移によりスキップ`⏎`load       success  <D>  終了コード 0`⏎`完了`(状態の語は英語のまま、`Run ID` は日本語でも `Run ID`。`所要時間` は表示幅 8 で、列の幅と同じ) | `flow_aux::fr3_1 watch の完了の行は日本語では完了`、`report_messages::watch の表は日本語の見出しと補足`、`report_messages::u6_br4_1 watch の所要時間の見出しは日本語では所要時間` |
| AC-176 | `@end` のスキップの補足 | `@end` で完了したステート(4.4 節) | `watch` | `b`、`c` の行の NOTE = `skipped after @end` | `report_messages` |
| AC-177 | 定義ファイルを読まない | 定義ファイルがない、または壊れている。ステートあり | `watch --config missing.toml` | 0。ステートが表示される(定義がなくても動く) | `binary_basic::watch と reset は定義ファイルがなくても動く`、`flow_aux::watch は定義ファイルを読まず検証しない` |
| AC-178 | ステートなし・定義なしでも 0 | 定義ファイルなし | `watch --config missing.toml` | 0。OUT に `no state` | `binary_basic::watch と reset は定義ファイルがなくても動く` |
| AC-179 | 壊れたステート | `garbage = = =` | `watch` | 65。ERR = `conductor: error: conductor.toml.state: state file is corrupt: TOML として読めません; run `conductor reset` to discard the state`(`` ` `` はバッククォート)。OUT 空 | `flow_aux::watch のステートの誤りは 65`、`flow_aux::ステートの読み込みの失敗の種類ごとに 65` |
| AC-180 | 壊れた完了のステート | 完了のステートだが `run_id` がファイル名と食い違う | `watch` | 65 | `flow_aux::fr3_4 watch は壊れた完了のステートを 65 にする` |
| AC-181 | ファイルへの出力に制御文字がない | `P(p; a:0, b:0)`、`run a` のあと | `watch`(標準出力をファイルに) | 0。ファイルに `\x1b` がなく、`a`、`success`、`pending` を含む | `binary_basic::watch をファイルに出力しても 制御文字がない`、`flow_aux::watch のファイルへの出力には制御文字がない` |
| AC-182 | 端末では状態に色(UNIX 系。疑似端末) | 同上 | 標準出力を端末にして `watch` | `success` が `\x1b[32m` と `\x1b[0m` で囲まれる(`warning` 33、`error` 31、`skipped` 36、`pending` は色なし)。列の幅は色の制御文字を除いた文字で決まる | `flow_aux::watch は端末なら色をつける`、`report_messages::色は端末で…`。区分: UNIX 系(疑似端末) |
| AC-183 | `NO_COLOR` で色なし | 同上、`NO_COLOR=1` | 同上 | 制御文字なし。空の `NO_COLOR` では色あり | `report_messages::色は端末で no_color が無いか空で unix のときだけ`。区分: UNIX 系(疑似端末) |
| AC-184 | Windows は色なし | Windows | `watch`(端末) | 制御文字なし | `report_messages`(08 章 6.3 節)。区分: Windows |
| AC-185 | ジョブIDの制御文字は表の中でもエスケープ | ジョブID に ESC を含む(TOML で `"a\u001bb"`)。実行済み | `watch` | 行に `a\u{1b}b`。生の ESC がない | `report_messages::ジョブ id の制御文字は表の中でもエスケープされる` |
| AC-186 | 標準出力に書けない(注入) | ステートあり | `watch`(標準出力を書き込み失敗にする) | 65。ERR = `conductor: error: cannot write to standard output`。Linux では `watch > /dev/full` でも起こせる(参照実装の自動テストは差し替えで検査。実バイナリでは未確認) | `flow_aux::watch の標準出力が閉じていたら 65`。区分: 注入 |
| AC-187 | 使えないオプション | なし | `watch -v` / `watch --remove-on-complete` / `watch --all` / `watch extra` | 68。ERR = `option not allowed for this command: '-v'`(それぞれ対象は `-v`、`--remove-on-complete`、`--all`)。`watch extra` は `unexpected argument: 'extra'` | `binary_basic::fr2_5 remove_on_complete は watch で 68`、`flow_aux::fr2_5`、`flow_aux::v は watch と reset と convert に付けると 68` |

### 4.6 `reset`

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-188 | ステートを消す | `P(p; a:0, b:7)`、`pipeline p` のあと | `reset` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state …` | `flow_aux::reset はステートを消す 何も表示しない` |
| AC-189 | 先頭から実行できる | 上のあと、`P(p; a:0, b:0)` に直す | `run a`(`TJ run` で記録) | 0。`a` が実行される | `flow_aux::reset のあとは先頭から実行できる`、`runtime::flow_with_state::ac4_3_3` |
| AC-190 | ステートがなくても成功 | ステートなし | `reset` | 0。OUT・ERR 空 | `flow_aux::reset はステートがなくても成功する` |
| AC-191 | 定義がなくても動く | 定義なし | `reset --config missing.toml` | 0 | `binary_basic::watch と reset は定義ファイルがなくても動く` |
| AC-192 | 壊れたステート・壊れた定義を迂回 | 壊れたステートと壊れた定義 | `reset` | 0。ステートが消える | `flow_aux::reset は壊れたステートと定義の誤りを迂回する` |
| AC-193 | 完了のステートも消す | 完了のステート(run ID あり・なし) | `reset`、`reset --run-id r1` | 0。消える | `flow_aux::fr4_1 reset は完了のステートも消す` |
| AC-194 | `--run-id` は 1 つだけ消す | `conductor.toml.state`、`.r1.state`、`.r2.state` がある | `reset --run-id r1` | `.r1.state` だけ消える。ほかは変わらない | `flow_run_id::reset の run id の指定は別の run id とステートに影響しない` |
| AC-195 | `--all` で全部消す | `P(p; a:0, b:7)`。`pipeline p`(run ID なし)、`--run-id a`、`--run-id b` で実行。`T` に、別の定義のステート(`conductor.toml.a.b.state`。中身はパイプライン名 `p`・ジョブ `a` の正常なステートで、`run_id` を持たない)、壊れたもの(`conductor.toml.c.state` = `garbage = = =`)、一時ファイル(`.conductor.toml.state.tmp.1.2.3`)、無関係なファイル(`conductor.toml.bak`、`other.toml.state`)を置く | `reset --all` | 終了 0。OUT 空。ERR は `conductor: warning: ` で始まり、`conductor.toml.c.state` を含み、`conductor.toml.a.b.state` を含まない。残るファイル: `.conductor.toml.state.tmp.1.2.3`、`conductor.toml`、`conductor.toml.a.b.state`、`conductor.toml.bak`、`conductor.toml.c.state`、`other.toml.state`。run ID なしと `.a.state`、`.b.state` は消える | `binary_run_id::reset all は定義のステートだけを消し…` |
| AC-196 | 警告の文言 | 上と同じ | `reset --all` | ERR = ``conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):``⏎`  conductor.toml.c.state`(ファイル名の行は先頭に空白 2 文字) | `flow_run_id::reset all は残したファイルを警告に出し 終了コードは 0`、08 章 4.10 節 |
| AC-197 | 壊れたものを個別に消す | 上のあと | `reset --run-id c` | 0。`conductor.toml.c.state` が消える | `binary_run_id::reset all は…` |
| AC-198 | 何もなくても 0 で出力なし | `T` に定義だけ | `reset --all` | 0。OUT・ERR 空 | `binary_run_id::reset all は何もなくても 0 で何も出力しない` |
| AC-199 | `--all` と `--run-id` の併用 | なし | `reset --all --run-id x` | 68。`invalid value: '--all'` | `binary_run_id::reset all は何もなくても…`、`args_grammar::all と run_id の併用と重複と値の欠落は誤り` |
| AC-200 | `--all` は環境変数を見ない | `CONDUCTOR_RUN_ID='bad id'` | `reset --all` | 0 | `flow_run_id::reset all は何も出力せず 0 で環境変数の値を見ない` |
| AC-201 | シンボリックリンクと別の定義は消さない(UNIX 系) | `conductor.toml.s.state` が通常のファイルへのシンボリックリンク。ディレクトリ `conductor.toml.d.state`。`conductor.toml.a.state` の中身の `run_id` がファイル名と食い違う | `reset --all` | リンク先のファイルは残る。リンクとディレクトリと、中身の `run_id` が食い違うファイルは消されない | `runtime::run_id_state::reset all はシンボリックリンクを辿らず消さない / reset all は別の定義のステートで run id のないものを消さない` |
| AC-202 | run ID なしのステートは中身を読まず消す | `conductor.toml.state` が壊れている(通常のファイル) | `reset --all` | 消える。警告なし | `runtime::run_id_state::reset all は対象だけを消し…`(06 章 7.4 節) |
| AC-203 | 環境変数の値の違反 | `CONDUCTOR_RUN_ID='bad id'` | `reset` | 68。`invalid value: 'run ID (environment variable)'` | `flow_run_id::reset と watch の環境変数の値の違反は 68` |
| AC-204 | 引数の誤り | なし | `reset extra`、`reset -v`、`reset --remove-on-complete` | 68。`unexpected argument: 'extra'`、`option not allowed for this command: '-v'`、同 `'--remove-on-complete'` | `flow_aux::fr2_5`、`args_grammar::watch と reset は位置引数を受け取らない` |
| AC-205 | 削除の失敗(UNIX 系、非 root) | `T/conductor.toml.state` が空でないディレクトリ | `reset` | 65。ERR に `cannot remove state file`。OUT 空 | `flow_aux::reset の失敗は 65`、`runtime::state_write::削除に失敗したら remove_failed`(派生の再現) |

### 4.7 `convert`

変換の入力 `CV1`(TOML)と `CV2`(YAML)は、次のとおり。

```toml
# CV1
format_version = 1

[pipeline]
name = "nightly"
description = "夜間バッチ(抽出、変換、読み込み、通知)"

[[job]]
id = "extract"
command = "extract --all --since yesterday"

[[job]]
id = "transform"
command = "transform \"in dir\" $HOME"
warning_exit_codes = [3, 4]
success_exit_codes = [0, 2]

[[job.transitions]]
on = "warning"
next = "notify"

[[job]]
id = "load"
command = "load"

[[job.transitions]]
on = "success"
next = "@end"

[[job]]
id = "notify"
command = "notify --channel ops"
```

```yaml
# CV2(アンカー、エイリアス、マージキーを含む)
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
defaults: &defaults
  success_exit_codes: [0, 2]
  warning_exit_codes: [3, 4]
job:
  - id: extract
    command: extract --all --since yesterday
    <<: *defaults
  - id: transform
    command: 'transform "in dir" $HOME'
    <<: *defaults
    success_exit_codes: [0]
    transitions:
      - on: warning
        next: notify
      - on: error
        next: "@end"
  - id: load
    command: load
    transitions:
      - { on: success, next: "@end" }
  - id: notify
    command: notify --channel ops
```

特に記さない限り、失敗の期待は、終了 67、OUT 空、ERR の最初の行が `conductor: error: ` で始まる。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-206 | TOML から YAML | `T/in.toml` = `CV1` | `convert in.toml --to yaml` | 終了 0。ERR 空。OUT は YAML で、`format_version: 1` と `id: extract` を含む。OUT を YAML として読んだ木が `CV1` の木と等しい。マップのキーは辞書順、インデントは 2 スペース、シーケンスは親のキーより 2 スペース深い(`job:` の次の行に `  - command: …`)。末尾は改行 | `convert_roundtrip::ac_7_1_1 toml から yaml`、`binary_basic::convert は変換結果だけを標準出力に出す` |
| AC-207 | YAML から TOML(アンカー展開) | `T/in.yaml` = `CV2` | `convert in.yaml --to toml` | 0。OUT に `<<` がなく、`[[job]]` を含む。OUT を TOML として読んだ木が、`CV2` を展開した木と等しい(`transform` の `success_exit_codes` は `[0]`、`extract` は `[0, 2]`) | `convert_roundtrip::ac_7_1_2 アンカーつきの yaml から toml` |
| AC-208 | 変換の出力は標準出力だけ | `T/in.yaml` = `format_version: 1⏎pipeline:⏎  name: p⏎job:⏎  - id: a⏎    command: echo a` | `convert in.yaml --to toml` | 0。ERR 空。OUT に `[[job]]` と `format_version = 1` | `binary_basic::convert は変換結果だけを標準出力に出す` |
| AC-209 | 入力形式は `--to` の反対で、拡張子は見ない | `T/looks-like.toml` = `a: 1⏎b: [x, y]` | `convert looks-like.toml --to toml` / `convert looks-like.toml --to yaml` | 前者: 0、OUT に `a = 1`。後者: 67(TOML として読んで `syntax error`) | `convert_ac::入力の形式は変換後の反対で 拡張子は見ない` |
| AC-210 | 文法の誤りは種類と行つき | `T/broken.yaml` = `a: [unclosed` | `convert broken.yaml --to toml` | 67。OUT 空。ERR の最初の行が `conductor: error: broken.yaml:1: syntax error: YAML として読めません`(v0.9.6 までは、理由が `文法の誤りです`) | `binary_basic::convert は…失敗: 標準出力は空 67`、`convert_ac::ac_7_1_3` |
| AC-211 | 入力がない | なし | `convert nofile --to toml` | 67。ERR = `conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません` | `flow_aux::convert の入力がないときも 67` |
| AC-212 | ディレクトリを入力に | `T/d` がディレクトリ | `convert d --to toml` | 67。`cannot read input file` | `convert_ac::変換はファイルの失敗を決まった順で報告する` |
| AC-212a | 書き手のいない名前付きパイプを入力に(UNIX 系。v0.9.8 以降) | `T/pipe.toml` が名前付きパイプ(`mkfifo`) | `convert pipe.toml --to yaml` | 止まらずに 67。`cannot read input file`(理由 `通常のファイルではありません`。開かない)。v0.9.7 までは、開くところで止まった | `convert_ac::名前付きパイプの入力は開かずに unreadable` |
| AC-213 | 入力が大きすぎる | 1,048,577 バイト(中身が壊れていてもよい) | `convert big.yaml --to toml` | 67。`input is too large` | `convert_ac::変換はファイルの失敗を決まった順で報告する` |
| AC-214 | ちょうど 1 MiB は受理 | `a: "xxx…"` で、ちょうど 1,048,576 バイト | `convert max.yaml --to toml` | 0。OUT が `a = "xxx` で始まる | `convert_ac::ちょうど1_mibの正しい入力は変換できる` |
| AC-215 | UTF-8 でない入力 | `a: ` のあとに不正なバイト列 | `convert bad.yaml --to toml` | 67。`cannot read input file`(文法を見る前) | `convert_ac::変換はファイルの失敗を決まった順で報告する` |
| AC-216 | 表せない値は場所つき | `T/in.yaml` = `job:⏎  - id: x⏎  - id: y⏎    note: ~` | `convert in.yaml --to toml` | 67。ERR に `value cannot be represented in the target format: null は表せません` と `(at: job[1].note)` | `flow_aux::convert は表せない値を場所つきで表示する`、`read_resolve::入力の順で最初の表せない値を キーの場所つきで報告する` |
| AC-217 | 表せない値の種類 | YAML: `a: ~` / `a: null` / `a:` / `a: !!binary aGk=` / `a: !mytag 1` / `a: 9223372036854775808` / `a: 1e999` / `1: x` / `true: x`。TOML: `d = 1979-05-27T07:32:00Z` / `d = 1979-05-27` / `t = 07:32:00` | `convert … --to <変換後>` | いずれも 67。原因の文は、順に `null は表せません`(3 つ)、`タグつきの値は表せません`(2 つ)、`数値が表せる範囲を超えています`(2 つ)、`文字列でないキーは表せません`(2 つ)、`日時の値は表せません`(3 つ) | `read_resolve::null の書き方と ほかの表せない種類 / toml の日時は表せない` |
| AC-218 | 複数の文書 | `a: 1⏎---⏎b: 2` | `convert in.yaml --to toml` | 67。`value cannot be represented in the target format: 複数の文書は表せません` | `read_resolve::複数の文書は表せない` |
| AC-219 | 最上位がマップでない | `- 1⏎- 2` | `convert in.yaml --to toml` | 67。`top level is not a map: 最上位がマップではありません`(行 1) | `read_resolve::最上位はマップでなければならない` |
| AC-220 | 空の YAML | 空ファイル(または `# c` だけ) | `convert in.yaml --to toml` | 67。`top level is not a map: 文書が空です` | `read_resolve`(09 章 5.3 節) |
| AC-221 | コメントは捨てられる | `# 先頭⏎a: 1 # 末尾` | `convert in.yaml --to toml` | 0。OUT = `a = 1`。OUT に `#` がない | `convert_ac::コメントつきの入力は変換でき コメントは捨てる` |
| AC-222 | キーの順序は辞書順 | `b: 1⏎a: 2` | `convert in.yaml --to toml` | OUT = `a = 2`⏎`b = 1`(入力の順を保たない) | 09 章 8.1 節(`convert_ac::出力は毎回同じ`) |
| AC-223 | 空のマップ | `{}`(YAML)、空(TOML) | `convert in.yaml --to toml` / `convert in.toml --to yaml` | 0。TOML の出力は空文字列。YAML の出力は `{}` | 09 章 9 節 |
| AC-224 | アンカーとマージキー | `base: &b {x: 1, y: 2}⏎d:⏎  <<: *b⏎  y: 9` | `convert in.yaml --to toml` | 0。`d` は `x = 1`、`y = 9`(自身のキーが優先) | `read_resolve::マージキーの単独と並びと優先順位` |
| AC-225 | マージキーの値がマップでない | `a:⏎  <<: 5` | `convert in.yaml --to toml` | 67。`マージキーの値がマップではありません`(`(at: a.<<)`) | `read_resolve::値がマップでないマージキーは表せない` |
| AC-226 | 重複するキー | `a: 1⏎b: 2⏎a: 3` | `convert in.yaml --to toml` | 67。`syntax error: キーが重複しています`(行 3) | `read_resolve`(09 章 5.1 節) |
| AC-227 | 再帰するアンカー | `a: &r [*r]` | `convert in.yaml --to toml` | 67。`アンカーが自分自身を含んでいます` | `read_resolve`(09 章 5.1 節) |
| AC-228 | スカラーの解決 | YAML: `a: yes`、`b: 0x1F`、`c: 007`、`d: '1'`、`e: 2001-01-01`、`f: 1_000` | `convert in.yaml --to toml` | `a = "yes"`、`b = 31`、`c = 7`、`d = "1"`、`e = "2001-01-01"`、`f = "1_000"`(キーは辞書順に並ぶ)。真偽値になるのは `true`/`false` の 3 通りの書き方だけ | `read_resolve::コアスキーマのスカラー`(09 章 5.2 節) |
| AC-229 | 展開後の要素数の上限 | `a: [0, 0, …]`(0 を n 個) | `convert in.yaml --to toml` | n = 99,997 は 0。n = 99,998 は 67(`input is too large`。原因 `展開後の要素数が上限(100000)を超えています`)。TOML 入力(`a = [0, 0, …]`)も同じ境界 | `read_resolve::要素数ちょうど100000は受理し 1つ多いと大きすぎる / toml の入力も要素数の上限は同じ` |
| AC-230 | アンカーの爆発は上限で止まる | アンカーの入れ子で、指数的に増える YAML | `convert in.yaml --to toml` | 30 秒以内に 67(`input is too large`)。メモリを使い果たさない | `read_resolve::エイリアスの爆発は メモリを使い果たさずに大きすぎる` |
| AC-231 | 入れ子の深さの上限 | YAML: `a: ` に続けて `[` を 31 個 / 32 個(閉じも書く) | `convert in.yaml --to toml` | 31 個: 0。32 個: 67(`syntax error: 入れ子が深すぎます(上限 32)`) | `read_resolve::読み取りの深さの上限 32 と 33` |
| AC-232 | 極端な深さでも異常終了しない | 1,000 段、100,000 段の入れ子(YAML・TOML) | `convert … ` | 67(`syntax error`)。異常終了しない | `read_resolve::極端に深い入力は 異常終了せずに文法の誤りで失敗する` |
| AC-233 | 往復で木が等しい | `CV1` | `convert in.toml --to yaml` の OUT を `convert --to toml` に通す | 結果の木が `CV1` の木と等しい。`command` が配列形式の定義、`[job.retry]` の表も同じ | `convert_roundtrip::定義の形の toml は yaml を通って往復する`、`配列形式の command は往復する *`、`retry の表は往復する *` |
| AC-234 | 数値の往復 | 整数の端(`i64` の最小・最大)、浮動小数(`1.0`、`-0.0`、`inf`、`nan`)、空の配列・マップ | 往復 | 型が変わらない(`1` と `1.0` を区別する。`0.0` と `-0.0` は別) | `convert_roundtrip::i64 の端は往復する / 浮動小数は両方向に往復する / スカラーの境界は往復する *` |
| AC-235 | ファイルを作らず環境に左右されない | `T` に入力のみ | `convert` を、別のカレントディレクトリ・異なる環境変数で実行 | `T` のファイル一覧が実行の前後で同じ。結果が同じ | `converter::nfr_security`、`flow_aux::convert は定義もステートも読まず シグナルも準備しない` |
| AC-236 | 標準出力に書けない(注入) | 正しい入力 | `convert`(標準出力を書き込み失敗にする) | 67。ERR = `conductor: error: cannot write to standard output` | `flow_aux::convert の標準出力が閉じていたら 67`。区分: 注入 |
| AC-237 | 引数の誤り | なし | `convert f`(`--to` なし)/ `convert f --to json` / `convert f --to YAML` / `convert f --to yaml --config c` | 68。`missing argument: '--to'` / `invalid value: '--to'` / `invalid value: '--to'` / `option not allowed for this command: '--config'` | `args_grammar::to は convert だけで必須 / to の値は yaml か toml だけ / to の値の誤りの対象は to で値を含まない / config は convert に付けられない` |
| AC-238 | 変換結果の全体を 1 回で書く | 失敗する入力 | `convert` | 失敗のとき、標準出力に 1 バイトも書かれない | `flow_aux::convert の失敗は 67 で標準出力は空`、`convert は変換結果を全部 1 回で標準出力に書く` |

### 4.8 run ID

特に記さない限り、定義は `two_jobs(n)` = `P(p; a:0, b:<n>)`(`a` は成功、`b` は終了コード `n`)。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-239 | run ID ごとに別のステート | `two_jobs(7)` | `pipeline p --run-id r1` と `pipeline p --run-id r2` を同時に | どちらも 7。OUT 空。`T` のファイルは `conductor.toml`、`conductor.toml.r1.state`、`conductor.toml.r2.state` の 3 つだけ。各ステートは `run_id` がその ID、`resume=b`、`a=success` | `binary_run_id::並行する 2 つの run id は別のステートで…` |
| AC-240 | 同じ run ID で再開し、他は変わらない | 上のあと、定義を `two_jobs(0)` に直す | `pipeline p --run-id r1` | 0。`r1` は完了のステートが残る(`completed=true`)。`r2` のステートファイルはバイト列が変わらない | 同上、`flow_run_id::同じ run id で呼び直すと止まったジョブから再開する` |
| AC-241 | ステートの形(run ID あり・なし) | `two_jobs(7)` | `pipeline p --run-id r1`、`pipeline p` | `.r1.state` は `run_id = "r1"` を持つ。run ID なしの `.state` は `run_id` キーを持たない | `flow_run_id::run id ごとに別のステートのファイルを読み書きする / run id なしは従来のファイル名と形のまま` |
| AC-242 | 環境変数 `CONDUCTOR_RUN_ID` | `two_jobs(7)`、`CONDUCTOR_RUN_ID=fromenv` | `pipeline p` | 7。`conductor.toml.fromenv.state` ができる | `binary_run_id::環境変数と空のオプションと定義の run_id_env はバイナリでも効く`、`flow_run_id::オプションは環境変数に優先し…` |
| AC-243 | 空のオプションは run ID なしに固定 | 同じ環境変数 | `pipeline p --run-id ""` | 7。`conductor.toml.state`(run ID なし)ができる | `flow_run_id::空の run id のオプションは環境変数があっても run id なしに固定する` |
| AC-244 | オプションは環境変数に優先 | 同じ環境変数 | `pipeline p --run-id fromopt` | `conductor.toml.fromopt.state` ができる(`fromenv` は使われない) | 同上 |
| AC-245 | 環境変数が空なら run ID なし | `CONDUCTOR_RUN_ID=` | `pipeline p` | 7。`conductor.toml.state` | `flow_run_id::環境変数が空なら run id なし` |
| AC-246 | 定義の `run_id_env` | `P(p run_id_env=MY_RUN; a:7)`、`MY_RUN=m1`、`CONDUCTOR_RUN_ID=ignored` | `run a` | 7。`conductor.toml.m1.state` ができる(`ignored` は使われない) | `binary_run_id::環境変数と空のオプションと定義の run_id_env…`、`flow_run_id::定義の run_id_env の名前の環境変数を読む` |
| AC-247 | `watch`・`reset` も `run_id_env` を読む | 上のあと、`MY_RUN=m1` | `watch`、`reset` | `watch` の OUT に `Run ID: m1`。`reset` で `conductor.toml.m1.state` が消える | `binary_run_id`、`flow_run_id::watch と reset は定義の run_id_env をゆるく読んで使う` |
| AC-248 | 有効な run ID | `two_jobs(0)` | `pipeline p --run-id <値>`(`1`、`r1`、`12345`、`build-2026.11_a`、`a.b`、`x` を 64 個) | いずれも 0 | `core`(05 章 6.1 節) `RunId::parse` |
| AC-249 | 無効な run ID(オプション) | `two_jobs(0)` | `pipeline p --run-id <値>`(`a/b`、`..`、`x` を 65 個、`a b`、`.x`、`日本語`) | 68。ERR の最初の行 = `conductor: error: invalid value: '--run-id'`。ERR にその値が現れない。ステートが作られない | `binary_run_id::無効な値は 68…`、`flow_run_id::オプションの値の違反は 68 で誤りの文に値を含めない` |
| AC-250 | 無効な run ID(環境変数) | 同じ値を `CONDUCTOR_RUN_ID` に | `pipeline p` | 68。ERR の最初の行 = `conductor: error: invalid value: 'run ID (environment variable)'`。値が現れない | `flow_run_id::環境変数の値の違反は 68 で誤りの文に値を含めない` |
| AC-251 | validate は run ID を受け付けない | `two_jobs(0)` | `validate --run-id x` | 68 | `binary_run_id::無効な値は 68…` |
| AC-252 | run ID の決定の順序 | 定義の誤り + `--run-id 'a b'` | `run a --run-id 'a b'` | 64(定義の検査が先。run ID は後) | `flow_run_id::run id の決定は定義の検査のあと ステートの読み込みの前` |
| AC-253 | run ID の決定はステートの読み込みの前 | 壊れたステート + `--run-id 'a b'` | `run a --run-id 'a b'` | 68(65 ではない) | 同上 |
| AC-254 | `watch` は `--run-id` があれば定義を読まない | 定義が壊れている。`.r1.state` あり | `watch --run-id r1` | 0。表が出る | `flow_run_id::watch は run id のオプションがあれば定義を読まない` |
| AC-255 | `watch`・`reset` は壊れた定義でも既定の環境変数名 | 定義が壊れている。`CONDUCTOR_RUN_ID=r1` | `reset` | 0。`.r1.state` だけが消える | `flow_run_id::reset は壊れた定義でも 既定の環境変数の名前で その run のステートだけを消す` |
| AC-256 | run ID なしの `watch` に `Run ID` の行はない | run ID なしのステート | `watch` | OUT に `Run ID` の行がない | `flow_run_id::watch は run id なしなら表示は変わらず run id の行を出さない` |
| AC-257 | ファイル名と中身の `run_id` の食い違い | `conductor.toml.r1.state` を `conductor.toml.r2.state` に複製する | `run b --run-id r2` | 65。ERR = ``conductor: error: conductor.toml.r2.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおり) | `flow_run_id::ファイル名と中身の run_id の食い違いは 65 で reset の案内を出す`、`runtime::run_id_state::中身の run_id がファイル名と一致しなければ corrupt` |
| AC-258 | run ID ありでも `--remove-on-complete` で消える | `two_jobs(0)` | `pipeline p --run-id r1 --remove-on-complete` | 0。`conductor.toml.r1.state` がない | `flow_run_id::fr2_1 run id ありでも remove_on_complete なら完了で消える` |
| AC-259 | `reset --run-id` は別の run に影響しない | `.r1.state`、`.r2.state`、`.state` | `reset --run-id r1` | `.r1.state` だけが消える | `flow_run_id::reset の run id の指定は別の run id とステートに影響しない` |
| AC-260 | 大文字小文字を区別しないファイルシステム | macOS の既定 / Windows | `pipeline p --run-id A`、`pipeline p --run-id a` | 同じファイルになる。食い違いは、中身の `run_id` との照合で 65 として検出される(利用者は大文字小文字だけで run ID を区別しない) | 06 章 1 節(派生)。区分: macOS / Windows。**未確認**(参照実装に専用のテストなし) |

### 4.9 リトライ

特に記さない限り、`RT` は `P(p; a:0{cmd="TJ flaky F <fails> 75", retry=(exit_codes=[75], max=3, interval="1s")})`(`F` は `T/flaky.count`。`<fails>` は失敗させる回数)。`flaky` は、実行のたびに `F` の回数を 1 増やす。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-261 | 2 回失敗して 3 回目に成功 | `RT`(`fails=2`) | `run a` | 0。OUT 空。ERR(-v なしでも)= `conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 1 seconds`⏎`conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 1 seconds`。`F` の値 = 3。ステート `a=success(0)` | `flow_retry::run 2 回目に成功すると 0 で終わり 知らせは v なしでも出る`、`docs` |
| AC-262 | -v で結果の行は 1 回だけ | 上と同じ | `run -v a` | ERR は上の 2 行のあとに `[conductor] job=a result=success (exit code 0)` の 1 行だけ(途中の失敗の結果行は出ない) | `flow_retry::run 結果の表示は 最終の結果について 1 回だけ` |
| AC-263 | 使い切る | `RT`(`fails=99`) | `run a` | 75。ERR は retry の行 3 つ(`retry 1 of 3`、`2 of 3`、`3 of 3`)。`F` の値 = 4(最大 4 回の実行)。ステート `a=error(75)`、`resume=a` | `flow_retry::run 使い切ると 最後の終了コードで終わり 再開位置はそのジョブ` |
| AC-264 | 対象外の終了コードはリトライしない | `exit_codes=[75]` で、ジョブが 3 で失敗(`flaky F 99 3`) | `run a` | 3。retry の行なし。`F` の値 = 1 | `flow_retry::run 対象外の結果は リトライしない` |
| AC-265 | 設定がなければ従来どおり | `P(p; a:0{cmd="TJ flaky F 99 75"})` | `run a` | 75。retry の行なし。`F` = 1 | `flow_retry::run リトライの設定がないジョブは 従来どおり` |
| AC-266 | 知らせは日本語でも | `RT`(`fails=1`)、`LANG=ja_JP.UTF-8` | `run a` | 0。ERR = `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。1 秒後にリトライします(1/3 回目)` | `flow_retry::run 知らせは日本語でも出る`、`report_messages::リトライの知らせの日本語の文は…` |
| AC-267 | 倍率と上限の待ち時間 | `retry=(exit_codes=[75], max=3, interval="1s", multiplier=2, max_interval="2s")`、`fails=99` | `run a` | 75。retry の行の待ち時間は順に `1`、`2`、`2`(`in 1 seconds`、`in 2 seconds`、`in 2 seconds`)。実行全体の時間は 5 秒以上 | `flow_retry::run 倍率と上限の待ち時間の列`、`core::retry_flow::倍率と上限では 10 20 40 60 60 になる` |
| AC-268 | 待ち時間の列(関数の検査) | `interval` / `multiplier` / `max_interval` / `max` | 待ち時間を求める関数 | `30s`、倍率なし、max 3 → 30、30、30。`10s`、2、`60s`、max 5 → 10、20、40、60、60。`60s`、3、`60s`、max 3 → 60、60、60。`1s`、10、`1h`、max 6 → 1、10、100、1000、3600、3600。`1h`、10、`24h`、max 4 → 3600、36000、86400、86400。極端に大きい値でも桁あふれしない(飽和) | `core::retry_flow::backoff waits / 極端に大きい値でも桁あふれしない`。区分: 単体 |
| AC-269 | 再実行の結果が成功なら遷移へ(pipeline) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(…)}, b:0)` | `pipeline p` | 0。`b` も実行される。完了のステート | `flow_retry::pipeline リトライして成功すると 次のジョブへ進み 完了する` |
| AC-270 | 使い切ると停止(pipeline) | `RT` に `b:0` を足し、`fails=99` | `pipeline p` | 75。`b` は実行されない。`resume=a` | `flow_retry::pipeline 使い切ると そこで停止し 再開位置はそのジョブ` |
| AC-271 | ジョブ自身の 127 は一覧にあればリトライ | `TJ exit 127`、`exit_codes=[127]`、`max=1`、`interval="1s"` | `run a` | 127。retry の行が 1 つ(1 回リトライしてから終わる) | `flow_retry::run ジョブ自身の 127 は 一覧にあればリトライし 起動の失敗はしない` |
| AC-272 | 起動の失敗はリトライしない | 存在しないプログラム、`exit_codes=[127]` | `run a` | 127。retry の行なし | 同上 |
| AC-273 | シグナルで終了したジョブはリトライしない(UNIX 系) | `argv=["sh","-c","kill -9 $$"]`、`exit_codes=[137]` | `run a` | 137。retry の行なし | `core::retry_flow::シグナルでの終了は停止` |
| AC-274 | 待機中のシグナル(UNIX 系) | `retry=(exit_codes=[75], max=3, interval="60s")`、`fails=99` | `run a` を起動し、ERR に最初の retry の行が出てから SIGTERM を送る | 143。ステート: `a=pending`(エラーとして記録しない)、`resume=a`。2 つ目の retry の行は出ない。ジョブは再び起動されない(`F` = 1) | `flow_retry::run 待機中のシグナルで 128 足す番号で終わり エラーとして記録しない` |
| AC-275 | 待機中の中断は 1 秒以内 | `interval="1h"`、`fails=99` | 最初の retry の行のあと SIGTERM | SIGTERM を送ってから 1 秒以内に、143 で終わる | `runtime::wait_interruptible::…`(3600 秒の待機に 200 ms 後の合図で、全体が 1.2 秒未満) |
| AC-276 | 再開は最初の実行から数え直す | 上のあと(`resume=a`) | 再度 `run a`(`interval="1s"`、`fails=1`) | `retry 1 of 3` の行から始まる。0 で完了 | `flow_retry::run 再開すると そのジョブを 最初の実行から数え直す` |
| AC-277 | 待機中のシグナル(pipeline) | 同様(pipeline) | `pipeline p`、待機中に SIGTERM | 143。以降のジョブを実行しない。`resume=a` | `flow_retry::pipeline 待機中のシグナルで 以降を実行せず 再開位置を記録する` |
| AC-278 | 結果の記録は最終の結果で 1 回だけ、試行の境目は試行ごとに書く(v0.10.0 で変更) | `RT`(`fails=2`) | `run a` | 実行の途中に、ステートファイルのジョブ `a` の状態が `success`/`error` にならない(結果の記録は最終の結果で 1 回だけ。耐久性つきの書き込み)。試行の開始のたびに(3 回)、その試行を足したステートが書かれる(同期を省く書き込み)。待機中のステートでは、`a` は `pending` で、最後の試行が終わりの時刻を持たない(開いている。`watch --follow` からは動いている最中に見える)。次の試行の開始の書き込みで、前の試行が、その試行の直後に取った終わりの時刻で閉じる(待ちの後の時刻ではない)。終わったあと、`a=success(0)` の `[[job.attempt]]` は 3 つ(古い順。すべて閉じている) | `flow_retry::run 結果は最後に 1 回だけ durable で書かれ 試行の境目は fast で書く`、`flow_retry::u6_br1_2 リトライの試行ごとに始まりと終わりを記録する`、`flow_retry::u7_br3_4 リトライの待ちの間のステートでは最後の試行が開いている`、`flow_retry::u7_br3_4 次の試行の書き込みで前の試行を本当の終わりの時刻で閉じる`。観測: 待機中にステートファイルを読む。区分: 注入(書き込みの順と時刻) |
| AC-279 | README のリトライの説明 | README | (検査) | 見出し、5 つのキー名(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)があり、載っている TOML の例が実際に読み取りを通る | `docs`。区分: 点検 |

### 4.10 シグナルと中断

UNIX 系の「実行」は、被試験体を起動し、`TJ sleep` の記録 `ready` が `O` に現れるのを待ってから、被試験体のプロセスにシグナルを送る。定義は `SL` = `P(p; sleeper:0{cmd="TJ --out O sleep"}, after:0)` を使う。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-280 | 実行中の SIGTERM(UNIX 系) | `SL` | `run sleeper` を起動し、`ready` のあと SIGTERM | 143。OUT 空。ステート `S[sleeper=error, after=pending ; resume=sleeper]`。`pid=` に記録されたジョブのプロセスが存在しない(孤児なし) | `binary_signal::ジョブの実行中の sigterm は 143 で 再開位置が残る` |
| AC-281 | HUP のあとすぐ INT(UNIX 系) | `SL`(`sleeper` だけでもよい) | `ready` のあと SIGHUP、続けて SIGINT | 129(最初の番号 = 1 が残る)。ステート `sleeper=error` | `binary_signal::続けて 2 つのシグナルを送っても 終了の処理は 1 回` |
| AC-282 | SIGINT / SIGQUIT 単独(UNIX 系) | `SL` | SIGINT / SIGQUIT | 130 / 131 | 07 章 7.2 節の例(派生。転送は `runtime::runner_signal::sigint も転送される`) |
| AC-283 | -v の表示(UNIX 系) | `SL` | `run -v sleeper`、SIGTERM | 143。ERR に `[conductor] job=sleeper result=error (interrupted by signal 15)` | 08 章 2.1 節(派生) |
| AC-284 | 時間を空けた 2 つのシグナルは最初の番号が残る | シグナル監視の単体 | INT のあと、時間を空けて HUP | 最初の番号 2 が残る | `signals_order::int のあとに hup を送ると int の番号が残る`。区分: 単体(UNIX 系) |
| AC-285 | 最初の番号・登録は 1 回 | シグナル監視の単体 | 監視を 1 回だけ登録し、続けてシグナルを受ける | 最初の番号が残る(登録はプロセスで 1 回だけ) | `signals_real::最初のシグナルの番号だけが残る`。区分: 単体(UNIX 系) |
| AC-286 | 転送は同じシグナルを 1 回だけ | 起動の部品の単体 | ジョブの実行中に合図を立てる(SIGTERM と SIGINT で別々に) | ジョブは、そのシグナルで終了する(`terminated_by_signal` = 15 / 2)。送信は 1 回(続けて合図を立てても追加で送らない) | `runner_signal::実行中の合図は sigterm として届く / sigint も転送される / シグナルは 1回だけ転送される`。区分: 単体(UNIX 系) |
| AC-287 | 許可外の番号は送らない | 起動の部品の単体 | 合図の番号が SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、-1、9999 | 送信しない。ジョブは自分で終わる。終わりの観測の直後に `interrupted = true`、`signal_number` がその番号 | `runner_signal::転送しない番号の合図では送られず interrupted だけが立つ`、`runtime::platform/unix` の単体。区分: 単体(UNIX 系) |
| AC-288 | ジョブは新しいプロセスグループ | 起動の部品の単体 | ジョブを起動して、プロセスグループ ID を調べる | ジョブのプロセスグループ ID が、ジョブの pid に等しく、`conductor`(呼び出し側)のものと異なる | `runner_signal::ジョブは新しいプロセスグループで起動される`。区分: 単体(UNIX 系) |
| AC-289 | 先に終わったジョブには転送しない | 起動の部品の単体 | ジョブが終わった後に合図が立つ | 送信しない | `runner_signal::先に終わったジョブは転送なしで観測される`。区分: 単体(UNIX 系) |
| AC-290 | 起動前の合図 | 起動の部品の単体 | 合図が立った状態で起動を呼ぶ | ジョブは起動されない(`started=false`、`interrupted=true`、`signal_number` = 番号)。分類は「分類なし」、終了コード `128 + 番号` | `runner_basic::起動前の合図が立っていれば起動しない`、`core::flow_sequences::起動しなかった中断は再開位置になり 128 足す番号を返す`。区分: 単体 |
| AC-291 | 終了の観測の直後の合図 | 起動の部品の単体 | ジョブの実行中に、転送されない番号の合図を立てる(ジョブは自分で終わる) | ジョブの終了コードを保ったまま `interrupted = true`、`signal_number` = その番号。分類は「エラー、`128 + 番号`」 | `runner_basic::終了の観測の直後に合図が立っていれば interrupted を足す`。区分: 単体 |
| AC-292 | 合図が立たなければ中断にならない | 起動の部品の単体 | 合図なしで `exit 1` | `interrupted = false`、`signal_number` なし | `runner_basic::実行中に合図が立たなければ interrupted にならない`。区分: 単体 |
| AC-293 | 起動前の中断の記録(注入) | 流れの制御(偽の起動部品) | 起動前に合図が立つ | ジョブを起動せず、`resume` = そのジョブ、エラーとして記録しない、終了 `128 + 番号` | `flow_run::起動の前に合図が立っていれば 起動せず 再開位置を記録して 128 足す番号`。区分: 注入 |
| AC-294 | 実行中の中断の記録(注入) | 同上 | 実行中に合図(2 つ来ても最初の番号) | ジョブを `error` として記録、終了 `128 + 最初の番号` | `flow_run::ジョブの実行中の合図は エラーとして記録して 128 足す番号 / 2 つの合図が来ても 最初の番号が残る`。区分: 注入 |
| AC-295 | ジョブのあいだの中断(pipeline、注入) | 同上 | 1 つ目のジョブが成功した直後に合図 | 次のジョブを起動せず、再開位置を記録して `128 + 番号`。-v で `[conductor] interrupted by signal N` | `flow_pipeline::ジョブのあいだの合図は 再開位置を記録して 128 足す番号`。区分: 注入 |
| AC-296 | 検査中の合図より誤りが優先 | 同上 | 定義の検査中に合図が立つ。定義の誤り / 順序外 / スキップ | 64 / 66 / 0(中断の終了コードにならない) | `flow_run::検査中の合図より 定義の誤りの終了コードが優先する / 検査中の合図より reject と skip が優先する`。区分: 注入 |
| AC-297 | シグナルの準備は検査の前(v0.10.0 で変更) | 同上 | `run` / `pipeline`、`watch --follow`、`validate`・`watch`・`reset`・`convert`・引数の誤り | `run`・`pipeline` は、定義の検査より前にシグナルを準備する。v0.10.0 以降は `watch --follow` も、run ID の決定より前に準備する。ほかは準備しない(`--follow` の無い `watch`、v0.9.0 以降の `graph`・`init`、`--dry-run` も準備しない) | `flow_run::シグナルの準備は 定義の検査より前に行う`、`flow_pipeline::個別実行と一括実行だけがシグナルを準備する`、`flow_run::使い方の誤りでは 何も実行せず シグナルも準備しない`、`flow_aux::u7_br1_1 follow の無い watch は合図の準備をしない`、`flow_aux::u7_br3_3 合図の準備に失敗したら 70 で何も読まない`。区分: 注入 |
| AC-298 | シグナルの準備の失敗は 70 | 同上 | 準備が失敗する | 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査もジョブの起動もしない | `flow_run::シグナルの準備に失敗したら 70 で 固定の英語の文`。区分: 注入 |
| AC-299 | (該当なし) | — | — | 登録はプロセスで 1 回だけで、「すでに登録済み」の場面は無い(番号は、範囲の参照のために残す) | — |
| AC-300 | 標準エラーが閉じていても本来の終了コード | 同上 | 標準エラーへの書き込みが失敗する | 本来の終了コードで終わる | `flow_run::標準エラーが閉じていても 本来の終了コードで終わる`。区分: 注入 |
| AC-301 | Windows の Ctrl+C(手動) | Windows の実機、コンソール。`SL` | `conductor run sleeper` の実行中に Ctrl+C を押す | (1) 終了コード 130。(2) ジョブの子プロセスが残らない。(3) 続く `watch` で `sleeper` が `error`、`after` が `pending`。(4) `run -v` の ERR に `interrupted by signal 2`。結果を記録欄に残す | `WINDOWS_CTRL_C_CHECKLIST`。区分: 手動(Windows)。参照実装では、記録欄が空で、実施の有無は未確認 |

### 4.11 ステートの異常系

ここの「実行」は、特に記さない限り `run a`(`P(p; a:0, b:0)`)。すべて、**ジョブは実行されず**、ステートファイルは**変わらず**、OUT は空。ERR の最初の行は `conductor: error: conductor.toml.state: <種類の文>: <理由>; <案内>`。
案内は ``run `conductor reset` to discard the state``(run ID あり: ``run `conductor reset --run-id <ID>` to discard the state``)。
(v0.10.0 以降)前提の「正しいステート」は、動いたジョブに試行(`[[job.attempt]]`)を持つもの(3.4 節)。試行の記録の誤り(試行が無い、最後でない試行に終わりが無い、時刻の書式の誤り)と、v0.9.9 までに書いたステートは、4.26 節(AC-559)。

| ID | 機能 | ステートの内容(前提) | 期待(終了 65) | 根拠 |
|----|------|------------------------|----------------|------|
| AC-302 | TOML として読めない | `garbage = = =` | `state file is corrupt: TOML として読めません` | `runtime::state_read::壊れた toml は corrupt で内容を表示に出さない`、`flow_run::ステートが壊れているときは 65` |
| AC-303 | UTF-8 でない | 不正なバイト列 | `state file is corrupt: UTF-8 の文字列ではありません` | `runtime::state_read::不正な utf8 は corrupt` |
| AC-304 | 大きさの上限 | 正しいステートに、コメント行を足して 1,048,577 バイト | `state file is too large: 大きさが上限(1 MiB)を超えています`。ちょうど 1,048,576 バイトは受理(実行が進む) | `runtime::state_read::上限ちょうどは受理し 1 バイト超えは too_large` |
| AC-305 | 書式の版が違う | `format_version = 2`(ほかに未知のキーがあっても)/ `0` | `state file version is not supported: 対応していない版です(reset が必要です)` | `runtime::state_read::将来の版は version_mismatch で未知のキーに影響されない`、`flow_run::ステートが読めないときは 65 で reset を案内する` |
| AC-306 | 書式の版がない・整数でない | `format_version` なし / `"1"` | `state file is corrupt: format_version がありません` / `format_version が整数ではありません` | `runtime::state_read::raw::fr3_4 版違い 壊れたファイル…` |
| AC-307 | 未知のキー | 最上位に未知のキー。旧形式の `jobs` | `state file is corrupt: 知らないキーがあります` | `runtime::state_read::旧キー jobs のステートは corrupt で拒否され内容を表示に出さない` |
| AC-308 | 必須のキーがない | `pipeline_name` などを欠く | `state file is corrupt: 必須のキーがありません` | `runtime::state_read::内容の矛盾は corrupt` |
| AC-309 | 値の型・状態の誤り | `status = "weird"`、`skip_reason` の誤り、`skipped` に `skip_reason` なし、`success` に `skip_reason` あり、`pending` に `exit_code` あり | `state file is corrupt: status の値が誤っています` / `skip_reason が status と合いません` / `exit_code が status と合いません` など(06 章 2.4 節の矛盾) | `runtime::state_read::内容の矛盾は corrupt` |
| AC-310 | ジョブの並びの矛盾 | ジョブID の重複 / `job_order` と `[[job]]` の不一致 / `resume_job_id` が `job_order` にない / `completed = true` と `resume_job_id` | `state file is corrupt: ジョブIDが重複しています` / `job の並びが job_order と一致しません` / `resume_job_id が job_order にありません` / `completed と resume_job_id が両立しません` | `runtime::state_read::内容の矛盾は corrupt` |
| AC-311 | 読めないファイル(UNIX 系、非 root) | ステートの権限を 000 に | `cannot read state file: 開く、または読み取ることができません` | `runtime::state_read::権限のないファイルは unreadable`、`flow_run::ステートが読めないときは 65 で reset を案内する` |
| AC-312 | 定義との食い違い: パイプライン名 | `pipeline_name` が定義と違う(定義側の名前を変えて `run`) | `conductor: error: conductor.toml.state: state does not match the definition: pipeline name; run `conductor reset` to discard the state` | `flow_run::定義との食い違いは 65 で reset を案内する`、`runtime::flow_with_state::ac4_4_1 から 3 食い違いはジョブを実行せず reset で回復する` |
| AC-313 | 定義との食い違い: ジョブID | ジョブの追加・改名のあと | `state does not match the definition: job ids; …` | 同上 |
| AC-314 | 定義との食い違い: ジョブの並び | ジョブの順序を入れ替えたあと | `state does not match the definition: job order; …` | 同上 |
| AC-315 | 食い違いの案内は run ID ありで変わる | `--run-id r1` のステートで定義を変える | `…; run `conductor reset --run-id <ID>` to discard the state` | `flow_run_id`、08 章 4.6 節 |
| AC-316 | 日本語 | `garbage = = =`、`LANG=ja_JP.UTF-8` | `conductor: error: conductor.toml.state: ステートファイルが壊れています: TOML として読めません; `conductor reset` でステートを初期化できます` | `report_messages::ステートの誤りの補足は文脈で決まる` |

ステートの異常のあとの回復と、正常な読み込み。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-317 | reset で回復する | 上のいずれかのあと | `reset`、続けて `run a` | 0。先頭から実行される | `runtime::flow_with_state::ac4_4_1 から 3` |
| AC-318 | pipeline も同じ | 上のいずれか | `pipeline p` | 65。ジョブは起動しない | `flow_pipeline::ステートの誤りと食い違いは 65 ジョブは起動しない` |
| AC-319 | ファイルなし・長さ 0 は「空」 | ステートなし / 長さ 0 | `run a` | 通常どおり 0 | `runtime::state_read::ステートファイルが無ければ空 / 長さ 0 のファイルは空` |
| AC-320 | 完了のまま残ったステートは「空」 | 完了のステート(正しい) | `pipeline p` | 先頭から実行して 0。読み込みでファイルは消されない(実行の最初の書き込みで上書きされる) | `runtime::state_read::completed のまま残ったステートは空として返しファイルは消さない` |
| AC-321 | 残った一時ファイルは読み込みに影響しない | `.conductor.toml.state.tmp.1.2.3` がある | `run a` | 通常どおり 0 | `runtime::state_write::残った一時ファイルは読み込みに影響しない` |
| AC-322 | 書き込みは原子的 | 書き込み前後 | (ステートを読み続ける) | 読み手は、直前の完全な内容か新しい完全な内容だけを見る。書き込みの失敗でも直前の内容が残る | `runtime::state_write::失敗しても以前のステートは壊れない / 置き換え先がディレクトリなら write_failed でステートは変わらず一時ファイルも残らない` |
| AC-323 | 排他制御はしない | 2 つの `run` を同時に | (2.1 節) | 両方が動く(2.1 節の「同時の `run`」) | `runtime::flow_with_state::ac2_5_2`。AC-404 を参照 |
| AC-324 | 終了と書き込みの間の停止は再実行(at-least-once) | 終了の直後、書き込みの前に `conductor` が止まる | 再度 `run a` | ジョブが再実行されうる。**外からは再現できない**(参照実装は、部品を差し替えて検査する) | `runtime::flow_with_state::at_least_once 終了のあと書き込みの前に止まるとジョブは再実行される`。区分: 注入 |

### 4.12 引数の文法

特に記さない限り、定義は `B0`、OUT は空、ERR は `conductor: error: <文>`⏎(空行)⏎`Usage:` から始まる使い方の全文。「68」の行は、終了 68。

| ID | 機能 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|
| AC-325 | 引数なし | `conductor` | 68。`missing argument` | `args_grammar::引数がない`、`binary_basic::使い方の誤りは 68…` |
| AC-326 | オプションだけ | `-v` | 68。`missing argument` | `args_grammar::引数がない` |
| AC-327 | 知らないコマンド | `bogus` / `''`(空文字) | 68。`unknown command: 'bogus'` / `unknown command: ''` | `args_grammar::知らないコマンドとオプション`、`binary_basic` |
| AC-328 | 必須の引数がない | `run` / `pipeline` / `convert` | 68。`missing argument` | `args_grammar::必須の引数の欠落と余分な引数` |
| AC-329 | 余分な引数 | `run a b` / `validate a b` / `watch x` / `reset x` | 68。`unexpected argument: 'b'` / `'b'` / `'x'` / `'x'` | 同上 |
| AC-330 | 使えないオプション | `watch -v`、`reset -v`、`run --all a`、`run --to yaml a`、`validate --remove-on-complete`、`reset --remove-on-complete`、`convert f --to yaml -v` | 68。`option not allowed for this command: '<そのオプション>'` | `args_grammar::v は run と pipeline だけ / all は reset だけが受け付ける / fr2_5…` |
| AC-331 | `--run-id` が使えるコマンド | `run`、`pipeline`、`watch`、`reset` に `--run-id x` | 文法として受理される(4 コマンド)。`validate`・`convert` では 68 | `args_grammar::run_id は 4 つのコマンドで解釈される / run_id は validate と convert で使えない` |
| AC-332 | 知らないオプション | `run --bogus a` / `run -x a` / `run --bogus=x a` | 68。`unknown option: '--bogus'` / `'-x'` / `'--bogus'` | `args_grammar::知らないコマンドとオプション` |
| AC-333 | 値の欠落 | `run a --config` / `run a --run-id` / `convert f --to` | 68。`option requires a value: '--config'` / `'--run-id'` / `'--to'` | `args_grammar::値の欠落は missing_value` |
| AC-334 | 重複 | `run -vv a`、`run -v -v a`、`run -v --verbose a`、`run a --config x --config y`、`run a --run-id x --run-id y`、`reset --all --all`、`run a --remove-on-complete --remove-on-complete`、`convert f --to yaml --to toml` | 68。`invalid value: '<オプション名>'`(`-vv` は `'-v'`) | `args_grammar::v はちょうど 1 回 / 重複する config と to は誤り / fr2_5 remove_on_complete は値を取らず 2 回の指定は誤り` |
| AC-335 | `--all` と `--run-id` | `reset --all --run-id x` | 68。`invalid value: '--all'` | `args_grammar::all と run_id の併用と重複と値の欠落は誤り` |
| AC-336 | 値を取らないオプションに `=値` | `run --verbose=x a`、`reset --all=x`、`run a --remove-on-complete=x` | 68。`invalid value: '--verbose'` / `'--all'` / `'--remove-on-complete'` | `args_grammar`(04 章 4 節) |
| AC-337 | `--to` の値 | `convert f --to json` / `YAML` / `Toml` | 68。`invalid value: '--to'`(値は含めない。非 UTF-8 の値も同じ) | `args_grammar::to の値は yaml か toml だけ / to の値の誤りの対象は to で値を含まない / utf8 でない to の値は invalid value で対象は to` |
| AC-338 | `--to` がない | `convert f` | 68。`missing argument: '--to'` | `args_grammar::to は convert だけで必須` |
| AC-339 | `--run-id` の値の検査 | `run a --run-id 'a b'` | 68。`invalid value: '--run-id'`(値を含めない) | `args_grammar::run_id の無効な値は 68 で誤りの文に値を含めない` |
| AC-340 | 空の `--config` | `run a --config=` | 64。`definition file does not exist`(空のパスとして受理され、後で誤りになる) | `args_grammar::config は空白と等号の両方の形を受け付ける`(04 章 4 節) |
| AC-341 | オプションの書き方 | `--config c.toml -v run a`、`run a -v`、`--config=c.toml`、`--run-id=r1`、`convert f --to=yaml` | すべて文法として正しい(`run` は実行される。`--run-id=` は run ID なしに固定) | `args_grammar::config は空白と等号の両方の形を受け付ける / オプションはコマンドの前にも書ける` |
| AC-342 | `--` のあとは位置引数 | `run -- -weird` / `-- --help` / `run a -- -v` | `job is not defined in the definition file: '-weird'`(68)/ `unknown command: '--help'`(68)/ `unexpected argument: '-v'`(68) | `args_grammar::ダブルハイフンのあとは位置引数 / ダブルハイフンのあとの help は処理しない` |
| AC-343 | ヘルプ(英語) | `--help`、`-h` | 終了 0。OUT = 08 章 8.1 節の使い方の全文(末尾は改行)。ERR 空 | `flow_aux::help は使い方を標準出力に出して 0` |
| AC-344 | ヘルプ(日本語) | `LANG=ja_JP.UTF-8` で `--help` | 0。OUT = 08 章 8.2 節の全文 | `report_messages::使い方は種類ごとの固定の文と使い方の一覧` |
| AC-345 | バージョン | `--version`、`-V` | 0。OUT = `conductor 0.10.2`(版は配布の版)。ERR 空。`LANG=ja_JP.UTF-8` でも英語・同じ | `binary_basic::version は 0 で版を標準出力に出す / fr6_1`、`flow_aux::version は版を標準出力に出して 0` |
| AC-346 | `-h`/`-V` は最優先 | `-vh`、`bogus --help`、`run -vv --help`、`-V -h`、`-h -V` | `-vh` は help(0)。`bogus --help` も help(0)。`-V -h` は version、`-h -V` は help(先に現れた方) | `args_grammar::help と version は先に処理される` |
| AC-347 | 使い方の誤りの形 | `bogus`(`LANG=C`)、`bogus`(`LANG=ja_JP.UTF-8`) | ERR の最初の行 = `conductor: error: unknown command: 'bogus'` / `conductor: error: 知らないコマンドです: 'bogus'`。2 行目は空行、3 行目以降は使い方の全文(英語 / 日本語)。OUT 空。ジョブは実行されない | `binary_basic::使い方の誤りは 68 で標準エラーに使い方を出す / 日本語のロケールでは エラーが日本語になる`、`flow_aux::使い方の誤りは日本語のロケールで日本語` |
| AC-348 | 使い方の誤りの全種類 | 上の各種類 | 8 種類(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)が、すべて 68 で、08 章 4.2 節の英語・日本語の文になる | テストの側の種類の一覧の写し(`tests/common/kinds.rs`。網羅の `match` で、種類が増えるとコンパイルで止まる)、`report_messages::使い方は種類ごとの固定の文…` |
| AC-349 | UTF-8 でない位置引数(UNIX 系) | 位置引数・`--run-id` の値に不正なバイト列 | 68。`invalid value`。`--config` と `convert` の入力パスは、そのまま通る(不正なバイト列のファイル名の定義が読める) | `args_grammar::utf8 でない… / utf8 でないパスは os_string のまま保つ`。区分: UNIX 系 |
| AC-447 | 値を取るオプションの値は `-h`/`--help` でも値 | `--config --help run a`、`--config -h run a`、`run a --config --version`(定義ファイルは存在しない) | 終了 64。help・version は出ない。ERR に、その語をパスとしたファイルの誤り(`--help: ...` など) | 派生(04 章 4.1・4.2) |
| AC-448 | 値を取るオプションが値を欠く | `--config --help` / `--help --config` | 前者は 68(`missing argument`。`--help` が値に取られて位置引数が 0 個)。後者は help(0) | 派生(04 章 4.2) |
| AC-449 | `-h`/`-V`/`-` と `--` の細部 | `-h=x`、`-V=1`、`--help=x`、`-vh=x`、`run -`、`-- run`、`-- run a` | `-h=x`・`--help=x`・`-vh=x` は help(0)、`-V=1` は version(0)。`run -` はジョブID `-`(定義がなければ 64)。`-- run` は 68(`missing argument`。`run` がコマンド名でジョブIDなし)。`-- run a` は `run` のジョブID `a`(定義がなければ 64) | 派生(04 章 4.1・10 節。実機で確認) |

### 4.13 出力とロケール

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-350 | 言語の決定: `LC_ALL` が最優先 | `LC_ALL=ja_JP.UTF-8`、`LANG=C` | `bogus` | ERR に `知らないコマンドです`(日本語) | `report_messages::ロケールの判定` |
| AC-351 | 最初の空でない値だけを見る | `LC_ALL=C`、`LANG=ja_JP.UTF-8` | `bogus` | 英語(`unknown command`) | 同上 |
| AC-352 | `LC_MESSAGES`、`LANG` の順 | `LC_MESSAGES=ja_JP`、`LANG=C` / `LANG=ja_JP`(ほか未設定) | `bogus` | 日本語 / 日本語 | 同上 |
| AC-353 | 空の値は飛ばす | `LC_ALL=`(空)、`LANG=ja_JP.UTF-8` | `bogus` | 日本語(空の `LC_ALL` は無視) | 同上 |
| AC-354 | 大文字小文字を区別する | `LANG=JA` / 未設定 | `bogus` | 英語 | 同上 |
| AC-355 | 標準出力と標準エラーの使い分け(v0.10.0 で変更) | `P(p; a:0)` | `run -v a` ほか | `run`・`pipeline`・`reset`・`validate` は、OUT に何も書かない。OUT に書くのは `watch`、`--help`、`--version`、`convert` と、v0.9.0 以降の `graph`、`--dry-run`、`init --config -` だけ。v0.10.0 以降は、`watch --follow` の描くたびの表(と、その前に置く画面の消去の列か空の 1 行)と `graph --format dot` も OUT。ジョブ本体の出力は、そのまま流れる(`log_dir` があっても、同じ流れに同じバイト列が出て、同時にログのファイルにも写す)。ログのファイルの 3 つの文は ERR | `binary_basic`、`report_messages::level ごとに出力先が決まる`、`report_messages::u7 継続表示は標準出力に出る`、`report_messages::ログを作れない文はエラーで 警告の 2 つは知らせの水準`、`binary_log::fr5_5 log_dir のあり なしで端末の出力が同じ` |
| AC-356 | 成功・スキップで表示なし | `P(p; a:0)`、`LANG` に不正な値 | `run a` | OUT・ERR 空(表示がなければ言語も色も判定しない) | `flow_run::成功は何も表示せず ロケールも判定しない`、`report_messages::言語と色の判定は 最初に必要になるまで行われない` |
| AC-357 | -v のトークンは言語によらず英語 | `LANG=ja_JP.UTF-8`、`P(p; a:1{w=[1]})` | `run -v a` | ERR = `[conductor] job=a result=warning (終了コード 1)`(`job=`、`result=`、`warning` は英語) | `flow_run::v は日本語のロケールで日本語になる` |
| AC-358 | -v の補足: 起動できない | 存在しないプログラム、`-v` | `run -v a` | ERR に `[conductor] job=a result=error (could not start)` と、続いて `conductor: error: job 'a': cannot start command '…'`。日本語は `起動できませんでした` | `report_messages::v の文言`、08 章 2.1 節 |
| AC-359 | -v の補足: シグナルで終了(UNIX 系) | `kill -9 $$` のジョブ | `run -v a`(日本語も) | `terminated by signal 9` / `シグナルで終了 9` | `report_messages::v の文言` |
| AC-360 | -v の補足: 中断(UNIX 系) | `SL`、SIGTERM | `run -v sleeper` | `interrupted by signal 15`(日本語 `シグナルで中断 15`)。起動前の中断は `interrupted by signal N, not started` / `シグナルで中断 N, 起動せず` | `report_messages::v の文言`(注入) |
| AC-361 | 全種類のイベントの形 | 08 章 1.1 節のイベント | (表示の部品を直接呼ぶ) | すべての文言が空でなく、改行で終わり、生の制御文字を含まない。誤りの行は `conductor: error: ` で始まる。英語の表示は日本語の文字を含まない(原因の文を除く) | `report_messages::全種類のイベントが空でなく 改行で終わり 生の制御文字を含まない / エラーの行は conductor error で始まり 場所を含む / 英語の表示は英語の文だけで日本語の文字を含まない`。区分: 単体 |
| AC-362 | 定義の誤りの形 | `P(…)` の誤り | `validate` | 1 行ずつ `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。原因は日本語の固定の文(英語のロケールでも) | `report_messages::定義の誤りは ファイル 行 種類 原因を 1 行ずつ` |
| AC-363 | 定義の誤りの種類の文(日本語) | `LANG=ja_JP.UTF-8` | 4.1 節の各誤り | 種類の文が 08 章 4.3 節の日本語(例: `定義にないキーです`、`ジョブが 1 つもありません`) | `report_messages::定義の誤りの日本語は…`、`リトライの設定の誤りは日本語の固定の文を持つ` |
| AC-364 | 定義の誤りの種類の文(英語・retry) | 4.1 節の retry の各誤り | `validate` | 08 章 4.3 節の英語の文(`retryNotTable` ほか 9 種) | `report_messages::リトライの設定の誤りは英語の固定の文を持つ` |
| AC-365 | ステートの誤りの補足 | 4.11 節の各誤り | (文脈ごと) | 読み込み: 案内 `; …`。書き込み: ` (the job was not started)` / ` (the job has already been executed)`。`reset`・`watch` 以外の読み込みは案内つき、`reset` の削除は補足なし | `report_messages::ステートの誤りの補足は文脈で決まる` |
| AC-366 | 拒否・完了判定不能・出力失敗の文言 | 08 章 4.7 節 | (表示の部品を直接呼ぶ) | `cannot run this job now: …`、`cannot determine whether the pipeline is complete`、`cannot write to standard output` の英語・日本語 | `report_messages`。区分: 単体 |
| AC-367 | 完了を判定できない | 全ジョブが「実行済み」で通過し、どこでも完了にならない状態(通常は起こらない) | `pipeline p` | 65。ERR = `conductor: error: conductor.toml.state: cannot determine whether the pipeline is complete`。ステートを初期化しない | `flow_pipeline::完了の判定ができないときは 65 で 初期化しない`。区分: 注入 |
| AC-368 | 内部の誤りの文は固定の英語 | 異常終了を起こす | (入口) | ERR = `conductor: error: internal error`(ロケールによらず)、終了 70。標準エラーが閉じていても 70 | `panic_hook::tests::フックの本体は固定の英語の文を書き 70 で終わる / 標準エラーに書けなくても 70 で終わる`(フックの本体の単体テスト。フックの設定そのものは自動のテストで確かめない)。区分: 単体 |
| AC-369 | 入口に異常終了を起こすコマンドがない | 製品の入口 | (点検) | 入口(引数の解釈の結果)に、わざと異常終了するコマンドがない。入口は 1 回だけ終了する | `nfr_source::テスト専用の経路なし`。点検の項目は AC-422 |

### 4.14 表示のエスケープ

入力に由来する文字列(ジョブID、名前、パス、キー名、コマンド名、使い方の誤りの対象)は、表示する前にエスケープする。**ジョブ本体の出力は通さない**(そのまま)。

規則: LF → `\n`、CR → `\r`、TAB → `\t`。そのほかの制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・方向制御の文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)は、`\u{<小文字の 16 進。先頭の 0 なし>}`。
そのほかの文字(日本語を含む)はそのまま。名前(`escape_name`)は、エスケープ後 64 文字で切る(エスケープ列は 1 かたまり。次の 1 文字を足すと 64 を超えるなら、そこで打ち切り、末尾に `…`(U+2026)を 1 つ付ける。出力は最大 65 文字)。パスは切らない。

外から確かめる方法: `B0` に対して `run '<入力>'`(定義にないジョブID)を実行し、ERR の最初の行が `conductor: error: job is not defined in the definition file: '<出力>'` になることを確かめる。終了 68。

| ID | 機能 | 入力 | 出力(期待) | 根拠 |
|----|------|------|--------------|------|
| AC-370 | LF と ESC | `a<LF>b<ESC>[0m` | `a\nb\u{1b}[0m` | `escape_table` |
| AC-371 | TAB | `k<TAB>x` | `k\tx` | 同上 |
| AC-372 | CR | `a<CR>b` | `a\rb` | 同上 |
| AC-373 | 日本語と空白 | `日本語 abc` | `日本語 abc`(そのまま) | 同上 |
| AC-374 | ゼロ幅スペース | `a<U+200B>b` | `a\u{200b}b` | 同上 |
| AC-375 | 右から左への上書き | `a<U+202E>b` | `a\u{202e}b` | 同上 |
| AC-376 | 方向の隔離 | `a<U+2066>b` | `a\u{2066}b` | 同上 |
| AC-377 | BOM | `a<U+FEFF>b` | `a\u{feff}b` | 同上 |
| AC-378 | 行・段落の区切り | `a<U+2028>b<U+2029>` | `a\u{2028}b\u{2029}` | 同上 |
| AC-379 | その他の制御文字 | `<U+007F><U+0085>`(NUL は引数に渡せないため、単体で確かめる) | `\u{7f}\u{85}`(NUL は `\u{0}`) | 同上(NUL は区分: 単体) |
| AC-380 | 64 文字ちょうど | `k` × 64 | `k` × 64(切らない) | 同上 |
| AC-381 | 64 文字を超える | `k` × 200 | `k` × 64 + `…` | 同上 |
| AC-382 | エスケープ後で数える | LF × 100 | `\n` × 32 + `…`(65 文字) | 同上 |
| AC-383 | 上限 3(単体) | `abcdef`、上限 3 | `abc…`。空の入力は空 | 同上。区分: 単体 |
| AC-384 | 全部の部品で同じ結果 | 上の表の入力 | 定義・ステート・変換の誤りを作る各部品が、この表と同じ結果を返す | `escape_table`(core・runtime・converter と揃える)。区分: 単体 |
| AC-385 | ジョブIDは 64 文字で切る | ジョブID が 200 文字の定義 | `watch` の行・`-v` の行・`run` の誤りで、ID が 64 文字 + `…` | `report_messages::ジョブ id は 64 文字で切り パスは切らない` |

表示の上限と、各部の適用。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-386 | パスは切らない | 300 文字のファイル名の定義(存在しない) | `validate --config <パス>` | ERR にパスの全体が現れる | 同上 |
| AC-387 | リトライの知らせのジョブID | ジョブID に制御文字・200 文字 | リトライの行 | エスケープされ、64 文字で切られる | `report_messages::リトライの知らせの job_id はエスケープされ 64 文字で切られる` |
| AC-388 | 定義の誤りのキー名 | 定義のキー名に制御文字 | `validate` | キー名がエスケープされ、64 文字で切られる | `core::definition_errors::キー名は 64 文字で切られて表示される`、`core::nfr_security` |

### 4.15 終了コード全件

「終了コードの全件」の確認。各コードを起こす受け入れテストを、表で示す。

| 終了コード | 意味 | 起こすテスト |
|------------|------|--------------|
| 0 | 成功・警告・スキップ・`watch`・`reset`・`validate`・`convert` の成功・`--help`・`--version`・`reset --all` の警告。v0.10.0 以降は `watch --follow` の終わり(完了・止まった・消えた・中断の合図)と、ログのファイルの警告だけのとき | AC-389 |
| 1 | 成功コードに含まれない 0、エラーで下位 8 ビットが 0 | AC-390 |
| ジョブの値(1〜255) | エラーのジョブの終了コード | AC-391 |
| 64 | 定義の誤り・`command` の誤り。v0.10.0 以降は `[pipeline.retry]`・`retry`・`env_file` の表・`unset_env`・`log_dir` の誤りと、`unset_env` と `env`・`.env` の重なり | 4.1 節、4.21〜4.23 節、4.25 節 |
| 65 | ステートの誤り・書き込み・削除の失敗・`watch` の出力失敗・完了判定不能。v0.10.0 以降は試行の記録の誤りと v0.9.9 までのステート、`watch --follow` の読めない 3 回・壊れた・出力失敗 | 4.11 節、4.3 節、4.6 節、4.5 節、4.26 節、4.27 節 |
| 66 | 拒否(順序外) | 4.3 節 |
| 67 | 変換の誤り・`convert` の出力失敗 | 4.7 節 |
| 68 | 使い方の誤り(知らないジョブID、run ID の誤りを含む)。v0.10.0 以降は `--follow`・`--interval` の誤り | 4.3、4.8、4.12 節、4.27 節 |
| 70 | 内部の誤り・シグナルの準備の失敗(v0.10.0 以降は `watch --follow` の準備の失敗も) | 4.10 節・4.13 節(注入)、4.27 節 |
| 127 | ジョブを起動できなかった。v0.10.0 以降はログのファイルを作れずに起動しなかったとき | 4.2 節、4.25 節 |
| 128 + 番号 | ジョブがシグナルで終了、または中断 | 4.3 節、4.9 節、4.10 節 |

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-389 | 終了 0 の全種類(v0.10.0 で変更) | 成功 / 警告(`w=[3]` で `exit 3`)/ スキップ(実行済み)/ `watch`・`reset`・`validate`・`convert`・`--help`・`--version` の成功 / `reset --all` の警告 / v0.10.0 以降は、`watch --follow` で完了したステート・止まったステート・消えたステート・中断の合図、`graph --format dot` の成功、成功したジョブのログのファイルへの書き込みの失敗・出力の待ち切れの警告 | 各実行 | すべて 0 | `binary_basic::警告は 0 スキップは 0 エラーはジョブの値` ほか、`binary_basic::u7_fr7_3 完了のステートを follow で見ると 1 回描いて 0 でパイプに制御の文字が無い`、`binary_basic::u7_fr7_3 止まったステートを follow で見ると 0 で終わる`、`flow_aux::u7_br3_1 一度見えたステートが 2 回続けて無いなら 0 で終わる`、`flow_aux::u7_br3_2 待ちの途中の合図で 0`、`flow_run::br5_2_br5_3 書き込みの失敗と待ち切れは警告を 1 行ずつ出し 終了コードは変わらない` |
| AC-390 | 終了 1 の 2 種類 | `P(p; a:0{s=[5]})`(成功コードにない 0)/ エラーで下位 8 ビットが 0 になる終了コード | `run a` | 1 / 1(`exit 256` は Windows で確かめる。UNIX 系では OS が 0 にするので成功。関数の検査は下) | `flow_run::成功コードに含まれない 0 は 1…`、`exit_codes::エラーで 0 になる場合は 1` |
| AC-391 | ジョブの値 | `exit 1`、`exit 2`、`exit 7`、`exit 255` | `run a` | 同じ値 | `binary_basic`、`core::flow_sequences` |
| AC-392 | 終了コードの範囲合わせ(関数の検査) | 報告する値 `reported`、`is_error` | 範囲合わせの関数 | `0..255` はそのまま。`259`→3、`511`→255、`-1`→255、`-2`→254、`0xC0000005`(32 ビット符号つき)→5。`256`・`512`・`-256`・32 ビットの最小値は、エラーなら 1、エラーでなければ 0 | `exit_codes::範囲内の値はそのまま / 範囲外の値は下位 8 ビット / エラーで 0 になる場合は 1 / エラーでなければ 0 のまま`。区分: 単体 |
| AC-393 | 中断の終了コード(関数の検査) | シグナル番号 n | `128 + n` を範囲合わせ(エラー扱い) | 15→143、2→130、1→129、3→131、300→172(範囲を超えない) | `exit_codes::中断の終了コードは 128 足す番号 / 中断の終了コードは範囲を超えない`。区分: 単体 |
| AC-394 | 分類の全 10 項目(関数の検査) | 07 章 4.4 節の表の各行 | 分類の関数 | 順 1: 起動前の中断 → 分類なし、`128 + n`。順 2: 起動なし・中断でもない → エラー 127。順 3: 起動済みの中断 → `128 + n`(ジョブの終了コードを見ない)。順 4: 起動失敗 → 127。順 5: シグナル終了 n → `128 + n`。順 6: 終了コードなし → 1(`conductor` のエラー)。順 7: 成功コード → 0。順 8: 警告コード → 0。順 9: 0 で成功コードにない → 1(`conductor` のエラー)。順 10: そのまま。優先順位は上の行ほど高い | `core::flow_sequences::優先順位 中断と起動失敗とシグナル`、`flow_run`。区分: 単体 |
| AC-395 | 終了コードの決定順序(v0.10.0 で変更) | 複数の誤りを同時に持つ入力 | `run` | 検査の順(引数 68 → 準備 70 → 定義 64 → `command` 64 → ジョブID 68 → run ID 68 → `.env` の読み取り 64(v0.9.0 以降。v0.10.0 以降は、`.env` の名前と `unset_env` の重なりを含む)→ ステートの読み込み 65 → 照合 65 → 起動前の判断 66/0 → 初期ステート 65(v0.10.0 以降は、試行ごとの試行の開始の書き込み。最初の試行の書き込みが初期化を兼ねる)→ ジョブ(v0.10.0 以降は、ログのファイルを作れなければ起動せず 127)→ 結果の書き込み 65 → 削除 65)で、最初の誤りだけが採られる。具体例は 4.3 節(定義の誤り > ジョブID)、4.8 節(定義 64 > run ID 68 > ステート 65) | `flow_run::順序は 判断 初期ステート ジョブ 記録の書き込み`、`flow_run::u6_br1_1 試行の始まりは起動の前に fast で書かれる`、`flow_run_id::run id の決定は定義の検査のあと ステートの読み込みの前`、`flow_env::env の名前と消す名前の重なりは 64 で範囲は読み取りの範囲`(10 章 2 節) |
| AC-396 | 128 + 番号の例 | ジョブが SIGKILL / 実行中に SIGTERM(UNIX 系) | `run a` | 137 / 143 | `flow_run::シグナルで終了したジョブは 128 足す番号`、`binary_signal` |

### 4.16 非機能要件の検証

2 章の各基準の受け入れテスト。「計測」は p95 などの測定、「強制」は基準超過で不合格(2 章の扱いの列)。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-397 | 定義の読み込みの性能 | ジョブ 10 個の定義(ファイルから読む) | 読み込みと検証を 100 回 | p95 が 20 ms 以下(目安。超えたら警告を記録) | `core::nfr_performance`。区分: 計測 |
| AC-398 | 起動前の処理の性能 | ジョブ 10 個の定義 | `command` の検査、ステートの読み、照合、速い書き込みを 100 回 | p95 が 20 ms 以下(仮。目安) | `runtime::nfr_performance`。区分: 計測 |
| AC-399 | `run` のオーバーヘッド | ジョブ 10 個の定義(各ジョブは `TJ timing`) | `run job0` を 20 回 | `conductor` 自身の時間が p95 100 ms 以下(目安) | `cli::nfr_perf`。区分: 計測 |
| AC-400 | `pipeline` のジョブあたり | 同上 | `pipeline` を 5 回 | ジョブ 1 つあたり p95 50 ms 以下(目安) | 同上。区分: 計測 |
| AC-401 | 大きな定義 | ジョブ 1,000 個の定義 | `run job0` | 1,000 ms 以内(仮。目安) | 同上。区分: 計測 |
| AC-402 | 大きな流れ | ジョブ 1,000 個の `pipeline`(プロセスもファイルも使わない流れの判断) | (内部の流れ) | 10,000 ms 以内(目安)。完了のステートが残ることは常に検査 | 同上。区分: 計測・単体 |
| AC-403 | 待機中の CPU 時間(UNIX 系) | `TJ sleep 3`(約 3 秒)のジョブ | `run a` の `conductor` の CPU 時間を、実行の最後に読む | 100 ms 以下。強制(CPU 時間が読めなければスキップ) | 同上。区分: UNIX 系 |
| AC-404 | 同時の 2 つの `run` | `P(p; a:0{cmd="TJ sleep 1"})`(終了コード 99) | 同じ定義に `run a` を 2 つ同時に | 両方が実行され(ジョブが 2 回始まる)、どちらも終了コード 99。ステートは正しい形で読み戻せる | 同上、`runtime::flow_with_state::ac2_5_2 排他制御はなく 2 つの実行がそれぞれ動く`。強制 |
| AC-405 | ランダムな引数 | 固定の種 | 語の表から 0〜6 個を選んだ引数列で 500 回 | 終了コードが 0〜255。標準エラーに、制御文字(改行を除く)と U+202E がない。異常終了しない | `cli::nfr_robust` |
| AC-406 | UTF-8 でない引数(UNIX 系) | `0xff 0xfe` を含むバイト列 | ジョブID・`--config`・オプション・`convert` の引数に渡す | 異常終了しない(終了コードが 0〜255) | `cli::nfr_robust`。区分: UNIX 系 |
| AC-407 | 壊れたステート | 状態・再開位置・フラグを乱したステート、ランダムなバイト列、巨大な値 | `run` / `watch` | 異常終了しない。壊れていれば 65 | `cli::nfr_robust`、`runtime::nfr_robust`、`runtime::state_read` |
| AC-408 | 空・制御文字のジョブID | ジョブID が空、制御文字を含む | `run <ID>` | 異常終了しない(68) | `cli::nfr_robust` |
| AC-409 | ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件(定義の `command` に入れる) | `validate` | 0 か 64 のどちらか。0 のとき `program` は空でない | `runtime::nfr_robust`(分割の単体も) |
| AC-410 | 大きな `command` | `prog ` + `'a b' "c\"d" e ` を 10 万回(約 30 万引数) | `validate` | 0。閉じない引用符を足したものは 64。30 秒以内 | `runtime::nfr_robust::大きな command` |
| AC-411 | 起動できない `command` | `''`、`'`、`""`、存在しない名前、ディレクトリ、`/` | `validate` / `run a` | 前 3 つは 64、残りは 127。異常終了しない | `runtime::nfr_robust::起動できない command` |
| AC-412 | 入れ子の深い入力 | 深さ 1,000 の TOML(定義、ステート) | `validate` / `run a` / `convert` | 64 / 65 / 67。異常終了せず、小さなスタックでも動く | `core::library_capability`、`runtime::toml_depth` |
| AC-413 | 変換の頑健性 | 範囲外・空・壊れた・切り詰めた入力、ランダムな構造 | `convert` | 異常終了しない。成功した構造は往復で等しい | `converter::nfr_security` |
| AC-414 | 内部の誤り | 異常終了を起こす | (入口) | 4.13 節の「内部の誤りの文」と同じ | `panic_hook` |
| AC-415 | 検証は副作用を持たない | 4.1 節の最後の項目 | — | 4.1 節 | `core::nfr_security` |
| AC-416 | 入力の値を表示に出さない | 各種類のエラー(定義・`command`・ステート・変換・使い方)に、秘密のつもりの文字列を含む入力 | 各コマンド | ERR に、その値が現れない(キー名は、エスケープして現れてよい)。`command` の誤りの文に `command` の中身が現れない | `core::nfr_security`、`runtime::nfr_content`、`converter::nfr_security`、`read_resolve::誤りの文は入力の値を含まない` |
| AC-417 | ステートに機微な値を書かない | 4.3 節の項目 | — | 4.3 節 | `runtime::nfr_content` |
| AC-418 | 変換に副作用がない | 4.7 節の項目 | — | 4.7 節 | `converter::nfr_security` |
| AC-419 | ネットワークに接続しない | 実装と依存 | 点検(ソースの検索、依存の一覧、実行中の通信の監視) | ネットワークの機能を使わない。通信の部品(HTTP・TLS など)が依存にない | `cli::nfr_source::ネットワークなし`、依存の検査。区分: 点検 |
| AC-420 | OS 固有コードの隔離 | ソース | 点検 | OS の分岐・OS 固有の呼び出しが、ジョブの起動の部品と入口の部品それぞれの「プラットフォーム層」の外にない。安全でない呼び出しはシグナル送信の 1 か所だけ(それがある言語では。入口の部品には 0 か所) | `runtime::nfr_source`(5 件)、`cli::nfr_source`(6 件)。区分: 点検 |
| AC-421 | 異常終了を起こす書き方がない | ソース | 点検 | ジョブの起動・ステートの部品の製品のコードに、未検査の取り出しや強制的な中断がない。変換も、入力の経路で同じ | `runtime::nfr_source`、`converter::nfr_security`。区分: 点検 |
| AC-422 | テスト専用の経路を持たない | ソース、配布物 | 点検 | 入口に、異常終了を起こす経路がない。テスト用の実行ファイル以外に、テスト専用のコードがない。配布物にテスト用ジョブが入っていない | `cli::nfr_source`、6 章。区分: 点検 |
| AC-423 | 著作権表示 | ソース | 点検 | 各ソースの先頭に Apache License 2.0 のヘッダがある | (規約)。区分: 点検 |
| AC-573 | 計画の模擬の性能(v0.10.1 以降) | ジョブ 1,000 個がすべて実行する定義(ステート・`.env` なし)/ 同じ定義で、`run j0` が `@end` に着いた後のステート(`j0` 成功、残り未実行、`end_reached`) | 計画の生成を、それぞれ 1 回(5 回の中央値) | 前者: 1,000 個すべてが `[run]`、結果は `completed`。後者: `j0` が実行済み、残りの 999 個が `@end` のあとのスキップ、結果は `@end` に到達。どちらも 150 ms 以内(目安。参照実装は開発用の版で強制として検査する。2.1 節)。計画の模擬は、実行と同じ判断を、ジョブの位置と済んだ前の部分の目印を持ち回って、1 つの判断あたり定数の手間で行い、計画の実行するジョブは本番の流れが起動するジョブと一致する(05 の 16.2) | `plan_simulate_perf::p1 千個のジョブがすべて実行する計画は 150ms 以内`、`plan_simulate_perf::p2 終点のあとの 999 個のスキップの計画は 150ms 以内`、`flow_dry_run::計画の実行するジョブは 本番の流れが起動するジョブと一致する`。区分: 計測 |


### 4.17 環境変数の注入と `.env`(v0.9.0 以降)

特に記さない限り、ジョブ `show` は `cmd="TJ env <名前>..."`(子の環境を、標準出力に `env[NAME]=set:q(値)` / `env[NAME]=unset` の行として出す)で、定義は `[pipeline] name = "p"` の下に `show` 1 つを置く。「環境」の欄は、被試験体に渡す環境変数(親の環境)。`CONDUCTOR_RUN_ID` は特に記さない限り未設定。「`.env` の誤り」の期待の「64」は、終了 64、ジョブを 1 つも起動しない、ステートファイルを作らない、を含む。値(秘密のつもりの文字列 `SECRET`)が、OUT・ERR のどこにも現れないことも、すべての項目で確かめる。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-450 | 層の優先順位と配列の順 | `[pipeline] env_file = "p.env"`、`[pipeline.env]` に `LAYER = "pipeline-env"`、`T_PENV = "pe"`。`show` に `env_file = ["j1.env", "j2.env"]`、`[job.env]` に `LAYER = "job-env"`、`T_JENV = "je w"`。`p.env` = `LAYER=pipeline-file⏎T_PFILE=pf⏎T_SHARED=from-p⏎T_OVERRIDDEN=from-env-file`、`j1.env` = `LAYER=j1⏎T_J1=one⏎T_SHARED=from-j1`、`j2.env` = `export T_J2=two⏎T_SHARED=from-j2⏎T_QUOTED="a b\n"`。環境: `T_INHERITED=from-parent`、`T_OVERRIDDEN=from-parent`。`show` は `LAYER T_PFILE T_PENV T_J1 T_J2 T_JENV T_SHARED T_QUOTED T_NONE T_INHERITED T_OVERRIDDEN` を出す | `run show` | 終了 0。OUT: `LAYER` = `"job-env"`、`T_PFILE` = `"pf"`、`T_PENV` = `"pe"`、`T_J1` = `"one"`、`T_J2` = `"two"`、`T_JENV` = `"je w"`、`T_SHARED` = `"from-j2"`(配列は後ろが勝つ)、`T_QUOTED` = `"a b\n"`(値は、`a`、空白、`b`、LF)、`T_NONE` = unset、`T_INHERITED` = `"from-parent"`(引き継いだ環境は残る)、`T_OVERRIDDEN` = `"from-env-file"`(`.env` が引き継いだ値を上書きする) | `binary_env::env と env_file の値と優先順位が子のプロセスの環境に見える` |
| AC-451 | 自動の変数 | `show` が `CONDUCTOR_PIPELINE CONDUCTOR_JOB_ID CONDUCTOR_RUN_ID CONDUCTOR_ATTEMPT` を出す | `pipeline p --run-id r-42` / `run show` | 前者の OUT: `CONDUCTOR_PIPELINE` = `"p"`、`CONDUCTOR_JOB_ID` = `"show"`、`CONDUCTOR_RUN_ID` = `"r-42"`、`CONDUCTOR_ATTEMPT` = `"1"`。後者(run ID なし): `CONDUCTOR_JOB_ID` = `"show"`、`CONDUCTOR_RUN_ID` = unset、`CONDUCTOR_ATTEMPT` = `"1"` | `binary_env::自動の変数が run id の有無で渡る` |
| AC-452 | run ID がないとき、引き継いだ `CONDUCTOR_RUN_ID` はそのまま | `show` が `CONDUCTOR_RUN_ID` を出す。環境: `CONDUCTOR_RUN_ID=zzz` | `run show --run-id ""`(run ID なしに固定) | 終了 0。OUT: `CONDUCTOR_RUN_ID` = `"zzz"`(`conductor` は足さず、取り除かない) | 05 章 14.3(派生) |
| AC-453 | `CONDUCTOR_ATTEMPT` はリトライごとに増え、再開で 1 に戻る(v0.10.0 で変更) | `show` に `[job.retry] exit_codes = [75], max = 2, interval = "1s"`。起動の部品の差し替えで、1・2 回目は終了コード 75、3 回目は 0 を返す | `run show`。続けて、リトライを使い切るジョブ(常に 75)を `run`、再び `run` | 起動のたびに渡された環境の `CONDUCTOR_ATTEMPT` は `1`、`2`、`3`。使い切って停止したあとの再実行では、また `1` から数える。v0.10.0 以降は、ステートに試行ごとの時刻(`[[job.attempt]]`)が残り、再実行の試行は前の試行の後ろに足される。`CONDUCTOR_ATTEMPT` は、ステートの試行の数から数えない(リトライの回数そのものはステートに書かれない) | `flow_env::リトライで attempt が 1 2 3 と増え再開で 1 に戻る`、`flow_run::u6_br1_4 再実行しても前の試行を残して後ろに足す`。区分: 注入 |
| AC-454 | `.env` の誤りで、何も実行せず 64 | `[pipeline] env_file = ["ok.env", "bad.env", "missing.env"]`。`ok.env` = `GOOD=SECRET`、`bad.env` = `NOEQ SECRET⏎T_OK=1⏎1BAD=SECRET⏎Q="SECRET\q"`(`missing.env` は無い) | `pipeline p` / `run show` / `validate` | すべて 64。OUT 空(ジョブが動かない)。ERR に、`bad.env:1`(missingEquals)、`bad.env:3`(invalidName)、`bad.env:4`(invalidEscape)、`missing.env`(fileMissing)の行がある。ERR に `SECRET` が現れない。ステートファイルがない | `binary_env::env の誤りは 64 で値が出力に無くジョブもステートも作らない` |
| AC-455 | 誤りは 20 件で打ち切る | `.env` に、`=` のない行が 25 行 | `validate` | 64。ERR は 21 行。最後の行 = `conductor: error: and 5 more`(日本語 `ほか 5 件`)。先頭の 20 行は、行番号 1〜20 の `missingEquals` | `flow_env::誤りは 20 件で打ち切り残りの件数を示す`、`report_messages::env の誤りは 20 件で打ち切り…` |
| AC-456 | `validate` の重複キーの警告 | `[pipeline] env_file = "p.env"`、`p.env` = `A=1⏎B=SECRET⏎A=2` | `validate` | 終了 0。OUT 空。ERR に `conductor: warning: ` で始まり、`p.env:3: duplicate key` と `` `A` `` を含む行(英語。日本語は `重複キー`)がある。`SECRET` が現れない。ステートを作らない | `binary_env::validate は重複キーの警告を標準エラーに出して 0` |
| AC-457 | `run` は自分のジョブの分だけ読む | `[pipeline]` に `env_file` なし。ジョブ `a`(`env_file` なし)と `b`(`env_file = "nofile.env"`。ファイルなし) | `run a` / `pipeline p` / `run b` | `run a` は 0(`b` の `.env` を読まない)。`pipeline p` と `run b` は 64(`pipeline` は全ジョブ分を読む) | `flow_env::run は pipeline の env_file とそのジョブの分だけを読む` |
| AC-458 | 再開でも読み直す | `[pipeline] env_file = "p.env"`。`b` がエラーで止まったステート | `p.env` を壊して、再度 `pipeline p` | 64。ジョブを起動せず、ステートは変わらない(`.env` を、呼び出しごとに読み直す) | `flow_env::再開でも env を読み直す` |
| AC-459 | 使い方の誤り(68)が `.env` の誤り(64)より先 | `.env` が無い定義 | `run nojob`(定義にないジョブID)/ `run a --run-id 'a b'` | どちらも 68 | `flow_env::使い方の誤りの 68 が env の誤りの 64 より先に勝つ` |
| AC-460 | `env`・`env_file` の定義の誤り | 各行の定義(`[job.env]` に書く)。ヘッダは `B0` | `validate` | 64。種類の文(`LANG=C`): `CONDUCTOR_X = "1"` と `conductor_x = "1"` → `reserved environment variable name`。`foo = "1"` と `FOO = "2"` → `environment variable names differ only in letter case`(2 つ目のキーの行)。`"1A" = "x"`、`"A B" = "x"` → `invalid environment variable name`。`A = "a\u0000b"` → `invalid environment variable value`。`A = 1` → `wrong value type`。`env = 5` → `wrong value type`。`env_file = ""`(空)→ `required value is empty`。`env_file = ["a", 3]`、`env_file = 5` → `wrong value type` | `env_definition`(各誤りと、予約名の大文字小文字、NUL、順序) |
| AC-461 | 予約名は `.env` でも拒否 | `.env` に `CONDUCTOR_X=1` / `conductor_x=1` / `Conductor_X=1` | `validate` | 64。ERR = `conductor: error: <.env>:1: reserved environment variable name: ` に続く理由(英語: ``names starting with `CONDUCTOR_` cannot be written``)。`foo=1⏎FOO=2` は `caseConflict`(`environment variable names differ only in letter case`、行 2) | `env_file_parse`(予約名、名前の誤り) |
| AC-462 | `.env` の書式(値) | `.env` の 1 行と、`show` が出す値(`run show`) | — | `A=1` → `"1"`。`export A=1`、`A = 1` → `"1"`。`A=x=y` → `"x=y"`。`A=x #y` → `"x"`、`A=x#y` → `"x#y"`。`A= "x"` → `"x"`。`A='no $expansion'` → `"no $expansion"`。`A="a\nb"` → `"a⏎b"`(LF)、`A="q\"\\"` → `q"\`。`A=` → `""`。`A=  # c` → `""`。`A=$HOME` → `"$HOME"`(展開しない)。先頭の BOM と CRLF の改行は受理(値に `\r` が入らない) | `env_file_parse`(各引用符、エスケープ、BOM、CRLF、`export`)。区分: 自動(値は子の環境で確かめる) |
| AC-463 | `.env` の書式(誤り) | `.env` の 1 行(`validate`) | — | 64。`A` → `missingEquals`。`=1`、`1A=x` → `invalidName`。`A='x` → `unterminatedQuote`。`A="x\q"` → `invalidEscape`。`A="x" y` → `trailingGarbage`。`A='x' # c`、`A="x" # c` は受理。値に NUL(U+0000)→ `nulInValue`。引用符が行をまたぐ書き方 → `unterminatedQuote` | 同上 |
| AC-464 | `.env` の上限と文字コード | `.env` を作る | `validate` | 大きさ 1,048,576 バイトちょうど → 0。1,048,577 バイト → 64(`tooLarge`。中身を読まず、行なし)。項目 10,000 個 → 0、10,001 個 → 64(`tooManyEntries`、10,001 個目の行)。UTF-8 でない → 64(`notUtf8`)。ディレクトリ / 権限 000(UNIX 系、非 root)→ 64(`unreadable`)。読む権限のない 1 MiB 超(UNIX 系、非 root)→ 64(`unreadable`。v0.9.6 までは `tooLarge`)。書き手のいない名前付きパイプ(UNIX 系)→ 64(`unreadable`。開かずに返る)。空のファイルは 0 | `env_loader::ちょうど 1 mib は通り 1 mib 足す 1 は誤りで…`、`utf8 でないファイルは誤り`、`無い ディレクトリ 読めないファイルは誤りで中身を読まない`、`権限のない 1 mib 超の env は unreadable`、`名前付きパイプの env は開かずに unreadable`、`env_file_parse::項目の上限` |
| AC-465 | パスの基準は定義ファイルのディレクトリ | `T/sub/c.toml`(`env_file = "x.env"`)、`T/sub/x.env` = `V=sub`、`T/x.env` = `V=cwd`。`show` は `V` を出す | `T` で `run show --config sub/c.toml` | 0。`V` = `"sub"`(実行ディレクトリの `x.env` は読まない)。絶対パス、`../x.env` も使える。`~` と変数は展開しない(`~/x.env` は、その名前のファイルを探して、無ければ `fileMissing`) | `env_loader::相対パスは定義ファイルのディレクトリが基準で絶対パスと dotdot も使える` |
| AC-466 | 同じ名前を 2 回書いた `.env` は後ろが勝つ | `.env` = `A=1⏎A=2` | `run show`(`A` を出す) | `A` = `"2"`。`validate` は警告(AC-456) | `env_file_parse`(重複) |
| AC-467 | 層をまたぐ大文字小文字だけが違う名前 | `[pipeline.env] foo = "p"`、`[job.env] FOO = "j"`。`show` は `foo FOO` を出す | `run show` | Linux・macOS: `foo` = `"p"`、`FOO` = `"j"`(両方が見える)。Windows: `FOO` = `"j"` だけ(OS が名前の大文字小文字を区別しない)。完全に同じ名前の優先順位は、どの OS でも同じ(後ろの層が勝つ) | `env_merge::層をまたぐ大文字小文字だけが違う名前は両方が並びに残り後ろが後`。区分: OS による差 |
| AC-468 | 環境変数の値がどこにも出ない | `[pipeline] env_file = "p.env"`(`T_TOKEN=SECRET`)、`[pipeline.env] T_PENV = "SECRET"`、`quiet`(`TJ exit 3`)に `[job.env] T_JENV = "SECRET"` | `pipeline -v p` / `run -v quiet` / `validate -v` | OUT・ERR のどこにも `SECRET` が現れない。失敗(終了 3)で書かれたステートファイルに、`SECRET`、`T_TOKEN`、`T_PENV` が現れない | `binary_env::秘密の値は標準出力 標準エラー ステートファイルに含まれない`、`flow_env::値は標準出力 標準エラー ステートのどこにも出ない` |
| AC-469 | 実行時に渡す環境の並び | 起動の部品の差し替え | `pipeline` | 起動に渡される足す環境変数の並びは、層の順(`[pipeline].env_file`、`[pipeline.env]`、`[job].env_file`、`[job.env]`、`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(run ID があるとき)、`CONDUCTOR_ATTEMPT`)。完全に同じ名前は 1 つ(後ろの層の位置) | `flow_env::run job に渡る環境の並びは層の順`、`env_merge`。区分: 注入 |
| AC-470 | README の環境変数の説明 | README | (検査) | 見出し、版の表記(`v0.9.0 以降`)、必須の語句があり、載っている TOML と `.env` の例が実際の読み取りと解析に通る | `docs`。区分: 点検 |

### 4.18 `graph`(v0.9.0 以降)

特に記さない限り、実行は `graph`(`LANG=C`)、期待は終了 0、OUT = 示した図、ERR 空、ステートファイルを作らない、ジョブを起動しない。定義は `P(p; ...)`(ジョブのコマンドは `TJ exit 0`)で、`on=success->X` は成功の遷移。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-471 | レーンの図: 飛び越しなし(G1) | `P(p; a:0, b:0, c:0)` | `graph` | OUT = `* a⏎* b⏎* c⏎o @end⏎` | `graph_report::レーンの図の例 g1 から g4 と g6` |
| AC-472 | レーンの図: 1 本の飛び越し(G2) | `P(p; a:0{on=success->d}, b:0, c:0, d:0, e:0)` | `graph` | OUT = `*   a`⏎`\|\`⏎`* \| b`⏎`* \| c`⏎`\|/`⏎`* d`⏎`* e`⏎`o @end`(各行の末尾は改行)。8 章 11.1 の G2 と同じ | 同上、`binary_graph::text と mermaid の出力が例と一致しジョブは動かない` |
| AC-473 | レーンの図: `@end` への飛び越し(G3)、入れ子(G4)、続く 2 本(G6) | G3: `P(p; a:0{on=success->@end}, b:0, c:0)`。G4: `P(p; a:0{on=success->f}, b:0{on=success->e}, c:0, d:0, e:0, f:0)`。G6: `P(p; a:0{on=success->c}, b:0, c:0, d:0{on=success->f}, e:0, f:0, g:0)` | `graph` | OUT は、8 章 11.1 の G3、G4、G6 と 1 文字も違わず一致する | `graph_report::レーンの図の例 g1 から g4 と g6` |
| AC-474 | 辺の一覧の整形(E1、E2) | 辺の一覧の配置(05 の 15.4 の範囲の外の図の構造。定義からは作れないので、整形の関数に辺の一覧の配置を直接渡す)。E1: `P(p; a:0{on=warning->e, on=success->e}, b:0{on=success->f}, c:0{on=success->e}, d:0, e:0, f:0, g:0)` の辺。E2: E1 と同じに、`--state`(すべて未実行) | 整形(`LANG=ja_JP.UTF-8` 相当) | OUT は、8 章 11.4 の E1、E2 と 1 文字も違わず一致する。英語の注記は 05 の 15.6 の文。E1 の定義そのものを `graph` に渡すと、レーンの図(8 章 11.4 の前の段落) | `graph_report::辺の一覧の例 e1 交差と e2 状態つき`、`graph_report::e1 の形は レーンの図になる` |
| AC-475 | 「対応する形」の判定(単体) | 辺の集合 | 判定 | すべてのジョブが、飛び越しなし(次の節への辺だけ)、通常の飛び越し(次の節への辺と、飛び越しの辺 1 本)、「両方の飛び越し」(辺は 1 本で成功と警告の両方、行き先は次の節でない)、分岐(辺は 2 本で、どちらも次の節でなく、行き先が異なる)のどれかなら、レーンの図。交差と同じ行き先は、どの組み合わせでも問わない(入れ子、続く飛び越し、同じ行き先への合流、交差する通常の飛び越し、交差に同じ行き先が混ざる形、交差し、かつグラフのどこかに「両方の飛び越し」・分岐を含む形、分岐の遠い行き先が最も内側の線の行き先より遠い形、後のジョブが分岐の線へ同じ行き先で合流する形は、すべてレーンの図)。定義から作れる形は、すべてレーンの図。辺が 3 本のジョブ、次の節への辺がなく成功だけの辺が 1 本のジョブ、後ろ向きの辺(図の構造を直接作る)は、辺の一覧 | `graph_drawable`。区分: 単体 |
| AC-476 | 配置の正解データ(単体) | 参照実装の正解データ(`git log --graph` の実際の出力: 行き先がすべて違う形の 537 例、同じ行き先を含む形の 1,517 例、交差する形のジョブ 3〜7 個の 940 例、交差と同じ行き先が混ざる形のジョブ 4〜6 個の 487 例。規則から導いた図で、git の出力ではない: 分岐を含む形の 871 例(主線の列 0 の規則で作り直した第 2 版)、「両方の飛び越し」か分岐を含むジョブ 3〜5 個のすべての形の 2,748 例、ジョブ 6〜7 個の 300 例) | 配置 | 全例で、行の並び(記号と、ラベルを置く位置)が一致する | `graph_layout_oracle`。区分: 単体 |
| AC-476c | 交差する飛び越しの図(C1〜C3)と、両方の飛び越し・分岐が混ざる形 | C1: `P(p; a:0{on=success->d}, b:0{on=success->e}, c:0, d:0, e:0)`。C2: 7 ジョブで `a→d`、`b→e`、`c→f`(横線 `_` を含む)。C3: 6 ジョブで `a→f`、`b→@end`、`c→e`。混ざった形: 交差する `a→d`、`b→e` と、離れたジョブの「両方の飛び越し」、分岐 | `graph` | C1〜C3 の OUT は、8 章 11.3c と 1 文字も違わず一致する。混ざった形も、レーンの図(交差の部分は C1 と同じで、「両方の飛び越し」・分岐の下の行の列 0 は空白) | `graph_shape_skip`、`graph_shape_both`、`graph_shape_branch`、`graph_report`、`binary_graph`。区分: 単体、結合 |
| AC-476e | 両方の飛び越し・分岐と、交差・同じ行き先の組み合わせ(N1〜N5)と、`-v`、`--state` | N1: `P(p; a:0{on=warning->d, on=success->d}, b:0{on=success->e}, c:0, d:0, e:0, f:0)`。N2: 6 ジョブで `a` の成功が `c`、警告が `e`、`b→d`。N3: 5 ジョブで `a` の両方が `d`、`b→d`。N4: 6 ジョブで `a` の両方が `d`、`b` の両方が `e`。N5: 6 ジョブで `a` の成功が `c`、警告が `e`、`c→e`。性能: 1,000 ジョブで、「両方の飛び越し」と分岐の入れ子、幅広い交差 | `graph`(N1 は `graph -v`、`graph --state` も。N2、N5 は `graph -v` も) | OUT は、8 章 11.3d と 1 文字も違わず一致する。主線の列 0(図の行と、`-v` の詳細の行)は、上のジョブが実行されうる、かつ次のジョブへの辺を持つときだけ `\|`。1,000 ジョブは 1 秒以内に、レーンの図になる | `graph_shape_both`(N1、N3、N4)、`graph_shape_branch`(N2、N5)、`graph_drawable`(主線の列 0)、`graph_report`、`binary_graph`、`graph_perf`。区分: 単体、結合 |
| AC-476d | 交差に同じ行き先が混ざる図(M2〜M4)と、`-v`、`--state` | M2: `P(p; a:0{on=success->e}, b:0{on=success->d}, c:0{on=success->e}, d:0, e:0)`。M3: 5 ジョブで `a→d`、`b→e`、`c→e`。M4: 6 ジョブで `a→d`、`b→f`、`c→e`(横線 `_` を含む)。性能: 1,000 ジョブで、前半の各ジョブ `i` が `N/2 + i % 250` へ飛ぶ | `graph`(M2 は `graph -v`、`graph --state` も) | M2〜M4 の OUT は、8 章 11.3c と 1 文字も違わず一致する。`-v` の詳細の行を除くと、`-v` なしの図と同じ。1,000 ジョブは 1 秒以内に、レーンの図になる | `graph_shape_same_target`、`graph_report`、`binary_graph`、`graph_perf`。区分: 単体、結合 |
| AC-476b | 分岐の図(B1〜B4) | B1: `P(p; a:0{on=success->c, on=warning->d}, b:0, c:0, d:0, e:0)`。B2: `a` の成功が `c`、警告が `@end`(4 ジョブ)。B3: `a` の成功が `g`、`b` の成功が `d`、警告が `e`(7 ジョブ)。B4: `a` の成功が `d`、警告が `f`(6 ジョブ) | `graph`、`graph -v`、`graph --state` | OUT は、8 章 11.3b の B1〜B4 と 1 文字も違わず一致する(実行されうるジョブの下の合流の行の列 0 は `\|`)。成功と警告のどちらが近い行き先でも同じ図。`-v` の詳細の行と `--state` の記号も、行の形を変えない(詳細の行の列 0 は、主線の列 0 の規則) | `graph_shape_branch`、`graph_report::分岐の図 b1 の全文`、`graph_report::分岐の図に v の詳細と state を重ねる`、`binary_graph::分岐の定義は 2本の線を同時に開くレーンの図で出る`、`docs` |
| AC-476a | 同じ行き先と「両方の飛び越し」の図(S1、D1、D2〜D6) | S1: `P(p; a:0{on=success->d}, b:0{on=success->d}, c:0, d:0, e:0)`。D1: `P(p; a:0{on=success->c, on=warning->c}, b:0, c:0, d:0)` | `graph` | OUT は、8 章 11.3a の S1、D1 と 1 文字も違わず一致する。D2〜D7 は 05 の 15.5 の規則から導いた図で、交差しない「両方の飛び越し」の図は、同じ飛び越しを成功だけで書いた通常の図と、主線の `|`(` \`、` /`、` \|` の先頭)を除いて一致する。交差する D3 もレーンの図 | `graph_report::同じ行き先の図 s1 と 両方の飛び越しの図 d1`、`graph_shape_both`、`binary_graph` |
| AC-477 | `--state` の重ね合わせ | `P(p; a:0{on=success->d}, b:0, c:0, d:1, e:0)` で `pipeline p`(`d` が終了 1 でエラー。`b`・`c` はスキップ、`a` は成功) | `graph --state` | OUT = `*   [OK] a`⏎`\|\`⏎`* \| [--] b`⏎`* \| [--] c`⏎`\|/`⏎`* [NG] d  <- resume here`⏎`* [  ] e`⏎`o @end`。ステートファイルのバイト列は変わらない。`--format mermaid` を足すと、`n3["[NG] d"]` を含み、末尾が `  classDef resume stroke-width:3px⏎  class n3 resume⏎` | `binary_graph::state は 実際に作ったステートの状態と再開位置を重ねる` |
| AC-478 | `--state` の例外 | (1) ステートなし。(2) 完了したステート。(3) 定義と食い違うステート | `graph --state` | (1) すべて `[  ]`、再開位置なし。(2) 状態の記号は重ねる(`[OK]`、`[--]`)が、再開位置の注記(`<- resume here`)はない。(3) 終了 65、OUT 空、ERR に `state does not match the definition` | `flow_graph::state でステートがなければ すべて 未実行 / state は ステートの状態と再開位置を重ね 完了したステートには再開位置がない / state のステートが定義と食い違えば 65…` |
| AC-479 | Mermaid | G2 の定義 | `graph --format mermaid` | OUT は、8 章 11.5 のとおり(`flowchart TD`、節 `n0`〜`n4` と `nend`、辺。主線の辺はラベルなし、それ以外は `-->\|"warning"\|` / `-->\|"success"\|`)。`-v` を足しても同じ。ジョブIDに `<`、`>`、`&`、`#`、`"`、`` ` `` を含むと、`#lt;`、`#gt;`、`#amp;`、`#35;`、`#quot;`、`#96;` にエスケープされ、図が壊れない | `graph_report::mermaid の例と state の再開位置 / mermaid のラベルは 特別な文字をエスケープし id を壊さない` |
| AC-480 | `-v` の詳細 | G2 の定義(`LANG=C`。ジョブの command は `TJ exit 0`) | `graph -v` | 各ジョブの行(と、続く分岐の行)のあとに、列の数だけ `\| ` を重ねた詳細の行が付く(列 0 は、主線の列 0 の規則。実行されないジョブと、次のジョブへの辺がないジョブの下は空白)。`a` の分岐の行の次に `\| \| command: <a の command>` と `\| \| success -> d (skips b, c), warning -> b`。`[job.retry]` があれば `retry: exit codes 75, 76, up to 3 times, every 5 s (x2, max 60 s)` の行。配列の `command` は、空・空白を含む要素だけ二重引用符で囲む。辺の一覧のときは、ジョブの行の下に 4 つの空白で字下げ。日本語は `コマンド: `、`リトライ: `、`成功 → d(b、c を飛び越す)、警告 → b` | `graph_report::v の詳細は g2 と g4 の例のとおり…`、`v は リトライと配列のコマンドと英語の文を出し…` |
| AC-481 | `validate` と同じ検査 | `env_file = ["missing.env"]` の定義 / `env_file` が指す `.env` に `DUP=one⏎DUP=SECRET` | `validate`、`graph` | 前者: どちらも 64。`graph` の OUT 空。ERR は `validate` と同じ。後者: `graph` は 0 で図を出し、ERR に重複キーの警告(`conductor: warning:`)。`SECRET` は現れない | `binary_graph::validate と同じ検査をする env の誤りは 64 で 重複キーは警告で 0`、`flow_graph::validate が 64 になる定義は graph も 64…` |
| AC-482 | 環境変数は図に出ない | `[pipeline]` と `[job]` に `env`、`env_file`(値 `SECRET`) | `graph`、`graph -v`、`graph --format mermaid`、`graph --state -v` | すべて 0。OUT・ERR に、`SECRET` も、環境変数の名前(`env` のキー、`.env` の名前)も現れない。`-v` は、コマンドだけを出す | `binary_graph::環境変数の値は どの出力にも出ない` |
| AC-483 | 使い方の誤り | — | `graph --format json` / `graph --format text --format mermaid` / `graph --run-id x` / `graph --state --state` / `graph -vv` / `watch --state` / `run --format text a` / `graph a b` | すべて 68。順に `invalid value: '--format'`、`invalid value: '--format'`、`option not allowed for this command: '--run-id'`、`invalid value: '--state'`、`invalid value: '-v'`、`option not allowed for this command: '--state'`、`option not allowed for this command: '--format'`、`unexpected argument: 'b'`。`graph --state --run-id x` は文法として正しい。ジョブを起動しない | `args_grammar`、`flow_graph::使い方の誤りは 68 で ジョブを起動しない` |
| AC-484 | 名前の不一致と標準出力の失敗 | `graph other`(定義の名前は `p`)/ 標準出力に書けない状態(注入) | `graph` | 前者は 64(`pipeline name does not match the definition`)。後者は 65、ERR = `conductor: error: cannot write to standard output` | `flow_graph::パイプライン名の不一致は 64 / 標準出力に書けなければ 65`。後者の区分: 注入 |
| AC-485 | ジョブを起動せず、ステートに触れず、1 回で書く | 起動の部品・ステートの部品・出力先の差し替え | `graph`、`graph --state` | ジョブは 1 つも起動されない。ステートは書かれず、消されず(読むだけ)。シグナルの準備はしない。標準出力への書き込みは 1 回で、図の全体 | `flow_graph::graph はジョブを起動せず ステートに触れず 標準出力に図だけを 1 回で書く`。区分: 注入 |
| AC-486 | 1,000 ジョブの性能 | 入れ子が最大の形(前半の各ジョブ `i` が `N-1-i` へ飛ぶ)/ 交差する形(`j(i)` → `j(i+2)`)/ 幅の広い交差の形 / 同じ行き先が離れて重なる形(前半の各ジョブ `i` が `N/2 + i % 250` へ飛ぶ)/ 「両方の飛び越し」と分岐の入れ子(前半の偶数のジョブ `i` が `N-1-i` へ両方の飛び越し、奇数のジョブが `N-2-i` と `N-1-i` へ分岐)/ 「両方の飛び越し」と分岐の幅広い交差(前半のジョブ `i` を 3 つおきに、`i+N/2` と `i+N/2+1` への分岐、`i+N/2` への両方の飛び越し、通常の飛び越し)。ジョブ 1,000 個 | 図の構造と配置 | 6 つとも 1 秒以内(強制)。幅の広い交差の形(前半の各ジョブ `i` が `i+N/2` へ飛ぶ)も含め、6 つともレーンの図 | `graph_perf`。区分: 計測(強制) |
| AC-487 | README の `graph` の説明 | README | (検査) | 見出し、版の表記、必須の語句があり、載っている図の例が、実際の出力と一致する | `docs`。区分: 点検 |

### 4.19 `--dry-run` と `--assume`(v0.9.0 以降)

特に記さない限り、期待は終了 0、ERR 空、ステートファイルを作らない、ジョブを起動しない(ジョブが書くはずのファイルができない)。`ENV3` は `CONDUCTOR_PIPELINE, CONDUCTOR_JOB_ID, CONDUCTOR_ATTEMPT`。`C` は、ジョブの分割後のコマンド(例: `["<TJ のパス>", "exit", "0"]`)。定義 `ABC` = パイプライン `demo`、`a:0`、`b:0{w=[3]}`、`c:0`。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-488 | `pipeline` の計画 | `ABC` | `pipeline demo --dry-run` | OUT = `plan (--dry-run): pipeline demo`⏎`resume: from the start`⏎`1. [run] a`⏎(5 つの空白)`command: C`⏎(5 つの空白)`env: ENV3`⏎`2. [run] b`(同じ 2 行)⏎`3. [run] c`(同じ 2 行)⏎`result (simulated): completed`。日本語は 8 章 11.6 の P1 | `binary_dry_run::pipeline の計画と assume の出力が例と一致し 何も作られない`、`plan_report::例 p1 から p4…` |
| AC-489 | `--assume` | `ABC` | `pipeline demo --dry-run --assume b=3` | OUT に `assume: b=3`、`2. [run (assumed: exit code 3 -> warning)] b`、最後の行 `result (simulated): completed`。`run a --dry-run --assume a=1` は、OUT = `plan (--dry-run): pipeline demo`⏎`target job: a`⏎`assume: a=1`⏎`1. [run (assumed: exit code 1 -> error)] a`(+ command、env の行)⏎`next step: (assumed: exit code 1 -> error) stop` | 同上 |
| AC-490 | 再開位置と本番の判断との一致 | `a:0, b:1, c:0` で `pipeline demo` を実行(`b` で停止、終了 1) | `pipeline demo --dry-run` / `run b --dry-run` | 前者の OUT に `resume: b (previous stop)`、`1. [already executed] a`、`2. [run] b  <- resume here`、`3. [run] c`。後者は `target job: b` を示し、最後の行が `next step: next job c`(再開位置の代わりに、対象と次の動き)。ステートファイルのバイト列は、どちらでも変わらない | `binary_dry_run::実際に作ったステートから再開位置が出て ステートのバイト列は変わらない` |
| AC-491 | 何も実行されない | `a`・`b` が、記録のファイルに書く(`TJ --out O run a 0`、`run b 0`) | `pipeline demo --dry-run` / `run a --dry-run` | どちらも 0。`O` ができない。ステートがない。(`--dry-run` なしの `pipeline demo` なら、`O` ができる) | `binary_dry_run::何も実行されない ジョブが書くはずのファイルができない` |
| AC-492 | 環境変数は名前だけ | `[pipeline] env_file = "p.env"`(`T_TOKEN=SECRET`)、`[pipeline.env] T_PENV = "SECRET"`、`a` の `[job.env] T_JENV = "SECRET"` | `pipeline demo --dry-run -v` / `run a --dry-run -v` / `run a --dry-run --assume a=1 --run-id r1` | OUT の `a` の項目に `env: T_TOKEN, T_PENV, T_JENV, CONDUCTOR_PIPELINE, …`(層の順。run ID があれば `CONDUCTOR_RUN_ID` を含む)。OUT・ERR に `SECRET` が現れない | `binary_dry_run::env と dotenv の値は 標準出力と標準エラーのどこにも無く 名前だけが出る`、`plan_report::環境変数の名前は 層の順で 最後に conductor の変数` |
| AC-493 | 遷移・`@end`・使われなかった仮定 | `P(p; a:0{on=success->c}, b:0, c:0)`、`--assume b=1` / `a:0{w=[3], on=warning->@end}, b, c`、`--assume a=3` | `pipeline p --dry-run …` | 前者: `assume: b=1`、`unused assumptions: b`、`2. [skip (transition)] b`、`result (simulated): completed`。後者: `1. [run (assumed: exit code 3 -> warning)] a`、`2. [skip (after @end)] b`、`3. [skip (after @end)] c`、`result (simulated): reached @end (completed)`(8 章 11.6 の P3、P4、P4e) | `plan_report`、`plan_assume::実行されないジョブの仮定は 使われなかった仮定になる 指定順` |
| AC-494 | 既定の 0 がエラーになるジョブ、`run` の次の動き | `a` が `s=[10]`、`b` / `a:0{on=success->c}, b, c, d`、`@end` への遷移 | `pipeline … --dry-run` / `run a --dry-run` | `1. [run (default: exit code 0 -> error)] a`、`2. [not run (after stop)] b`、`result (simulated): stopped (error: a)`。`run a` は、最後の行が `next step: next job c (skipped: b)`、`@end` なら `next step: reached @end (the rest are skipped when called)`、エラーなら `next step: stop`。実行済みの対象は次の動きの行を出さない(8 章 11.6、11.7) | `plan_report`、`plan_assume::成功コードに 0 を含まないジョブは 既定の 0 でエラーになる`、`plan_run` |
| AC-495 | リトライの対象の注記 | `a` に `[job.retry] exit_codes=[75,76], max=3, interval="5s"`、`--assume a=75` | `pipeline … --dry-run` | `[run (assumed: exit code 75 -> error, retry target)] a` と、`retry: exit codes 75, 76, up to 3 times, every 5 s` の行。模擬は、リトライを繰り返さない(1 つの終了コードを最終の結果として進める) | `plan_assume::リトライの対象の終了コードを仮定すると 注記される`、`plan_report::例 p2` |
| AC-496 | 完了したステートは「ステートなし」 | 完了したステート | `pipeline demo --dry-run` | `resume: from the start`、全ジョブ `[run]`(本番の `pipeline` と同じ扱い。`graph --state` は完了したステートの状態を重ねる。意図した違い) | `plan_pipeline`、`flow_dry_run` |
| AC-497 | `.env`・定義・ステートの誤り、拒否 | 各状況 | `pipeline demo --dry-run` / `run c --dry-run`(`a` が未実行) | `.env` が無い、定義の誤り、`command` の誤り → 64(計画は出ない、OUT 空)。ステートが壊れている、定義と食い違う → 65。`run c --dry-run` は 66、ERR = `conductor: error: job 'c': cannot run this job now: earlier jobs have not finished (out of order)`、OUT 空(本番と同じ表示) | `flow_dry_run::定義と command と env の誤りは 64 で 計画を出さない / ステートの食い違いと読めないステートは 65 / 順序外の run は 66 で 本番と同じ表示` |
| AC-498 | `--assume` と `--dry-run` の使い方の誤り | — | `pipeline demo --dry-run --assume b=300` / `--assume b` / `--assume b=x` / `--assume b=1 --assume b=2` / `--assume nojob=1` / `pipeline demo --assume b=1` / `watch --dry-run` / `pipeline demo --dry-run --dry-run` | すべて 68。順に `invalid value: '--assume'`(4 つ)、`job is not defined in the definition file`(対象なし。ジョブIDを表示しない)、`option not allowed for this command: '--assume'`、`option not allowed for this command: '--dry-run'`、`invalid value: '--dry-run'`。入力の値(`300`、`x`、`nojob`)が ERR に現れない。ジョブを起動しない | `flow_dry_run::assume の不正な入力は 68 で 誤りの表示に入力の値を含まない / assume だけと dry run の付ける場所の誤りは 68` |
| AC-499 | 本番の流れとの一致(注入) | 起動の部品の差し替え | 同じ定義・ステートで、`pipeline --dry-run` と、(すべて成功する)`pipeline` | 計画の `[run]` のジョブ = 本番が起動したジョブ(順序も)。新規と再開(エラーで止まったあと)の両方 | `flow_dry_run::計画の実行するジョブは 本番の流れが起動するジョブと一致する / 前回の停止からの再開も…`。区分: 注入 |
| AC-500 | シグナルの準備・ステート・出力 | 差し替え | `--dry-run`、`--run-id r1 --remove-on-complete --dry-run` | シグナルを準備しない。ステートは読むだけ(書かない、消さない。`--remove-on-complete` は効果なし)。計画の全体を、標準出力に 1 回で書く。書けなければ 65 | `flow_dry_run::dry run はジョブを起動せず ステートに書かず 消さず シグナルも準備しない / run id と remove on complete は併用でき ステートは読むだけ / 計画を出せたら エラーで停止する模擬でも 0 で 出力できなければ 65`。区分: 注入 |
| AC-501 | 1,000 ジョブの計画の性能 | すべてのジョブが `env` と `env_file` を持つ、ジョブ 1,000 個 | 計画の生成 | 1 秒以内(強制)。全ジョブが `[run]`、結果は `completed` | `plan_simulate_perf`。区分: 計測(強制) |
| AC-502 | README の `--dry-run` の説明 | README | (検査) | 見出し、版の表記、必須の語句があり、載っている計画の例が、実際の出力と一致する | `docs`。区分: 点検 |

### 4.20 `init`(v0.9.0 以降)

特に記さない限り、実行は、空のディレクトリ `T` で(`T` が実行ディレクトリ)。ひな形の全文は 8 章 12 章。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-503 | 既定の作成 | `T` が空 | `init` | 終了 0。ERR = `created conductor.toml`⏎。OUT 空。`T/conductor.toml` ができ、UTF-8、改行は LF(CR を含まない)、`name = "my-pipeline"` を含む。続く `validate` は 0。続く `pipeline my-pipeline --dry-run` は 0 で、計画の先頭が `plan (--dry-run): pipeline my-pipeline`、`[run] hello` と `[run] flaky` を含む。ステートを作らない | `binary_init::空のディレクトリで init すると validate に通り dry run の計画が出る`、`flow_init::既定の名前とパスで ひな形が書かれ 標準エラーに作成の 1 行` |
| AC-504 | 既存のファイルは壊さない | `T/conductor.toml` = `# 利用者の定義⏎` | `init` | 終了 69。ERR = `conductor: error: conductor.toml already exists (use --force to overwrite)`⏎。OUT 空。ファイルは変わらない。日本語は `conductor: error: conductor.toml は既にあります(上書きするには --force)` | `binary_init::もう一度 init すると 69 でファイルは変わらない`、`flow_init::既にあるときは 69 と 既にありますの 1 行で 標準出力は空` |
| AC-505 | `--force` | 同上 | `init --force nightly` | 終了 0。ERR = `replaced conductor.toml`⏎(日本語 `conductor.toml を置き換えました`)。`name = "nightly"`。続く `validate nightly` は 0 | `binary_init::force で上書きされ validate に通る` |
| AC-506 | `--config -`(標準出力) | `T` が空 | `init --config -` | 終了 0。OUT = ひな形の全文(AC-503 で作られるファイルと同じ内容)。ERR 空。ファイルを作らない。`--force` を足しても同じ | `binary_init::config ハイフンの出力は ファイルの内容と同じで ファイルを作らない`、`flow_init::config ハイフンは標準出力にひな形だけを出し force を無視し ファイルを作らない` |
| AC-507 | 名前のエスケープ | `T` が空 | `init --config d<i>.toml '<名前>'`、続けて `validate --config d<i>.toml '<名前>'`(名前: `a"b`、`a\b`、`夜間 バッチ`、`# x`) | すべて 0(TOML として正しい名前になる)。名前は、コメントには書かれない。`init --config missing/conductor.toml` は 65、ERR = `conductor: error: cannot create missing/conductor.toml: the parent directory does not exist`(親のディレクトリを作らない) | `binary_init::特殊な文字の名前で init したファイルも validate を通る`、`init_template::名前は toml の基本文字列としてエスケープされる / 名前は コメントに書かれない` |
| AC-508 | ひな形の全文(単体) | 言語、名前 | ひな形の生成 | 日本語と英語の全文(8 章 12 章)と、1 文字も違わず一致する。UTF-8、LF、末尾に改行 1 つ。どの言語でも、定義として読め、`command` も分けられる。`[pipeline.env]` と `[job.retry]` の例は、有効な行。名前以外の利用者の値は入らない | `init_template::日本語と英語のひな形は 原本と 1 文字も違わず一致する / ひな形は utf8 で lf だけで 末尾に改行が 1 つ / どのひな形も 定義として読め command も分けられる / ひな形に 名前以外の利用者の値は入らない`。区分: 単体 |
| AC-509 | 作れないときの表示(注入) | ファイル作成の部品の差し替え | `init` | `parentMissing` → 終了 65、ERR = `conductor: error: cannot create conductor.toml: the parent directory does not exist`。`isDirectory` → `…: it is a directory`。`permissionDenied` → `…: permission denied`。`otherIo` → `…: I/O error`。日本語は `conductor: error: conductor.toml を作れません: 親のディレクトリがありません` / `ディレクトリです` / `書く権限がありません` / `入出力の誤りです`。OS のエラーの文は含まない。`--config -` で標準出力に書けない → 65、ERR = `conductor: error: cannot write the template to standard output` | `flow_init::作れないときは 65 と 種類ごとの固定の文で os の文を含まない / 標準出力に書けないときは 65`、`report_messages::init のメッセージは 仕様の表のとおり`。区分: 注入 |
| AC-510 | ファイルの作成の規則(単体) | 一時ディレクトリ | ファイルの作成 | 新規: 内容がそのまま(LF のまま)。既存は壊さず `alreadyExists`。宙に浮いたシンボリックリンクは `alreadyExists`(`force` ならリンクそのものを置き換える)。`force`: 内容が新しくなり、一時ファイルが残らない。親が無い → `parentMissing`(作らない)。作る先がディレクトリ → `isDirectory`。書く権限がないディレクトリ → `permissionDenied`(UNIX 系、非 root)。同時に作る 2 つのうち、ちょうど 1 つだけが `created`(もう 1 つは `alreadyExists`) | `file_create::新規に作る 内容は書いたとおりで 改行は lf のまま / 既にあるファイルは壊さない / force で置き換える… / 同時に作る 2 つのうち ちょうど 1 つだけが created になる`。区分: 単体 |
| AC-511 | 引数の文法 | — | `init -v` / `init --run-id x` / `init --all` / `init a b` / `validate --force` / `init --force --force` / `init ''` / `init '  '` | すべて 68。順に `option not allowed for this command: '-v'`、`'--run-id'`、`'--all'`、`unexpected argument: 'b'`、`option not allowed for this command: '--force'`(`validate`)、`invalid value: '--force'`、`invalid value: 'pipeline name'`(2 つ)。ファイルに触れない(引数の誤りは、ファイルを作る前に返る) | `args_grammar`、`flow_init::引数の誤りは ファイルに触れる前に 68` |
| AC-512 | 何も読まず、シグナルも準備しない(注入) | 差し替え | `init` | 定義、`.env`、ステートを読まない。シグナルを準備しない。成功は標準エラーの 1 行だけ | `flow_init::定義と環境ファイルとステートを読まず シグナルも準備しない`。区分: 注入 |
| AC-513 | README の `init` の説明 | README | (検査) | 見出し、版の表記、終了コード 69 の記述、必須の語句があり、載っているひな形の例が、実際の日本語のひな形と一致する | `docs`。区分: 点検 |

### 4.21 パイプライン共通のリトライの既定と `retry = false`(v0.10.0 以降)

規定は 03 の 7.5・7.8 と 05 の 7 章。特に記さない限り、`PR` は `[pipeline]` の下の `[pipeline.retry]` に `exit_codes = [75]`、`max = 3`、`interval = "1s"` を書いたもの(既定のリトライ)。`retry=false` は、ジョブに `retry = false` を書いたもの。`flaky` と `F` は 4.9 節と同じ。誤りの期待は、終了 64、ERR の最初の行が `conductor: error: conductor.toml:<行>: <種類の文>: <理由>`(4.1 節)。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-519 | 既定のリトライの当てはめ | `P(p; a:0{cmd="TJ flaky F 2 75"})` に `PR`(`a` は `retry` を書かない) | `run a` / `pipeline p` | どちらも 0。ERR = `conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 1 seconds`⏎`conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 1 seconds`(-v なしでも出る)。`F` の値 = 3。ステート `a=success(0)`。既定はインラインの表(`retry = { exit_codes = [75], max = 3, interval = "1s" }`)でも書け、`multiplier`・`max_interval` も読める | `flow_retry::既定 retry を書かないジョブは既定の終了コードでリトライして成功する`、`core::definition_retry::br1_2 既定を読み retry を書かないジョブに既定が入る`、`core::definition_retry::br1_2 既定の倍率と上限も読める`、`core::definition_retry::br1_2 既定はインラインテーブルでも書ける` |
| AC-520 | `retry = false` は既定を使わない | `P(p; a:0{cmd="TJ flaky F 99 75", retry=false}, b:0)` に `PR` | `run a` / `pipeline p` | どちらも 75。retry の行なし。`F` の値 = 1。`pipeline p` は `a` で止まり、`b` を実行しない(`resume=a`)。既定の無い定義の `retry = false` も誤りにならない(`validate` は 0、リトライしない) | `flow_retry::既定 retry が false のジョブは既定があってもリトライしない`、`flow_retry::既定 pipeline では既定のジョブはリトライし false のジョブはそこで止まる`、`core::definition_retry::br2_2 retry が false のジョブは既定があっても none`、`core::definition_retry::br2_3 既定が無くても retry が false は誤りにならず none` |
| AC-521 | `[job.retry]` は既定を丸ごと置き換える | `PR` に `multiplier = 2`、`max_interval = "10m"` を足す。ジョブ `a` は `[job.retry]` に `exit_codes = [9]`、`max = 5`、`interval = "2s"`、ジョブ `b` は `retry` を書かない | (定義の読み取り) | `a` の実効のリトライは、自分の表だけ(`exit_codes = [9]`、`max = 5`、`interval` 2 秒、倍率と上限なし。既定の倍率を混ぜない)。`b` は既定(倍率と上限つき)。既定を当てはめたジョブは、同じ値の表を書いたジョブと同じ定義になる | `core::definition_retry::br2_2 ジョブの表があれば丸ごと使い既定の倍率を混ぜない`、`core::definition_retry::br2_4 既定を当てはめたジョブは同じ値の表を書いたジョブと同じ定義になる`。区分: 単体 |
| AC-522 | `[pipeline.retry]` の検査の誤り | 各行の定義(`validate`) | `validate` | `[pipeline]` の `retry = false`(`true`、`3`、`"x"` も)→ `retry must be a table`(理由 ``` `[pipeline.retry]` はテーブルで書きます ```、`retry` の行)。`[pipeline.retry]` の必須のキーの欠け・範囲の誤り・未知のキー → ジョブの表と同じ種類の文(4.1 節)で、既定の表の中の位置。`interval = "10m"`、`multiplier = 2`、`max_interval = "5m"` の既定は、ジョブが 3 つでも `retry max_interval is smaller than interval` が 1 行だけ。`PR` で、`retry` を書かない `a:0{w=[75]}` と `b:0{s=[0,75]}` → `retry exit_codes overlap the success or warning exit codes` が、`a` と `b` のそれぞれの `[[job]]` の行に 1 行ずつ(理由 ``` 既定のリトライ(`[pipeline.retry]`)の `exit_codes` に、このジョブの成功または警告の終了コードと同じ値があります ```)。`retry=false` のジョブと、自分の表を書いたジョブは、既定と比べない。`name = ""` でも、既定の誤りと既定との重なりの誤りは報告する。`run` も同じ文で 64、値は ERR に現れない | `core::definition_retry::br1_1 pipeline の retry が表でない`、`core::definition_retry::br1_2 既定の誤りはジョブの表と同じ種類で既定の中の位置`、`core::definition_retry::br3_5 既定の上限が間隔より小さい誤りはジョブの数によらず 1 回`、`core::definition_retry::br3_1_br3_2 既定との重なりはジョブごとにそのジョブの行で報告する`、`core::definition_retry::br3_3 false のジョブと自分の表のジョブは既定と重なっても誤りにならない`、`core::definition_retry::name が空でも既定の誤りと既定との重なりの誤りが出る`、`binary_basic::validate は既定のリトライとの重なりで 64 になり 値を出さない`、`flow_validate::既定のリトライとの重なりは 64 で run と同じ誤りの文` |
| AC-523 | ジョブの `retry` の値の型の誤り | `B0` の `a` に `retry = true`(または `retry = 5`、`retry = "x"`) | `validate` | 64。`retry must be a table`(行は `retry` の値の行)。理由は ``` `retry` はテーブルか `false` で書きます ```(v0.9.x までは ``` `retry` はテーブルで書きます ```)。日本語の種類の文も 08 の 4.3 のとおり | `core::definition_retry::br2_1 ジョブの retry が true なら表か false で書く誤り`、`core::definition_retry::br1_2 retry が表でない` |
| AC-524 | 計画と `graph -v` に当てはめた既定が表れる | `P(p; a:0)` に `PR`(`a` は `retry` を書かない)と、同じ値の `[job.retry]` を `a` に書いた定義 | `pipeline p --dry-run` / `graph -v` | どちらの定義でも、OUT が 1 文字も違わない(計画の `a` の項目と、`graph -v` の `a` の詳細の行に `retry: exit codes 75, up to 3 times, every 1 s`) | `plan_report::既定のリトライを当てはめたジョブの計画は同じ値の表を書いたときと一文字も違わない`、`graph_report::v は既定のリトライを当てはめたジョブを同じ値の表を書いたジョブと同じに出す`。区分: 単体 |

### 4.22 無くてもよい `.env`(`env_file` の表)(v0.10.0 以降)

規定は 03 の 7.6・7.7 と 05 の 14.5。表記は 4.17 節と同じ(ジョブ `show` が子の環境を出す。`SECRET` が OUT・ERR に現れないことも確かめる)。「同じ表示」は、`T` のパスの違いを除いて、OUT・ERR がバイト列で同じこと。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-525 | 無くてもよい `.env` が無ければ読み飛ばす | 定義 A: `[pipeline] env_file = ["p.env", { path = "local.env", required = false }]`、`show` に `env_file = { path = "nodir/j-local.env", required = false }`。定義 B: A から無くてもよい 2 つを除いたもの(`env_file = "p.env"`)。どちらも `p.env` = `T_P=p` だけを置く(`local.env` と `nodir/` は無い)。`show` は `T_P T_LOCAL T_JLOCAL` を出す | `run show` / `pipeline p` / `validate` / `pipeline p --dry-run` | 4 つとも、A は 0 で、OUT・ERR が B と同じ表示。ERR に `local.env` が現れない(誤りにも警告にもしない)。`run show` の OUT は `T_P` = `"p"`、`T_LOCAL`・`T_JLOCAL` = unset。親のディレクトリが無いときも「無い」として扱う | `binary_env::無くてもよいファイルが無ければ 4 つのコマンドは 0 で出力はそのファイルを書かない定義と同じ`、`flow_env::無くてもよいファイルが無ければ run と pipeline は 0 でその変数は渡らず何も表示しない`、`runtime::env_loader::無いファイルは必ず要るなら file_missing 要らないなら誤りにせず引き当てにも載せない`、`runtime::env_loader::親のディレクトリが無いときも無いとして扱う` |
| AC-526 | 既定(必須)と文字列の形は今までどおり | `env_file = "a.env"` / `env_file = { path = "a.env" }` / `env_file = { path = "a.env", required = true }`(`a.env` は無い)。`[pipeline]` と `show` のどちらに書いても、表 1 つでも配列の要素でもよい | `validate` / `run show` | 3 つの書き方は同じ意味(必ず要る)。`a.env` が無ければ 64(`fileMissing`。種類の文 `environment file does not exist`)。`a.env` があれば、その項目が渡る | `core::env_definition::env_file の表は path と required を読み required を省くと必ず要る`、`core::env_definition::env_file は表 1 つでも書け pipeline と job の両方で読める`、`runtime::env_loader::無いファイルは必ず要るなら file_missing 要らないなら誤りにせず引き当てにも載せない` |
| AC-527 | 同じパスは、どこかで必ず要るなら必ず要る | `[pipeline] env_file = { path = "x.env", required = false }`、`show` に `env_file = "x.env"`(`x.env` は無い)/ 同じパスを別の書き方(`./x.env` など)で書いたもの | `run show` / `pipeline p` / `validate` | 64(`fileMissing` が 1 行。同じパスは 1 回だけ読み、1 回だけ報告する)。ジョブを起動せず、ステートを作らない | `runtime::env_loader::同じパスに解決される記述はどれか 1 つが必ず要るなら必ず要る`、`flow_env::同じファイルをどこかで必ず要るとしていれば無いとき 64` |
| AC-528 | 表の誤り | 各行の `env_file`(`[pipeline]` か `show` に書く)| `validate` | 64。`{ path = "" }` → `required value is empty`(理由 ``` `env_file` に空の文字列は書けません ```)。`{ required = false }` → `wrong value type`(理由 ``必須の値 `path` がありません``、表の先頭の行)。`{ path = 1 }` → `wrong value type`(``` `path` は文字列で書きます ```)。`{ path = "a", required = 1 }` → `wrong value type`(``` `required` は真偽値(`true` または `false`)で書きます ```)。`{ path = "a", x = 2 }` → `unknown key`。`[{ path = "" }, true, { path = "a", required = 1, x = 2 }]` → 位置の順に `required value is empty`、`wrong value type`、`wrong value type`、`unknown key` の 4 行。`env_file = 5` → `wrong value type`(理由 ``` `env_file` は、文字列かテーブル、またはその配列で書きます ```。v0.9.x までは ``` `env_file` は、文字列、または文字列の配列で書きます ```) | `core::env_definition::env_file の表の誤りは固定の理由で返す` |
| AC-529 | あるが読めない・誤りのある `.env` は、無くてもよい指定でも 64 | `env_file = { path = "local.env", required = false }` と、同じパスの `env_file = "local.env"` の 2 つの定義。`local.env` は、`A=SECRET` のあとに不正なバイト列(UTF-8 でない)/ ディレクトリ / 書式の誤り | `run show` / `pipeline p` / `validate` / `pipeline p --dry-run` | 4 つとも 64、OUT 空。ERR は、必ず要る定義と同じ表示(`local.env` を含み、`SECRET` を含まない)。ステートを作らない。誤りの並びは記述の出現順で、読み飛ばしたファイルは入らない | `binary_env::無くてもよいファイルでもあって utf8 でなければ 64 で必ず要るときと同じ表示`、`runtime::env_loader::必ず要らないファイルでもあって読めないものは今までどおりの誤り`、`runtime::env_loader::誤りの並びは記述の出現順で読み飛ばしたファイルは入らない` |

### 4.23 `unset_env`(v0.10.0 以降)

規定は 03 の 7.7(`.env` との重なり)・7.10、05 の 14 章、07 の 4.1・4.2。表記は 4.17 節と同じ。「環境」の欄の変数は、被試験体を起動した側(親)に置く。誤りの期待の 64 は、ジョブを 1 つも起動せず、ステートファイルを作らないことを含む。値(`SECRET`)が OUT・ERR に現れないことも確かめる。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-530 | 引き継いだ環境から名前を消す | `[pipeline] unset_env = ["T_INHERITED"]`、`show` に `unset_env = ["PATH"]` と `[job.env] T_JENV = "je"`。環境: `T_INHERITED=from-parent`、`T_OVERRIDDEN=from-parent`。`show` は `T_INHERITED PATH T_OVERRIDDEN T_JENV` を出す | `run show` / `pipeline p` | どちらも 0。`T_INHERITED` = unset、`PATH` = unset、`T_OVERRIDDEN` = `"from-parent"`(消さない名前は残る)、`T_JENV` = `"je"`(消したあとに足す)。親に無い名前を消しても何も起きない。名前の比べ方は OS に従う(UNIX 系は完全一致、Windows は大文字小文字を区別しないので、親の `Path` も消える) | `binary_env::unset_env の名前は親から引き継いだ環境から消えてほかは残る`、`runtime::runner_basic::消す名前は引き継いだ環境から除かれる`、`runtime::runner_basic::消す名前で引き継いだ環境に無い名前は何もせずほかの変数は残る`、`runtime::runner_basic::消してから足した名前は足した値になる`、`runtime::runner_basic::大文字小文字だけが違う名前の消し方は os に従う` |
| AC-531 | パイプラインとジョブの `unset_env` を合わせる | `[pipeline] unset_env = ["HTTP_PROXY", "A", "HTTP_PROXY"]`、ジョブに `unset_env = ["a", "B"]` / どちらにも書かない / `unset_env = []` | (起動に渡る消す名前の並び) | 消す名前の並びは、パイプラインの名前のあとにジョブの名前、書いた順。完全に同じ名前は最初の 1 つだけ(`HTTP_PROXY`、`A`、`a`、`B`)。大文字小文字だけが違う名前(`A` と `a`)は別の名前として残す。書かない・空の配列は、消す名前が空で、足す環境変数の並びは v0.9.x と同じ | `core::env_merge::removals はパイプラインの次にジョブの順で完全に同じ名前は最初の 1 つ`、`core::env_merge::unset_env を書かなければ removals は空で entries は今と同じ`、`core::env_definition::unset_env を pipeline と job で書いた順に読み同じ名前は最初の 1 つにまとめる`、`flow_env::run_job に消す名前がパイプラインの次にジョブの順で渡る`。区分: 単体、注入 |
| AC-532 | `env` との重なり(`unsetEnvConflict`) | `[pipeline] unset_env = ["T_TOKEN"]` と、`show` の `[job.env] t_token = "SECRET"`(大文字小文字だけが違う) | `run show` / `pipeline p` / `validate` / `pipeline p --dry-run` | 4 つとも 64、OUT 空。ERR の最初の行は `conductor: error: conductor.toml:<unset_env の要素の行>: a name in unset_env is also set by env: ` に続く理由(``` `unset_env` の名前が、`env` の名前と重なっています ```)。名前ごとに 1 行。`[pipeline].unset_env` は `[pipeline.env]` とすべてのジョブの `[job.env]` と比べ、ジョブの `unset_env` は `[pipeline.env]` とそのジョブの `[job.env]` だけと比べる(ほかのジョブの `env` とは比べない) | `core::env_definition::unset_env と同じジョブに当てはまる env の重なりは大文字小文字を区別せずに誤り`、`binary_env::unset_env と env や env の重なりは 64 で値が出ずジョブもステートも作らない`、`flow_validate::unset_env と env や env の重なりは 64 で run と同じ誤りの文` |
| AC-533 | `.env` との重なり(`unsetConflict`) | `[pipeline] env_file = "p.env"`、`show` に `unset_env = ["T_TOKEN"]`。`p.env` = `A=1⏎T_TOKEN=SECRET` | `run show` / `pipeline p` / `validate` / `pipeline p --dry-run` | 4 つとも 64、OUT 空。ERR に `p.env:2: name is also listed in unset_env` の行(`.env` の誤りの形。08 の 4.12)。同じファイルを複数のジョブが読んで重なっても、「ファイルと行」で 1 行。調べる範囲は読む範囲と同じ(`run` はそのジョブと `[pipeline]` の分。`[pipeline].unset_env` は、全ジョブの `env_file` と比べる)。`required = false` で無かったファイルは調べない | `binary_env::unset_env と env や env の重なりは 64 で値が出ずジョブもステートも作らない`、`core::env_merge::env との重なりはファイルと行の誤りで同じファイルと行は 1 つにまとめる`、`core::env_merge::重なりの範囲は run ならそのジョブだけ`、`core::env_merge::パイプラインの unset_env は全ジョブの env_file と比べ 重ならなければ空`、`flow_env::env の名前と消す名前の重なりは 64 で範囲は読み取りの範囲`、`flow_env::同じ env を 2 つのジョブが読んで重なっても誤りは 1 つ` |
| AC-534 | 名前と型の誤り | 各行の `unset_env`(`[pipeline]` か `show` に書く) | `validate` | 64。`unset_env = "A"` → `wrong value type`(理由 ``` `unset_env` は、文字列の配列で書きます ```)。`[1]` → `wrong value type`(``` `unset_env` の要素は、文字列で書きます ```)。`[""]` → `required value is empty`。`["1A"]`、`["A-B"]` → `invalid environment variable name`。`["CONDUCTOR_X"]`、`["conductor_x"]` → `reserved environment variable name`。理由は `unset_env` の文(03 の 7.10)で、値を含めない | `core::env_definition::unset_env の型の誤りは固定の理由で返す`、`core::env_definition::unset_env の空の名前と規則違反と予約名は誤りで理由は unset_env の文`、`core::env_definition::誤りの表示に値が含まれない` |
| AC-535 | 計画に消す名前が表れる | `[pipeline] unset_env = ["T_INHERITED"]`、ジョブに `unset_env = ["T_OVERRIDDEN"]`。環境: `T_INHERITED=from-parent`、`T_OVERRIDDEN=from-parent` | `pipeline p --dry-run` | 0。ジョブの項目の、環境変数の行のあとに `     unset env: T_INHERITED, T_OVERRIDDEN`(5 つの空白。日本語は `消す環境変数: `)。消す名前の無いジョブと、実行しないジョブには、この行を出さない。親の値(`from-parent`)は OUT・ERR に現れない。ジョブを起動せず、ステートを作らない | `binary_env::dry_run は消す名前を出し親の値を出さない`、`plan_report::消す名前のあるジョブだけ 環境変数の行の後に消す環境変数の行を英日で出す`、`plan_report::消す名前の行は実行しないジョブには出さない`、`flow_dry_run::dry_run は消す名前の行を出し ジョブを起動しない`、`flow_dry_run::dry_run も env と消す名前の重なりは 64 で計画を出さない` |

### 4.24 `graph --format dot`(v0.10.0 以降)

規定は 04 の 7.7、05 の 15.8、08 の 11.8。特に記さない限り、表記と期待は 4.18 節と同じ(`LANG=C`、終了 0、ERR 空、ステートファイルを作らない、ジョブを起動しない)。`G2` は 4.18 節の G2 の定義 `P(p; a:0{on=success->d}, b:0, c:0, d:0, e:0)`。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-536 | DOT の全文(G2) | `G2` | `graph --format dot` / `graph --format dot -v` | どちらも OUT = `digraph conductor {`⏎`  rankdir=TB;`⏎`  n0 [label="a", shape=box];`⏎`  n1 [label="b", shape=box];`⏎`  n2 [label="c", shape=box];`⏎`  n3 [label="d", shape=box];`⏎`  n4 [label="e", shape=box];`⏎`  nend [label="@end", shape=oval];`⏎`  n0 -> n1 [label="warning"];`⏎`  n0 -> n3 [label="success"];`⏎`  n1 -> n2;`⏎`  n2 -> n3;`⏎`  n3 -> n4;`⏎`  n4 -> nend;`⏎`}`(各行の末尾は改行。`-v` は DOT に影響しない)。日本語はラベルの語が `"警告"`・`"成功"`(08 の 11.8 と 1 文字も違わない) | `binary_graph::format_dot の出力は例と一致し v を付けても同じで ジョブは動かない`、`graph_dot::g2 の全文は mermaid と同じ節と辺を dot の記法で書く` |
| AC-537 | `--state` の記号と再開位置の太い枠 | `G2` の `d` を `d:1` にして `pipeline p`(1 で止まる。`a` 成功、`b`・`c` スキップ、`d` エラー) | `graph --format dot --state` | 0。節の定義が `  n0 [label="[OK] a", shape=box];`⏎`  n1 [label="[--] b", shape=box];`⏎`  n2 [label="[--] c", shape=box];`⏎`  n3 [label="[NG] d", shape=box, penwidth=3];`⏎`  n4 [label="[  ] e", shape=box];`。`penwidth=3` は再開位置の 1 つだけ。ステートファイルのバイト列は変わらない。`--state` なしでは、状態の記号も太い枠も出ない | `binary_graph::format_dot と state は 状態の記号と再開位置の太い枠を出す`、`graph_dot::state のときは mermaid と同じ記号で 再開位置だけが太い枠`、`graph_dot::state でなければ 記号も太い枠も無い` |
| AC-538 | ラベルのエスケープと、ジョブ ID のほかを書かないこと | ジョブ ID に `q"1`、`b\s`、`x<LF>y`(TOML で `"x\ny"`)を含む定義 / `command` とパイプライン名に目印の文字列を含む定義 | `graph --format dot`(`-v`、`--state` も) | ラベルは、まず 08 の 9 章の見える形にしてから、`\` を `\\`、`"` を `\"` にする(`q"1` → `q\"1`、`b\s` → `b\\s`、`x<LF>y` → `x\\ny`)。各行が決まった形(`n<k> [label="…", shape=box];` か `<元> -> <行き先>[ [label="…"]];`)で、文書が壊れない。節の ID は連番で、ジョブ ID はラベルにだけ書く。OUT・ERR に、`command` とパイプライン名の目印が現れない | `graph_dot::escape_dot_label の入力と出力の対の表`、`graph_dot::引用符と バックスラッシュと 改行を含むジョブ id でも 各行が決まった形になる`、`binary_graph::format_dot は コマンドとパイプライン名を出さない` |
| AC-539 | Mermaid と同じ節と辺、ラベルの語、使い方と誤り | 分岐・飛び越し・`@end` を含む定義 / 各行の引数 | `graph --format dot` と `graph --format mermaid` / `graph --format svg` / `graph --format dot --format text` / 誤りのある定義で `graph --format dot` | DOT のジョブの節と辺の数は、Mermaid と同じ(行き先の分からない辺は、どちらも出さない)。主線の辺はラベルなし、成功だけ・警告だけ・両方の辺は `"success"`・`"warning"`・`"success, warning"`(日本語 `"成功"`・`"警告"`・`"成功・警告"`)。`--format svg` → 68、`invalid value: '--format'`(`svg` は ERR に現れない。使い方に `--format <text\|mermaid\|dot>`)。重複も 68 の `invalid value: '--format'`。誤りのある定義は、`--format text` と同じ ERR と終了コード。標準出力への書き込みは 1 回で、DOT の全体 | `graph_dot::分岐と飛び越しと終点を含む定義で ジョブの節と辺の数が mermaid と同じ`、`graph_dot::行き先の分からない辺は mermaid と同じく出さない`、`graph_dot::主線の辺はラベルなしで 成功だけ 警告だけ 両方の辺は語のラベル 英日`、`binary_graph::format の値が text mermaid dot の外なら 68 で 値は表示しない`、`args_grammar::graph の format は text か mermaid か dot だけで 重複と値の欠けは誤り`、`flow_graph::format_dot は 0 で終わり 標準出力にdotを1回で書き v で変わらない`、`flow_graph::誤りのある定義は format_dot でも text と同じ標準エラーと終了コード` |
| AC-540 | 1 万ジョブの描画の性能 | 直線の 1 万ジョブの図(ジョブ `i` から `i+1` へ、最後は `@end`。再開位置は真ん中の 1 つ) | DOT の描画 | 1 秒以内(強制)。行の数 = 節(1 万 + 終点)+ 辺(1 万)+ 3 = 20,004。`penwidth=3` は 1 つ。最後の辺は `  n9999 -> nend;` | `graph_dot::一万のジョブの直線の図も 時間の上限の内に描け 行の数が期待どおり`。区分: 計測(強制)、単体 |

### 4.25 ジョブの出力をファイルに残す(`log_dir`)(v0.10.0 以降。v0.10.1 の名前の空白・読む前の印を含む)

規定は 03 の 7.9、05 の 19 章、07 の 4.1・4.2・4.5、08 の 4.14。特に記さない限り、定義の `[pipeline]` に `log_dir = "logs"` を書き、`L` はそのディレクトリ(`T/logs`)。`<時刻>` は試行の開始の時刻の詰めた形 `YYYYMMDDTHHMMSSZ`(UTC。実行の前後に取った時刻のあいだ)。ログの 3 つの文は 08 の 4.14 の文で、`<path>` はそのファイルのパス。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-541 | ファイルの中身はジョブの出力だけ | `P(p; a:0{cmd="TJ say hello oops"})` | `pipeline p` | 0。OUT = `hello`、ERR = `oops`(端末には今までどおり出る)。`L` にファイルが 1 つ。中身は `hellooops` か `oopshello`(2 つの流れを読んだ順に 1 つのファイルへ書く。各流れの中の順は保つ。`conductor` は何も書き足さない) | `binary_log::fr5_2 ファイルの中身はジョブの出力で 名前は安全な id 時刻 試行の番号`、`runtime::runner_log::br4_1 標準出力と標準エラーの両方がファイルに残り 各流れの中の順を保つ`、`runtime::runner_log::br4_2 ファイルにはジョブの出力だけを書く` |
| AC-542 | ファイルの名前 | ジョブID `a/b` の `P(p; a/b:0{cmd="TJ say hello oops"})` | `pipeline p` | ファイルの名前は `a_b.<時刻>.1.log`(`<安全な ID>.<時刻>.<試行の番号>.log`)。`<時刻>` は、ステートのその試行の `started_at` と同じ時刻。日本語のジョブID(`夜間の処理`)や大きな試行の番号(101)でも作れる | `binary_log::fr5_2 ファイルの中身はジョブの出力で 名前は安全な id 時刻 試行の番号`、`flow_run::br3_1 log_dir があれば 定義のディレクトリで解決した指定を試行の時刻とともに渡す`、`runtime::log_file::br3_1 ディレクトリを親を含めて作り 名前の形で作る`、`runtime::log_file::日本語の id と 試行の番号の大きな値でも作れる` |
| AC-543 | 安全な ID | ジョブID と、ログの名前の部分(安全な ID) | 安全な ID を求める関数 | `a/b`・`a\b` → `a_b`、`../x` → `_._x`、`.env` → `_env`、`..` → `__`、`a.` → `a_`、`a ` → `a_`、`a.b` → `a.b`、`CON` → `_CON`、`con` → `_con`、`com1` → `_com1`、`NUL.x` → `_NUL.x`、`console`・`CONX`・`COM0`・`x.CON` → そのまま、`a:b*c?d"e<f>g\|h` → `a_b_c_d_e_f_g_h`、制御文字(U+0000〜U+001F、U+007F〜U+009F)→ `_`、`日本語` → そのまま。(v0.10.1 以降)予約名と比べるときだけ前後の空白を除く: `con ` → `_con_`、` con` → `_ con`、`con .x` → `_con .x`、`nul .tar.gz` → `_nul .tar.gz`。100 バイトまでに切り、UTF-8 の文字の途中では切らない(`あ` × 70 → `あ` × 33)。切った後の末尾の `.` も `_` に。どの入力(`../../etc/passwd`、`C:\x` など)でも、`/`・`\` が残らず、`.` で始まらない。名前の全体は 255 バイトを超えない | `runtime::log_file::br3_2 安全な名前の対の表`、`runtime::log_file::br3_2 末尾の点は 置き換えた後に見る`、`runtime::log_file::br3_2 長い id は 100 バイトまでで 文字の途中で切らない`、`runtime::log_file::nfr6_3 置き換えた名前にディレクトリの区切りが残らない`、`runtime::log::tests::名前の最長でも 255 バイトを超えない`。区分: 単体 |
| AC-544 | UNIX 系はファイルの権限が 0600 | `P(p; a:0)` | `run a` | `L` のファイルの権限が `0600`(番号つきの名前も同じ)。Windows は何もしない | `binary_log::br3_4 unix 系のログのファイルは 0600`、`runtime::log_file::br3_4 unix 系のファイルの権限は 0600`。区分: UNIX 系 |
| AC-545 | リトライで試行ごとのファイル | `P(p; flaky:0{cmd="TJ say try- '' 75", retry=(exit_codes=[75], max=2, interval="1s")})` | `run flaky` | 75。OUT = `try-try-try-`。`L` に `flaky.<時刻>.1.log`、`flaky.<時刻>.2.log`、`flaky.<時刻>.3.log` の 3 つ(試行の番号は `CONDUCTOR_ATTEMPT` と同じ)。各ファイルの中身は `try-` | `binary_log::fr5_3 リトライで試行ごとに 3 つのファイルができる`、`flow_retry::br2_3 リトライの試行ごとに時刻を取り 試行の番号が増える`、`flow_pipeline::br3_1 各ジョブの試行ごとにログの指定と時刻を渡す` |
| AC-546 | ディレクトリの作成とパスの基準 | 定義 `T/sub/c.toml` に `log_dir = "logs"` / 絶対パスの `log_dir` / `log_dir = "a/b/logs"`(無い)/ ジョブID `../x/y` / 同じ名前のファイルがすでにある | `T` で `run a --config sub/c.toml` ほか | 相対の `log_dir` は定義ファイルのディレクトリが基準(`T/sub/logs`。実行ディレクトリではない)、絶対パスはそのまま、`~` と変数は展開しない。無いディレクトリは親を含めて作る。ファイルは `log_dir` の直下にできる(`_._x_y.<時刻>.1.log`)。同じ名前があれば `…1-2.log`、`…1-3.log` と番号を足して作り、元のファイルは上書きも追記もしない。`conductor` はログのファイルを消さない(`reset` も触れない) | `flow_run::br3_1 log_dir があれば 定義のディレクトリで解決した指定を試行の時刻とともに渡す`、`flow_run::br3_1 絶対パスの log_dir はそのまま渡す`、`runtime::log_file::br3_1 ディレクトリを親を含めて作り 名前の形で作る`、`runtime::log_file::br3_1 ファイルは log_dir の直下にできる`、`runtime::log_file::br3_3 同じ名前があれば番号を足し 元のファイルは変わらない`、`runtime::runner_log::br3_1 ディレクトリが無ければ作ってから起動する`(`reset` は派生。07 の 4.5) |
| AC-547 | 作れなければ起動せず 127 | `log_dir = "conductor.toml"`(定義ファイルそのもの。ディレクトリを作れない)。`a` は `cmd="TJ --out O run a 0"` | `run a` | 127。OUT 空。ERR はちょうど 1 行で `conductor: error: cannot create the log file: <T>/conductor.toml/a.<時刻>.1.log`(番号なしの名前のパス。日本語は `conductor: error: ログのファイルを作れません: <path>`)。`O` ができない(ジョブは起動しない)。`127` を `exit_codes` に持つリトライでもリトライしない。リトライの 2 回目以降の試行で作れなければ、その試行の 1 行で終わる。`pipeline` は、そのジョブで 127 で止まる。`-1000` まで使い切ったときも、番号なしの名前で作れない | `binary_log::br5_1 ログを作れなければ起動せず 127 と 1 行`、`flow_run::br5_1 ログを作れなければ 127 で ログの 1 行だけを出す`、`flow_run::br5_1 ログを作れないときの日本語の 1 行`、`flow_retry::br5_1 ログを作れない起動の失敗は 127 が一覧にあってもリトライしない`、`flow_retry::br5_1 リトライの後の試行でログを作れなければ その試行の 1 行で終わる`、`flow_pipeline::br5_1 途中のジョブでログを作れなければ 127 で止まり 1 行だけ`、`runtime::runner_log::br5_1 ログを作れなければ起動せず 作れないパスを返す`、`runtime::log_file::br5_1 ディレクトリを作れなければ 番号なしの名前を返す`、`runtime::log::tests::番号を使い切ったら作らず 番号なしの名前を返す` |
| AC-548 | 端末の出力は同じ(16 MiB を含む) | `P(p; one:0{cmd="TJ say out-1 err-1"}, two:0{cmd="TJ say out-2 err-2 3"})` の、`log_dir` のある定義と無い定義 / `P(p; big:0{cmd="TJ flood 16777216"})`(16 MiB) | `pipeline p` / `run big` | 前者: どちらも 3、OUT = `out-1out-2`、ERR = `err-1err-2` で、`log_dir` のあり・なしで同じ。`log_dir` の無い定義はディレクトリを作らない。後者: OUT は 16,777,216 バイトで `flood` の並びと一致し、ログのファイルの中身も同じ並び(強制。時間の基準なし)。端末への書き込みに失敗しても、読み続けてファイルへ書く | `binary_log::fr5_5 log_dir のあり なしで端末の出力が同じ`、`binary_log::nfr1_3 16 mib の出力でも端末とファイルが一致する`、`runtime::runner::tests::読んだ分をそのまま端末とファイルへ書き 端末は書くたびに flush する`、`runtime::runner::tests::端末への書き込みに失敗しても読み続けてファイルへ書く` |
| AC-549 | `-v` の行はジョブの出力の後 | `P(p; a:0{cmd="TJ say A-out A-err"}, b:0{cmd="TJ say B-out B-err"})` | `pipeline -v p` | 0。ERR = `A-err[conductor] job=a result=success (exit code 0)`⏎`B-err[conductor] job=b result=success (exit code 0)`⏎(ジョブの出力を写し終えてから、結果の行を出す)。OUT = `A-outB-out` | `binary_log::nfr1_9 v の行はジョブの出力の後に出る` |
| AC-550 | 出力を持ち続ける子の子、書けないとき、「止める」印 | `cmd="TJ orphan 30"`(子の子が 30 秒、出力を持ち続ける)/ ファイルへの書き込みが途中で失敗する(注入)/ 読み取りの処理の「止める」印 | `run a` ほか | 子の子があっても、ジョブの終了の後、最大 2 秒で読み取りを打ち切って戻り(10 秒以内)、試行の後に `conductor: warning: output continued after the job ended; the rest is not in the log file: <path>`。書き込みの失敗は、以後ファイルに書かず(端末には書き続ける)、試行の後に `conductor: warning: failed to write to the log file; output after that point is not in the file: <path>`。どちらも 1 行ずつで、終了コードは変えない。警告はリトライの知らせの前、`pipeline` では次のジョブの前に出る。「止める」印は読む前と読んだ後に見る(v0.10.1 以降は読む前にも見る。読む前に立っていれば読まずに、読んだ後なら読んだ分をどこにも書かずに終わる)。ログがあっても、シグナルは今までどおり転送される | `runtime::runner_log::br4_5 孫が出力を持ち続けても 10 秒以内に戻り 待ち切れを返す`、`runtime::runner_log::br4_4 ログありでもシグナルは今と同じく転送され 出力を待って戻る`、`flow_run::br5_2_br5_3 書き込みの失敗と待ち切れは警告を 1 行ずつ出し 終了コードは変わらない`、`flow_retry::br5_2 警告は試行ごとに リトライの知らせの前に出る`、`flow_pipeline::br5_3 待ち切れの警告は次のジョブの前に出て 終了コードは変わらない`、`runtime::runner::tests::ファイルへの書き込みに失敗したら以後ファイルに書かず 端末には書き続ける`、`runtime::runner::tests::止める印が立っていればどこにも書かずに終わる`、`runtime::runner::tests::br2_3 止める印が立っていれば read を呼ばずに終わる`、`runtime::runner::tests::br2_4 読んでいる間に印が立てば 読んだ分を書かずに終わる`。区分: 単体、注入 |
| AC-551 | 作らないコマンドと `log_dir` の誤り | `P(p; a:0)` / 各行の `log_dir` | `validate`、`graph`、`run --dry-run a`、`pipeline --dry-run p` / `validate`・`run a` | 前者: すべて 0 で、`L` を作らない(`--dry-run` は時刻もログの指定も使わない)。後者: 64。`log_dir = 1` → `wrong value type`(理由 ``` `log_dir` は文字列で書きます ```)、`log_dir = ""` → `required value is empty`(``` `log_dir` が空です ```)、`[[job]]` に `log_dir` → `unknown key`。ほかの誤りと一緒に、位置の順に集める | `binary_log::br5_4 validate graph dry_run はログのディレクトリを作らない`、`binary_log::br1_1 log_dir の誤りは 64`、`flow_pipeline::br5_4 validate graph dry_run はログの指定も時刻も使わない`、`flow_run::br5_4 dry_run は時刻もログの指定も使わない`、`core::definition_errors::br1_1 log_dir が文字列でなければ型の誤り`、`core::definition_errors::br1_1 log_dir が空なら空の誤り`、`core::definition_errors::br1_1 log_dir を job に書けば知らないキー`、`core::definition_errors::br1_1 log_dir の誤りはほかの誤りと一緒に集める` |
| AC-552 | README の `log_dir` の説明 | README | (検査) | 見出しと必須の語句(ファイルの名前の形 `<ジョブ ID>.<開始の時刻>.<試行の番号>.log`、`-2`・`-3` の番号、`0600`、2 つの流れの間の順、最大 2 秒の待ち、古いログを消さないこと、作れないとき)と、ログの 3 つの文(日本語の 3 つと、作れないときの英語)があり、載っている TOML の例が実際の読み取りに通る(`log_dir = "logs"`) | `docs::readme に log_dir の必須の語句と版の表記がある`、`docs::readme の log_dir の例の定義を実際の読み取りに通せる`。区分: 点検 |

### 4.26 試行ごとの時刻と `watch` の所要時間(v0.10.0 以降)

規定は 05 の 18 章、06 の 2.2a・2.3・5.1・6.1、08 の 6 章。「試行」は、ステートのジョブの `[[job.attempt]]` の 1 つ(`started_at`、`ended_at`)。時刻は実行のたびに変わるので、実バイナリの項目は、形(UTC の `YYYY-MM-DDTHH:MM:SSZ`)と、実行の前後に取った時刻のあいだにあることを確かめる。時刻の値そのものを比べる項目は、実行の部品の時計を差し替えて(注入)確かめる。ステートの誤りの期待の 65 は、4.11 節と同じ(ジョブを起動せず、ステートは変わらず、OUT 空)。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-553 | 試行の開始でステートに書く | `P(p; a:0, b:3{w=[3]})` / ステートが空でない(前に `run a` した)/ 試行の開始の書き込みが失敗する(注入)/ スキップするジョブ | `pipeline p` / `run b` | `pipeline p` は 0 で、ステートに `[[job.attempt]]` が 2 つ(`a`、`b` に 1 つずつ。どちらも閉じている)。試行ごとに、起動の**前に**、試行を足したステートを同期を省いて書く(ステートが空なら、これが初期化を兼ね、空でなくても書く)。試行の開始の書き込みに失敗したら 65 で、ジョブを起動しない(`(the job was not started)`)。スキップしたジョブには試行が無い。ステートに足されるのは試行の時刻だけで、`command`・環境変数の値・ジョブの出力は書かない | `binary_basic::u6_fr6_1 ステートに試行の始まりと終わりの時刻が書かれる`、`flow_run::u6_br1_1 試行の始まりは起動の前に fast で書かれる`、`flow_run::u6_br1_1 空でないステートでも試行の始まりを fast で書いてから起動する`、`flow_run::u6_br1_1 始まりの書き込みに失敗したら 65 で起動しない`、`flow_pipeline::u6_br1_1 各ジョブの試行の時刻を記録する`、`flow_pipeline::u6_br1_1 スキップしたジョブには試行が無い`、`runtime::nfr_content::ステートファイルに command と環境変数の値が現れない` |
| AC-554 | 試行の終わりの時刻 | 偽の時計(呼ぶたびに 1 秒進む)で `P(p; a:0)` / 起動できない `command` / ログのファイルを作れない | `run a` | 試行の始まりは、起動の前に取った時刻、終わりは、試行の後に取った時刻(`a` の試行 = (0 回目, 1 回目))。起動に失敗した試行と、ログのファイルを作れずに起動しなかった試行も、足して閉じる(結果はエラー)。結果の記録(耐久性つき)は、試行を閉じたあとのステート | `flow_run::u6_br1_1 試行の始まりと終わりは偽の時計の値`、`flow_run::u6_br1_2 起動の失敗も試行を足して閉じ エラーになる`、`flow_run::u6_br1_2 ログを作れずに起動しなかった試行も足して閉じる`、`core::attempt_flow::br1_1 試行を始めると終わりの無い試行が後ろに足される`、`core::attempt_flow::br1_2 試行を閉じると最後の試行に終わりが入る`。区分: 注入、単体 |
| AC-555 | リトライで試行が増え、待ちのあいだは開いたまま | 偽の時計で `RT`(`fails=2`) | `run a` | 0。`a` の試行は 3 つ((0, 1)、(2, 3)、(4, 5))。同期を省く書き込みが 3 回、耐久性つきが 1 回。待ちのあいだのステートでは最後の試行が開いていて(`a` は `pending`)、`watch --follow` からは動いている最中に見える(止まったと見ない)。次の試行の開始の書き込みで、前の試行を、試行の直後に取った時刻で閉じる(AC-278) | `flow_retry::u6_br1_2 リトライの試行ごとに始まりと終わりを記録する`、`flow_retry::u7_br3_4 リトライの待ちの間のステートでは最後の試行が開いている`、`flow_retry::u7_br3_4 次の試行の書き込みで前の試行を本当の終わりの時刻で閉じる`、`flow_retry::u7_br3_4 リトライ 1 回で fast 2 回 durable 1 回`、`core::pipeline_activity::br3_4 リトライの待ちの間の形は running 初回`、`core::pipeline_activity::br3_4 リトライの待ちの間の形は running 再開`。区分: 注入 |
| AC-556 | 中断・強制終了の試行の閉じ方 | 偽の時計で、起動の前に合図が立つ / リトライの待ちの途中で合図が立つ / `conductor` が強制終了されて、最後の試行が開いたまま残ったステート | `run a` | 起動の前の中断: その試行を取り除く(試行は増えない)。待ちの途中の中断: 覚えていた終わりの時刻で閉じ、新しい試行を足さずに耐久性つきで書く(`a` は `pending` のまま。エラーとして記録しない)。強制終了の後の再実行: 開いたままの最後の試行を、**自身の `started_at`** で閉じてから、新しい試行を足す | `flow_run::u6_br1_3 起動の前に合図が立っていた試行は取り除く`、`flow_retry::u7_br3_4 待ちの途中の中断では持っていた時刻で閉じて durable で書く`、`flow_retry::u6_br1_3 リトライの待ちの中断では新しい試行を足さない`、`flow_run::u6_br1_4 強制終了で閉じ忘れた試行は自身の始まりで閉じてから足す`、`core::attempt_flow::br1_3 起動しなかった試行は取り除く`、`core::attempt_flow::br1_4 閉じ忘れの最後の試行は自身の始まりで閉じてから足す`。区分: 注入、単体 |
| AC-557 | 再開で試行を足し、前の試行を消さない | `P(p; a:7)` で `run a`(エラー)のあと、`a:0` に直す | `run a` | 0。`a` の試行は 2 つ(古い順。前の試行は残る)。`CONDUCTOR_ATTEMPT` は 1(ステートの試行の数から数えない。AC-453)。結果とスキップの記録は、試行を変えない。`--dry-run` は試行を記録せず、時刻も取らない | `flow_run::u6_br1_4 再実行しても前の試行を残して後ろに足す`、`core::attempt_flow::br1_4 再開しても前の試行を残して後ろに足す`、`core::attempt_flow::結果の記録とスキップの記録は試行を保つ`、`flow_dry_run::u6 dry_run は試行を記録せず 時刻も取らない` |
| AC-558 | 時刻の書式 | 時刻(UNIX 時刻の秒) | 整形・読み取り・差の関数 | ステート用は `YYYY-MM-DDTHH:MM:SSZ`(例 `2026-11-25T09:12:48Z` = 1,795,597,968)、詰めた形は `YYYYMMDDTHHMMSSZ`(`20261125T091248Z`)。範囲は 1970-01-01T00:00:00Z〜9999-12-31T23:59:59Z(範囲の外は作らず、読まない)。うるう年と各月末(2024 年の各月末とその翌日)で整形と読み取りが往復する。読み取りは、ステート用の形だけ(小数の秒、時差、`Z` なし、詰めた形、13 月は読まない)。差は秒で、負にならない(終わりが前なら 0) | `core::time_format::br2_1 境界の表で整形と詰めた形が一致する`、`core::time_format::br2_2 境界の表で読み取りが整形と往復する`、`core::time_format::br2_1 2024 年の各月末とその翌日`、`core::time_format::br2_1 日ごとに往復する`、`core::time_format::br2_2 形の違うものは読まない`、`core::time_format::br2_2 範囲の外の値は読まない`、`core::time_format::br2_1 範囲は 1970 年から 9999 年まで`、`core::time_format::nfr2_2 差は秒で 負にならない`、`runtime::state_read::u6 試行の時刻を読む`。区分: 単体 |
| AC-559 | 試行の記録の誤りと v0.9.9 までのステート | `P(p; a:0, b:0)`。各行のステート | `run b` / `pipeline p` / `watch` / `graph --state` / `pipeline p --dry-run` | 65。動いたジョブ(`success`・`warning`・`error`)に試行が無い(v0.9.9 までに書いたステート)→ `state file is corrupt: 動いたジョブに試行の記録がありません(古い形式の可能性があります)`。最後でない試行に `ended_at` が無い → `終わりの時刻の無い試行が、最後の試行ではありません`。時刻の書式の誤り(`2026-11-25T09:12:48`、`…+09:00`、`…48.5Z`、`2026-13-25T…Z`、`20261125T091248Z`、空)→ `時刻の書式が誤っています`。試行に知らないキー → `知らないキーがあります`、`started_at` が無い → `必須のキーがありません`、時刻が文字列でない・`attempt` が表の配列でない → `値の型が誤っています`。どれも `reset` を案内し(`watch` も)、ERR にステートの値を出さない。`reset` の後は動く。`pending`・`skipped` は試行が無くてよく、最後の試行は終わりが無くても、終わりが始まりより前でも読める | `runtime::state_read::u6_br3_1 動いたジョブに試行が無ければ no attempt`、`runtime::state_read::u6_br3_2 最後でない試行に終わりが無ければ unfinished attempt`、`runtime::state_read::u6_br2_1 時刻の書式の誤りは bad time`、`runtime::state_read::u6_br2_2 試行の知らないキー 欠け 型の誤り`、`runtime::state_read::u6_br3_1 pending と skipped は試行が無くてよい`、`runtime::state_read::u6_br3_2 最後の試行は終わりが無くても 終わりが始まりより前でも読める`、`runtime::state_read::u6 試行の誤りの表示にステートの値を出さない`、`runtime::nfr_content::試行の時刻の誤りの表示に入力の値が現れない`、`binary_basic::u6_fr6_4 古い形のステートは 65 で reset を案内し reset の後は動く`、`flow_run::u6_br3_1 古い形のステートは 65 で reset を案内し起動しない`、`flow_pipeline::u6_br3_1 古い形のステートは 65 で reset を案内し起動しない`、`flow_aux::u6_br3_1 watch は古い形のステートを 65 にして reset を案内する`、`flow_graph::u6_br3_1 state で古い形のステートは 65 で reset を案内し図を出さない`、`flow_dry_run::u6_br3_1 古い形のステートは 65 で reset を案内し計画を出さない` |
| AC-560 | `watch` の所要時間の列 | `P(p; a:0, b:0)` で `run a` のあと / 偽の時計で、`b` が `[job.retry]` で 2 回の試行のあと / `b` の試行を始めたまま止まったステート(強制終了の後と同じ形) | `watch` | 表の見出しは `JOB  STATUS   DURATION  NOTE`(日本語 `所要時間`)。終わったジョブ(`success`・`warning`・`error`)は、最初の試行の始まりから最後の試行の終わりまで(リトライの待ちを含む。`b    success  3s`)。実バイナリの短いジョブは `<秒>s` の形。動いていない `pending` は空(`b    pending`)。動いている最中の `pending`(最後の試行に終わりが無い)は、最初の始まりから今まで(`b    pending  1m05s`)。今の時刻は、表を作る前に 1 回だけ取る | `binary_basic::u6_fr6_5 watch の表に所要時間の列がある`、`flow_aux::u6_br4_1 watch の所要時間は偽の時計の始まりから終わりまで`、`flow_aux::u6_br4_1 watch は動いている最中のジョブに今までの経過時間を出す`、`report_messages::u6_br4_1 watch の所要時間の見出しは日本語では所要時間` |
| AC-561 | 所要時間の書き方 | ジョブの状態と試行と今の時刻 | 所要時間を求めて書く関数 | 1 分未満は `<秒>s`(`17s`、`59s`)、1 時間未満は `<分>m<秒 2 桁>s`(`1m05s`、`1m00s`)、それ以上は `<時>h<分 2 桁>m<秒 2 桁>s`(`2h03m04s`、`1h00m00s`)。終わりが始まりより前(時計の戻り)は `0s`。空: 試行が無い、最後の試行が終わった `pending`、`skipped`、結果のあるジョブで最後の試行に終わりが無い | `report_messages::u6_br4_1 watch の所要時間は表のとおり`、`core::time_format::u6_br4_2 所要時間は単位つきの短い形`、`core::time_format::u6_br4_1 試行が無ければ所要時間は空`、`core::time_format::u6_br4_1 結果のあるジョブは最初の始まりから最後の終わりまで`、`core::time_format::u6_br4_1 結果のあるジョブで最後の試行に終わりが無ければ空`、`core::time_format::u6_br4_1 動いている最中の pending は今までの経過時間`、`core::time_format::u6_br4_1 最後の試行が終わった pending と skipped は空`、`core::time_format::u6_br4_1 終わりが始まりより前なら 0 秒`。区分: 単体 |

### 4.27 `watch --follow`(v0.10.0 以降)

規定は 04 の 7.3、05 の 20 章、07 の 7.6、08 の 6.4、10 の 2 章。特に記さない限り、定義は `P(p; a:0, b:0)`、標準出力はパイプ(端末ではない)。自動の項目の「描いた回数」は、OUT の `Pipeline: p` の行の数。読み直しの結果の並び(有る・無い・読めないなど)と待ちの秒数を決めて確かめる項目は、ステートの読み取りと待ちを差し替えて(注入)行う。どの項目でも、`watch --follow` はステートを読むだけで、書かない・消さない。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-562 | 完了したステートで 1 回描いて 0 | `pipeline p`(0)のあと | `watch --follow --interval 1` | 0。OUT は表 1 回分(`Pipeline: p` で始まり、`completed` で終わる)。制御の文字(`\x1b`)が無い。ERR 空。待たない。シグナルの準備をする | `binary_basic::u7_fr7_3 完了のステートを follow で見ると 1 回描いて 0 でパイプに制御の文字が無い`、`flow_aux::u7_br3_1 完了のステートは 1 回描いて 0`、`flow_aux::u7 run id のステートを読む` |
| AC-563 | 止まったステートで 0 | `P(p; a:7, b:0)` で `pipeline p`(7)のあと / 中断で止まった `pending`(再開位置のジョブが閉じた試行を持つ) | `watch --follow` | 0。表を 1 回描いて終わる(エラーのジョブがある、または再開位置のジョブが試行を持つ)。待たない。制御の文字が無い | `binary_basic::u7_fr7_3 止まったステートを follow で見ると 0 で終わる`、`flow_aux::u7_br3_1 止まったステートは 1 回描いて 0`、`core::pipeline_activity::br3_1 エラーのジョブの後の間は stopped`、`core::pipeline_activity::br3_1 再開の位置のジョブが試行を持てば stopped` |
| AC-564 | ステートが消えたら 0 | 読み直しの結果が、動いている → 無い → 無い / 動いている → 無い → 動いている → 完了 | `watch --follow` | 一度見えたステートが **2 回続けて**無いときだけ、消えたとして 0 で終わる(描いたのは 1 回。ERR 空)。1 回だけの「無い」は何も描かずに続ける | `flow_aux::u7_br3_1 一度見えたステートが 2 回続けて無いなら 0 で終わる`、`flow_aux::u7_br3_1 一度見えたステートが 1 回だけ無いなら続ける`。区分: 注入 |
| AC-565 | ステートが無いときは待つ | 読み直しの結果が、無い → 無い → 動いている → 完了 | `watch --follow` | 一度も見ていない「無い」は、毎回 `no state (nothing has run yet)` を描いて待つ。OUT = `no state (nothing has run yet)`⏎⏎`no state (nothing has run yet)`⏎⏎`Pipeline: p`…(2 回目から空の 1 行を置く)。待ちは 2 秒が 3 回。「無い」では今の時刻を取らない。開始と終わりの判定(続ける・終わる)は 05 の 20 章(まだ始まっていない・ジョブとジョブのあいだは続ける) | `flow_aux::u7_br4_1 ステートが無ければ待ち できたら表を描く`、`core::pipeline_activity::br3_1 始まる前は between`、`core::pipeline_activity::br3_1 ジョブとジョブの間は between`、`core::pipeline_activity::br3_1 終わりの無い試行があれば running`。区分: 注入 |
| AC-566 | 読めないのが 3 回続くと 65 | 読み直しの結果が、動いている → 読めない × 3 / 読めない × 2 → 無い → 読めない × 2 → 完了 / 動いている → 壊れた(大きすぎる、版の違い)/ v0.9.9 までのステート | `watch --follow` | 読めない(`unreadable`)ときは何も描かず、1 秒後に読み直す。3 回続けば、ERR に `cannot read state file` の行を 1 つ出して 65(待ちは 2、1、1 秒)。読めた(無いを含む)後は数え直す(0 で終わる)。読めないときは、「無い」の続きも 0 に戻す。壊れた・大きすぎる・版の違いは、`watch` と同じ表示ですぐ 65。v0.9.9 までのステートは `reset` を案内して 65、OUT 空 | `flow_aux::u7 読めないのが 3 回続けば 65`、`flow_aux::u7 読めないときは何も描かず 1 秒後に読み直し 読めたら続ける`、`flow_aux::u7 読めた後は読めない回数を数え直す`、`flow_aux::u7 読めないときは無いの続きを戻す`、`flow_aux::u7_br4_2 壊れたステートは今の watch と同じ表示で 65`、`flow_aux::u7_br4_2 古い形のステートは reset を案内して 65`。区分: 注入 |
| AC-567 | 中断の合図で 0 | 動いているステート。待ちの途中で合図が立つ / 描く前に合図が立っている / シグナルの準備が失敗する | `watch --follow` | 待ちの途中の合図: 0、ERR 空、合図の後は読まない。描く前の合図: 何も読まず、何も描かずに 0。準備の失敗: 70、ERR = `conductor: error: failed to set up signal handling`、何も読まない。`--follow` の無い `watch` は準備しない(AC-297)。UNIX 系は SIGHUP・SIGINT・SIGQUIT・SIGTERM、Windows は Ctrl+C で終わる(07 の 7.6) | `flow_aux::u7_br3_2 待ちの途中の合図で 0`、`flow_aux::u7_br3_2 描く前に合図が立っていれば何も読まずに 0`、`flow_aux::u7_br3_3 合図の準備に失敗したら 70 で何も読まない`、`flow_aux::u7_br1_1 follow の無い watch は合図の準備をしない`。区分: 注入 |
| AC-568 | `--interval` の範囲と既定 | 読み直しの結果が、ジョブのあいだ → 動いている × 2 → 完了 / 動いている → 完了 | `watch --follow` / `watch --follow --interval 5` | 既定の間隔は 2 秒(待ちは 2、2、2。描くたびに今の時刻を 1 回取る)。`--interval 5` は 5 秒。値は 1〜3600 の整数の秒(`1`、`2`、`60`、`3600` は受理。`--interval=5` の形も、`--follow` の前に書いても可)。`--config`・`--run-id` と一緒に使える | `flow_aux::u7_br2_1 動いている間は既定の 2 秒おきに描き直す`、`flow_aux::u7_br1_2 interval の秒数で待つ`、`args_grammar::u7_br1_2 watch は follow と interval を受け取る`、`args_grammar::u7_br1_1 watch は follow なしなら今のまま`。区分: 注入、単体 |
| AC-569 | 使い方の誤り | — | `watch --follow --interval <値>`(`0`、`3601`、`99999abc`、`1.5`、`-1`、`+5`、`abc`、空、` 5`、`5 `、`99999999999`、UTF-8 でない値)/ `watch --follow --interval` / `watch --interval 5` / `run x --follow`(`pipeline`・`reset`・`validate`・`graph`・`init`・`convert` も)/ `run x --follow --interval 5` / `run x --interval 5` / `watch --follow --follow` / `watch --follow --interval 1 --interval 2` / `watch --follow=1` | すべて 68、OUT 空、ERR の最初の行のあとに空行と使い方。順に `invalid value: '--interval'`(値は ERR に現れない)、`option requires a value: '--interval'`、`option not allowed for this command: '--interval'`、`option not allowed for this command: '--follow'`(`--interval` と一緒でも `--follow` の誤り)、`option not allowed for this command: '--interval'`、`invalid value: '--follow'`、`invalid value: '--interval'`、`invalid value: '--follow'` | `binary_basic::u7_fr7_2 interval の誤りは 68 で値を出さない`、`args_grammar::u7_br1_2 interval の誤った値は invalid value で値を含まない`、`args_grammar::u7_br1_2 follow の無い interval は使えない`、`args_grammar::u7_br1_2 watch 以外の follow と interval は使えない`、`args_grammar::u7_br1_2 follow と interval の重複と follow への値は誤り`、`args_grammar::u7_br1_2 utf8 でない interval の値は invalid value で対象は interval` |
| AC-570 | 描き方(端末とパイプ) | 読み直しの結果が、動いている → 完了。標準出力がパイプ / 端末(`NO_COLOR=1`) | `watch --follow` | パイプ(と Windows の端末): 1 回目は何も置かず、2 回目から空の 1 行を置いて追記する(OUT に `\n\nPipeline: p\n`。制御の文字なし)。1 回分ずつ 1 回で書く。UNIX 系の端末: 毎回、表の前に `ESC[H ESC[2J`(`\x1b[H\x1b[2J`)を置き(2 回)、空の行は置かない。中身は 1 回の `watch` と同じ(英語・日本語、ステートの有無)。標準出力に書けなければ `cannot write to standard output` で 65(「無い」の表示でも) | `flow_aux::u7_br2_2 端末でないときは 2 回目から空の 1 行を置いて追記する`、`flow_aux::u7_br2_2 unix の端末では毎回画面を消してから描く`、`report_messages::u7_br2_2 継続表示の前置きは画面を消すか 2 回目から空の 1 行`、`report_messages::u7_nfr1_9 継続表示の中身は 1 回の watch と同じ`、`flow_aux::u7 標準出力に書けなければ 65`。区分: 注入、単体 |
| AC-571 | ステートに書き込まない(1,000 回の繰り返し) | 読み直しの結果が、動いている × 999 → 完了 | `watch --follow --interval 1` | 0。表を 1,000 回描き、待ちは 999 回。ステートを書く・消す呼び出しが 1 回も無い(強制。時間の基準なし) | `flow_aux::u7 1000 回の繰り返しでも動き 書き込みをしない`。区分: 注入 |
| AC-572 | README の `watch --follow` の説明 | README | (検査) | 見出しと必須の語句(`--follow`、`--interval <秒>`、`1〜3600 の整数`、既定は 2、画面を消して描き直すこと、空の 1 行を置くこと、完了した・止まった・消えたで終わること、読めないが 3 回続けば 65、`--follow` の無い `--interval` の誤り)があり、使い方の `watch` の行が、使い方の文(英語・日本語)と README で揃う | `docs::u7 readme に watch follow の必須の語句と版の表記がある`、`docs::u7_nfr1_10 使い方の watch の行は args messages readme で揃う`。区分: 点検 |

---

## 5. デモ

### 5.1 構成

デモは、`conductor` の主な動きを、1 本のスクリプトで順に見せる。利用者は、リポジトリの根から実行する。スクリプトは、一時ディレクトリに定義とジョブを写して実行し、終了時にそのディレクトリを消す(リポジトリには何も残らない)。

```text
demo/README.md                 使い方と各場面の説明
demo/conductor.toml            UNIX 系の定義(パイプライン nightly)
demo/retry.toml                UNIX 系のリトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/env.toml, demo/demo.env   UNIX 系の環境変数の定義(パイプライン env-demo)と `.env`。場面 11 だけが使う(v0.9.0 以降)
demo/log.toml                  UNIX 系のログのファイルの定義(パイプライン log-demo)。場面 15 だけが使う(v0.10.0 以降)
demo/run-demo.sh               UNIX 系のスクリプト(シェル)
demo/scripts/{extract,transform,load,notify,flaky,show-env}.sh   UNIX 系のジョブ(show-env は v0.9.0 以降)
demo/windows/conductor.toml, retry.toml, env.toml, demo.env, log.toml, run-demo.cmd   Windows 版(コマンドプロンプトのバッチ。log.toml は v0.10.0 以降)
demo/windows/scripts/{extract,transform,load,notify,flaky,show-env}.cmd   Windows 版のジョブ
```

### 5.2 実行の仕方(UNIX 系と Windows の差)

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 実行 | `demo/run-demo.sh`(リポジトリの根から) | `demo\windows\run-demo.cmd` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`。なければ、リリースの形のビルドの成果物(`target/release/conductor`)。それも無ければ、実装のビルド手順で、リリースの形のビルドを作る | `CONDUCTOR`。なければ `target\release\conductor.exe`。それも無ければ同様 |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | `DEMO_LANG`(既定 `ja_JP.UTF-8`。`en_US` のようにも書ける)。`LC_ALL`、`LC_MESSAGES` は空にする。コードページを実行中だけ 65001(UTF-8)にして、終わりに戻す |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ。場面 4 では、`watch --follow` の前でも待つ(端末では画面が消えるので、その前の出力を読めるように。v0.10.1 以降) | 各場面のあとで待つのは同じ。場面 4 の `watch --follow` の前では待たない(Windows は画面を消さず、追記するため) |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。小数可) | `DEMO_SLEEP`(整数の秒。既定 1。`0` で待たない。小数は使えない) |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | 一時ディレクトリを作り、定義とスクリプトを写して実行。終了時に消す | `%TEMP%` の下の一時ディレクトリ(`conductor-demo-<乱数>`)に写して実行し、終わりに消す |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を消す(利用者の環境の影響を除く) | 同じ(空にする) |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色。`watch` も色あり(標準出力が端末のとき) | 画面に色なし |
| 各コマンドの出力 | `$ <コマンド>` の行、実行、`→ 終了コード <N>` | `$ conductor <引数>`、実行、`→ 終了コード <N>` |
| ステートの一覧 | `ls conductor.toml*.state`(なければ `(なし)`) | `dir /b conductor.toml*.state`(なければ `(なし)`) |
| 場面の見出し | `=== <番号>. <題> ===` | 同じ |

### 5.3 定義とジョブ

`demo/conductor.toml`(Windows 版は、`command` が `scripts\<名>.cmd`、`notify` は `["cmd", "/c", "scripts\\notify.cmd"]`):

```toml
format_version = 1

[pipeline]
name = "nightly"
description = "デモ用の夜間バッチ"
run_id_env = "DEMO_RUN_ID"

[[job]]
id = "extract"
command = "sh scripts/extract.sh"
warning_exit_codes = [1]

[[job.transitions]]
on = "warning"
next = "notify"

[[job]]
id = "transform"
command = "sh scripts/transform.sh"

[[job]]
id = "load"
command = "sh scripts/load.sh"

[[job]]
id = "notify"
command = ["sh", "scripts/notify.sh"]
```

`demo/retry.toml`: `[pipeline]` の `name = "retry-demo"`(`description` は `リトライのデモ`)、ジョブ `flaky`(`command = "sh scripts/flaky.sh"`)、`[job.retry]` の `exit_codes = [75]`、`max = 3`、`interval = "1s"`。

ジョブのスクリプトの仕様(`extract`・`transform`・`load`・`notify`): `  [<名>] 実行中...` を表示する → `DEMO_SLEEP` の秒だけ待つ → `  [<名>] 終了コード <N> で終わります` を表示する → 環境変数 `DEMO_<名の大文字>_EXIT`(既定 0)の値 N で終わる。
`flaky` の仕様: カレントディレクトリの `flaky.count` の回数を 1 増やす(なければ 0 から)→ `  [flaky] <n> 回目の実行...` を表示する → 待つ → `n` が `DEMO_FLAKY_FAILS`(既定 2)以下なら `  [flaky] 終了コード 75 で終わります(一時的な失敗)` を表示して 75 で終わる。超えたら `  [flaky] 終了コード 0 で終わります` を表示して 0 で終わる。
(Windows 版のジョブは、待ちに `ping` を使うなど、実現の仕方が違ってよい。表示の文言は同じ。)

`demo/env.toml`(v0.9.0 以降。場面 11): `[pipeline]` の `name = "env-demo"`、`description = "環境変数のデモ"`、`env_file = ["demo.env", { path = "local.env", required = false }]`(v0.10.0 以降。`local.env` は置いていないので、読み飛ばす。v0.9.x までは `env_file = "demo.env"`)、`[pipeline.env]` に `DEMO_GREETING = "hello from [pipeline.env]"` と `DEMO_STAGE = "pipeline"`、ジョブ `show-env`(`command = "sh scripts/show-env.sh"`)に `unset_env = ["DEMO_PROXY"]`(v0.10.0 以降)と、`[job.env]` に `DEMO_STAGE = "job"`。`demo/demo.env`: `DEMO_FROM_FILE=from demo.env`、`export DEMO_QUOTED="two words"`(コメントつき)、`DEMO_STAGE=dotenv`(コメントで、優先順位の例を説明)。`show-env` の仕様: `DEMO_GREETING`、`DEMO_FROM_FILE`、`DEMO_QUOTED`、`DEMO_STAGE`、`CONDUCTOR_PIPELINE`、`CONDUCTOR_JOB_ID`、`CONDUCTOR_RUN_ID`(なければ `(なし)`)、`CONDUCTOR_ATTEMPT`、(v0.10.0 以降)`DEMO_PROXY`(なければ `(なし)`)を、`  [show-env] <名前>=<値>` の形で 1 行ずつ表示する。

`demo/log.toml`(v0.10.0 以降。場面 15): `[pipeline]` の `name = "log-demo"`、`description = "ログのファイルのデモ"`、`log_dir = "logs"`、`[pipeline.retry]` に `exit_codes = [75]`、`max = 3`、`interval = "1s"`(既定のリトライ)、ジョブ `flaky`(`command = "sh scripts/flaky.sh"`。`retry` を書かないので既定を当てはめる)。

### 5.4 場面

| 場面 | 見せること | 実行するコマンド(UNIX 系。Windows も同じ引数) |
|------|-----------|--------------------------------------------------|
| 0 | 定義の中身 | `cat conductor.toml`(Windows は `type`) |
| 1 | 全部成功。書いた順に 1 つずつ | `pipeline -v nightly` |
| 2 | `extract` が警告(`DEMO_EXTRACT_EXIT=1`)→ `transform`・`load` をスキップして `notify` へ | `pipeline -v nightly` |
| 3 | `load` のエラー(`DEMO_LOAD_EXIT=2`)で止まり、終了コード 2。止まった位置 | `pipeline -v nightly`、続けて `watch` |
| 4 | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す(v0.10.0 以降は所要時間の列つき)。(v0.10.0 以降)`watch --follow` は、完了したステートなので 1 回描いて 0 で終わる。`--remove-on-complete` で完了時にステートを消す | `pipeline -v nightly`、`watch`、(v0.10.0 以降)`watch --follow`(UNIX 系は `DEMO_PAUSE=1` なら、その前で待つ。v0.10.1 以降)、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` |
| 5 | 個別実行と初期化 | `run -v extract`、`reset` |
| 6 | TOML から YAML への変換 | `convert conductor.toml --to yaml` |
| 7 | 検査。正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない | `validate -v`、誤りの定義 `bad.toml`(`command = "sh -c 'echo hi"`)を作って `validate --config bad.toml`、一覧 |
| 8 | run ID ごとに別のステートファイル(`conductor.toml.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 |
| 9 | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` |
| 10 | `flaky` が 2 回、終了コード 75 で失敗し、`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) | `rm -f flaky.count`(Windows は `del`)、`pipeline -v --config retry.toml retry-demo` |
| 11(v0.9.0 以降) | 環境変数。`show-env` が、`.env` < `[pipeline.env]` < `[job.env]` の優先順位で足された環境変数(`DEMO_STAGE=job`)と、自動の `CONDUCTOR_*`(run ID がなければ `CONDUCTOR_RUN_ID=(なし)`、`CONDUCTOR_ATTEMPT=1`)を表示する。(v0.10.0 以降)無くてもよい `local.env` は黙って読み飛ばす。親に置いた `DEMO_PROXY=from-parent` は、`unset_env` で消えて `DEMO_PROXY=(なし)`。`--dry-run` は、消す名前の行(`unset env: DEMO_PROXY`)を出す。`.env` が無いと、ジョブを動かさずに 64。`validate` は、重複キーを警告して 0。値は `conductor` 自身の出力に出ない | `cat env.toml`、`cat demo.env`、`DEMO_PROXY=from-parent` で `pipeline -v --config env.toml env-demo`、(v0.10.0 以降)`pipeline --dry-run --config env.toml env-demo`。`env_file = "missing.env"` の定義 `bad-env.toml` で `pipeline`。`A=1`・`A=2` の `dup.env` を指す定義 `dup-env.toml` で `validate` |
| 12(同) | 遷移の図。`extract` が警告なら `notify` へ飛ぶ遷移の図(右の線が飛び越し)、`-v` の詳細、Mermaid、(v0.10.0 以降)DOT、`--state` の `[OK]`・`[NG]`・`[  ]` と再開位置。`graph` はステートを読むだけ | `graph`、`graph -v`、`graph --format mermaid`、(v0.10.0 以降)`graph --format dot`、`DEMO_LOAD_EXIT=2` で `pipeline nightly`(`load` で止める)、`graph --state`、`reset` |
| 13(同) | 実行計画。計画(ジョブは実行されない)、`extract` が警告で `transform`・`load` が遷移でスキップされ `notify` へ、`load` で停止、`run` の次の動き。ステートは作られない | `pipeline nightly --dry-run`、`--assume extract=1`、`--assume load=2`、`run extract --dry-run`、ステートの一覧 |
| 14(同) | ひな形。`created new.toml`、ひな形が `validate` に通る、2 回目は終了コード 69、`--force` で `replaced new.toml`、`--config -` は標準出力にひな形 | `init --config new.toml starter`、`validate --config new.toml starter`、もう一度 `init`、`init --config new.toml --force starter`、`init --config - starter` |
| 15(v0.10.0 以降) | ログのファイル。`flaky` が 2 回失敗してから成功し(既定のリトライ)、試行ごとに 1 つのファイル `flaky.<開始の時刻(UTC。詰めた形)>.<試行の番号>.log` ができる(3 つ)。UNIX 系は `-rw-------`(0600)。ファイルの中身はジョブの出力だけ。端末の出力は `log_dir` が無いときと同じ。`conductor` は古いログを消さない(片づけは利用者) | `cat log.toml`、`rm -f flaky.count`、`pipeline --config log.toml log-demo`、`ls -l logs`、各 `logs/*.log` の中身(`--- <パス>` の見出しのあと)、`rm -r logs`、`reset --config log.toml`。Windows 版は `ls -l` の代わりに `dir /b logs`、`rm -r` の代わりに `rmdir /s /q logs`(権限の表示は無い) |

最後に `デモは以上です。`(Windows 版も同じ)を出す。

### 5.5 デモの受け入れテスト

実行は、`CONDUCTOR=<被試験体>`、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、一時ディレクトリの親(UNIX 系は `TMPDIR`)を空のディレクトリ `D` にして行う。標準入力は空。

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-424 | UNIX 系: スクリプトが全部通る(v0.10.0 で変更) | 上の設定 | `sh demo/run-demo.sh` | 終了 0。見出し `=== 0. ` から `=== 15. ` までが順に出る(v0.9.x までは `=== 14. ` まで)。場面 3 の `watch` の表の見出しが `JOB        STATUS   DURATION  NOTE` で、`extract`・`transform`・`load` の所要時間が `<秒>s` の形、`notify` の行が `notify     pending`。場面 15 に、`log_dir = "logs"`、試行 1〜3 のファイルの見出し `--- logs/flaky.<詰めた時刻>.<番号>.log`(番号は 1、2、3)、`-rw-------`、3 回目のファイルの中身(`  [flaky] 3 回目の実行...`⏎`  [flaky] 終了コード 0 で終わります`)、`$ rm -r logs` が現れる | `unix_demo::デモは 0 で完走し 場面 0 から 15 の要点の行が出る`。区分: UNIX 系 |
| AC-425 | UNIX 系: 場面 7(検査) | 同上 | 同上 | 出力に `definition is valid`、`command has an unclosed quote`、`→ 終了コード 64` が現れる。誤りの定義の検査のあと、ステートの一覧が `(なし)` | 同上 |
| AC-426 | UNIX 系: 場面 4(再開・完了)(v0.10.0 で変更) | 同上 | 同上 | `completed` の行、`skipped (already executed)` が現れる。v0.10.0 以降は、`watch --follow` が 1 回だけ描き(`Pipeline: nightly` が 1 回、`completed` あり。標準出力がパイプなので画面の消去の列 `ESC[H` が無い)、`→ 終了コード 0` で終わる。`DEMO_PAUSE=1` のときは、1 回の `watch` の完了の行のあと、`watch --follow` の前で待つ(場面 4 の待ちは、場面の終わりと合わせて 2 回。v0.10.1 以降) | 同上、`unix_demo::br5_5 demo pause のとき場面 4 は watch follow の前で待つ` |
| AC-427 | UNIX 系: 場面 8(run ID) | 同上 | 同上 | `conductor.toml.build-1.state` が一覧に現れる。再開のあと `completed`。`reset` のあと一覧は `(なし)` | 同上 |
| AC-428 | UNIX 系: 場面 9(環境変数と後始末) | 同上 | 同上 | `conductor.toml.opt-1.state` が現れ、最後の `watch` が `no state` | 同上 |
| AC-429 | UNIX 系: 場面 10(リトライ) | 同上 | 同上 | `conductor: retry: job 'flaky' exited with code 75; retry 1 of 3 in 1 seconds` と `… retry 2 of 3 in 1 seconds` の 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success` が現れる | 同上 |
| AC-430 | UNIX 系: 利用者の環境に左右されない | `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、`LC_ALL=C`、`LC_MESSAGES=C`、`NO_COLOR=1` を設定 | 同上 | 出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ。この前提では、デモが `LC_ALL` と `LC_MESSAGES` を消したかは、表示からは見分けられない | 同上 |
| AC-431 | UNIX 系: 何も残さない | 同上 | 同上 | 終了後、一時ディレクトリの親 `D` に何も残らない。リポジトリの作業ツリーは変わらない | 同上 |
| AC-432 | Windows 版: 定義 | 定義 `demo\windows\conductor.toml` | (検査) | 4 ジョブ(`extract`、`transform`、`load`、`notify`)と、`extract` の警告の遷移(`notify` へ)を持つ。`validate` が 0 | `windows_demo`。区分: Windows |
| AC-433 | Windows 版: 場面 1〜2 | 一時ディレクトリに定義とスクリプトを写す | `pipeline nightly`(場面 1)、`DEMO_EXTRACT_EXIT=1` で `pipeline nightly`(場面 2) | 場面 1 は 0。場面 2 は `notify` へ進み 0 | 同上 |
| AC-434 | Windows 版: 場面 3〜4(v0.10.0 で変更) | 同上 | `DEMO_LOAD_EXIT=2` で `pipeline`(2)、`watch`、続けて `pipeline`(`load` から再開) | 2 のあと、`watch` の表に所要時間の列(`DURATION`)が出る。再実行で `load` から再開して 0 | `windows_demo::場面 3 と 4 load の失敗は 2 で watch に所要時間が出て 再実行すると load から再開する` |
| AC-435 | Windows 版: 場面 5〜7 | 同上 | `run extract`、`reset`、`convert conductor.toml --to yaml`、`validate`、誤りの定義の `validate` | `run`・`reset`・`convert`・`validate` は 0。誤りの定義は 64 | 同上 |
| AC-436 | Windows 版: 場面 8〜9 | 同上 | run ID を使う一連(場面 8、9) | run ID ごとにステートが分かれる。`--run-id` が環境変数に勝つ。`reset --run-id` と `--all` が働く | 同上 |
| AC-437 | Windows 版: 場面 10 | 同上 | `pipeline --config retry.toml retry-demo` | 2 回失敗してから成功する | 同上 |
| AC-438 | Windows 版: 利用者の run ID に影響されない | 利用者の run ID の環境変数を設定 | 同上 | 結果が変わらない | 同上 |
| AC-514 | UNIX 系: 場面 11(環境変数。v0.9.0 以降)(v0.10.0 で変更) | 上の設定 | `sh demo/run-demo.sh` | `[show-env] DEMO_GREETING=hello from [pipeline.env]`、`DEMO_FROM_FILE=from demo.env`、`DEMO_QUOTED=two words`、`DEMO_STAGE=job`、`CONDUCTOR_PIPELINE=env-demo`、`CONDUCTOR_JOB_ID=show-env`、`CONDUCTOR_RUN_ID=(なし)`、`CONDUCTOR_ATTEMPT=1` の行と、`missing.env`(`.env` が無い誤り)、``duplicate key `A` ``(警告)が現れる。v0.10.0 以降は、`[show-env] DEMO_PROXY=(なし)` が現れ、`[show-env] DEMO_PROXY=from-parent` は現れない。`--dry-run` の計画に `     unset env: DEMO_PROXY` の行が現れる | `unix_demo::デモは 0 で完走し 場面 0 から 15 の要点の行が出る`(場面 11)。区分: UNIX 系 |
| AC-515 | UNIX 系: 場面 12(`graph`)(v0.10.0 で変更) | 同上 | 同上 | `*   extract`⏎`\|\`⏎`* \| transform`⏎`* \| load`⏎`\|/`⏎`* notify`⏎`o @end` の図、`command: sh scripts/extract.sh`、`success -> transform`、`flowchart TD`、`[NG] load  <- resume here`、`[OK] extract` が現れる。v0.10.0 以降は、`digraph conductor {` の行(DOT)も現れる | 同上(場面 12) |
| AC-516 | UNIX 系: 場面 13(`--dry-run`) | 同上 | 同上 | `plan (--dry-run): pipeline nightly`⏎`resume: from the start`⏎`1. [run] extract` で始まる計画、`assume: extract=1`、`2. [skip (transition)] transform`、`3. [skip (transition)] load`、`4. [run] notify`、`assume: load=2`、`result (simulated): stopped (error: load)`、`target job: extract`、`next step: next job transform` が現れ、続くステートの一覧は `(なし)` | 同上(場面 13) |
| AC-517 | UNIX 系: 場面 14(`init`) | 同上 | 同上 | `created new.toml`、`name = "starter"`、`already exists (use --force to overwrite)`、`→ 終了コード 69`、`replaced new.toml`、`format_version = 1`(`--config -` の出力)が現れる | 同上(場面 14) |
| AC-518 | Windows 版: 場面 11〜15(v0.10.0 で変更) | 一時ディレクトリに定義とスクリプトを写す | 場面 11〜15 の一連 | 場面 11: 環境変数が子に渡り、優先順位は `[job.env]` が最も強い。`.env` が読めないと 64、`validate` は重複キーを警告して 0。v0.10.0 以降は、`unset_env` の名前は、親に置いても(大文字小文字によらず)子に渡らず、`--dry-run` は消す名前を出す。場面 12: `graph` の図と `--state`。場面 13: `--dry-run` は何も実行せずステートも作らない。場面 14: `init` は 2 回目に 69、`--force` で上書き。場面 15(v0.10.0 以降): `log_dir` で試行ごとにファイルができ(3 つ)、中身はジョブの出力 | `windows_demo::場面 11 環境変数が子に渡り 優先順位は job env が最も強い`、`windows_demo::場面 11 unset env の名前は親に置いても大文字小文字によらず渡らず dry run は消す名前を出す`、`windows_demo::場面 11 env が読めないと 64 で validate は重複キーを警告して 0`、`windows_demo::場面 12 graph は遷移を図にし state で状態と再開位置を重ねる`、`windows_demo::場面 13 dry run は計画を出し 何も実行せず ステートも作らない`、`windows_demo::場面 14 init はひな形を作り 2 回目は 69 で force で上書きできる`、`windows_demo::場面 15 log dir で試行ごとにファイルができ 中身はジョブの出力`。区分: Windows |

---

## 6. リリース物の要件

### 6.1 対応 OS と配布の形

`conductor` は、外部のランタイムを要しない単一の実行ファイルとして配る。Linux・macOS・Windows で、同じ定義に同じ結果を返す。

| 対象 | OS・CPU | 形式 | 実行ファイル | 備考 |
|------|---------|------|--------------|------|
| Linux x86_64 | Linux、x86_64 | `tar.gz` | `conductor` | 静的リンク(OS の共有ライブラリに依存しない) |
| macOS Apple シリコン | macOS、aarch64 | `tar.gz` | `conductor` | 動作確認あり |
| macOS Intel | macOS、x86_64 | `tar.gz` | `conductor` | クロスビルド。**動作の確認はしていない** |
| Windows x86_64 | Windows、x86_64 | `zip` | `conductor.exe` | |

- 配布物の名前: `conductor-<版>-<対象>.<拡張子>`(`<版>` は `v` なし)。対象の名前は、実装のツールチェーンの対象の名前(例: `x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-apple-darwin`、`x86_64-pc-windows-msvc`)を使う。
- 展開すると、最上位に `conductor-<版>-<対象>/` のディレクトリがあり(zip も同じ)、その中に実行ファイルと `LICENSE` がある。
- チェックサム: 全成果物の SHA-256 を、`sha256sum` の形式で `SHA256SUMS` に書き、一緒に配る。署名はしない(転送の破損の検知まで。配布元の改ざんは防げない)。macOS の署名・公証はしない(Gatekeeper の警告の回避手順を、利用者向けの文書に書く)。
- 配布するのは `conductor` だけ。テスト用ジョブは、配布物に含めない。
- リリースの注記: 自動生成に、固定の 1 文を加える(`Intel macOS (x86_64-apple-darwin): 動作の確認なし(クロスビルド)。SHA256SUMS で、ダウンロードしたファイルを確かめてください。`)。
- 下書き(draft)として作り、人が確認して公開する(自動では公開しない)。版に `-` を含む(例: `0.9.0-rc.1`)ものは、プレリリースにする。

### 6.2 リリースの手順と関門

1. すべての部品の版を新しい版に上げ、`main` に取り込む。
2. `main` のそのコミットに、タグ `v<版>` を付けて push する。タグのコミットが `main` の祖先であること。タグの `v` を除いた文字列が、実行ファイルの部品の版と**完全に一致**すること。
3. 自動の流れが、タグの検査、通常の品質ゲート(7 章)の全項目、4 つの対象のビルド、成果物の動作確認、下書きのリリースの作成を行う。

### 6.3 受け入れテスト

| ID | 機能 | 前提 | 実行 | 期待 | 根拠 |
|----|------|------|------|------|------|
| AC-439 | 成果物の名前と形 | リリース | (成果物の一覧) | 4 つのファイル(`conductor-<版>-x86_64-unknown-linux-musl.tar.gz`、`…-aarch64-apple-darwin.tar.gz`、`…-x86_64-apple-darwin.tar.gz`、`…-x86_64-pc-windows-msvc.zip`)と `SHA256SUMS` がある | 参照実装のリリースの自動の流れ、README。区分: 点検 |
| AC-440 | 展開の構造 | 各成果物 | 展開 | 最上位に `conductor-<版>-<対象>/` があり、その中に実行ファイル(Windows は `conductor.exe`)と `LICENSE` がある。`conductor` 以外の実行ファイル(テスト用ジョブ)がない | 同上。区分: 点検 |
| AC-441 | チェックサム | `SHA256SUMS` | `sha256sum` の形式で検査 | すべての成果物の行があり、検査が通る | 同上。区分: 点検 |
| AC-442 | 版の表示 | 展開した実行ファイル(macOS Intel を除く) | `<実行ファイル> --version` | 0。OUT = `conductor <版>` | 同上(動作確認)。区分: 点検 |
| AC-443 | 静的リンク(Linux) | Linux の成果物 | `file` など | 静的リンクである(`statically linked` または `static-pie linked`) | 同上。区分: 点検 |
| AC-444 | タグの検査 | タグ `v<版>` | リリースの流れ | タグが `main` 上にない、または版が部品の版と違うと失敗する(タグが `main` 上にない旨 / タグと版の不一致の旨) | 同上。区分: 点検 |
| AC-445 | 下書きとプレリリース | リリースの作成 | (作成されたリリース) | 下書きである。版に `-` があればプレリリース。注記に固定の 1 文がある | 同上。区分: 点検 |
| AC-446 | 3 OS で受け入れテストが通る | Linux・macOS・Windows の実機(または同等の実行環境) | 4 章の自動の項目(各 OS で行える項目) | 全項目が合格する(OS の区分の項目は、その OS で) | 参照実装の CI のテスト(3 OS)。区分: 自動 |

---

## 7. 品質ゲートの要件(言語中立)

マージの前に、次のすべてを通す。参照実装は、これらを CI(継続的インテグレーション)で自動で行い、リリースの前に同じ検査を再度行う。

| 検査 | 基準 | 失敗の扱い |
|------|------|------------|
| 整形 | 差分が 1 つもない | マージを止める |
| 静的検査 | 警告が 1 つもない(警告をエラー扱い) | マージを止める |
| ビルド | 全部品(テストを含む)が、Linux・Windows で、最小の対応版の道具でもビルドできる。C のコンパイルを要する依存、ネットワークの部品がない | マージを止める |
| テスト | Linux・macOS・Windows で、全テスト(単体と統合)が通る。最小の対応版の道具でも通る。プロセスを起動するテストは 30 秒の上限つき | マージを止める |
| 行カバレッジ | 全体で **80% 以上**(テスト用ジョブを除く)。Linux で計測する(Windows 専用のコードは計測の対象に入らない) | マージを止める |
| 文書の検査 | 利用者向けの文書(README)に、必要な節・語が残っている(`SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert`) | マージを止める |
| 最小の対応版の記載の一致 | 最小の対応版の記載(設定、文書、CI の道具の版)が、すべて一致する | マージを止める |
| 脆弱性の検査 | 依存の既知の脆弱性。毎週(月曜 03:00 UTC)と、変更のたびに | 警告のみ(マージを止めない) |
| 起動の性能の測定 | 2.1 節。3 OS で、最適化した版で測定し、サマリに出す | 失敗しても全体は止めない(強制に切り替えられる) |
| Windows の Ctrl+C | 4.10 節の手動の項目を、実機で確かめる | リリースの前に実施し、記録を残す |

ブランチ保護(どの検査を必須にするか)の設定は、リポジトリの外にあり、未確認(8 章)。

---

## 8. 未確認の事項

- ブランチ保護(どの検査を必須にするか)は、リポジトリの設定であり、リポジトリ内のファイルからは確認できない。
- 現在の行カバレッジの実測値、各性能の計測値は、この調査では実行していないため不明(基準だけを記した)。
- 依存の脆弱性の検査の現在の結果、CI の実行環境の将来の変更による影響は未確認。
- Windows の Ctrl+C の手動の確認の記録欄は空で、手動確認の実施の有無は不明。
- 起動の性能の測定を強制に切り替える予定の有無は、コードのコメント(「配分の確定のあと、CI が切り替える」)以上の情報がない。
- 標準出力への書き込みの失敗(`watch`・`convert`)は、参照実装では差し替えた出力先で検査しており、実際の実行ファイルで(例: Linux の `/dev/full`)確かめた記録はない。
- シグナルの準備の失敗(終了 70)と内部の誤り(終了 70)は、実行ファイルの外からは再現できない(注入で確かめる)。
- 大文字小文字を区別しないファイルシステムでの run ID の衝突は、専用のテストがなく、挙動(65 として検出される)は仕様の記述に基づく。
- 4.2 節以降の「派生」の項目(根拠に「派生」とあるもの)は、参照実装に同じ形のテストがなく、規定から導いた。参照実装で通るかどうかは、この調査では実行していない。
- テスト用ジョブの `flaky` モードは、参照実装の補助の実行ファイルにはない(デモのスクリプトに同じ役のものがある)。他の言語の実装は、この仕様どおりに作る必要がある。
- ほぼ同時に届いた複数のシグナルのどれが「最初」になるかは規定していない(07 章)。
- (v0.9.0 以降)統合した環境変数の全体の大きさが OS の上限を超えたときの、起動の失敗(127)の実機の挙動は未確認。層をまたぐ、大文字小文字だけが違う環境変数の名前の、Windows と UNIX 系の違い(AC-467)は、許容として README に書かれている(Windows の実機は未確認)。
- (v0.9.0 以降)`graph` のレーンの配置の規則(05 の 15.5)は、参照実装の正解データ(AC-476)の例から導いたもので、正解データが原本。他の実装は、正解データと同じ結果を返すこと。
- (v0.9.0 以降)AC-452(引き継いだ `CONDUCTOR_RUN_ID`)と AC-496(完了したステートの計画)の一部、AC-453 の「再開で 1 に戻る」の外からの確認は、参照実装に同じ形のテストがなく、規定(05 の 14 章、16 章)から導いた。
- この調査では、テストやビルドは実行していない(ソースとテストの読み取りに基づく)。
