# 11 品質とビルド(非機能要件、テスト、受け入れテスト、CI、リリース、デモ、依存、レイアウト)

対象: `conductor` v0.8.0 の TypeScript(Node.js)版。この文書は、TypeScript で `conductor` を再実装した人が、(1) 同じ品質の基準を満たし、(2) 同じテストを書き、(3) 同じ CI とリリースの仕組みを再現するための仕様である。
この文書の基準を満たし、受け入れテスト(4 章)がすべて通れば、利用者から見て同じ挙動のツールになる。

| 章 | 内容 |
|----|------|
| 1 | この文書の位置づけと、TypeScript 版の前提 |
| 2 | 非機能要件(性能、堅牢性、セキュリティ、保守性)の基準値と、強制か目安かの区別 |
| 3 | テスト構成(単体テスト、受け入れテストの仕組み、テスト用ジョブ、偽の部品、Vitest の設定、OS 別のテスト、略記、判定の書き方) |
| 4 | 受け入れテスト一覧(機能ごと。ID は `AC-xxx`) |
| 5 | CI(GitHub Actions)の全ジョブと、品質ゲートの再現手順 |
| 6 | リリース |
| 7 | デモ |
| 8 | 依存ライブラリと `package.json` / ロックファイルの運用 |
| 9 | リポジトリのレイアウト |
| 10 | 未確認の事項と、他の章との整合が要る点 |

他章との関係: 振る舞いの規定は [03](03-definition-format.md)(定義)、[04](04-cli.md)(CLI)、[05](05-execution-flow.md)(流れ)、[06](06-state-file.md)(ステート)、[07](07-process-and-signals.md)(プロセスとシグナル)、[08](08-output-and-i18n.md)(出力)、[09](09-converter.md)(変換)、[10](10-errors-and-exit-codes.md)(終了コード)にある。
構成は [02](02-architecture.md)、再構築の手順は [12](12-rebuild-guide.md) にある。受け入れテストは、それらを「前提 → 実行 → 期待」の形で、外から確かめる。

---

## 1. この文書の位置づけと、TypeScript 版の前提

- 受け入れテストは、**ビルドした `dist/bin/conductor.js` を、`node` で外から実行して**確かめる(終了コード、標準出力、標準エラー、作られるファイル)。外からは確かめにくい項目は、区分を「単体」「注入」「点検」「手動」「計測」として示す(3.6 節)。
- 定義ファイルは YAML(`.yaml` / `.yml`)か JSON(`.json`)。既定のファイル名は `conductor.yaml`。ステートファイルは JSON で、名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`)。`convert` は YAML と JSON の相互変換(`--to yaml|json`)。
- 受け入れテストの ID は `AC-xxx`。001〜449 は、`conductor` が満たすべき基本の項目。450 以降は、YAML と JSON の扱いに固有の項目(450〜475)、品質・CI・版の項目(476〜487)、Node.js(非同期の入出力、イベントループ、ビルドの成果物)に固有の項目(488 以降)。番号は、この文書の中で一意。
- 期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま([08](08-output-and-i18n.md)、[10](10-errors-and-exit-codes.md))。理由の文の正確な字句は、08・10 章が正本で、この文書の例は、それと同じ字句を使う。
- 確認できなかった点は「未確認」と書き、10 章にまとめる。この文書の作成時には、Node.js、npm、TypeScript、ESLint、Prettier、Vitest、YAML ライブラリ、GitHub Actions の挙動を実機で動かして確かめていない。ツールの設定の書式・オプション名・終了コードの規則は、**実装時に確認する**と書いたものを含め、採用した版の文書で確かめてから使う。
- 版の数字(Node.js、npm、各ライブラリ、GitHub Actions の部品)は、例として書いたものを除き、「実装時に確認する」。

---

## 2. 非機能要件

「強制」は、基準を超えると受け入れテストが不合格になるもの。「目安」は、基準値で、超えたら警告として記録する(不合格にしない)。目安は、利用者から見える挙動を損なわない範囲で、別の値を採ってよい。「仮」は、暫定として置いた値で、実測してから見直す(この文書の作成時には測っていない)。

### 2.1 性能

p95 は、昇順に並べた測定値の `ceil(n × 0.95)` 番目(1 始まり)。測定値は、標準エラーに `[bench] <行>` として出し、環境変数 `GITHUB_STEP_SUMMARY` があれば、そのファイルに `- <行>` として追記する。基準を超えたときは、`警告: <名前>: <値> ms が基準 <限度> ms を超えた` を記録する。
環境変数 `STARTUP_BENCH_ENFORCE` が文字列 `true` のときだけ、目安の基準超えを、テストの失敗(`expect` の不一致)にする(CI の既定は `"false"`)。

Node.js は、起動のたびに JavaScript の実行環境を立ち上げ、ES モジュールを読み込む。そのため、`conductor` を 1 回起動する時間は、ネイティブの実行ファイルより長い。起動時間は目安として測り、利用者から見える挙動の基準にはしない。

| 項目 | 基準 | 扱い | 受け入れテスト |
|------|------|------|----------------|
| `conductor --version` の起動から終了まで(Node の起動と ES モジュールの読み込みを含む。20 回) | p95 250 ms(Linux・macOS)、500 ms(Windows)(仮) | 目安 | AC-488 |
| 定義(ジョブ 10 個)の読み込みと検証(取り込みから結果まで。プロセス内で呼ぶ)を、最初の 5 回を捨てて 100 回 | p95 20 ms | 目安 | AC-397 |
| 起動前の処理(`command` の検査、ステートの読み、照合、速い書き込み)を、最初の 5 回を捨てて 100 回。ジョブ 10 個の定義 | p95 20 ms(仮)。`fsync` つきの耐久性のある書き込みは基準なし(記録のみ) | 目安 | AC-398 |
| `run` の `conductor` 自身のオーバーヘッド(ジョブ 10 個の定義、20 回。`conductor` 自身の Node の起動を含む) | p95 350 ms(仮) | 目安 | AC-399 |
| `pipeline` のジョブ 1 つあたりの増分(ジョブ 10 個と 1 個の `pipeline` の差から求める。5 回) | p95 50 ms | 目安 | AC-400 |
| ジョブ 1,000 個の定義で `run job0` が終わるまで(起動を含む) | 1,500 ms(仮) | 目安 | AC-401 |
| ジョブ 1,000 個の `pipeline` の流れの判断(偽の `Deps` を使い、プロセスとファイルなし) | 10,000 ms | 目安。ただし、完了のステートが残ることは常に検査する | AC-402 |
| ジョブの実行中の `conductor` の CPU 時間の増分(Linux。待機の 1 秒後から 2.5 秒後までの増分) | 50 ms 以下 | **強制**(`/proc` を読めない環境ではスキップ) | AC-403 |
| 同じ定義への 2 つの `run` を同時に | 両方が実行され、どちらもジョブの終了コードで終わり、ステートは読み戻せる(排他制御はしない) | **強制** | AC-404 |
| リトライの待機の中断 | 合図から 1 秒以内に返る。確認の間隔(ポーリング)を持たない | **強制**(1 秒) | AC-275 |
| ジョブの終了の待ち | イベント(`close`)で待ち、確認の間隔を持たない。合図から転送までの遅れは 10 ms 以下 | 目安 | [07](07-process-and-signals.md) |
| ジョブが大量の出力(16 MiB)を標準出力に書いても、`conductor` がそれを溜め込まない | `stdio: 'inherit'` のため、出力は `conductor` を通らない。出力の全体がそのまま読み手に届く | **強制** | AC-493 |

測定のしかた:

- 「`conductor` 自身のオーバーヘッド」は、`conductor` を起動してから終了するまでの時間から、ジョブ自身の時間を引いた値。ジョブ自身の時間は、テスト用ジョブ `timing`(3.4 節)が出す `boot_ms`(Node の起動から、スクリプトの最初の行までの時間)と、`start_ns`・`end_ns` の差の和。
- 「ジョブ 10 個の定義」は、10 個とも同じ形の簡単なジョブ(テスト用ジョブの `exit 0` か `timing`)。
- 「`pipeline` のジョブ 1 つあたりの増分」は、`(10 個の pipeline の全体 − 1 個の pipeline の全体 − 9 × ジョブ 1 つの自身の時間) / 9`。
- 測定の対象は、リリースと同じ作り方の成果物(`npm run build` で作った `dist/`。6 章)。
- Node の JIT(実行時の最適化)の影響を除くため、プロセス内の測定は、最初の 5 回を捨てる。
- 性能のテストは、`test/perf/` に置き、通常のテスト(`vitest run`)とは別の設定(`vitest.perf.config.ts`)で、ファイルを並列に走らせず(`fileParallelism: false`)実行する。測定は 3 OS(Linux・macOS・Windows)で行い、失敗しても全体は不合格にしない(目安)。

### 2.2 堅牢性

外部の入力(コマンドライン、`command` の文字列、ステートファイル、定義ファイル、変換の入力、OS の結果)に対して、**異常終了しない**(失敗は終了コードとメッセージで返す)。【強制】
TypeScript(JavaScript)では、投げられた例外(`throw`)と、拒否された Promise は、捕まえられるが、**捕まえそこねると、`uncaughtException` と `unhandledRejection` になり、既定ではプロセスが落ちる**。また、再帰が深すぎると `RangeError: Maximum call stack size exceeded` が起きる。これは捕まえられる場合があるが、起きた時点の状態は信頼できない。したがって、入れ子の深さと入力の大きさの上限は、再帰や展開に入る前に検査する(AC-412、AC-232)。入口は、`process.on('uncaughtException')` と `process.on('unhandledRejection')` で、残った失敗を 1 行(`conductor: error: internal error`)にして、終了コード 70 で終わる(AC-414、AC-489、AC-490)。

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| ランダムな引数 | 固定の種のランダムな引数列(語の表から 0〜6 個を選ぶ。乱数は 3.1 節の `mulberry32`)500 回で、終了コードが 0〜255、標準エラーに制御文字(改行を除く)と U+202E がない | AC-405 |
| UTF-8 でない引数(UNIX 系) | ジョブID、`--config`、オプション、`convert` の引数に、不正なバイト列を渡しても異常終了しない(Node は、`process.argv` の不正なバイトを U+FFFD に置き換えるとされる。未確認) | AC-406 |
| 壊れたステート | 壊れた JSON、ランダムなバイト列、状態・再開位置・フラグを乱したステート、巨大な値で、異常終了しない(終了コード 65) | AC-407 |
| 空・制御文字を含むジョブID | 異常終了しない | AC-408 |
| ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件で、分割が成功したら `program` は空でない。成功か誤り(64)かのどちらか | AC-409 |
| 大きな `command` | 約 30 万引数が分割できる。閉じない引用符は誤り。再帰に依存しない | AC-410 |
| 起動できない `command` | 空文字列、`'`、`''`、存在しない名前、ディレクトリのパス、`/` は、すべて起動失敗(または定義の誤り)。範囲外のシグナル番号の合図でも起動せず中断として扱う | AC-411 |
| 入れ子の深い入力 | 深さ 1,000 の YAML・JSON(定義・ステート・変換)で、異常終了せず失敗を返す(`node` の `--stack-size` を既定のままで)。`RangeError` が漏れて 70 にならない | AC-412 |
| 製品のコードの書き方 | 外部の入力の経路に、異常終了を起こす書き方(非 null 表明 `!`、型表明 `as`、`any`、`JSON.parse`、`process.exit` の入口以外での呼び出し、待たない Promise、`throw` の非 `Error` 値)がない | AC-421 |
| 内部の誤り | 入口の `try/catch` と `uncaughtException`・`unhandledRejection` が、固定の英語 `conductor: error: internal error` を標準エラーに 1 行書き、終了コード 70。標準エラーが閉じていても 70 | AC-414、AC-489、AC-490 |
| 変換の入力 | 範囲外・空・壊れた入力、ランダム・切り詰めた入力で異常終了しない。成功したランダムな構造は往復で等しい | AC-413 |
| 標準出力の失敗 | 標準出力への書き込みの失敗(`EPIPE`、`ENOSPC` など)で、`'error'` イベントが未処理にならない。`watch` は 65、`convert` は 67 で、メッセージを標準エラーに出す | AC-186、AC-236、AC-492、AC-494 |
| 標準出力の取りこぼし | 標準出力がパイプで、読み手が遅くても、終了前に書き込みの完了を待つ。出力が途中で切れない(`process.exit()` を書き込みの完了前に呼ばない) | AC-493 |
| 終了後に残らない | すべてのサブコマンドが、出力の完了から 1 秒以内に、プロセスとして終了する(タイマー・シグナルの登録・ストリームが残ってハングしない) | AC-495 |
| 警告を出さない | Node が標準エラーに出す警告(`(node:<pid>) ... Warning`)が、通常の実行で出ない | AC-496 |
| 時間の上限 | プロセスを起動するテストは、すべて 30 秒の上限つき(超えたら子プロセスを強制終了して不合格) | 3.3 節 |

### 2.3 セキュリティ

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| シェルを呼ばない | 起動は、`command` の分割(または配列)で得た `program` と引数を、`node:child_process` の `spawn`(`shell: false`)に渡すだけ。`exec`・`execSync`・`execFile`・`fork`、`shell: true`、`sh`・`cmd`・`bash`・`powershell`・`-c`・`/C` を挟む経路がない(Windows の `.cmd` / `.bat` の安全策は 07 章) | AC-116(点検)と 4.2 節 |
| 引数にメタ文字が入っても注入にならない | `a;b`、`&&`、`\|`、`$HOME`、`*`、`>x` などが、そのまま引数として渡る。Windows の `.cmd` への危険な引数で、続くコマンドが実行されない | 4.2 節 |
| 検証は副作用を持たない | 定義の検証は、コマンドを解釈・展開・起動しない。定義ファイルのディレクトリは変わらない | AC-415 |
| 入力の値を表示に出さない | すべての種類のエラーの表示に、入力の値(秘密のつもりの文字列、`command`、ライブラリのエラー文)が現れない。キー名は制御文字をエスケープして表示。**YAML ライブラリが返すエラー文と、例外のスタックを、そのまま表示に出さない**(入力の断片を含みうるため。行・列の位置だけを使う) | AC-416 |
| ステートに機微な値を書かない | ステートファイルに、`command`、引数、環境変数の値、標準出力の内容が現れない | AC-417 |
| 表示のエスケープ | 制御文字・方向制御文字を無効化する(4.14 節の表)。定義・ステート・変換の各部が、同じ入力に同じ結果を返す | 4.14 節 |
| ネットワークに接続しない | 実行中に外向きの通信をしない。製品のコードが `node:net`・`node:http`・`node:https`・`node:http2`・`node:tls`・`node:dgram`・`node:dns` を import せず、グローバルの `fetch`・`WebSocket` を使わない。実行時の依存(YAML ライブラリ)も、それらを使わない | AC-419(点検) |
| 動的なコードの実行をしない | 製品のコードが、`eval`、`new Function`、`node:vm`、`node:worker_threads`、ネイティブのアドオン(`process.dlopen`、`.node` ファイルの読み込み)、`process.binding` を使わない | AC-420(点検) |
| シグナルの送信 | ジョブの子プロセスの pid だけ(0 以下にならない)、許可した 4 つのシグナルだけ、1 回だけ(プロセスグループには送らない) | 4.10 節の単体の項目 |
| テスト専用の経路を製品に持たない | 製品の入口に、わざと異常終了するコマンドや、テスト用の環境変数・隠しオプションがない。配布物に `test/` を含めない | AC-422(点検)、6 章 |
| 変換の副作用 | 変換の部品(`src/convert/`)が `node:child_process`・ネットワークのモジュールを import しない。`node:fs` は、入力ファイルを読む入口だけが使い、本体(文字列から文字列への変換)と書き出しは、ファイルにも環境変数にもカレントディレクトリにも触れない。`convert` は入力以外のファイルを作らず、カレントディレクトリの変更と環境変数に左右されない | AC-418 |
| 既知の脆弱性 | 依存の既知の脆弱性を、定期的に(毎週)と、変更のたびに `npm audit` で検査する。結果は警告で、マージをブロックしない | 5 章 |
| 供給網(ビルドの再現) | `package-lock.json` をコミットし、CI は `npm ci` だけを使う(ロックファイルと `package.json` が食い違えば失敗)。依存のインストールスクリプトに頼らない(実行時の依存は 1 つ。8 章) | 5 章、AC-478 |
| 配布物の完全性 | チェックサムのファイル(`SHA256SUMS`)を付ける(署名はしない。転送の破損の検知まで) | 6 章 |

### 2.4 保守性

| 項目 | 基準 | 受け入れテスト |
|------|------|----------------|
| OS 固有コードの隔離 | OS ごとの分岐(`process.platform`、`os.platform()`)は、`src/runner/platform.ts` の 1 か所だけ。ほかの部品は、そこが出す関数・値(UNIX 系か Windows かの判定、受け取るシグナルの一覧、ディレクトリの同期の可否)だけを使う | AC-420 |
| 外部の入力で異常終了しない書き方 | 2.2 節 | AC-421 |
| 型の検査 | `tsc --noEmit` が `strict` で通る。`noUncheckedIndexedAccess`、`exactOptionalPropertyTypes`、`noImplicitOverride`、`noFallthroughCasesInSwitch`、`verbatimModuleSyntax` を有効にする(9.3 節) | AC-480、AC-483 |
| 整形と静的検査 | `prettier --check` の差分、ESLint の警告・エラー(`--max-warnings 0`)が 1 つでもあればマージを止める | 5 章 |
| 命名 | TypeScript の標準の命名規則(型・クラスは PascalCase、関数・変数は camelCase、定数は SCREAMING_SNAKE_CASE か camelCase、ファイル名は小文字とハイフン)に従う。独自の規則は設けない | (規約) |
| 部品の分け方 | `src/definition/` と `src/flow/`(入出力なし。ファイルとプロセスに触れずに検査できる)、`src/runner/` と `src/state/`(入出力あり)、`src/convert/`、`src/report/`、`src/cli/`(引数・流れの組み立て・`Deps`)を分ける([02](02-architecture.md)) | (規約) |
| ソースへの著作権表示 | 各ソースファイル(`.ts`、`.js`、`.mjs`、`.yml`、`.sh`、`.cmd`)の先頭に、Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)を置く | AC-423(点検) |

---

## 3. テスト構成

### 3.1 方針

- 方法論は test-after(実装してから、層ごとにテストを書いて実行する)。単体テストと、ビルドした `dist/bin/conductor.js` を `node:child_process` で起動する受け入れテストを持つ。
- テストの枠組みは Vitest(`vitest run`)。アサーションは Vitest の `expect` を使う。テスト用の別のライブラリ(アサーション、モック、擬似端末など)は入れない(8 章)。
- 一時ディレクトリは、各テストが `fs.mkdtempSync(path.join(os.tmpdir(), 'conductor-test-'))` で作り(以下 `T`)、`afterEach` で `fs.rmSync(T, { recursive: true, force: true })` で消す。テストごとに別の `T` を使うので、**ステートファイルの名前(`<定義>.state`)は、テストの間で衝突しない**。環境変数は、テストのプロセスの `process.env` を書き換えず、子プロセスの `env` に渡す(プロセス内の単体テストは、偽の `Deps` で環境を渡す)。
- 入力と期待の組が多いものは、**テーブル駆動**(`it.each([...])` か、配列を `for ... of` で回して `it(...)` を作る)で書く。ID(`AC-xxx`)は、テストの名前に含める(例: `it('AC-092 一重引用符が閉じていない', ...)`)。
- テストの名前は、日本語でも英語でもよい。
- 行カバレッジの下限は **80%**(5 章)。
- プロセスを起動するテストは、30 秒の上限で、ハングだけを検知する(3.3 節)。
- 乱数が要るテストは、固定の種の擬似乱数 `mulberry32` を使う(`Math.random` は種を決められないので使わない)。実装(32 ビット符号なしの演算。`Math.imul` を使う):

```ts
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
```

### 3.2 部品ごとの単体テスト

`src/<部品>/` の各部品は、`test/<部品>/` に単体テスト(`*.test.ts`)を置く。

| 部品 | 主な試験対象 | 方法 | 主な AC |
|------|--------------|------|---------|
| `src/definition/` | 定義の読み込み・検査・型・エラーの種類と行番号。YAML と JSON の読み取り(タグ・複数文書・重複キー・非文字列キーの拒否、深さと大きさの上限)。`command` の分割。範囲外の整数の字句検査(`bigint`) | テーブル駆動。`test/fixtures/*.yaml`・`*.json` の入力例 | AC-001〜098、AC-450〜 |
| `src/convert/` | YAML ⇔ JSON の変換、往復、表せない値、アンカーの展開、暴走する入力の上限 | テーブル駆動。往復の性質テスト(固定の種) | AC-206〜238 |
| `src/runid/` | run ID の検査(文字種と長さの規定は [05](05-execution-flow.md))と決定の順序 | テーブル駆動 | AC-239〜260 |
| `src/flow/` | 純粋な判断: 分類 10 項目、遷移、再開、リトライの待ち時間(`bigint` の飽和演算)、終了コードの範囲合わせ | テーブル駆動。ファイル・プロセスなし | AC-268、AC-392〜394、AC-499 |
| `src/state/` | ステートの読み書き(実ファイル、`T`)、整合性検査(14 段階)、原子的な置換、reset(`--all`、シンボリックリンク) | 実ファイル。書き込みの失敗は、置き換え先をディレクトリにして起こす | AC-302〜324 |
| `src/runner/` | 子プロセスの起動と待ち、環境変数・作業ディレクトリの引き継ぎ、シグナルの転送(1 回だけ・許可した番号だけ)、シグナル名と番号の対応 | テスト用ジョブ `TJ` を実際に起動 | AC-099〜116、AC-280〜292、AC-502 |
| `src/report/` | 表示の全種類の文言(英語・日本語)、ロケールの決定、色、エスケープ、`watch` の表 | テーブル駆動 | AC-350〜388 |
| `src/cli/` | 引数の文法、各サブコマンドの流れ、終了コードの決定 | **偽の `Deps`**(3.7 節)。ファイル・プロセス・シグナルなし | AC-125〜、AC-293〜300、AC-325〜349 |
| `test/support/` | 偽の部品、一時ディレクトリの補助、30 秒の上限つきの実行、記録の読み取り | (試験の道具。単体試験は不要。カバレッジの対象から除く) | — |

### 3.3 受け入れテストの仕組み(`child_process` の統合テスト)

受け入れテストは、ビルドした `conductor` と、テスト用ジョブ `TJ`(3.4 節)を、`node:child_process` の `spawn` で起動して確かめる。置き場所は **`test/acceptance/`** の `*.test.ts`。機能ごとにファイルを分ける。

| ファイル | 内容 |
|----------|------|
| `test/support/global-setup.ts` | Vitest の `globalSetup`。ビルドと後始末(下記) |
| `test/support/harness.ts` | 共通の補助(3.8 節): `runConductor`、定義とステートの書き出し・読み取り、30 秒の上限 |
| `test/acceptance/validate.test.ts` | 4.1 節、4.2 節、450〜 の定義の検査 |
| `test/acceptance/run.test.ts` | 4.3 節、4.4 節(`run`、`pipeline`) |
| `test/acceptance/aux.test.ts` | 4.5 節、4.6 節(`watch`、`reset`)、4.8 節(run ID) |
| `test/acceptance/convert.test.ts` | 4.7 節 |
| `test/acceptance/retry.test.ts` | 4.9 節 |
| `test/acceptance/signal.test.ts` | 4.10 節(UNIX 系の項目は `describe.skipIf(process.platform === 'win32')`) |
| `test/acceptance/state.test.ts` | 4.11 節 |
| `test/acceptance/args.test.ts` | 4.12 節〜4.15 節 |
| `test/acceptance/nfr.test.ts` | 4.16 節(堅牢性・点検) |
| `test/acceptance/node.test.ts` | 4.20 節(Node.js に固有の項目) |
| `test/acceptance/demo.test.ts` | 4.17 節(デモ。UNIX 系と Windows で `skipIf`) |
| `test/perf/*.test.ts` | 2.1 節の計測(別の設定 `vitest.perf.config.ts`) |
| `test/inspect/*.test.ts` | 点検(TypeScript のコンパイラ API で、ソースを調べる。3.6 節) |

`globalSetup` の手順:

1. 環境変数 `CONDUCTOR_BIN` があれば、そのパスを `conductor` として使う(配布物を試験するため。AC-446)。パスの拡張子が `.js` または `.mjs` なら、`process.execPath`(試験を走らせている `node`)で実行する。それ以外(単一の実行ファイルなど)は、そのパスを直接実行する。環境変数 `CONDUCTOR_TESTJOB` があれば、`TJ` のスクリプトのパスとして使う(なければ `test/jobs/tj.mjs`)。
2. `CONDUCTOR_BIN` がなければ、リポジトリの根をカレントディレクトリにして、`npx tsc -p tsconfig.build.json`(`npm run build` と同じ)を 1 回だけ実行し、`dist/bin/conductor.js` を使う。リポジトリの根は、`globalSetup` のファイルの場所(`import.meta.dirname` または `fileURLToPath(import.meta.url)`)から `../..` と決める。
3. 決めたパスを、`provide`(Vitest の `project.provide`)か、環境変数(子の試験のプロセスが継ぐ `process.env.CT_BIN`)で、各テストのファイルに渡す。

各テストは、`T` をカレントディレクトリにして、`spawn(process.execPath, [conductorJs, ...args], { cwd: T, env, stdio: ['pipe' か 'ignore', 'pipe', 'pipe'], shell: false })` で実行する。`env` は 3.5 節の既定の環境変数だけ(親の環境を引き継がない)。
30 秒を超えたら、`conductor` を `child.kill('SIGKILL')`(Windows は `child.kill()`)で止め、記録(`pid=`)に書かれたジョブの pid があればそれも強制終了して、テストを失敗にする(`harness.runConductor` の 30 秒の `setTimeout`)。加えて、Vitest の `testTimeout`(3.9 節)で、テストの全体にも上限を付ける。

### 3.4 テスト用ジョブ(`test/jobs/tj.mjs`、以下 `TJ`)

受け入れテストで使う、小さな Node のスクリプト。配布物には含めない(6 章)。TypeScript のビルドを要さないよう、**素の JavaScript(`.mjs`)**で書き、`node:fs` と `node:process` だけを import する(Node の起動を速くし、読み込む部品を最小にするため)。
定義の中では、`TJ` は、2 つの語 `"<process.execPath>" "<tj.mjs の絶対パス>"` を表す(各語を二重引用符で囲む。パスに空白があってもよい)。例: `command: "TJ exit 3"` は、定義の中で `"/usr/bin/node" "/repo/test/jobs/tj.mjs" exit 3` という `command` になる。
定義を書き出す補助(`harness.writeDefinition`)は、すべての文字列を `JSON.stringify` で出す。JSON の文字列は、YAML の二重引用符の文字列としても同じ値を表すので、Windows のパス(`\`)も、引用符も、そのまま正しく書ける。

```text
TJ [--out <ファイル>] <モード> [引数...]
```

- 1 回の `TJ` の起動は、Node の起動(数十 ms の見込み。実測は実装時に確認する)を伴う。起動の費用を抑えるため、(1) `tj.mjs` は、上記の 2 つの組み込みモジュール以外を読まない、(2) 受け入れテストは、`vitest` の `pool: 'forks'` の複数のプロセスで並列に走る(3.9 節)、(3) 大量のジョブを要するテスト(AC-402 など)は、`TJ` を使わず、偽の `Deps` で行う、(4) 性能の測定では、`TJ` 自身の起動時間を `timing` が `boot_ms` として出し、差し引く(2.1 節)。
- `--out <ファイル>` が先頭にあれば、**記録の行**をそのファイルに追記する(`fs.appendFileSync(file, line + '\n')`、1 行 1 件)。なければ標準出力に書く。ジョブの標準出力を被試験体越しに読めない場合があるため、結果はファイルで受け取る。
- 記録に出す文字列(値)は、`q(s)` で表す: 二重引用符で囲み、`"` と `\` の前に `\` を置き、改行は `\n`、復帰は `\r`、タブは `\t` と書く(`JSON.stringify` とは別。非 ASCII はそのまま。他の制御文字は変換しない)。
- 未知のモードは、終了コード 2。
- 終わり方は、`process.exitCode = n` を設定してイベントループが空になるのを待つか、書き込みを終えてから `process.exit(n)` を呼ぶ(`TJ` は製品ではないので、後者でよい。ただし `say` は、書き込みの完了(`write` のコールバック)を待ってから終わる)。

| モード | 動作 |
|--------|------|
| `exit <n>` | 終了コード n で終わる(省略・整数でないときは 2) |
| `run <label> <n>` | 記録 `ran=<label>` を出し、終了コード n(省略・整数でないときは 0) |
| `echo <args...>` | 各引数を、記録 `arg[i]=q(引数)`(i は 0 から)として 1 行ずつ出し、終了コード 0 |
| `env <NAME>...` | 各環境変数を、設定されていれば `env[NAME]=set:q(値)`、なければ `env[NAME]=unset` として出す(`NAME in process.env`)。終了コード 0 |
| `cwd` | カレントディレクトリ(`process.cwd()`)を `cwd=q(パス)` として出す。終了コード 0 |
| `pid` | 自分のプロセス識別子を `pid=<N>`(`process.pid`)として出す |
| `say <out> <err> [n]` | `<out>` を標準出力に、`<err>` を標準エラーに、改行を付けずにそのまま書き(書き込みの完了を待つ)、終了コード n(省略は 0)。`--out` に従わない |
| `sleep [秒]` | `pid=<N>` と `ready` を記録し、眠る(`setInterval(() => {}, 1 << 30)` で、イベントループを保つ)。秒があれば、その秒のあと終了コード 99。なければ眠り続ける。SIGTERM・SIGINT・SIGHUP を受けたら、ハンドラが自分のシグナルのハンドラを外し、`process.kill(process.pid, <受けたシグナル>)` で自分に同じシグナルを送り直す(シグナルで終了したように観測されるようにする。確認は実装時。10 章) |
| `trap <ms> <code>` | `pid=<N>` と `ready` を記録する。SIGTERM・SIGINT・SIGHUP を受けたら、記録 `trapped` を出し、`<ms>` ミリ秒待ってから、記録 `done` を出し、終了コード `<code>` で終わる(シグナルで中断された後も、ジョブが終わるまで `conductor` が待つことを確かめる。AC-503) |
| `flaky <counter-file> <fails> <code>` | `<counter-file>` の整数(なければ 0)に 1 を足して書き戻す。足した値が `<fails>` 以下なら終了コード `<code>`、超えたら 0 |
| `timing` | `boot_ms=<起動から最初の行までのミリ秒。performance.now()>`、`start_ns=`・`end_ns=`(UNIX 時刻のナノ秒を `BigInt(Math.round((performance.timeOrigin + performance.now()) * 1e6))` で求める)を、標準出力に書く(性能の測定用) |
| `burn <ms>` | 指定のミリ秒(既定 100)、CPU を使い続けて終わる(CPU 時間の測定用) |
| `stdin` | 標準入力を終わりまで読み、読んだ内容を記録 `stdin=q(内容)` として出し、終了コード 0(標準入力の引き継ぎの確認。AC-507) |
| `big <MiB>` | 標準出力に、指定の MiB(既定 16)の `x` を、書き込みの完了を待ちながら書き、終了コード 0(AC-493) |

補足:

- `sleep` モードで SIGQUIT を受けると、Node の既定の動作(コアダンプを伴う終了)になる(実装時に確認する)。したがって、AC-282 の SIGQUIT の項目は、`conductor` の終了コード 131 だけを確かめ、ジョブの終わり方は確かめない。
- UNIX 系の受け入れテストでは、シグナルで終わるジョブを `["sh", "-c", "kill -9 $$"]`(配列形式)で作る(`TJ` にモードを足さない)。
- `TJ` の `env`・`cwd` の記録の値は、`q(s)` で出すので、OS によらず同じ書式になる。

### 3.5 定義とステートの略記と、実行環境

**実行環境**

- 実行ファイル `conductor`(以下「被試験体」)と `TJ` を用意する(3.3 節)。
- 各テストは、空の一時ディレクトリ(`T`)で行う。`T` の中に定義ファイル(既定の名前 `conductor.yaml`)を作り、`T` をカレントディレクトリにして被試験体を起動する。ステートファイルは、定義ファイルと同じディレクトリにできる(例: `T/conductor.yaml.state`)。
- 環境変数は、特に記さない限り、次のとおり。`LANG=C`。`LC_ALL`・`LC_MESSAGES`・`NO_COLOR`・`CONDUCTOR_RUN_ID`・`NODE_OPTIONS` は未設定にする(表示を英語に固定し、利用者の環境の影響を除く)。`PATH` と(Windows の)`SystemRoot`・`ComSpec`・`PATHEXT`・`TEMP`・`TMP` は、親から引き継ぐ。
- 標準入力は空(`stdio` の 0 番を `'ignore'`。`/dev/null` に相当)にする。標準入力を使う項目は、`'pipe'` にして書き込む。
- パスに空白や非 ASCII を含めない(特に記す場合を除く)。

**定義の略記** — 前提の「定義」は、次の略記で書く。

```text
P(<パイプライン名> [run_id_env=<ENV>] [remove_on_complete=<true|false>]; <ジョブ>, <ジョブ>, ...)
```

| 略記 | 展開 |
|------|------|
| `P(p; ...)` | `format_version: 1`、`pipeline:` の `name: p`、続いて `job:` の列(`- id: ...`)を、書いた順に並べる |
| `<id>:<n>` | `job` の要素に `id: <id>`、`command: "TJ exit <n>"` |
| `<id>:<n>{...}` | 同上に、波括弧の中の修飾を足す |
| `w=[…]` | `warning_exit_codes: […]` |
| `s=[…]` | `success_exit_codes: […]` |
| `on=success->X` / `on=warning->X` | `transitions:` に `- on: success`(または `warning`)と `next: X`(X はジョブID か `@end`。**`@end` は YAML では `"@end"` と二重引用符で囲む**。`@` で始まる値は、引用符なしでは書けない) |
| `retry=(exit_codes=[…], max=N, interval="…", multiplier=M, max_interval="…")` | `retry:` の写像(書いたキーだけ) |
| `cmd=<文字列>` | `command` を、その文字列(YAML の文字列。`TJ` は 3.4 節の 2 つの語。必要なら二重引用符で囲む)にする。`id:n` の `n` は使わない |
| `argv=[…]` | `command` を、その文字列の列(YAML の流れ形式の列)にする。`TJ` を書くときは、`process.execPath` と `tj.mjs` のパスを、別々の要素にする |
| `O` | 記録の出力ファイル(`T/out.txt` など。`--out` に渡す) |

例: `P(p; a:0, b:7{w=[3]}, c:0{on=success->@end})` は、次の定義(`TJ` は、3.4 節の 2 つの語の略)。

```yaml
format_version: 1
pipeline:
  name: p
job:
  - id: a
    command: "TJ exit 0"
  - id: b
    command: "TJ exit 7"
    warning_exit_codes: [3]
  - id: c
    command: "TJ exit 0"
    transitions:
      - on: success
        next: "@end"
```

JSON の定義(`conductor.json`)で試験する項目は、同じ構造を JSON のオブジェクトで書く(例: `{"format_version": 1, "pipeline": {"name": "p"}, "job": [{"id": "a", "command": "TJ exit 0"}]}`)。キー名と構造は、YAML と同じ([03](03-definition-format.md))。

**ステートの略記** — ステートの期待は、次の略記で書く。`end` と `completed` は、書かなければ `false`。`resume` は、書かなければ「キーがない」。

```text
S[<id>=<状態>(<終了コード>), ... ; resume=<id>, end=<true|false>, completed=<true|false>, run_id=<id>]
```

状態は `pending`・`success`・`warning`・`error`・`skipped`。スキップは `skipped/byTransition` か `skipped/byEnd`。終了コードがなければ括弧を省く。例: `S[a=success(0), b=error(7), c=pending ; resume=b]`。ステートファイルの書式(JSON)は [06](06-state-file.md)。受け入れテストは、ステートファイルを(テストでは)`JSON.parse` で読み、論理項目(`pipeline_name`、`job_order`、`job` の各要素の `id`・`status`・`skip_reason`・`exit_code`、`resume_job_id`、`end_reached`、`completed`、`run_id` など)を比べる。`JSON.parse` を使ってよいのは、テストだけである(製品のコードは自前の読み取りを使う。AC-421)。

### 3.6 判定の書き方と区分

| 欄 | 意味 |
|----|------|
| 実行 | 被試験体の引数。`T` をカレントディレクトリにする。`--config` を省くと `T/conductor.yaml` |
| 終了 | 終了コード |
| OUT | 標準出力。「空」は 0 バイト。引用は完全一致。「含む」は部分一致 |
| ERR | 標準エラー。同上。複数行は `⏎` で区切って書く(行の終わりの改行は省く) |
| 前提 | 実行の前に作るもの(定義、ステート、環境変数) |

- どのテストも、**30 秒の上限**で実行する(ハングの検知)。
- 「ジョブが実行された」は、`TJ run <label> <n>` の記録 `ran=<label>` で確かめる(`O` を読む)。「実行されない」は、その記録がないこと。
- 「ステートが変わらない」は、ステートファイルの内容(バイト列)が、実行の前後で同じこと。「ステートがない」は、ファイルがないこと。
- 「UNIX 系」「Windows」「非 root」と書かれた項目は、その環境でだけ行う(`it.skipIf(process.platform === 'win32')`、`it.skipIf(process.platform !== 'win32')`。root かどうかは `process.getuid?.() === 0`。3.10 節)。

| 区分 | 意味 |
|------|------|
| (自動) | 被試験体を外から実行して確かめる |
| 単体 | 内部の関数(分割、終了コードの範囲合わせ、エスケープ、判断の純粋な関数)を直接呼んで確かめる。外から再現しにくい入力のため |
| 注入 | 標準出力への書き込みの失敗、シグナル登録の失敗など、外から作れない失敗を、偽の `Deps`(3.7 節)で起こして確かめる。外から作れる範囲では、`node --import <注入のスクリプト>` で、被試験体の前に別のスクリプトを読み込ませる(製品に試験用の経路を持たせない。AC-489) |
| 点検 | ソースや依存を、TypeScript のコンパイラ API(`typescript` の `createSourceFile` と構文木の走査)、`package.json`、`npm ls`、`npm pack --dry-run --json` で調べて確かめる(実行ではない。`test/inspect/` に置く) |
| 手動 | 人が実機で確かめて、記録を残す |
| 計測 | 性能の測定(2.1 節) |

### 3.7 偽の部品(`Deps`)を使うテスト

`src/cli/` の `Deps` は、次の役割を TypeScript の interface で持つ([02](02-architecture.md))。`test/support/fakes.ts` が、それぞれの偽物を提供する。

| 役割 | 本物 | 偽物(`test/support/fakes.ts`) |
|------|------|-------------------------------|
| 環境変数 | `process.env` | `FakeEnv`(`Record<string, string>`) |
| 時刻・待ち | `Date.now`、`setTimeout`(`AbortSignal` で中断できる待ち) | `FakeClock`(待ちを記録し、すぐに返す。合図で中断するか選べる) |
| ファイルシステム | `node:fs` と `src/state/` の原子的な書き込み | `MemFs`(メモリ上のステート。書き込み・削除の失敗を起こせる) |
| プロセス実行 | `src/runner/` | `FakeRunner`(用意した結果を順に返す。起動の回数と引数を記録) |
| シグナル | `process.on(<シグナル>)` | `FakeSignals`(任意の時点で番号を立てる。準備の失敗・「登録済み」を返せる) |
| 標準出力・標準エラー | `process.stdout` / `process.stderr` | 文字列を溜める `MemWriter`、書き込みが常に失敗する(または `EPIPE` を返す)`FailingWriter` |

偽の部品で確かめる項目(区分「注入」)は、AC-293〜300、AC-324、AC-366〜368、AC-485、AC-491 など。`src/cli/` の流れのテストは、偽の部品の上で、`run`・`pipeline`・`watch`・`reset`・`convert`・`validate` を呼び、終了コード・標準出力・標準エラー・`MemFs` のステートを比べる。
本物の部品との結合は、受け入れテスト(3.3 節)が確かめる。

### 3.8 テストの補助(`test/support/harness.ts`)

| 関数 | 役割 |
|------|------|
| `makeTempDir()` | `T` を作る(3.1 節)。後始末は `afterEach` |
| `defaultEnv(extra?)` | 3.5 節の既定の環境変数に `extra` を足した `env` を返す |
| `writeDefinition(T, name, source)` | 略記(3.5 節)の定義を、YAML か JSON で書き出す。文字列は `JSON.stringify` で出す |
| `runConductor(args, { cwd, env, stdin?, timeoutMs = 30_000 })` | 被試験体を `spawn` し、`{ code, signal, stdout, stderr }`(文字列。UTF-8 で復号)を返す。30 秒で強制終了して失敗にする。標準出力・標準エラーは、終了(`close`)まで読み切る |
| `startConductor(args, opts)` | 実行中に操作する項目(シグナルを送る、標準エラーの行を待つ)のために、`child` と、行ごとの待ち受け(`waitForStderr(pattern)`)を返す |
| `readRecords(file)` | `TJ` の記録の行を読む |
| `readState(T, name)` | ステートファイルを読んで、`JSON.parse` した値を返す |
| `listDir(T)` | `T` のファイル名の一覧(ソート済み) |

### 3.9 Vitest の設定

`vitest.config.ts`(通常):

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    exclude: ['test/perf/**', 'node_modules/**', 'dist/**'],
    globalSetup: ['test/support/global-setup.ts'],
    pool: 'forks',
    testTimeout: 60_000,
    hookTimeout: 120_000,
    teardownTimeout: 10_000,
    coverage: {
      provider: 'v8',
      include: ['src/**/*.ts'],
      exclude: ['src/bin/**'],
      reporter: ['text', 'json-summary'],
      thresholds: { lines: 80 },
    },
  },
});
```

`vitest.perf.config.ts`(性能。通常の設定を土台にして、対象と並列を変える):

```ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['test/perf/**/*.test.ts'],
    globalSetup: ['test/support/global-setup.ts'],
    pool: 'forks',
    fileParallelism: false,
    testTimeout: 300_000,
    hookTimeout: 120_000,
  },
});
```

| 項目 | 決め | 理由 |
|------|------|------|
| `pool` | `'forks'`(子プロセスで試験を走らせる) | 受け入れテストが子プロセスを起動し、シグナルを扱う。`worker_threads` のプールでは、`process.chdir()` やシグナルの扱いに制約があるため、プロセスで分ける(Vitest の版により既定が異なるため、明示する。実装時に確認) |
| `testTimeout` | 60 秒 | プロセスを起動するテストは 30 秒の上限(3.3 節)を自分で持つ。Vitest の上限は、それより長く置き、自前の 30 秒の失敗の説明を先に出す |
| `hookTimeout` | 120 秒 | `globalSetup` のビルド(`tsc`)に使う余裕 |
| ファイルの並列 | 既定(並列)。受け入れテストは、テストごとに別の `T` を使うので、ステートファイルは衝突しない | 実行時間を短くするため。同じ `T` を複数のテストで共有しない |
| 性能の試験の並列 | 並列にしない(`fileParallelism: false`) | 同時に走るテストが、測定値を乱すため |
| カバレッジ | `provider: 'v8'`。対象は `src/` の `.ts`、除外は `src/bin/`(入口。薄い。受け入れテストが通す)。**受け入れテストで起動した子プロセスの中のコードは、数えない**(子のプロセスのカバレッジの収集は、この仕様では行わない)。下限は、行 80% | 5.3 節 |
| スナップショット | 使わない(期待は、文言そのものをテストに書く) | 文言の変更に気づくため |
| `thresholds` の書式 | `coverage.thresholds.lines`(版により書式が異なる。実装時に確認)。5.3 節の `check-coverage.mjs` が、書式によらない独立の検査を行う | |

### 3.10 OS 別のテスト

- UNIX 系だけのテスト(シグナル、権限、シンボリックリンク、新しいプロセスグループ)は、`describe.skipIf(process.platform === 'win32')` か `it.skipIf(process.platform === 'win32')` で囲む。Windows だけのテスト(`.cmd` の安全策、Ctrl+C のチェックリストに対応する自動の部分)は、`skipIf(process.platform !== 'win32')`。Linux だけのテスト(`/dev/full`、`/proc` による CPU 時間)は、`skipIf(process.platform !== 'linux')`。
- 非 root のテスト(権限の拒否を起こす項目)は、`skipIf(process.getuid?.() === 0)` を加える。`process.getuid` は Windows にない(`undefined`)ので、オプショナルの呼び出しにする。
- `skipIf` でスキップされた項目は、結果に「skipped」として残る。CI では、OS ごとにスキップされる件数が、想定の範囲であることを、実装時に確認する。
- OS 固有の製品のコードは、`src/runner/platform.ts` に集める。他のファイルから見ると、OS の違いは見えない(AC-420)。
- どの OS でも、`tsc --noEmit` と ESLint が通る(OS 固有の分岐は、実行時の `process.platform` による分岐であり、型の検査は OS によらず同じに通る)。

### 3.11 テストの実行

```sh
npm ci                              # ロックファイルのとおりに依存を入れる
npm run build                       # tsc で dist/ を作る(受け入れテストの globalSetup も、なければ作る)
npm test                            # vitest run(単体 + 受け入れ。3 OS で通る)
npm run test:coverage               # vitest run --coverage(行 80%。Linux)
npm run test:perf                   # 性能の測定(2.1 節)
CONDUCTOR_BIN=<配布物の dist/bin/conductor.js> npx vitest run test/acceptance   # 配布物の試験(AC-446)
```

`package.json` の `scripts`(9.2 節)に、上の `npm run` の名前を定義する。

---
## 4. 受け入れテスト一覧

ID は、`AC-` と通し番号。表は、ID・機能・前提・実行・期待の順。期待の標準エラーの文言は、特に記さない限り `LANG=C`(英語)。理由の文(日本語の固定の文)は、英語の表示でも日本語のまま。
YAML と JSON に固有の項目は 4.19 節(AC-450〜487)、Node.js に固有の項目は 4.20 節(AC-488 以降)にまとめた。この節の定義は、特に記さない限り YAML の `conductor.yaml`。

### 4.1 定義の検査(`validate`、`run`、`pipeline` 共通)

特に記さない限り、実行は `validate`(`--config` を省く)。「64」の期待は、終了 64、OUT 空、ステートファイルを作らない、ジョブを起動しない、を含む。
ERR の期待は、**最初の行**のうち、`conductor: error: <ファイル>[:<行>]: ` に続く種類の文([08](08-output-and-i18n.md))を示す。ファイル名は、`--config` に渡した表記のまま。行は、誤りの位置の行(1 始まり)。
基本の定義 `B0` は、`P(p; a:0)` とする。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-001 | 正しい定義 | `B0` | `validate` | 終了 0。OUT・ERR 空。`T/conductor.yaml.state` を作らない |
| AC-002 | validate の -v(1 ジョブ) | `B0` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (1 job)`。OUT 空 |
| AC-003 | validate の -v(複数) | `P(p; a:0, b:0, c:0)` | `validate -v` | 終了 0。ERR = `[conductor] definition is valid (3 jobs)` |
| AC-004 | validate の -v(日本語) | `P(p; a:0, b:0, c:0)`、`LANG=ja_JP.UTF-8` | `validate -v` | 終了 0。ERR = `[conductor] 定義は正しいです(ジョブ 3 個)` |
| AC-005 | 定義ファイルがない | ファイルなし | `validate --config missing.yaml` | 64。ERR = `conductor: error: missing.yaml: definition file does not exist: 指定の場所にファイルがありません` |
| AC-006 | 定義ファイルがない(日本語) | 同上、`LANG=ja_JP.UTF-8` | 同上 | 64。ERR = `conductor: error: missing.yaml: 定義ファイルが存在しません: 指定の場所にファイルがありません` |
| AC-007 | 通常のファイルでない | `T/d.yaml` がディレクトリ | `validate --config d.yaml` | 64。ERR の種類の文 = `cannot read definition file`(理由 `通常のファイルではありません`) |
| AC-008 | 空のファイル(YAML) | 長さ 0 の `conductor.yaml`(コメントだけのファイルも同じ) | `validate` | 64。ERR = `conductor: error: conductor.yaml: format_version is missing: …`(行なし) |
| AC-009 | 文法の誤り(行つき) | `format_version: 1⏎pipeline: ]⏎` | `validate` | 64。ERR の最初の行が `conductor: error: conductor.yaml:2: syntax error: ` で始まる |
| AC-010 | キーの重複(YAML。文法の誤り) | `format_version: 1⏎format_version: 1⏎` | `validate` | 64。種類の文 = `syntax error`(行 2) |
| AC-011 | 書式の版が 1 でない | `format_version: 2` に `B0` の他を足す | `validate` | 64。`unsupported format_version` |
| AC-012 | 書式の版が整数でない | `format_version: "1"`、`format_version: 1.0`、`format_version: true` | `validate` | 64。`unsupported format_version` |
| AC-013 | 版の確認が先 | `format_version` なし、未知のキーと型の誤りを含む | `validate` | 64。ERR は 1 行だけ。種類の文 = `format_version is missing` |
| AC-014 | `jobs`(複数形)のキー | `B0` の `job:` を `jobs:` に | `validate` | 64。`unknown key` |
| AC-015 | 未知のキー(ジョブ) | `B0` の job に `colour: red` | `validate` | 64。`unknown key`(行はそのキーの行) |
| AC-016 | 未知のキー(パイプライン) | `pipeline:` に `owner: x`(または `remove_on_complet: true`) | `validate` | 64。`unknown key` |
| AC-017 | `pipeline` がない | `format_version: 1` と `job:` だけ | `validate` | 64。`wrong value type`(行 1) |
| AC-018 | `pipeline` が写像でなく列 | `pipeline: [{name: p}]` | `validate` | 64。`wrong value type` |
| AC-019 | パイプライン名が空 | `name: "  "` | `validate` | 64。`required value is empty` |
| AC-020 | ジョブがない | `job` を書かない(または `job: []`) | `validate` | 64。`no jobs are defined` |
| AC-021 | ジョブIDの重複 | `P(p; a:0, a:0)` | `validate` | 64。`duplicate job id` |
| AC-022 | ジョブIDが不正 | ジョブID が `""`、または `"@x"` | `validate` | 64。`invalid job id` |
| AC-023 | 遷移先がない | `P(p; a:0{on=success->nowhere})` | `validate` | 64。`transition target job is not defined` |
| AC-024 | エラー結果への遷移 | `transitions` の `on: error` | `validate` | 64。`no transition can be written for the error result` |
| AC-025 | `on` の値が不正 | `on: later`、または `on: Success` | `validate` | 64。`wrong value type` |
| AC-026 | 自分自身・前のジョブへの遷移 | `P(p; a:0{on=success->a}, b:0)`、`P(p; a:0, b:0{on=success->a})` | `validate` | 64。`transition target must be a later job or @end` |
| AC-027 | `@end` への遷移は正しい | `P(p; a:0{on=success->@end}, b:0)` | `validate` | 0 |
| AC-028 | 同じ結果の遷移の重複 | `on: success` の遷移を 2 つ | `validate` | 64。`duplicate transition for the same result` |
| AC-029 | 0 を警告にできない | `P(p; a:0{w=[0]})` | `validate` | 64。`0 cannot be a warning exit code` |
| AC-030 | 成功と警告の重なり | `P(p; a:0{w=[3], s=[0,3]})` | `validate` | 64。`success and warning exit codes overlap` |
| AC-031 | 終了コードの型・範囲の誤り | `w=["x"]`、`w=[1.5]`、`w=[99999999999999999999]`、`w=[2147483648]`、`w=[-2147483649]`、`w=[9007199254740993]`(2^53 + 1。number に落とすと別の値になる) | `validate` | 64。`wrong value type`(要素ごとに 1 行)。範囲の検査は、字句から `bigint` で行い、number に落とさない(AC-500) |
| AC-032 | 負の終了コード・空の成功コード・範囲の端 | `w=[-1]`、`s=[]`、`w=[2147483647]`、`s=[-2147483648]` | `validate` | 0(受理する) |
| AC-033 | `command` の型・欠落・空 | `command` なし / `""` / `"   "` / `[]` / `[""]` / `["  ", "x"]` / `5` / `["a", 5]` | `validate` | 64。なし・`5`・`["a",5]` は `wrong value type`。ほかは `required value is empty` |
| AC-034 | 配列形式の `command` の受理 | `argv=["my prog", "a b", "$HOME", ""]`、`argv=["echo", ""]` | `validate` | 0 |
| AC-035 | `run_id_env` の名前の形 | `run_id_env` が `"1A"` / `"A-B"` / `""` / `"A B"` / `"A=B"` / `"A.B"` | `validate` | 64。`invalid environment variable name`。整数 `1` なら `wrong value type`。`_a1`・`GITHUB_RUN_ID`・`A` は 0 |
| AC-036 | `remove_on_complete` の型 | `"true"` / `1` / `[true]` / `yes` | `validate` | 64。`wrong value type`(`yes` は YAML の真偽値ではなく文字列)。`true`・`false`・省略は 0 |
| AC-037 | 誤りは位置の順に 1 行ずつ | `ME`(この節の末尾の定義) | `validate` | 64。ERR は 4 行。順に `…:2: unknown key`、`…:10: wrong value type`、`…:13: unknown key`、`…:16: transition target job is not defined` |
| AC-038 | 行番号は CRLF でも同じ | `ME` を CRLF 改行で | `validate` | 同上(行番号が同じ) |
| AC-039 | BOM つきの定義 | `B0` の先頭に UTF-8 の BOM | `validate` | 0 |
| AC-040 | 定義の大きさの上限(受理) | `B0` に、コメント行(`#` で始まる行)を足して、ちょうど 1,048,576 バイト | `validate` | 0 |
| AC-041 | 定義の大きさの上限(超過) | 1,048,577 バイト(UTF-8 でなくてもよい) | `validate` | 64。`definition file is too large` |
| AC-042 | UTF-8 でない定義 | 上限以内で、不正なバイト列を含む | `validate` | 64。`cannot read definition file`(復号は `TextDecoder('utf-8', { fatal: true })` 相当で、不正なバイト列を置換せず拒否する) |
| AC-043 | 入れ子の深さ(受理) | `a: ` に `[` を 32 個、`]` を 32 個続けただけのファイル(`format_version` なし。根のマップは数えず、32 段まで受理。[03](03-definition-format.md)) | `validate` | 64。`format_version is missing`(深さの検査は通った) |
| AC-044 | 入れ子の深さ(超過) | `a: ` に `[` を 33 個・1,000 個(閉じも書く) | `validate` | 64。`syntax error`(理由 `入れ子が深すぎます(上限は 32 段)`。行は 33 段目の `[` の行) |
| AC-045 | パイプライン名の照合 | `P(p; a:0)` | `validate p` / `validate wrong` / `validate` | 0 / 64(`pipeline name does not match the definition`)/ 0(照合しない) |
| AC-046 | 名前の不一致の誤りに値を含めない | `name: SECRET-NAME` | `validate other-name` | 64。ERR に `SECRET-NAME` も `other-name` も現れない |
| AC-047 | 誤りの文に入力の値を含めない | `format_version: 1`、`pipeline:` の `name: p`、`job:` の `id: a`、`command: 5`、`warning_exit_codes: ["SENTINEL-VALUE"]` | `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |
| AC-048 | `command` の引用符が閉じていない | `P(p; a:0{cmd="TJ exit 0 'x"})` | `validate` | 64。ERR = `conductor: error: conductor.yaml: job 'a': command has an unclosed quote` |
| AC-049 | `command` が空になる | `cmd="''"` | `validate` | 64。`job 'a': command is empty` |
| AC-050 | `command` の誤りは全ジョブ報告 | 2 つのジョブがそれぞれ誤り | `validate` | 64。ERR は 2 行(ジョブごと) |
| AC-051 | `command` の誤り(日本語) | 同上(1 つ目の形)、`LANG=ja_JP.UTF-8` | `validate` | ERR = `conductor: error: conductor.yaml: job 'a': command の引用符が閉じていません` |
| AC-052 | `run` でも `command` の誤りは 64 でジョブは起動しない | 1 つ目が正しく、2 つ目が誤り | `run a` | 64。`ran=` の記録なし。ステートなし |
| AC-053 | 起動できないプログラムでも validate は 0 | `cmd="/no/such/program"` | `validate` | 0 |
| AC-054 | 壊れたステートでも validate は 0 | `B0`、`T/conductor.yaml.state` に `{garbage` | `validate p` | 0。ステートは変わらない |
| AC-055 | validate の引数の誤り | `B0` | `validate a b` / `validate --run-id x` / `validate --to yaml` | 68 |
| AC-056 | `run_id_env` が不正な定義は run でも 64 | `run_id_env: "1A"` | `run a` | 64 |
| AC-057 | validate は副作用を持たない | `B0` の `command` が「`TJ run a 0`」(`O` に記録) | `validate` | 0。`O` が作られない。`T` のファイル一覧が実行の前後で同じ |

#### `retry` の検査

`R0` は、`P(p; a:75{retry=(exit_codes=[75], max=3, interval="1s")})`。各行は、`R0` の `retry` を変えて `validate` する。誤りは終了 64、種類の文は ERR の最初の行。

| ID | 機能 | 変更後の `retry` | 期待 |
|----|------|------------------|------|
| AC-058 | 受理(固定の間隔) | `R0` のまま | 0 |
| AC-059 | 受理(倍率と上限) | `multiplier: 2`、`max_interval: 1m` を足す | 0 |
| AC-060 | 受理(境界) | `exit_codes: [1,255,127]`、`max: 1` と `max: 100`、`interval` が `1s`・`24h`・`86400s`・`1440m`、`multiplier` が 2 と 10、`max_interval` が `interval` と同じ | いずれも 0 |
| AC-061 | 受理(流れ形式の写像) | `retry: { exit_codes: [75], max: 3, interval: 1s }` | 0 |
| AC-062 | `retry` が写像でない | `retry: 5`、または `retry: [{exit_codes: [75], max: 3, interval: 1s}]` | 64。`retry must be a map` |
| AC-063 | 必須のキーがない | `retry: {}` | 64。`a required key of retry is missing` が 3 行(`exit_codes`、`max`、`interval` の順) |
| AC-064 | 未知のキー | `retries: 2` を足す | 64。`unknown key` |
| AC-065 | `exit_codes` が空 | `exit_codes: []` | 64。`retry exit_codes is empty` |
| AC-066 | `exit_codes` の要素の範囲 | `exit_codes: [0, 75, 256]` | 64。`retry exit_codes has an invalid exit code` が 2 行(0 と 256。要素ごと)。`-1`、`1.5`、`"75"`、`true`、`9007199254740993` も同じ |
| AC-067 | 成功・警告の終了コードと重なる | `{s=[0,5]}` で `exit_codes: [5,75]`、`{w=[4]}` で `exit_codes: [4]` | 64。`retry exit_codes overlap the success or warning exit codes` |
| AC-068 | 最大回数の範囲 | `max: 0`、`101`、`-1`、`1.5`、`"3"`、`4294967296` | 64。`retry max is invalid` |
| AC-069 | 間隔の書式 | `interval` が `10`(整数)/ `1.5s` / `0s` / `25h` / `1S` / `86401s` / `1ms` / `" 1s"` / `"1 s"` | 64。`retry interval is invalid`(`max_interval` も同じ規則) |
| AC-070 | 倍率の範囲 | `multiplier` が `1`、`11`、`0`、`2.0`、`"2"`(`max_interval` を添える) | 64。`retry multiplier is invalid` |
| AC-071 | 倍率と上限は片方だけ不可 | `multiplier: 2` だけ、`max_interval: 1m` だけ | 64。`retry multiplier and max_interval must be given together or not at all` |
| AC-072 | 上限が間隔より小さい | `interval: 1m`、`multiplier: 2`、`max_interval: 30s` | 64。`retry max_interval is smaller than interval` |
| AC-073 | 誤りは位置の順、ほかの誤りと一緒に | `retry` の誤り 2 つと、別の定義の誤りを混ぜる | 64。ERR は位置の順に 1 行ずつ |
| AC-074 | `run` と同じ誤りの文 | 上のいずれかの誤りの定義で `run a` を実行 | `validate` と同じ文で 64 |
| AC-075 | 誤りの文に入力の値を含めない | `interval: SENTINEL-VALUE` で `validate` | 64。ERR に `SENTINEL-VALUE` が現れない |

`ME`(誤りが複数ある定義。行番号は、この並びのとおり):

```yaml
format_version: 1
unknown_top: true

pipeline:
  name: p

job:
  - id: a
    command: x
    success_exit_codes: ["zero"]
  - id: b
    command: y
    colour: red
    transitions:
      - on: success
        next: nowhere
```

JSON で同じ構造を書いた `ME`(`conductor.json`)では、誤りの行は、順に 3、11、16、20 になる(AC-475)。

### 4.2 `command` の分割と起動

「分割」の行は、`P(p; a:0{cmd="TJ --out O echo <X>"})` を作り、`run a` を実行する(`TJ`、`O` は 3.5 節)。`<X>` は、各行の「`command` の引数部」。
期待は、終了 0、OUT・ERR 空、`O` の記録(`arg[0]=…` 以降)。記録は `q(…)` の形(3.4 節)。ここでは、引数の値だけを書く(`arg[i]` の添字は先頭から 0)。YAML に書くときは、`JSON.stringify` で出した二重引用符の文字列(3.4 節)で書き、文字列として同じ値になるようにする。

| ID | 機能 | `command` の引数部(`TJ --out O echo` の後ろ) | 期待する引数の並び |
|----|------|------------------------------------------------|---------------------|
| AC-076 | 分割: 基本 | `a b` | `a`、`b` |
| AC-077 | 分割: 空白・タブ・改行の混在 | `␣␣a ⇥ b ⏎ c␣␣` | `a`、`b`、`c` |
| AC-078 | 分割: 一重引用符 | `'a b' 'c\d' '"x"'` | `a b`、`c\d`、`"x"` |
| AC-079 | 分割: 二重引用符と `\"` | `"a \" b"` | `a " b` |
| AC-080 | 分割: 二重引用符の中の `\n` `\\` | `"a\nb" "c\\d"` | `a\nb`、`c\\d`(バックスラッシュは、そのまま。`\n` は 2 文字) |
| AC-081 | 分割: `\\"` は引用符を閉じない | `"\\"x"` | `\"x` |
| AC-082 | 分割: 引用符の外のバックスラッシュ | `a\ b` | `a\`、`b` |
| AC-083 | 分割: 末尾が `\` のパスは一重引用符 | `'C:\dir\'` | `C:\dir\` |
| AC-084 | 分割: `--msg='a b'` | `--msg='a b'` | `--msg=a b` |
| AC-085 | 分割: 引用符のつなぎ | `a"b c"'d e'f` | `ab cd ef`(1 つ) |
| AC-086 | 分割: 空の引数 | `'' "" x` | (空)、(空)、`x` |
| AC-087 | 分割: 非 ASCII | `'あ い' う` | `あ い`、`う` |
| AC-088 | 分割: メタ文字は解釈しない | `a;b && c \| d > e $HOME * `` `x` `` (y)` | `a;b`、`&&`、`c`、`\|`、`d`、`>`、`e`、`$HOME`、`*`、`` `x` ``、`(y)`。`T` に `e` というファイルができない |
| AC-089 | 分割: 区切りは 4 文字だけ | `a<U+00A0>b`(U+00A0 は空白に見えるが区切りではない) | `a<U+00A0>b`(1 つ) |
| AC-090 | 配列形式: 要素はそのまま | `argv=[<TJ の 2 要素>, "--out", "O", "echo", "a b", "  lead and trail  ", "$HOME", "a;b", "*", ""]` | `a b`、`  lead and trail  `、`$HOME`、`a;b`、`*`、(空) |
| AC-091 | 文字列と配列は同じ結果 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 7"})` と、同じ内容の `argv` 形式 | それぞれ `run a` → 0、`pipeline p` → 7、OUT・ERR 空。`O` = `ran=a`⏎`ran=b` が同じ |

分割の誤り(拒否)。実行は `validate`。終了 64。ERR の最初の行は `conductor: error: conductor.yaml: job 'a': <文>`。

| ID | 機能 | `command`(文字列) | 期待する文 |
|----|------|---------------------|------------|
| AC-092 | 拒否: 一重引用符が閉じていない | `echo 'a` | `command has an unclosed quote` |
| AC-093 | 拒否: 二重引用符が閉じていない | `echo "a` | 同上 |
| AC-094 | 拒否: 末尾が二重引用符の中の `\` | `echo "a\` | 同上 |
| AC-095 | 拒否: `\\"` で閉じない | `echo "a\\" b` | 同上 |
| AC-096 | 拒否: パスの末尾の `\"` | `prog "C:\dir\"` | 同上 |
| AC-097 | 拒否: 空の語だけ | `''`、`'' x` | `command is empty` |
| AC-098 | 拒否: 区切りだけ | `"  \t\n"`(YAML の二重引用符の文字列)。定義の検査が、空白だけの `command` を先に拒否する | `required value is empty`(定義の検査。64) |

起動と実行環境。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-099 | 環境変数の引き継ぎ | `P(p; a:0{cmd="TJ --out O env CT_VAR CT_NO_SUCH"})`。環境に `CT_VAR=hello`、`CT_NO_SUCH` は未設定 | `run a` | 終了 0。`O` = `env[CT_VAR]=set:"hello"`⏎`env[CT_NO_SUCH]=unset` |
| AC-100 | 作業ディレクトリは引き継ぐ | `cmd="TJ --out O cwd"`。定義は `T/sub/x.yaml` | `T` で `run a --config sub/x.yaml` | `O` = `cwd=q(T のパス)`(`T/sub` ではない) |
| AC-101 | run ID を環境変数として渡さない | `cmd="TJ --out O env CONDUCTOR_RUN_ID"` | `run a --run-id r1` | `O` = `env[CONDUCTOR_RUN_ID]=unset` |
| AC-102 | 標準出力・標準エラーはそのまま | `cmd="TJ say 'OUT to stdout' 'ERR to stderr' 0"` | `run a` | 終了 0。OUT = `OUT to stdout`(改行なし、完全一致)。ERR = `ERR to stderr`(完全一致) |
| AC-103 | 存在しないプログラム | `cmd="/no/such/program-for-conductor-test"` | `run a` | 終了 127。ERR に `conductor: error: job 'a': cannot start command '/no/such/program-for-conductor-test'`。OUT 空。ステート `S[a=error ; resume=a]`(`spawn` の `'error'` イベント(`ENOENT`)を、起動失敗として扱う) |
| AC-104 | 配列形式の起動失敗は先頭の要素の名前 | `argv=["no-such-prog-zzz", "--flag"]` | `run a` | 127。ERR に `cannot start command 'no-such-prog-zzz'` |
| AC-105 | 空白を含む先頭の要素 | `argv=["my prog", "x"]`(`my prog` というファイルはない) | `run a` | 127。`cannot start command 'my prog'` |
| AC-106 | 実行できないファイル(UNIX 系) | `cmd` が実行権限のない通常のファイル(`T/data.txt`)のパス | `run a` | 127(`EACCES`) |
| AC-107 | ディレクトリ・`/` を指定(UNIX 系) | `cmd="/"`、`cmd` がディレクトリのパス | `run a` | 127(`EACCES` など) |
| AC-108 | 起動失敗は表示の言語に従う | `LANG=ja_JP.UTF-8`、存在しないプログラム | `run a` | 127。ERR に `コマンドを起動できません` |
| AC-109 | 終了コードの返り方 | `exit 0`、`exit 3`、`exit 255` のジョブ(警告の指定なし) | `run a` | 0 / 3 / 255。OUT・ERR 空 |
| AC-110 | 大きな終了コード | `TJ exit 300` | `run a` | 44(UNIX 系も Windows も `300 mod 256`) |
| AC-111 | 256 の終了コード(Windows) | `TJ exit 256`、`-v` | `run -v a` | 1。ERR に `exit code 256, conductor exits with 1` |
| AC-112 | 負の終了コード(Windows) | `TJ exit -1`、`-v` | `run -v a` | 255。ERR に `conductor exits with 255`(Windows の `exit` イベントが、終了コード `-1` を `-1` で返すか `4294967295` で返すかは未確認。どちらでも、32 ビット符号つきに直してから範囲合わせをして、255 になる。AC-392) |
| AC-113 | `.cmd` への危険な引数(文字列形式。Windows) | `T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + ` 'x" & echo injected > T\injected.txt'` | `run a` | 127(禁止文字を含む引数なので、起動を拒否する。[07](07-process-and-signals.md))。`T/injected.txt` ができない |
| AC-114 | `.cmd` への危険な引数(配列形式。Windows) | 同じスクリプト。`argv=[パス, "x\" & echo injected > T\injected.txt"]` | `run a` | 同上 |
| AC-115 | `.cmd` への安全な引数(Windows) | `T/ok.cmd`(`exit /b 7`)。`cmd` = そのパス + ` plain` | `run a` | 7 |
| AC-116 | シェルを使わないことの点検 | ソース | (構文木の走査) | `node:child_process` から import される名前が `spawn` だけ(`exec`・`execSync`・`execFile`・`execFileSync`・`spawnSync`・`fork` を import しない)。`spawn` の呼び出しが `src/runner/` に 1 か所だけで、オプションの `shell` が `false`(リテラル)。`shell: true` がどこにもない。シェルの名前(`sh`・`bash`・`cmd`・`powershell`・`-c`・`/C`)を `spawn` の第 1・第 2 引数に渡す経路がない。**Windows の `.cmd` / `.bat` を起動する経路だけは、07 章が定める安全策(禁止文字の拒否ほか。Node で `.cmd` を `shell: false` で起動できない場合の方法は 07 章で決める。AC-508)を通り、その経路は `platform.ts` の 1 か所に集める**。区分: 点検 |

### 4.3 `run`(個別実行)

特に記さない限り、`P(…)` の `<id>:<n>` は、3.5 節のとおり `TJ exit <n>`。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-117 | 成功 | `P(p; a:0)` | `run a` | 終了 0。OUT・ERR 空。ステート: `S[a=success(0) ; completed=true]` が残る(最後のジョブで完了。完了のステートは残る) |
| AC-118 | 警告 | `P(p; warn:3{w=[3]}, fail:7, last:0)` | `run warn` | 0。OUT・ERR 空。ステート `warn=warning(3)` |
| AC-119 | 実行済みのスキップ | 上のあと | `run warn` | 0。OUT・ERR 空。ステートは変わらない |
| AC-120 | エラーはジョブの終了コード | 上のあと | `run fail` | 7。OUT・ERR 空。ステート `S[warn=warning(3), fail=error(7), last=pending ; resume=fail]` |
| AC-121 | エラーのあとの再実行 | 上のあと、定義の `fail` を `fail:0` に直す | `run fail` | 0。`fail=success(0)`。続けて `run last` → 0、`completed=true` |
| AC-122 | 自動の再実行はない | `P(p; a:7)` | `run a` | 7。ジョブは 1 回だけ実行(`TJ run` で確認) |
| AC-123 | 順序外は拒否(66) | `P(p; a:0, b:0, c:0)`、ステートなし | `run b` | 66。ERR = `conductor: error: job 'b': cannot run this job now: earlier jobs have not finished (out of order)`。OUT 空。ステートを作らない |
| AC-124 | 順序外でステートは変わらない | `S[a=success(0), b=error(1), c=pending ; resume=b]` | `run c` | 66。ステートは変わらない |
| AC-125 | 知らないジョブID | `B0` | `run nojob` | 68。ERR の最初の行 = `conductor: error: job is not defined in the definition file: 'nojob'`。続けて空行と使い方。OUT 空。ステートなし |
| AC-126 | 定義の誤りは知らないジョブIDより優先 | 定義に誤りあり | `run nojob` | 64 |
| AC-127 | ジョブIDなし | `B0` | `run` | 68。`missing argument` |
| AC-128 | 既定の定義ファイル | `T/conductor.yaml` | `run a`(`--config` なし) | 定義を読む。ステートは `T/conductor.yaml.state` |
| AC-129 | `--config` の場所 | `T/sub/x.yaml` | `run a --config sub/x.yaml` | ステートは `T/sub/x.yaml.state` |
| AC-130 | -v: 成功 | `cmd="TJ say hello ''"` | `run -v a` | 0。OUT = `hello`。ERR = `[conductor] job=a result=success (exit code 0)` |
| AC-131 | -v: 警告の遷移 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run -v a` | 0。OUT 空。ERR = `[conductor] job=a result=warning (exit code 1)`⏎`[conductor] transition: warning -> c (skipped: b)` |
| AC-132 | -v: `@end` への遷移 | `P(p; a:0{on=success->@end}, b:0)` | `run -v a` | 0。ERR = `[conductor] job=a result=success (exit code 0)`⏎`[conductor] transition: success -> @end`。ステート `end=true`、`completed=false` |
| AC-133 | -v: 日本語 | `LANG=ja_JP.UTF-8`、`P(p; a:0, b:0)` | `run -v a`、続けて `run -v a` | ERR = `[conductor] job=a result=success (終了コード 0)`。2 回目に `[conductor] job=a スキップ (実行済み)` |
| AC-134 | -v: スキップの行 | `P(p; a:0, b:0)`、`a` を実行済み | `run -v a` | 0。ERR = `[conductor] job=a skipped (already executed)` |
| AC-135 | `@end` のあとの個別実行 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `run a`、`run -v b`、`run -v c` | すべて 0。`b`・`c` は実行されない。`run b` の ERR = `[conductor] job=b skipped (skipped after @end)`。ステート: `b=skipped/byEnd`、`c=skipped/byEnd`、`end=true`。最後に `completed=true` |
| AC-136 | 遷移でスキップされたジョブの個別実行 | `P(p; a:1{w=[1], on=warning->c}, b:0, c:0)` | `run a`、`run -v b` | `run a` のあと `b=skipped/byTransition`。`run b` は 0、`b` は実行されず、ERR = `[conductor] job=b skipped (skipped by a transition)` |
| AC-137 | 成功コードに含まれない 0 | `P(p; a:0{s=[5]})` | `run a` | 1。ERR = `conductor: error: job 'a': exit code 0 is not listed in success_exit_codes`(-v なしでも出る)。ステート `a=error(0)`、`resume=a` |
| AC-138 | 成功コードに含まれない 0(-v) | 同上 | `run -v a` | 1。ERR = `[conductor] job=a result=error (exit code 0, conductor exits with 1)`⏎`conductor: error: job 'a': exit code 0 is not listed in success_exit_codes` |
| AC-139 | 成功コードの指定 | `P(p; a:5{s=[5]})`、`P(p; a:0{s=[0,2]})`、`P(p; a:2{s=[2]})` | `run a` | いずれも 0(成功) |
| AC-140 | シグナルで終了したジョブ(UNIX 系) | `argv=["sh", "-c", "kill -9 $$"]` | `run -v a` | 137。ERR = `[conductor] job=a result=error (terminated by signal 9)`。ステート `a=error`(終了コードなし)、`resume=a`(`exit` イベントの `code` が `null`、`signal` が `'SIGKILL'` で、番号 9 に直す。AC-502) |
| AC-141 | 完了のステートが残っていても先頭から | `P(p; a:0, b:0)`。`run a`、`run b`(完了)のあと | `run a`(`TJ run` で記録) | 0。`a` が再び実行される。`completed` のステートは上書きされる |
| AC-142 | 完了のあと先頭以外は拒否 | 上の完了のステート | `run b` | 66 |
| AC-143 | `--remove-on-complete`(run) | `P(p; a:0)` | `run a --remove-on-complete` | 0。OUT・ERR 空。`T/conductor.yaml.state` がない |
| AC-144 | 定義の `remove_on_complete` | `P(p remove_on_complete=true; a:0)` | `run a` | 0。ステートがない |
| AC-145 | 完了でなければ消さない | `P(p; a:0, b:0)` | `run a --remove-on-complete` | 0。ステートが残る(`a=success`、`completed=false`) |
| AC-146 | ステートの形(エラーで停止) | 定義 `c2.yaml`(`P(n2; a:0, b:3, c:0)`) | `pipeline n2 --config c2.yaml` | 3。`T/c2.yaml.state` の内容は JSON のオブジェクトで、`completed=false`、`definition_key="c2.yaml"`、`definition_path="c2.yaml"`、`end_reached=false`、`format_version=1`、`job_order=["a","b","c"]`、`pipeline_name="n2"`、`resume_job_id="b"`、`job` は `a=success exit_code=0`、`b=error exit_code=3`、`c=pending`(`run_id` キーなし) |
| AC-147 | ステートの書式 | 上のステート | (ファイルを読む) | 1 つの JSON のオブジェクト(テストが `JSON.parse` で読める)。末尾は改行。インデントと、キーの並び(固定の順。実装が、キーの順にオブジェクトを組み立てて `JSON.stringify` で出す)は [06](06-state-file.md) のとおり。同じ内容のステートを書くと、毎回同じバイト列になる |
| AC-148 | ステートに機微な値を書かない | `cmd="TJ say SECRET-OUT '' 3"`(終了コード 3 のエラーになる)。環境変数 `SECRET_ENV=SECRET-ENVVAL` を設定 | `run a` | ステートファイル(終了コード 3、`resume=a`)に、`command` の文字列、`SECRET-OUT`、`SECRET-ENVVAL` が現れない |
| AC-149 | 初期ステートの書き込みに失敗(UNIX 系、非 root) | `T` を書き込み不可にする(`fs.chmodSync(T, 0o555)`。定義は読める) | `run a` | 65。ERR に `cannot write state file` と `(the job was not started)`。ジョブは実行されない |
| AC-150 | ジョブのあとの書き込みの失敗(UNIX 系、非 root) | 定義 `argv=["sh", "-c", "chmod a-w . ; exit 7"]`(ジョブが `T` を書き込み不可にする) | `run a` | 65(ジョブの 7 ではない)。ERR に `cannot write state file` と `(the job has already been executed)` |
| AC-151 | 定義の誤り(ファイルなし・誤り) | 定義なし / `format_version: 1` だけ | `run a` | 64。ERR は `conductor: error: ` で始まる。OUT 空 |

### 4.4 `pipeline`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-152 | 全部成功: 記述順に実行 | `P(p; a:0{cmd="TJ --out O run a 0"}, b:0{cmd="TJ --out O run b 0"})`(2 ジョブ) | `pipeline p` | 0。OUT・ERR 空。`O` = `ran=a`⏎`ran=b`。ステート `S[a=success(0), b=success(0) ; completed=true]` が残る |
| AC-153 | 完了のステートの形 | 上のあと | (ステートを読む) | `resume_job_id` なし、`completed=true`、`end_reached=false`。形式は変わらない |
| AC-154 | 再実行は先頭から | 上のあと | `pipeline p` | 0。`a`、`b` が再び実行される(`O` に 2 組) |
| AC-155 | 警告の遷移で間をスキップ | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)` | `pipeline -v nightly --run-id r1` | 0。ERR = `[conductor] job=extract result=warning (exit code 1)`⏎`[conductor] transition: warning -> load (skipped: transform)`⏎`[conductor] job=transform skipped (skipped by a transition)`⏎`[conductor] job=load result=success (exit code 0)`。`transform` は実行されない |
| AC-156 | `@end` で完了 | `P(p; a:0{on=success->@end}, b:0, c:0)` | `pipeline p` | 0。`b`・`c` は実行されない。ステート `S[a=success(0), b=skipped/byEnd, c=skipped/byEnd ; end=true, completed=true]` |
| AC-157 | エラーで停止 | `P(p; a:0, b:7, c:0)`(各ジョブは `TJ run` 形式で記録) | `pipeline p` | 7。`c` は実行されない。ステート `S[a=success(0), b=error(7), c=pending ; resume=b]` |
| AC-158 | エラーからの再開 | 上のあと、定義の `b` を `b:0` に直す | `pipeline -v p` | 0。`a` は再実行されない(ERR に `[conductor] job=a skipped (already executed)`)。`b`・`c` が実行される。完了のステート |
| AC-159 | 個別実行の続きから一括実行 | `P(p; a:0, b:0, c:0)`。`run a` のあと | `pipeline p` | 0。`a` は再実行されない。`b`・`c` が実行される |
| AC-160 | 起動できないジョブで停止 | `P(p; a:0, b:0, c:0)` の `b` を存在しないプログラムに | `pipeline p` | 127。`c` は実行されない。ERR に `cannot start command` |
| AC-161 | パイプライン名の不一致 | `B0` | `pipeline wrongname` | 64。`pipeline name does not match the definition`。ジョブは実行されない |
| AC-162 | パイプライン名なし | `B0` | `pipeline` | 68。`missing argument` |
| AC-163 | `--remove-on-complete` | `P(p; a:0, b:0)` | `pipeline p --remove-on-complete` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state (nothing has run yet)` |
| AC-164 | 定義の `remove_on_complete: true` | `P(p remove_on_complete=true; a:0, b:0)` | `pipeline p` | 0。ステートがない |
| AC-165 | オプションと定義の組み合わせ | `remove_on_complete=false` の定義 | `pipeline p --remove-on-complete` | ステートがない(オプションが優先して消す) |
| AC-166 | 完了でないとき消さない | `P(p remove_on_complete=true; a:0, b:7)` | `pipeline p` | 7。ステートが残る(エラーで停止) |
| AC-167 | 完了のあと先頭以外の run は 66 | 完了のステート | `run b` | 66 |
| AC-168 | 初期ステートが書けない(UNIX 系、非 root) | `T` を書き込み不可 | `pipeline p` | 65。ジョブは起動されない |
| AC-169 | 完了のステートが残るとき定義の変更は照合されない | `pipeline p` で完了 → 定義のジョブIDを変える | `pipeline p` | 0(完了のステートは「空」として扱う。65 にならない) |

### 4.5 `watch`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-170 | 表の形(英語) | `P(p; a:0, b:7, c:0)` で `pipeline p`(7 で終わる)のあと | `watch` | 終了 0。OUT = `Pipeline: p`⏎`JOB  STATUS   NOTE`⏎`a    success  exit code 0`⏎`b    error    exit code 7`⏎`c    pending`(行末の空白なし)。ERR 空 |
| AC-171 | ステートなし | ステートなし | `watch` | 0。OUT = `no state (nothing has run yet)`(1 行) |
| AC-172 | ステートなし(日本語) | `LANG=ja_JP.UTF-8` | `watch` | 0。OUT = `ステートはありません(まだ実行されていません)` |
| AC-173 | 長さ 0 のファイルも「なし」 | 長さ 0 のステートファイル | `watch` | 同上(`no state …`) |
| AC-174 | 完了の行 | `P(nightly; extract:1{w=[1], on=warning->load}, transform:0, load:0)`、`pipeline nightly --run-id r1` のあと | `watch --run-id r1` | OUT = `Pipeline: nightly`⏎`Run ID: r1`⏎`JOB        STATUS   NOTE`⏎`extract    warning  exit code 1`⏎`transform  skipped  skipped by a transition`⏎`load       success  exit code 0`⏎`completed` |
| AC-175 | 日本語の表 | 同上、`LANG=ja_JP.UTF-8` | `watch --run-id r1` | OUT = `パイプライン: nightly`⏎`Run ID: r1`⏎`ジョブ     状態     補足`⏎`extract    warning  終了コード 1`⏎`transform  skipped  遷移によりスキップ`⏎`load       success  終了コード 0`⏎`完了`(状態の語は英語のまま、`Run ID` は日本語でも `Run ID`) |
| AC-176 | `@end` のスキップの補足 | `@end` で完了したステート(4.4 節) | `watch` | `b`、`c` の行の NOTE = `skipped after @end` |
| AC-177 | 定義ファイルを読まない | 定義ファイルがない、または壊れている。ステートあり | `watch --config missing.yaml` | 0。ステートが表示される(定義がなくても動く) |
| AC-178 | ステートなし・定義なしでも 0 | 定義ファイルなし | `watch --config missing.yaml` | 0。OUT に `no state` |
| AC-179 | 壊れたステート | `{garbage` | `watch` | 65。ERR = ``conductor: error: conductor.yaml.state: state file is corrupt: JSON として読めません; run `conductor reset` to discard the state``(`` ` `` はバッククォート)。OUT 空 |
| AC-180 | 壊れた完了のステート | 完了のステートだが `run_id` がファイル名と食い違う | `watch` | 65 |
| AC-181 | ファイルへの出力に制御文字がない | `P(p; a:0, b:0)`、`run a` のあと | `watch`(標準出力をファイルに) | 0。ファイルに `\x1b` がなく、`a`、`success`、`pending` を含む |
| AC-182 | 端末では状態に色(UNIX 系。疑似端末) | 同上 | 標準出力を端末(`process.stdout.isTTY` が真)にして `watch` | `success` が `\x1b[32m` と `\x1b[0m` で囲まれる(`warning` 33、`error` 31、`skipped` 36、`pending` は色なし)。列の幅は色の制御文字を除いた文字で決まる。区分: UNIX 系(疑似端末)。疑似端末を作る方法は未確認(10 章)。代わりに、端末かどうかの判定を `Deps` の関数にして、偽の `Deps` で「端末」を返す単体の試験を併せて置く(区分: 注入) |
| AC-183 | `NO_COLOR` で色なし | 同上、`NO_COLOR=1` | 同上 | 制御文字なし。空の `NO_COLOR` では色あり。区分: UNIX 系(疑似端末か、注入) |
| AC-184 | Windows は色なし | Windows | `watch`(端末) | 制御文字なし。区分: Windows |
| AC-185 | ジョブIDの制御文字は表の中でもエスケープ | ジョブID に ESC を含む(YAML で `"a\u001bb"`)。実行済み | `watch` | 行に `a\u{1b}b`。生の ESC がない |
| AC-186 | 標準出力に書けない(注入) | ステートあり | `watch`(標準出力を書き込み失敗にする `FailingWriter`) | 65。ERR = `conductor: error: cannot write to standard output`。Linux では `watch > /dev/full` でも起こせる(AC-494)。パイプの読み手が閉じた場合(`EPIPE`)は AC-492 |
| AC-187 | 使えないオプション | なし | `watch -v` / `watch --remove-on-complete` / `watch --all` / `watch extra` | 68。ERR = `option not allowed for this command: '-v'`(それぞれ対象は `-v`、`--remove-on-complete`、`--all`)。`watch extra` は `unexpected argument: 'extra'` |

### 4.6 `reset`

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-188 | ステートを消す | `P(p; a:0, b:7)`、`pipeline p` のあと | `reset` | 0。OUT・ERR 空。ステートがない。続く `watch` → `no state …` |
| AC-189 | 先頭から実行できる | 上のあと、`P(p; a:0, b:0)` に直す | `run a`(`TJ run` で記録) | 0。`a` が実行される |
| AC-190 | ステートがなくても成功 | ステートなし | `reset` | 0。OUT・ERR 空 |
| AC-191 | 定義がなくても動く | 定義なし | `reset --config missing.yaml` | 0 |
| AC-192 | 壊れたステート・壊れた定義を迂回 | 壊れたステートと壊れた定義 | `reset` | 0。ステートが消える |
| AC-193 | 完了のステートも消す | 完了のステート(run ID あり・なし) | `reset`、`reset --run-id r1` | 0。消える |
| AC-194 | `--run-id` は 1 つだけ消す | `conductor.yaml.state`、`.r1.state`、`.r2.state` がある | `reset --run-id r1` | `.r1.state` だけ消える。ほかは変わらない |
| AC-195 | `--all` で全部消す | `P(p; a:0, b:7)`。`pipeline p`(run ID なし)、`--run-id a`、`--run-id b` で実行。`T` に、別の定義のステート(`conductor.yaml.a.b.state`。中身はパイプライン名 `p`・ジョブ `a` の正常なステートで、`run_id` を持たない)、壊れたもの(`conductor.yaml.c.state` = `{garbage`)、一時ファイル(`.conductor.yaml.state.tmp.1.2.3`)、無関係なファイル(`conductor.yaml.bak`、`other.yaml.state`)を置く | `reset --all` | 終了 0。OUT 空。ERR は `conductor: warning: ` で始まり、`conductor.yaml.c.state` を含み、`conductor.yaml.a.b.state` を含まない。残るファイル: `.conductor.yaml.state.tmp.1.2.3`、`conductor.yaml`、`conductor.yaml.a.b.state`、`conductor.yaml.bak`、`conductor.yaml.c.state`、`other.yaml.state`。run ID なしと `.a.state`、`.b.state` は消える |
| AC-196 | 警告の文言 | 上と同じ | `reset --all` | ERR = ``conductor: warning: kept state files that could not be read (remove one with `conductor reset --run-id <ID>`):``⏎`  conductor.yaml.c.state`(ファイル名の行は先頭に空白 2 文字) |
| AC-197 | 壊れたものを個別に消す | 上のあと | `reset --run-id c` | 0。`conductor.yaml.c.state` が消える |
| AC-198 | 何もなくても 0 で出力なし | `T` に定義だけ | `reset --all` | 0。OUT・ERR 空 |
| AC-199 | `--all` と `--run-id` の併用 | なし | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-200 | `--all` は環境変数を見ない | `CONDUCTOR_RUN_ID='bad id'` | `reset --all` | 0 |
| AC-201 | シンボリックリンクと別の定義は消さない(UNIX 系) | `conductor.yaml.s.state` が通常のファイルへのシンボリックリンク。ディレクトリ `conductor.yaml.d.state`。`conductor.yaml.a.state` の中身の `run_id` がファイル名と食い違う | `reset --all` | リンク先のファイルは残る。リンクとディレクトリと、中身の `run_id` が食い違うファイルは消されない(`fs.lstat` で種類を調べ、リンクをたどらない) |
| AC-202 | run ID なしのステートは中身を読まず消す | `conductor.yaml.state` が壊れている(通常のファイル) | `reset --all` | 消える。警告なし |
| AC-203 | 環境変数の値の違反 | `CONDUCTOR_RUN_ID='bad id'` | `reset` | 68。`invalid value: 'run ID (environment variable)'` |
| AC-204 | 引数の誤り | なし | `reset extra`、`reset -v`、`reset --remove-on-complete` | 68。`unexpected argument: 'extra'`、`option not allowed for this command: '-v'`、同 `'--remove-on-complete'` |
| AC-205 | 削除の失敗(UNIX 系、非 root) | `T/conductor.yaml.state` が空でないディレクトリ | `reset` | 65。ERR に `cannot remove state file`。OUT 空 |

### 4.7 `convert`(YAML と JSON の相互変換)

`convert <入力ファイル> --to yaml|json`。入力の形式は `--to` の反対(`--to json` なら YAML、`--to yaml` なら JSON)で、入力ファイルの拡張子は見ない([09](09-converter.md))。

変換の入力 `CV1`(YAML)、`CV1J`(`CV1` を JSON にしたもの)、`CV2`(YAML。アンカー、エイリアス、マージキーを含む)は、次のとおり。

```yaml
# CV1
format_version: 1
pipeline:
  name: nightly
  description: 夜間バッチ(抽出、変換、読み込み、通知)
job:
  - id: extract
    command: extract --all --since yesterday
  - id: transform
    command: 'transform "in dir" $HOME'
    warning_exit_codes: [3, 4]
    success_exit_codes: [0, 2]
    transitions:
      - on: warning
        next: notify
  - id: load
    command: load
    transitions:
      - on: success
        next: "@end"
  - id: notify
    command: notify --channel ops
```

```json
{"format_version": 1,
 "pipeline": {"name": "nightly", "description": "夜間バッチ(抽出、変換、読み込み、通知)"},
 "job": [
  {"id": "extract", "command": "extract --all --since yesterday"},
  {"id": "transform", "command": "transform \"in dir\" $HOME",
   "warning_exit_codes": [3, 4], "success_exit_codes": [0, 2],
   "transitions": [{"on": "warning", "next": "notify"}]},
  {"id": "load", "command": "load", "transitions": [{"on": "success", "next": "@end"}]},
  {"id": "notify", "command": "notify --channel ops"}]}
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

特に記さない限り、失敗の期待は、終了 67、OUT 空、ERR の最初の行が `conductor: error: ` で始まる。「木」は、テストが `JSON.parse` と、YAML の読み取り(製品の読み取りの部品、またはテスト専用に用意した小さな読み取り)で得る、写像・列・文字列・整数・浮動小数点数・真偽値の構造。整数と浮動小数点数の区別(`1` と `1.0`)は、`JSON.parse` の結果(どちらも number)では付かないので、**木の比較は、テスト用に、数値の字句(出力の文字列に小数点・指数があるか)を見て区別する補助(`test/support/tree.ts`)を使う**。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-206 | YAML から JSON | `T/in.yaml` = `CV1` | `convert in.yaml --to json` | 終了 0。ERR 空。OUT は JSON で、`"format_version": 1` と `"id": "extract"` を含む。OUT を JSON として読んだ木が `CV1` の木と等しい。キーの順は入力の記述順、インデントは 2 スペース、末尾は改行(細部は [09](09-converter.md) のとおり) |
| AC-207 | YAML から JSON(アンカー展開) | `T/in.yaml` = `CV2` | `convert in.yaml --to json` | 0。OUT に `<<` も `&defaults` もない。OUT を JSON として読んだ木が、`CV2` を展開した木と等しい(`transform` の `success_exit_codes` は `[0]`、`extract` は `[0, 2]`) |
| AC-208 | 変換の出力は標準出力だけ | `T/in.yaml` = `format_version: 1⏎pipeline:⏎  name: p⏎job:⏎  - id: a⏎    command: echo a` | `convert in.yaml --to json` | 0。ERR 空。OUT に `"job": [` と `"format_version": 1` |
| AC-209 | 入力形式は `--to` の反対で、拡張子は見ない | `T/looks-like.json` = `a: 1⏎b: [x, y]`(YAML の内容) | `convert looks-like.json --to json` / `convert looks-like.json --to yaml` | 前者: 0、OUT を JSON として読むと `{"a": 1, "b": ["x","y"]}`。後者: 67(JSON として読んで `syntax error`) |
| AC-210 | 文法の誤りは種類と行つき | `T/broken.yaml` = `a: [unclosed` | `convert broken.yaml --to json` | 67。OUT 空。ERR の最初の行が `conductor: error: broken.yaml:` で始まり、種類の文が `syntax error`、理由が `YAML の文法に誤りがあります`(行は採用ライブラリの検出位置。未確認) |
| AC-211 | 入力がない | なし | `convert nofile --to json` | 67。ERR = `conductor: error: nofile: input file does not exist: 指定の場所にファイルがありません` |
| AC-212 | ディレクトリを入力に | `T/d` がディレクトリ | `convert d --to json` | 67。`cannot read input file` |
| AC-213 | 入力が大きすぎる | 1,048,577 バイト(中身が壊れていてもよい) | `convert big.yaml --to json` | 67。`input is too large` |
| AC-214 | ちょうど 1 MiB は受理 | `a: "xxx…"` で、ちょうど 1,048,576 バイト | `convert max.yaml --to json` | 0。OUT が `{⏎  "a": "xxx` で始まる |
| AC-215 | UTF-8 でない入力 | `a: ` のあとに不正なバイト列 | `convert bad.yaml --to json` | 67。`cannot read input file`(文法を見る前) |
| AC-216 | 表せない値は場所つき | `T/in.yaml` = `job:⏎  - id: x⏎  - id: y⏎    note: .nan` | `convert in.yaml --to json` | 67。ERR に `value cannot be represented in the target format: NaN と無限大は JSON で表せません` と `(at: job[1].note)` |
| AC-217 | 表せない値と、表せる値の境界 | YAML(`--to json`): `a: .inf` / `a: .nan` / `a: -.inf` / `a: [1, .inf]` / `a: 9223372036854775808` / `a: 0xFFFFFFFFFFFFFFFFFF` / `a: 1e999`。JSON(`--to yaml`): `{"a": 9223372036854775808}` / `{"a": 1e400}` / キーが 1,025 文字のオブジェクト。受理されるもの: YAML の `a: ~` / `a: null` / `a:`(値は `null`)、JSON の `{"a": null}`、`{"a": 9223372036854775807}`、キーが 1,024 文字のオブジェクト | `convert … --to <変換後>` | 表せないものは 67(種類 `unrepresentable`)。原因の文は、順に `NaN と無限大は JSON で表せません`(`.inf`・`.nan`・`-.inf`・`[1, .inf]` の 4 つ)、`数値が表せる範囲を超えています`(YAML の 3 つと JSON の 2 つ)、`キーが長すぎます(上限 1024 文字)`(1,025 文字のキー)。受理されるものは 0 で、`null` は `null` のまま出る。64 ビット符号つき整数の範囲の端(`9223372036854775807`)は、number では表せないので、`bigint` か字句のまま扱って、桁を落とさずに出す(AC-499) |
| AC-218 | 複数の文書・タグ・文字列でないキー | `a: 1⏎---⏎b: 2` / `a: !!str 1` / `1: x` / `true: x` | `convert in.yaml --to json` | 67。種類の文は `syntax error`。理由は順に `複数の文書は書けません(1 ファイルに書ける文書は 1 つです)`、`タグは使えません`、`キーは文字列で書きます`、`キーは文字列で書きます` |
| AC-219 | 最上位がマップでない | YAML `- 1⏎- 2`(`--to json`)、JSON `[1, 2]`(`--to yaml`) | `convert` | 67。`top level is not a map: 最上位がマップではありません`(行 1) |
| AC-220 | 空の入力 | 空の YAML ファイル(または `# c` だけ)を `--to json` で。空の JSON ファイル(0 バイト)を `--to yaml` で | `convert` | YAML: 67。`top level is not a map: 文書が空です`。JSON: 67。`syntax error`(JSON では空の入力は誤り) |
| AC-221 | コメントは捨てられる | `# 先頭⏎a: 1 # 末尾` | `convert in.yaml --to json` | 0。OUT = `{⏎  "a": 1⏎}`。OUT に `#` がない |
| AC-222 | キーの順序は入力の記述順 | `b: 1⏎a: 2` | `convert in.yaml --to json` | OUT = `{⏎  "b": 1,⏎  "a": 2⏎}`(辞書順に並べ替えない。マージキーで取り込まれたエントリは `<<` の位置に並ぶ。[09](09-converter.md))。**JavaScript のオブジェクトは、整数に見えるキー(`"1"`、`"2"`)を、挿入順ではなく昇順に並べる**ので、木の保持には `Map` か、キーの順を別に持つ構造を使う(`b: 1⏎1: 2` のようなキーを持つ入力で、順序が保たれることも確かめる。AC-509) |
| AC-223 | 空のマップ | `{}`(YAML)、`{}`(JSON) | `convert in.yaml --to json` / `convert in.json --to yaml` | 0。JSON の出力は `{}`。YAML の出力は `{}` |
| AC-224 | アンカーとマージキー | `base: &b {x: 1, y: 2}⏎d:⏎  <<: *b⏎  y: 9` | `convert in.yaml --to json` | 0。`d` は `"x": 1`、`"y": 9`(自身のキーが優先) |
| AC-225 | マージキーの値がマップでない | `a:⏎  <<: 5` | `convert in.yaml --to json` | 67。種類の文は `syntax error`、理由は `マージキーの値がマップではありません`(行 2。場所 `(at: …)` は付かない) |
| AC-226 | 重複するキー | YAML `a: 1⏎b: 2⏎a: 3`(`--to json`)、JSON `{"a": 1, "b": 2, "a": 3}`(`--to yaml`) | `convert` | 67。`syntax error: キーが重複しています`(YAML は行 3) |
| AC-227 | 再帰するアンカー | `a: &r [*r]` | `convert in.yaml --to json` | 67。`アンカーが自分自身を含んでいます` |
| AC-228 | スカラーの解決 | YAML: `a: yes`、`b: 0x1F`、`c: 007`、`d: '1'`、`e: 2001-01-01`、`f: 1_000`、`g: on`、`h: ~~`、`i: 'null'` | `convert in.yaml --to json` | `"a": "yes"`、`"b": 31`、`"c": 7`、`"d": "1"`、`"e": "2001-01-01"`、`"f": "1_000"`、`"g": "on"`、`"h": "~~"`、`"i": "null"`(キーは入力の記述順に並ぶ)。真偽値になるのは `true`/`false` の 3 通りの書き方(`true`、`True`、`TRUE` と `false` の同様の形)だけ |
| AC-229 | 展開後の要素数の上限 | `a: [0, 0, …]`(0 を n 個)。JSON は `{"a": [0, 0, …]}` | `convert in.yaml --to json` / `convert in.json --to yaml` | n = 99,997 は 0。n = 99,998 は 67(種類の文 `input is too large`。理由 `要素数が上限(100000)を超えています`)。JSON も同じ境界 |
| AC-230 | アンカーの爆発は上限で止まる | アンカーの入れ子で、指数的に増える YAML | `convert in.yaml --to json` | 30 秒以内に 67(`input is too large`。理由は `要素数が上限(100000)を超えています` または `文字の大きさが上限(16 MiB)を超えています`)。メモリを使い果たさない(ヒープの上限で落ちない) |
| AC-231 | 入れ子の深さの上限 | YAML: `a: ` に続けて `[` を 32 個 / 33 個(閉じも書く)。JSON: `{"a": ` に続けて `[` を 32 個 / 33 個(閉じも書く) | `convert` | 32 個: 0。33 個: 67(`syntax error: 入れ子が深すぎます(上限は 32 段)`)。最上位の写像(根)は数えず、その直下のコレクションを 1 段と数える([03](03-definition-format.md)) |
| AC-232 | 極端な深さでも異常終了しない | 1,000 段、100,000 段の入れ子(YAML・JSON) | `convert … ` | 67(`syntax error`)。異常終了しない(`RangeError: Maximum call stack size exceeded` が漏れて 70 にならない。2.2 節) |
| AC-233 | 往復で木が等しい | `CV1` | `convert in.yaml --to json` の OUT を `convert --to yaml` に通す | 結果の木が `CV1` の木と等しい。`command` が配列形式の定義、`retry` の写像も同じ。`CV1J` を先に `--to yaml`、次に `--to json` に通しても等しい |
| AC-234 | 数値の往復 | 整数の端(64 ビット符号つき整数の最小・最大)、浮動小数(`1.0`、`-0.0`、`0.5`、`1e21`)、空の配列・写像 | 往復(YAML → JSON → YAML、JSON → YAML → JSON) | 型が変わらない(`1` と `1.0` を区別する。`0.0` と `-0.0` は別。JSON の出力で、浮動小数は、小数点か指数を持つ書き方になる)。整数の端は、桁を落とさない(number に直さない) |
| AC-235 | ファイルを作らず環境に左右されない | `T` に入力のみ | `convert` を、別のカレントディレクトリ・異なる環境変数で実行 | `T` のファイル一覧が実行の前後で同じ。結果が同じ |
| AC-236 | 標準出力に書けない(注入) | 正しい入力 | `convert`(標準出力を `FailingWriter` に) | 67。ERR = `conductor: error: cannot write to standard output` |
| AC-237 | 引数の誤り | なし | `convert f`(`--to` なし)/ `convert f --to xml` / `convert f --to YAML` / `convert f --to Json` / `convert f --to yaml --config c` | 68。`missing argument: '--to'` / `invalid value: 'xml'` / `invalid value: 'YAML'` / `invalid value: 'Json'` / `option not allowed for this command: '--config'` |
| AC-238 | 変換結果の全体を 1 回で書く | 失敗する入力 | `convert` | 失敗のとき、標準出力に 1 バイトも書かれない |

### 4.8 run ID

特に記さない限り、定義は `two_jobs(n)` = `P(p; a:0, b:<n>)`(`a` は成功、`b` は終了コード `n`)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-239 | run ID ごとに別のステート | `two_jobs(7)` | `pipeline p --run-id r1` と `pipeline p --run-id r2` を同時に | どちらも 7。OUT 空。`T` のファイルは `conductor.yaml`、`conductor.yaml.r1.state`、`conductor.yaml.r2.state` の 3 つだけ。各ステートは `run_id` がその ID、`resume=b`、`a=success` |
| AC-240 | 同じ run ID で再開し、他は変わらない | 上のあと、定義を `two_jobs(0)` に直す | `pipeline p --run-id r1` | 0。`r1` は完了のステートが残る(`completed=true`)。`r2` のステートファイルはバイト列が変わらない |
| AC-241 | ステートの形(run ID あり・なし) | `two_jobs(7)` | `pipeline p --run-id r1`、`pipeline p` | `.r1.state` は `"run_id": "r1"` を持つ。run ID なしの `.state` は `run_id` キーを持たない |
| AC-242 | 環境変数 `CONDUCTOR_RUN_ID` | `two_jobs(7)`、`CONDUCTOR_RUN_ID=fromenv` | `pipeline p` | 7。`conductor.yaml.fromenv.state` ができる |
| AC-243 | 空のオプションは run ID なしに固定 | 同じ環境変数 | `pipeline p --run-id ""` | 7。`conductor.yaml.state`(run ID なし)ができる |
| AC-244 | オプションは環境変数に優先 | 同じ環境変数 | `pipeline p --run-id fromopt` | `conductor.yaml.fromopt.state` ができる(`fromenv` は使われない) |
| AC-245 | 環境変数が空なら run ID なし | `CONDUCTOR_RUN_ID=` | `pipeline p` | 7。`conductor.yaml.state` |
| AC-246 | 定義の `run_id_env` | `P(p run_id_env=MY_RUN; a:7)`、`MY_RUN=m1`、`CONDUCTOR_RUN_ID=ignored` | `run a` | 7。`conductor.yaml.m1.state` ができる(`ignored` は使われない) |
| AC-247 | `watch`・`reset` も `run_id_env` を読む | 上のあと、`MY_RUN=m1` | `watch`、`reset` | `watch` の OUT に `Run ID: m1`。`reset` で `conductor.yaml.m1.state` が消える |
| AC-248 | 有効な run ID | `two_jobs(0)` | `pipeline p --run-id <値>`(`1`、`r1`、`12345`、`build-2026.11_a`、`a.b`、`x` を 64 個) | いずれも 0 |
| AC-249 | 無効な run ID(オプション) | `two_jobs(0)` | `pipeline p --run-id <値>`(`a/b`、`..`、`x` を 65 個、`a b`、`.x`、`日本語`) | 68。ERR の最初の行 = `conductor: error: invalid value: '--run-id'`。ERR にその値が現れない。ステートが作られない |
| AC-250 | 無効な run ID(環境変数) | 同じ値を `CONDUCTOR_RUN_ID` に | `pipeline p` | 68。ERR の最初の行 = `conductor: error: invalid value: 'run ID (environment variable)'`。値が現れない |
| AC-251 | validate は run ID を受け付けない | `two_jobs(0)` | `validate --run-id x` | 68 |
| AC-252 | run ID の決定の順序 | 定義の誤り + `--run-id 'a b'` | `run a --run-id 'a b'` | 64(定義の検査が先。run ID は後) |
| AC-253 | run ID の決定はステートの読み込みの前 | 壊れたステート + `--run-id 'a b'` | `run a --run-id 'a b'` | 68(65 ではない) |
| AC-254 | `watch` は `--run-id` があれば定義を読まない | 定義が壊れている。`.r1.state` あり | `watch --run-id r1` | 0。表が出る |
| AC-255 | `watch`・`reset` は壊れた定義でも既定の環境変数名 | 定義が壊れている。`CONDUCTOR_RUN_ID=r1` | `reset` | 0。`.r1.state` だけが消える |
| AC-256 | run ID なしの `watch` に `Run ID` の行はない | run ID なしのステート | `watch` | OUT に `Run ID` の行がない |
| AC-257 | ファイル名と中身の `run_id` の食い違い | `conductor.yaml.r1.state` を `conductor.yaml.r2.state` に複製する | `run b --run-id r2` | 65。ERR = ``conductor: error: conductor.yaml.r2.state: state file is corrupt: run_id がファイル名の run ID と一致しません; run `conductor reset --run-id <ID>` to discard the state``(`<ID>` は文字どおり) |
| AC-258 | run ID ありでも `--remove-on-complete` で消える | `two_jobs(0)` | `pipeline p --run-id r1 --remove-on-complete` | 0。`conductor.yaml.r1.state` がない |
| AC-259 | `reset --run-id` は別の run に影響しない | `.r1.state`、`.r2.state`、`.state` | `reset --run-id r1` | `.r1.state` だけが消える |
| AC-260 | 大文字小文字を区別しないファイルシステム | macOS の既定 / Windows | `pipeline p --run-id A`、`pipeline p --run-id a` | 同じファイルになる。食い違いは、中身の `run_id` との照合で 65 として検出される(利用者は大文字小文字だけで run ID を区別しない)。区分: macOS / Windows。**未確認**(仕様の記述に基づく) |

### 4.9 リトライ

特に記さない限り、`RT` は `P(p; a:0{cmd="TJ flaky F <fails> 75", retry=(exit_codes=[75], max=3, interval="1s")})`(`F` は `T/flaky.count`。`<fails>` は失敗させる回数)。`flaky` は、実行のたびに `F` の回数を 1 増やす。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-261 | 2 回失敗して 3 回目に成功 | `RT`(`fails=2`) | `run a` | 0。OUT 空。ERR(-v なしでも)= `conductor: retry: job 'a' exited with code 75; retry 1 of 3 in 1 seconds`⏎`conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 1 seconds`。`F` の値 = 3。ステート `a=success(0)` |
| AC-262 | -v で結果の行は 1 回だけ | 上と同じ | `run -v a` | ERR は上の 2 行のあとに `[conductor] job=a result=success (exit code 0)` の 1 行だけ(途中の失敗の結果行は出ない) |
| AC-263 | 使い切る | `RT`(`fails=99`) | `run a` | 75。ERR は retry の行 3 つ(`retry 1 of 3`、`2 of 3`、`3 of 3`)。`F` の値 = 4(最大 4 回の実行)。ステート `a=error(75)`、`resume=a` |
| AC-264 | 対象外の終了コードはリトライしない | `exit_codes: [75]` で、ジョブが 3 で失敗(`flaky F 99 3`) | `run a` | 3。retry の行なし。`F` の値 = 1 |
| AC-265 | `retry` がなければ 1 回だけ実行 | `P(p; a:0{cmd="TJ flaky F 99 75"})` | `run a` | 75。retry の行なし。`F` = 1 |
| AC-266 | 知らせは日本語でも | `RT`(`fails=1`)、`LANG=ja_JP.UTF-8` | `run a` | 0。ERR = `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。1 秒後にリトライします(1/3 回目)` |
| AC-267 | 倍率と上限の待ち時間 | `retry=(exit_codes=[75], max=3, interval="1s", multiplier=2, max_interval="2s")`、`fails=99` | `run a` | 75。retry の行の待ち時間は順に `1`、`2`、`2`(`in 1 seconds`、`in 2 seconds`、`in 2 seconds`)。実行全体の時間は 5 秒以上 |
| AC-268 | 待ち時間の列(関数の検査) | `interval` / `multiplier` / `max_interval` / `max` | 待ち時間を求める関数(`src/flow/`。結果は `bigint`) | `30s`、倍率なし、max 3 → 30、30、30。`10s`、2、`60s`、max 5 → 10、20、40、60、60。`60s`、3、`60s`、max 3 → 60、60、60。`1s`、10、`1h`、max 6 → 1、10、100、1000、3600、3600。`1h`、10、`24h`、max 4 → 3600、36000、86400、86400。極端に大きい値でも桁あふれしない(飽和。AC-499)。区分: 単体 |
| AC-269 | 再実行の結果が成功なら遷移へ(pipeline) | `P(p; a:0{cmd="TJ flaky F 1 75", retry=(…)}, b:0)` | `pipeline p` | 0。`b` も実行される。完了のステート |
| AC-270 | 使い切ると停止(pipeline) | `RT` に `b:0` を足し、`fails=99` | `pipeline p` | 75。`b` は実行されない。`resume=a` |
| AC-271 | ジョブ自身の 127 は一覧にあればリトライ | `TJ exit 127`、`exit_codes: [127]`、`max: 1`、`interval: 1s` | `run a` | 127。retry の行が 1 つ(1 回リトライしてから終わる) |
| AC-272 | 起動の失敗はリトライしない | 存在しないプログラム、`exit_codes: [127]` | `run a` | 127。retry の行なし |
| AC-273 | シグナルで終了したジョブはリトライしない(UNIX 系) | `argv=["sh","-c","kill -9 $$"]`、`exit_codes: [137]` | `run a` | 137。retry の行なし |
| AC-274 | 待機中のシグナル(UNIX 系) | `retry=(exit_codes=[75], max=3, interval="60s")`、`fails=99` | `run a` を起動し、ERR に最初の retry の行が出てから SIGTERM を送る(`child.kill('SIGTERM')`) | 143。ステート: `a=pending`(エラーとして記録しない)、`resume=a`。2 つ目の retry の行は出ない。ジョブは再び起動されない(`F` = 1) |
| AC-275 | 待機中の中断は 1 秒以内 | `interval="1h"`、`fails=99` | 最初の retry の行のあと SIGTERM | SIGTERM を送ってから 1 秒以内に、143 で終わる(待ちは、`AbortSignal` で中断できるタイマー。確認の間隔を持たない)。単体では、3600 秒の待機に 200 ms 後の合図で、全体が 1.2 秒未満 |
| AC-276 | 再開は最初の実行から数え直す | 上のあと(`resume=a`) | 再度 `run a`(`interval="1s"`、`fails=1`) | `retry 1 of 3` の行から始まる。0 で完了 |
| AC-277 | 待機中のシグナル(pipeline) | 同様(pipeline) | `pipeline p`、待機中に SIGTERM | 143。以降のジョブを実行しない。`resume=a` |
| AC-278 | ステートは最終の結果で 1 回だけ | `RT`(`fails=2`) | `run a` | 実行の途中に、ステートファイルのジョブ `a` の状態が `success`/`error` にならない(最終の結果のときだけ変わる。途中の失敗はステートに書かれない)。リトライの回数はステートに書かれない。観測: 待機中にステートファイルを読む |
| AC-279 | README のリトライの説明 | README | (検査) | 見出し、5 つのキー名(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)があり、載っている YAML の例が実際に読み取りを通る。区分: 点検 |

### 4.10 シグナルと中断

UNIX 系の「実行」は、被試験体を起動し(`startConductor`)、`TJ sleep` の記録 `ready` が `O` に現れるのを待ってから、被試験体のプロセスにシグナルを送る(`child.kill('SIGTERM')` など)。定義は `SL` = `P(p; sleeper:0{cmd="TJ --out O sleep"}, after:0)` を使う。
シグナル名(`'SIGTERM'` など)と番号の対応は、`conductor` の中の対応表で行う(AC-502)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-280 | 実行中の SIGTERM(UNIX 系) | `SL` | `run sleeper` を起動し、`ready` のあと SIGTERM | 143。OUT 空。ステート `S[sleeper=error, after=pending ; resume=sleeper]`。`pid=` に記録されたジョブのプロセスが存在しない(孤児なし。`process.kill(pid, 0)` が `ESRCH`) |
| AC-281 | HUP のあとすぐ INT(UNIX 系) | `SL`(`sleeper` だけでもよい) | `ready` のあと SIGHUP、続けて SIGINT | 129(最初の番号 = 1 が残る)。ステート `sleeper=error` |
| AC-282 | SIGINT / SIGQUIT 単独(UNIX 系) | `SL` | SIGINT / SIGQUIT | 130 / 131(SIGQUIT では、`TJ` の終わり方は実装時に確認。3.4 節。確かめるのは `conductor` の終了コードだけ) |
| AC-283 | -v の表示(UNIX 系) | `SL` | `run -v sleeper`、SIGTERM | 143。ERR に `[conductor] job=sleeper result=error (interrupted by signal 15)` |
| AC-284 | 時間を空けた 2 つのシグナルは最初の番号が残る | シグナル監視の単体 | INT のあと、時間を空けて HUP | 最初の番号 2 が残る。区分: 単体(UNIX 系) |
| AC-285 | 最初の番号・登録は 1 回 | シグナル監視の単体 | 監視を登録し、HUP を受ける。再度の登録 | 最初の番号 1 が残る。2 回目の登録は「すでに登録済み」(失敗ではない)で、`process.listenerCount('SIGHUP')` などが増えない(リスナーが二重にならない)。区分: 単体(UNIX 系) |
| AC-286 | 転送は同じシグナルを 1 回だけ | 起動の部品(`src/runner/`)の単体 | ジョブの実行中に合図を立てる(SIGTERM と SIGINT で別々に) | ジョブは、そのシグナルで終了する(`exit` イベントの `signal` が `'SIGTERM'` / `'SIGINT'`、番号 15 / 2)。送信は 1 回(続けて合図を立てても追加で送らない)。区分: 単体(UNIX 系) |
| AC-287 | 許可外の番号は送らない | 起動の部品の単体 | 合図の番号が SIGKILL(9)、SIGSTOP、SIGCONT、SIGUSR1、0、-1、9999 | 送信しない。ジョブは自分で終わる。終わりの観測の直後に `interrupted = true`、`signal_number` がその番号。区分: 単体(UNIX 系) |
| AC-288 | ジョブは新しいプロセスグループ | 起動の部品の単体。[07](07-process-and-signals.md) が、ジョブを新しいプロセスグループに置くと規定する場合(Node では `spawn` の `detached: true`。UNIX 系では新しいセッションを作るとされる。未確認) | ジョブを起動して、プロセスグループ ID を調べる(`ps -o pgid= -p <pid>`) | ジョブのプロセスグループ ID が、ジョブの pid に等しく、`conductor`(呼び出し側)のものと異なる。07 章が規定しない場合は、この項目を行わない。区分: 単体(UNIX 系) |
| AC-289 | 先に終わったジョブには転送しない | 起動の部品の単体 | ジョブが終わった後に合図が立つ | 送信しない(終了済みの pid に `kill` しない)。区分: 単体(UNIX 系) |
| AC-290 | 起動前の合図 | 起動の部品の単体 | 合図が立った状態で起動を呼ぶ | ジョブは起動されない(`started=false`、`interrupted=true`、`signal_number` = 番号)。分類は「分類なし」、終了コード `128 + 番号`。区分: 単体 |
| AC-291 | 終了の観測の直後の合図 | 起動の部品の単体 | ジョブの実行中に、転送されない番号の合図を立てる(ジョブは自分で終わる) | ジョブの終了コードを保ったまま `interrupted = true`、`signal_number` = その番号。分類は「エラー、`128 + 番号`」。区分: 単体 |
| AC-292 | 合図が立たなければ中断にならない | 起動の部品の単体 | 合図なしで `exit 1` | `interrupted = false`、`signal_number` なし。区分: 単体 |
| AC-293 | 起動前の中断の記録(注入) | 流れの制御(偽の `FakeRunner`・`FakeSignals`) | 起動前に合図が立つ | ジョブを起動せず、`resume` = そのジョブ、エラーとして記録しない、終了 `128 + 番号`。区分: 注入 |
| AC-294 | 実行中の中断の記録(注入) | 同上 | 実行中に合図(2 つ来ても最初の番号) | ジョブを `error` として記録、終了 `128 + 最初の番号`。区分: 注入 |
| AC-295 | ジョブのあいだの中断(pipeline、注入) | 同上 | 1 つ目のジョブが成功した直後に合図 | 次のジョブを起動せず、再開位置を記録して `128 + 番号`。-v で `[conductor] interrupted by signal N`。区分: 注入 |
| AC-296 | 検査中の合図より誤りが優先 | 同上 | 定義の検査中に合図が立つ。定義の誤り / 順序外 / スキップ | 64 / 66 / 0(中断の終了コードにならない)。区分: 注入 |
| AC-297 | シグナルの準備は検査の前 | 同上 | `run` / `pipeline`、`validate`・`watch`・`reset`・`convert`・引数の誤り | `run`・`pipeline` だけが、定義の検査より前にシグナルを準備する。ほかは準備しない。区分: 注入 |
| AC-298 | シグナルの準備の失敗は 70 | 同上 | 準備が失敗する | 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査もジョブの起動もしない。区分: 注入 |
| AC-299 | 登録済みでも続ける | 同上 | 準備が「すでに登録済み」を返す | 流れは成功と同様に続く。区分: 注入 |
| AC-300 | 標準エラーが閉じていても本来の終了コード | 同上 | 標準エラーへの書き込みが失敗する(`FailingWriter`) | 本来の終了コードで終わる。区分: 注入 |
| AC-301 | Windows の Ctrl+C(手動) | Windows の実機、コンソール。`SL` | `conductor run sleeper` の実行中に Ctrl+C を押す | (1) 終了コード 130。(2) ジョブの子プロセスが残らない。(3) 続く `watch` で `sleeper` が `error`、`after` が `pending`。(4) `run -v` の ERR に `interrupted by signal 2`。結果を記録欄に残す(`test/WINDOWS_CTRL_C_CHECKLIST.md`)。区分: 手動(Windows)。実施の記録は、この文書の作成時点では存在しない(未確認) |

### 4.11 ステートの異常系

ここの「実行」は、特に記さない限り `run a`(`P(p; a:0, b:0)`)。すべて、**ジョブは実行されず**、ステートファイルは**変わらず**、OUT は空。ERR の最初の行は `conductor: error: conductor.yaml.state: <種類の文>: <理由>; <案内>`。
案内は ``run `conductor reset` to discard the state``(run ID あり: ``run `conductor reset --run-id <ID>` to discard the state``)。ステートは JSON の 1 つのオブジェクト([06](06-state-file.md))。

| ID | 機能 | ステートの内容(前提) | 期待(終了 65) |
|----|------|------------------------|----------------|
| AC-302 | JSON として読めない | `{garbage` | `state file is corrupt: JSON として読めません` |
| AC-303 | UTF-8 でない | 不正なバイト列 | `state file is corrupt: UTF-8 の文字列ではありません` |
| AC-304 | 大きさの上限 | 正しいステートの JSON の末尾に、空白(スペース)を足して 1,048,577 バイト | `state file is too large: 大きさが上限(1 MiB)を超えています`。ちょうど 1,048,576 バイトは受理(実行が進む) |
| AC-305 | 書式の版が違う | `"format_version": 2`(ほかに未知のキーがあっても)/ `0` | `state file version is not supported: 対応していない版です(reset が必要です)` |
| AC-306 | 書式の版がない・整数でない | `format_version` なし / `"1"` / `1.0` | `state file is corrupt: format_version がありません` / `format_version が整数ではありません`(`1.0` も同じ) |
| AC-307 | 未知のキー | 最上位に未知のキー。`jobs` というキー | `state file is corrupt: 知らないキーがあります` |
| AC-308 | 必須のキーがない | `pipeline_name` などを欠く | `state file is corrupt: 必須のキーがありません` |
| AC-309 | 値の型・状態の誤り | `status: "weird"`、`skip_reason` の誤り、`skipped` に `skip_reason` なし、`success` に `skip_reason` あり、`pending` に `exit_code` あり | `state file is corrupt: status の値が誤っています` / `skip_reason が status と合いません` / `exit_code が status と合いません` など([06](06-state-file.md) の矛盾) |
| AC-310 | ジョブの並びの矛盾 | ジョブID の重複 / `job_order` と `job` の不一致 / `resume_job_id` が `job_order` にない / `completed: true` と `resume_job_id` | `state file is corrupt: ジョブIDが重複しています` / `job の並びが job_order と一致しません` / `resume_job_id が job_order にありません` / `completed と resume_job_id が両立しません` |
| AC-311 | 読めないファイル(UNIX 系、非 root) | ステートの権限を 000 に | `cannot read state file: 開く、または読み取ることができません` |
| AC-312 | 定義との食い違い: パイプライン名 | `pipeline_name` が定義と違う(定義側の名前を変えて `run`) | `conductor: error: conductor.yaml.state: state does not match the definition: pipeline name; run `conductor reset` to discard the state` |
| AC-313 | 定義との食い違い: ジョブID | ジョブの追加・改名のあと | `state does not match the definition: job ids; …` |
| AC-314 | 定義との食い違い: ジョブの並び | ジョブの順序を入れ替えたあと | `state does not match the definition: job order; …` |
| AC-315 | 食い違いの案内は run ID ありで変わる | `--run-id r1` のステートで定義を変える | `…; run `conductor reset --run-id <ID>` to discard the state` |
| AC-316 | 日本語 | `{garbage`、`LANG=ja_JP.UTF-8` | `conductor: error: conductor.yaml.state: ステートファイルが壊れています: JSON として読めません; `conductor reset` でステートを初期化できます` |

ステートの異常のあとの回復と、正常な読み込み。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-317 | reset で回復する | 上のいずれかのあと | `reset`、続けて `run a` | 0。先頭から実行される |
| AC-318 | pipeline も同じ | 上のいずれか | `pipeline p` | 65。ジョブは起動しない |
| AC-319 | ファイルなし・長さ 0 は「空」 | ステートなし / 長さ 0 | `run a` | 通常どおり 0 |
| AC-320 | 完了のまま残ったステートは「空」 | 完了のステート(正しい) | `pipeline p` | 先頭から実行して 0。読み込みでファイルは消されない(実行の最初の書き込みで上書きされる) |
| AC-321 | 残った一時ファイルは読み込みに影響しない | `.conductor.yaml.state.tmp.1.2.3` がある | `run a` | 通常どおり 0 |
| AC-322 | 書き込みは原子的 | 書き込み前後 | (ステートを読み続ける) | 読み手は、直前の完全な内容か新しい完全な内容だけを見る(一時ファイルへ書いて、`FileHandle.sync()` で同期し、`fs.rename` で置き換える)。書き込みの失敗でも直前の内容が残る。置き換え先がディレクトリなら書き込みの失敗でステートは変わらず、一時ファイルも残らない |
| AC-323 | 排他制御はしない | 2 つの `run` を同時に | (2.1 節) | 両方が動く。AC-404 を参照 |
| AC-324 | 終了と書き込みの間の停止は再実行(at-least-once) | 終了の直後、書き込みの前に `conductor` が止まる | 再度 `run a` | ジョブが再実行されうる。**外からは再現できない**(偽の部品で検査する)。区分: 注入 |

### 4.12 引数の文法

特に記さない限り、定義は `B0`、OUT は空、ERR は `conductor: error: <文>`⏎(空行)⏎`Usage:` から始まる使い方の全文。「68」の行は、終了 68。

| ID | 機能 | 実行 | 期待 |
|----|------|------|------|
| AC-325 | 引数なし | `conductor` | 68。`missing argument` |
| AC-326 | オプションだけ | `-v` | 68。`missing argument` |
| AC-327 | 知らないコマンド | `bogus` / `''`(空文字) | 68。`unknown command: 'bogus'` / `unknown command: ''` |
| AC-328 | 必須の引数がない | `run` / `pipeline` / `convert` | 68。`missing argument` |
| AC-329 | 余分な引数 | `run a b` / `validate a b` / `watch x` / `reset x` | 68。`unexpected argument: 'b'` / `'b'` / `'x'` / `'x'` |
| AC-330 | 使えないオプション | `watch -v`、`reset -v`、`run --all a`、`run --to yaml a`、`validate --remove-on-complete`、`reset --remove-on-complete`、`convert f --to yaml -v` | 68。`option not allowed for this command: '<そのオプション>'` |
| AC-331 | `--run-id` が使えるコマンド | `run`、`pipeline`、`watch`、`reset` に `--run-id x` | 文法として受理される(4 コマンド)。`validate`・`convert` では 68 |
| AC-332 | 知らないオプション | `run --bogus a` / `run -x a` / `run --bogus=x a` | 68。`unknown option: '--bogus'` / `'-x'` / `'--bogus'` |
| AC-333 | 値の欠落 | `run a --config` / `run a --run-id` / `convert f --to` | 68。`option requires a value: '--config'` / `'--run-id'` / `'--to'` |
| AC-334 | 重複 | `run -vv a`、`run -v -v a`、`run -v --verbose a`、`run a --config x --config y`、`run a --run-id x --run-id y`、`reset --all --all`、`run a --remove-on-complete --remove-on-complete`、`convert f --to yaml --to json` | 68。`invalid value: '<オプション名>'`(`-vv` は `'-v'`) |
| AC-335 | `--all` と `--run-id` | `reset --all --run-id x` | 68。`invalid value: '--all'` |
| AC-336 | 値を取らないオプションに `=値` | `run --verbose=x a`、`reset --all=x`、`run a --remove-on-complete=x` | 68。`invalid value: '--verbose'` / `'--all'` / `'--remove-on-complete'` |
| AC-337 | `--to` の値 | `convert f --to xml` / `YAML` / `Json` | 68。`invalid value: 'xml'` など(値そのもの) |
| AC-338 | `--to` がない | `convert f` | 68。`missing argument: '--to'` |
| AC-339 | `--run-id` の値の検査 | `run a --run-id 'a b'` | 68。`invalid value: '--run-id'`(値を含めない) |
| AC-340 | 空の `--config` | `run a --config=` | 64。`definition file does not exist`(空のパスとして受理され、後で誤りになる) |
| AC-341 | オプションの書き方 | `--config c.yaml -v run a`、`run a -v`、`--config=c.yaml`、`--run-id=r1`、`convert f --to=yaml` | すべて文法として正しい(`run` は実行される。`--run-id=` は run ID なしに固定) |
| AC-342 | `--` のあとは位置引数 | `run -- -weird` / `-- --help` / `run a -- -v` | `job is not defined in the definition file: '-weird'`(68)/ `unknown command: '--help'`(68)/ `unexpected argument: '-v'`(68) |
| AC-343 | ヘルプ(英語) | `--help`、`-h` | 終了 0。OUT = [08](08-output-and-i18n.md) の使い方の全文(末尾は改行)。ERR 空 |
| AC-344 | ヘルプ(日本語) | `LANG=ja_JP.UTF-8` で `--help` | 0。OUT = 08 章の日本語の全文 |
| AC-345 | バージョン | `--version`、`-V` | 0。OUT = `conductor 0.8.0`(版は配布の版)。ERR 空。`LANG=ja_JP.UTF-8` でも英語・同じ |
| AC-346 | `-h`/`-V` は最優先 | `-vh`、`bogus --help`、`run -vv --help`、`-V -h`、`-h -V` | `-vh` は help(0)。`bogus --help` も help(0)。`-V -h` は version、`-h -V` は help(先に現れた方) |
| AC-347 | 使い方の誤りの形 | `bogus`(`LANG=C`)、`bogus`(`LANG=ja_JP.UTF-8`) | ERR の最初の行 = `conductor: error: unknown command: 'bogus'` / `conductor: error: 知らないコマンドです: 'bogus'`。2 行目は空行、3 行目以降は使い方の全文(英語 / 日本語)。OUT 空。ジョブは実行されない |
| AC-348 | 使い方の誤りの全種類 | 上の各種類 | 8 種類(`UnknownCommand`、`MissingArgument`、`UnexpectedArgument`、`UnknownOption`、`OptionNotAllowed`、`MissingValue`、`InvalidValue`、`UnknownJob`)が、すべて 68 で、08 章の英語・日本語の文になる |
| AC-349 | UTF-8 でない位置引数(UNIX 系) | 位置引数・`--run-id` の値に不正なバイト列(AC-406 と同じ方法で渡す) | 68。`invalid value`。`--config` と `convert` の入力パスは、Node が不正なバイトを U+FFFD に置き換えて受け取るとされるので(未確認)、そのパスのファイルは見つからず、終了 64(`--config`)/ 67(`convert`)になる(不正なバイト列のファイル名の定義は、Node 版では読めない制約。10 章)。いずれも異常終了しない。区分: UNIX 系 |
| AC-447 | 値を取るオプションの値は `-h`/`--help` でも値 | `--config --help run a`、`--config -h run a`、`run a --config --version`(定義ファイルは存在しない) | 終了 64。help・version は出ない。ERR に、その語をパスとしたファイルの誤り(`--help: ...` など) |
| AC-448 | 値を取るオプションが値を欠く | `--config --help` / `--help --config` | 前者は 68(`missing argument`。`--help` が値に取られて位置引数が 0 個)。後者は help(0) |
| AC-449 | `-h`/`-V`/`-` と `--` の細部 | `-h=x`、`-V=1`、`--help=x`、`-vh=x`、`run -`、`-- run`、`-- run a` | `-h=x`・`--help=x`・`-vh=x` は help(0)、`-V=1` は version(0)。`run -` はジョブID `-`(定義がなければ 64)。`-- run` は 68(`missing argument`。`run` がコマンド名でジョブIDなし)。`-- run a` は `run` のジョブID `a`(定義がなければ 64) |

引数は、`node:util` の `parseArgs` を使わず自前で解析する([04](04-cli.md))。上の表の文法(`--opt=value`、短縮形の連結、`--`、値を取るオプションの値が `-h` でも値になること)を `parseArgs` が満たさない点は、実装時に確かめる(未確認。10 章)。

### 4.13 出力とロケール

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-350 | 言語の決定: `LC_ALL` が最優先 | `LC_ALL=ja_JP.UTF-8`、`LANG=C` | `bogus` | ERR に `知らないコマンドです`(日本語) |
| AC-351 | 最初の空でない値だけを見る | `LC_ALL=C`、`LANG=ja_JP.UTF-8` | `bogus` | 英語(`unknown command`) |
| AC-352 | `LC_MESSAGES`、`LANG` の順 | `LC_MESSAGES=ja_JP`、`LANG=C` / `LANG=ja_JP`(ほか未設定) | `bogus` | 日本語 / 日本語 |
| AC-353 | 空の値は飛ばす | `LC_ALL=`(空)、`LANG=ja_JP.UTF-8` | `bogus` | 日本語(空の `LC_ALL` は無視) |
| AC-354 | 大文字小文字を区別する | `LANG=JA` / 未設定 | `bogus` | 英語 |
| AC-355 | 標準出力と標準エラーの使い分け | `P(p; a:0)` | `run -v a` ほか | `run`・`pipeline`・`reset`・`validate` は、OUT に何も書かない。OUT に書くのは `watch`、`--help`、`--version`、`convert` だけ。ジョブ本体の出力は、そのまま流れる |
| AC-356 | 成功・スキップで表示なし | `P(p; a:0)`、`LANG` に不正な値 | `run a` | OUT・ERR 空(表示がなければ言語も色も判定しない) |
| AC-357 | -v のトークンは言語によらず英語 | `LANG=ja_JP.UTF-8`、`P(p; a:1{w=[1]})` | `run -v a` | ERR = `[conductor] job=a result=warning (終了コード 1)`(`job=`、`result=`、`warning` は英語) |
| AC-358 | -v の補足: 起動できない | 存在しないプログラム、`-v` | `run -v a` | ERR に `[conductor] job=a result=error (could not start)` と、続いて `conductor: error: job 'a': cannot start command '…'`。日本語は `起動できませんでした` |
| AC-359 | -v の補足: シグナルで終了(UNIX 系) | `kill -9 $$` のジョブ | `run -v a`(日本語も) | `terminated by signal 9` / `シグナルで終了 9` |
| AC-360 | -v の補足: 中断(UNIX 系) | `SL`、SIGTERM | `run -v sleeper` | `interrupted by signal 15`(日本語 `シグナルで中断 15`)。起動前の中断は `interrupted by signal N, not started` / `シグナルで中断 N, 起動せず`(注入) |
| AC-361 | 全種類のイベントの形 | [08](08-output-and-i18n.md) のイベント | (表示の部品を直接呼ぶ) | すべての文言が空でなく、改行で終わり、生の制御文字を含まない。誤りの行は `conductor: error: ` で始まる。英語の表示は日本語の文字を含まない(原因の文を除く)。区分: 単体 |
| AC-362 | 定義の誤りの形 | `P(…)` の誤り | `validate` | 1 行ずつ `conductor: error: <ファイル>[:<行>]: <種類の文>: <原因>`。原因は日本語の固定の文(英語のロケールでも) |
| AC-363 | 定義の誤りの種類の文(日本語) | `LANG=ja_JP.UTF-8` | 4.1 節の各誤り | 種類の文が 08 章の日本語(例: `定義にないキーです`、`ジョブが 1 つもありません`) |
| AC-364 | 定義の誤りの種類の文(英語・retry) | 4.1 節の retry の各誤り | `validate` | 08 章の英語の文(`retry` の 9 種) |
| AC-365 | ステートの誤りの補足 | 4.11 節の各誤り | (文脈ごと) | 読み込み: 案内 `; …`。書き込み: ` (the job was not started)` / ` (the job has already been executed)`。`reset`・`watch` 以外の読み込みは案内つき、`reset` の削除は補足なし |
| AC-366 | 拒否・完了判定不能・出力失敗の文言 | 08 章 | (表示の部品を直接呼ぶ) | `cannot run this job now: …`、`cannot determine whether the pipeline is complete`、`cannot write to standard output` の英語・日本語。区分: 単体 |
| AC-367 | 完了を判定できない | 全ジョブが「実行済み」で通過し、どこでも完了にならない状態(通常は起こらない) | `pipeline p` | 65。ERR = `conductor: error: conductor.yaml.state: cannot determine whether the pipeline is complete`。ステートを初期化しない。区分: 注入 |
| AC-368 | 内部の誤りの文は固定の英語 | `src/cli/` が例外(`throw new Error(...)`)を投げる(偽の部品に投げさせる) | (入口の `main` の `try/catch`) | ERR = `conductor: error: internal error`(ロケールによらず。例外のメッセージとスタックは出さない)、終了 70。標準エラーが閉じていても 70。区分: 注入 |
| AC-369 | 入口に異常終了を起こすコマンドがない | 製品の入口 | (点検) | 入口(引数の解釈の結果)に、わざと異常終了するコマンドがない。入口は 1 回だけ終了を要求する(`process.exit` は `src/bin/conductor.ts` の 1 か所以下)。点検の項目は AC-422、AC-498 |

### 4.14 表示のエスケープ

入力に由来する文字列(ジョブID、名前、パス、キー名、コマンド名、使い方の誤りの対象)は、表示する前にエスケープする。**ジョブ本体の出力は通さない**(そのまま)。

規則: LF → `\n`、CR → `\r`、TAB → `\t`。そのほかの制御文字(U+0000〜U+001F、U+007F〜U+009F)と、書式・方向制御の文字(U+200B〜U+200F、U+2028、U+2029、U+202A〜U+202E、U+2060〜U+2064、U+2066〜U+2069、U+FEFF)は、`\u{<小文字の 16 進。先頭の 0 なし>}`。
そのほかの文字(日本語を含む)はそのまま。名前用は、エスケープ後 64 文字(**コードポイント**。UTF-16 の単位ではない。`for (const ch of s)` で数える)で切る(エスケープ列は 1 かたまり。次の 1 文字を足すと 64 を超えるなら、そこで打ち切り、末尾に `…`(U+2026)を 1 つ付ける。出力は最大 65 文字)。パスは切らない。サロゲートペア(絵文字など)を、途中で切らない。不正な UTF-8 のバイトは、U+FFFD として扱う(Node の引数・ファイルの復号の結果)。

外から確かめる方法: `B0` に対して `run '<入力>'`(定義にないジョブID)を実行し、ERR の最初の行が `conductor: error: job is not defined in the definition file: '<出力>'` になることを確かめる。終了 68。

| ID | 機能 | 入力 | 出力(期待) |
|----|------|------|--------------|
| AC-370 | LF と ESC | `a<LF>b<ESC>[0m` | `a\nb\u{1b}[0m` |
| AC-371 | TAB | `k<TAB>x` | `k\tx` |
| AC-372 | CR | `a<CR>b` | `a\rb` |
| AC-373 | 日本語と空白 | `日本語 abc` | `日本語 abc`(そのまま) |
| AC-374 | ゼロ幅スペース | `a<U+200B>b` | `a\u{200b}b` |
| AC-375 | 右から左への上書き | `a<U+202E>b` | `a\u{202e}b` |
| AC-376 | 方向の隔離 | `a<U+2066>b` | `a\u{2066}b` |
| AC-377 | BOM | `a<U+FEFF>b` | `a\u{feff}b` |
| AC-378 | 行・段落の区切り | `a<U+2028>b<U+2029>` | `a\u{2028}b\u{2029}` |
| AC-379 | その他の制御文字 | `<U+007F><U+0085>`(NUL は引数に渡せないため、単体で確かめる) | `\u{7f}\u{85}`(NUL は `\u{0}`。区分: 単体) |
| AC-380 | 64 文字ちょうど | `k` × 64 | `k` × 64(切らない) |
| AC-381 | 64 文字を超える | `k` × 200 | `k` × 64 + `…` |
| AC-382 | エスケープ後で数える | LF × 100 | `\n` × 32 + `…`(65 文字) |
| AC-383 | 上限 3(単体)とコードポイントの数え方 | `abcdef`、上限 3。`😀` × 64 と `😀` × 65(`😀` は U+1F600。UTF-16 で 2 単位) | `abc…`。空の入力は空。`😀` × 64 は切らない、`😀` × 65 は `😀` × 64 + `…`(サロゲートペアの途中で切らない。`.length` で数えると誤る)。区分: 単体 |
| AC-384 | 全部の部品で同じ結果 | 上の表の入力 | 定義・ステート・変換の誤りを作る各部品が、この表と同じ結果を返す(共通の関数を使ってよい)。区分: 単体 |
| AC-385 | ジョブIDは 64 文字で切る | ジョブID が 200 文字の定義 | `watch` の行・`-v` の行・`run` の誤りで、ID が 64 文字 + `…` |

表示の上限と、各部の適用。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-386 | パスは切らない | 300 文字のファイル名の定義(存在しない) | `validate --config <パス>` | ERR にパスの全体が現れる |
| AC-387 | リトライの知らせのジョブID | ジョブID に制御文字・200 文字 | リトライの行 | エスケープされ、64 文字で切られる |
| AC-388 | 定義の誤りのキー名 | 定義のキー名に制御文字 | `validate` | キー名がエスケープされ、64 文字で切られる |

### 4.15 終了コード全件

「終了コードの全件」の確認。各コードを起こす受け入れテストを、表で示す。

| 終了コード | 意味 | 起こすテスト |
|------------|------|--------------|
| 0 | 成功・警告・スキップ・`watch`・`reset`・`validate`・`convert` の成功・`--help`・`--version`・`reset --all` の警告 | AC-389 |
| 1 | 成功コードに含まれない 0、エラーで下位 8 ビットが 0 | AC-390 |
| ジョブの値(1〜255) | エラーのジョブの終了コード | AC-391 |
| 64 | 定義の誤り・`command` の誤り | 4.1 節 |
| 65 | ステートの誤り・書き込み・削除の失敗・`watch` の出力失敗・完了判定不能 | 4.11 節、4.3 節、4.6 節、4.5 節 |
| 66 | 拒否(順序外) | 4.3 節 |
| 67 | 変換の誤り・`convert` の出力失敗 | 4.7 節 |
| 68 | 使い方の誤り(知らないジョブID、run ID の誤りを含む) | 4.3、4.8、4.12 節 |
| 70 | 内部の誤り(未捕捉の例外・Promise の拒否を含む)・シグナルの準備の失敗 | 4.10 節・4.13 節(注入)、AC-489、AC-490 |
| 127 | ジョブを起動できなかった | 4.2 節 |
| 128 + 番号 | ジョブがシグナルで終了、または中断 | 4.3 節、4.9 節、4.10 節 |

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-389 | 終了 0 の全種類 | 成功 / 警告(`w=[3]` で `exit 3`)/ スキップ(実行済み)/ `watch`・`reset`・`validate`・`convert`・`--help`・`--version` の成功 / `reset --all` の警告 | 各実行 | すべて 0 |
| AC-390 | 終了 1 の 2 種類 | `P(p; a:0{s=[5]})`(成功コードにない 0)/ エラーで下位 8 ビットが 0 になる終了コード | `run a` | 1 / 1(`exit 256` は Windows で確かめる。UNIX 系では OS が 0 にするので成功。関数の検査は下) |
| AC-391 | ジョブの値 | `exit 1`、`exit 2`、`exit 7`、`exit 255` | `run a` | 同じ値 |
| AC-392 | 終了コードの範囲合わせ(関数の検査) | 報告する値 `reported`、`is_error` | 範囲合わせの関数(`src/flow/`) | `0..255` はそのまま。`259`→3、`511`→255、`-1`→255、`-2`→254、`0xC0000005`(32 ビット符号つき。符号なしの `3221225477` で渡されても同じ)→5、`4294967295`(符号なしで渡された -1)→255。`256`・`512`・`-256`・32 ビットの最小値は、エラーなら 1、エラーでなければ 0。number の演算で足りる範囲(32 ビットの範囲)。区分: 単体 |
| AC-393 | 中断の終了コード(関数の検査) | シグナル番号 n | `128 + n` を範囲合わせ(エラー扱い) | 15→143、2→130、1→129、3→131、300→172(範囲を超えない)。区分: 単体 |
| AC-394 | 分類の全 10 項目(関数の検査) | [07](07-process-and-signals.md) の分類の表の各行 | 分類の関数(`src/flow/`) | 順 1: 起動前の中断 → 分類なし、`128 + n`。順 2: 起動なし・中断でもない → エラー 127。順 3: 起動済みの中断 → `128 + n`(ジョブの終了コードを見ない)。順 4: 起動失敗 → 127。順 5: シグナル終了 n → `128 + n`。順 6: 終了コードなし → 1(`conductor` のエラー)。順 7: 成功コード → 0。順 8: 警告コード → 0。順 9: 0 で成功コードにない → 1(`conductor` のエラー)。順 10: そのまま。優先順位は上の行ほど高い。区分: 単体 |
| AC-395 | 終了コードの決定順序 | 複数の誤りを同時に持つ入力 | `run` | 検査の順(引数 68 → 準備 70 → 定義 64 → `command` 64 → ジョブID 68 → run ID 68 → ステートの読み込み 65 → 照合 65 → 起動前の判断 66/0 → 初期ステート 65 → ジョブ → 結果の書き込み 65 → 削除 65)で、最初の誤りだけが採られる。具体例は 4.3 節(定義の誤り > ジョブID)、4.8 節(定義 64 > run ID 68 > ステート 65) |
| AC-396 | 128 + 番号の例 | ジョブが SIGKILL / 実行中に SIGTERM(UNIX 系) | `run a` | 137 / 143 |

### 4.16 非機能要件の検証

2 章の各基準の受け入れテスト。「計測」は p95 などの測定、「強制」は基準超過で不合格(2 章の扱いの列)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-397 | 定義の読み込みの性能 | ジョブ 10 個の定義(ファイルから読む) | 最初の 5 回を捨てて、読み込みと検証を 100 回(プロセス内) | p95 が 20 ms 以下(目安。超えたら警告を記録)。区分: 計測 |
| AC-398 | 起動前の処理の性能 | ジョブ 10 個の定義 | `command` の検査、ステートの読み、照合、速い書き込みを、最初の 5 回を捨てて 100 回 | p95 が 20 ms 以下(仮。目安)。区分: 計測 |
| AC-399 | `run` のオーバーヘッド | ジョブ 10 個の定義(各ジョブは `TJ timing`) | `run job0` を 20 回 | `conductor` 自身の時間(全体 − `boot_ms` − (`end_ns` − `start_ns`))が p95 350 ms 以下(仮。目安)。区分: 計測 |
| AC-400 | `pipeline` のジョブあたり | 同上 | 10 個と 1 個の `pipeline` を各 5 回 | ジョブ 1 つあたりの増分(2.1 節の式)が p95 50 ms 以下(目安)。区分: 計測 |
| AC-401 | 大きな定義 | ジョブ 1,000 個の定義 | `run job0` | 1,500 ms 以内(仮。目安)。区分: 計測 |
| AC-402 | 大きな流れ | ジョブ 1,000 個の `pipeline`(偽の部品で、プロセスもファイルも使わない) | (内部の流れ) | 10,000 ms 以内(目安)。完了のステートが残ることは常に検査。区分: 計測・単体 |
| AC-403 | 待機中の CPU 時間(Linux) | `TJ sleep 4`(約 4 秒)のジョブ | `run a` を起動し、`conductor` の pid の `/proc/<pid>/stat` の `utime + stime`(14・15 番目の値。単位は時計の刻み。`getconf CLK_TCK` で秒に直す)を、起動の 1 秒後と 2.5 秒後に読む | 増分が 50 ms 以下(待機中に、タイマーの確認やイベントループの空回りで CPU を使わない)。強制(`/proc` が読めなければスキップ)。区分: Linux |
| AC-404 | 同時の 2 つの `run` | `P(p; a:0{cmd="TJ sleep 1"})`(終了コード 99) | 同じ定義に `run a` を 2 つ同時に | 両方が実行され(ジョブが 2 回始まる)、どちらも終了コード 99。ステートは正しい形で読み戻せる。強制 |
| AC-405 | ランダムな引数 | 固定の種(`mulberry32(1)`) | 語の表から 0〜6 個を選んだ引数列で 500 回(`src/cli/` を偽の部品で、入口の関数を通して呼ぶ) | 終了コードが 0〜255。標準エラーに、制御文字(改行を除く)と U+202E がない。例外が漏れない |
| AC-406 | UTF-8 でない引数(UNIX 系) | `0xff 0xfe` を含むバイト列。`child_process` の引数は文字列で、不正なバイト列を渡せないので、`spawn('sh', ['-c', 'exec "$0" "$@"', ...])` ではなく、`sh -c 'exec node dist/bin/conductor.js run "$(printf "\377\376")"'` のように、シェルの `printf` で作る(試験の道具としてシェルを使うのは可。製品はシェルを使わない) | ジョブID・`--config`・オプション・`convert` の引数に渡す | 異常終了しない(終了コードが 0〜255)。区分: UNIX 系 |
| AC-407 | 壊れたステート | 状態・再開位置・フラグを乱したステート、ランダムなバイト列、巨大な値 | `run` / `watch` | 異常終了しない。壊れていれば 65 |
| AC-408 | 空・制御文字のジョブID | ジョブID が空、制御文字を含む | `run <ID>` | 異常終了しない(68) |
| AC-409 | ランダムな `command` | 文字集合 `a b ⇥ ⏎ ' " \ ; \| & $ * あ` から長さ 0〜39 の文字列 2,000 件(定義の `command` に入れる) | `validate`(分割の単体も) | 0 か 64 のどちらか。0 のとき `program` は空でない |
| AC-410 | 大きな `command` | `prog ` + `'a b' "c\"d" e ` を 10 万回(約 30 万引数) | `validate` | 0。閉じない引用符を足したものは 64。30 秒以内。`Array.prototype.push(...大きな配列)` や再帰で `RangeError` にならない |
| AC-411 | 起動できない `command` | `''`、`'`、`""`、存在しない名前、ディレクトリ、`/` | `validate` / `run a` | 前 3 つは 64、残りは 127。異常終了しない |
| AC-412 | 入れ子の深い入力 | 深さ 1,000 の YAML と JSON(定義、ステート) | `validate` / `run a` / `convert` | 64 / 65 / 67。異常終了せず(`RangeError` が漏れて 70 にならず)、`node` の `--stack-size` を変えなくても動く |
| AC-413 | 変換の頑健性 | 範囲外・空・壊れた・切り詰めた入力、ランダムな構造 | `convert` | 異常終了しない。成功した構造は往復で等しい |
| AC-414 | 内部の誤り | 異常終了を起こす | (入口) | 4.13 節の「内部の誤りの文」と同じ(AC-368)。実際のプロセスでの検査は AC-489、AC-490 |
| AC-415 | 検証は副作用を持たない | 4.1 節の最後の項目 | — | AC-057 |
| AC-416 | 入力の値を表示に出さない | 各種類のエラー(定義・`command`・ステート・変換・使い方)に、秘密のつもりの文字列を含む入力。YAML ライブラリが、入力の断片を含むエラー文を返す入力(例: 不正な文字を含む行)も含む | 各コマンド | ERR に、その値が現れない(キー名は、エスケープして現れてよい)。`command` の誤りの文に `command` の中身が現れない。例外のスタック(`at ` で始まる行)が現れない |
| AC-417 | ステートに機微な値を書かない | 4.3 節の項目 | — | AC-148 |
| AC-418 | 変換に副作用がない | 4.7 節の項目 | — | AC-235。加えて、`src/convert/` が `node:child_process`・ネットワークのモジュールを import せず、`node:fs` を使うのが入力ファイルの読み取りの入口だけであること(点検) |
| AC-419 | ネットワークに接続しない | 実装と依存 | 点検(構文木で `import` と `require`、グローバルの参照を走査。`npm ls --omit=dev --all --parseable` の一覧の各パッケージのソースの文字列検索) | 製品のコードが `node:net`・`node:http`・`node:https`・`node:http2`・`node:tls`・`node:dgram`・`node:dns`(接頭辞なしの `net` なども)を import せず、`fetch`・`WebSocket`・`XMLHttpRequest` を参照しない。実行時の依存のソースにも、それらの import がない(YAML ライブラリは未確認)。区分: 点検 |
| AC-420 | OS 固有コードの隔離と、動的実行・ネイティブの不使用 | ソース | 点検(構文木の走査) | `process.platform`、`os.platform()`、`os.type()` の参照が、`src/runner/platform.ts` の外にない。`eval`、`new Function`、`node:vm`、`node:worker_threads`、`process.dlopen`、`process.binding`、`.node` の読み込み、`import.meta.resolve` による動的な場所の決定が、どこにもない(0 か所)。区分: 点検 |
| AC-421 | 異常終了を起こす書き方がない | ソース、ESLint の設定 | 点検(構文木の走査と、ESLint の規則) | 製品のコード(`src/`)に、(1) `process.exit(`(`src/bin/conductor.ts` の 1 か所以下を除く。AC-498)、(2) 非 null 表明(`x!`)、(3) 型表明(`x as T`。`as const` を除く)、(4) `any` の型注釈と `@ts-ignore`・`@ts-expect-error`・`eslint-disable` による無効化、(5) `JSON.parse`(自前の読み取りを使う)、(6) 待たない Promise(`@typescript-eslint/no-floating-promises`)、(7) `Error` 以外の値の `throw`(`@typescript-eslint/only-throw-error`)がない。配列・マップの添字は `noUncheckedIndexedAccess` の下で、`undefined` を検査してから使う。区分: 点検 |
| AC-422 | テスト専用の経路を持たない | ソース、配布物 | 点検 | 入口に、異常終了を起こす経路や、テスト用の環境変数・隠しオプションがない。`test/` を、`src/` が import しない。`npm pack --dry-run --json` のファイル一覧に、`test/`・`src/`・`scripts/`・設定ファイルが含まれない(含まれるのは `dist/`、`package.json`、`LICENSE`、`README.md`)。テスト用ジョブ `tj.mjs` が入っていない |
| AC-423 | 著作権表示 | ソース | 点検 | 各 `.ts`・`.js`・`.mjs` ファイルの先頭(`#!` の行があればその次)に Apache License 2.0 のヘッダ(`Copyright 2026 agwlvssainokuni`)がある。`.yml`・`.sh`・`.cmd` も同じ(コメントの書式は各言語に合わせる)。`node_modules/`・`dist/`・`coverage/`・`test/fixtures/` は対象外。区分: 点検 |

### 4.17 デモの受け入れテスト

デモの仕様は 7 章。実行は、`CONDUCTOR=<被試験体>`(`dist/bin/conductor.js` か、実行ファイルのパス)、`DEMO_LANG=en_US.UTF-8`、`DEMO_SLEEP=0`、一時ディレクトリの親(UNIX 系は `TMPDIR`)を空のディレクトリ `D` にして行う。標準入力は空。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-424 | UNIX 系: スクリプトが全部通る | 上の設定 | `sh demo/run-demo.sh` | 終了 0。見出し `=== 0. ` から `=== 10. ` までが順に出る。区分: UNIX 系 |
| AC-425 | UNIX 系: 場面 7(検査) | 同上 | 同上 | 出力に `definition is valid`、`command has an unclosed quote`、`→ 終了コード 64` が現れる。誤りの定義の検査のあと、ステートの一覧が `(なし)` |
| AC-426 | UNIX 系: 場面 4(再開・完了) | 同上 | 同上 | `completed` の行、`skipped (already executed)` が現れる |
| AC-427 | UNIX 系: 場面 8(run ID) | 同上 | 同上 | `conductor.yaml.build-1.state` が一覧に現れる。再開のあと `completed`。`reset` のあと一覧は `(なし)` |
| AC-428 | UNIX 系: 場面 9(環境変数と後始末) | 同上 | 同上 | `conductor.yaml.opt-1.state` が現れ、最後の `watch` が `no state` |
| AC-429 | UNIX 系: 場面 10(リトライ) | 同上 | 同上 | `conductor: retry: job 'flaky' exited with code 75; retry 1 of 3 in 1 seconds` と `… retry 2 of 3 in 1 seconds` の 2 行、`[flaky] 3 回目の実行`、`job=flaky result=success` が現れる |
| AC-430 | UNIX 系: 利用者の環境に左右されない | `CONDUCTOR_RUN_ID=zzz`、`DEMO_RUN_ID=zzz`、`LC_ALL=ja_JP.UTF-8`、`LC_MESSAGES=ja_JP.UTF-8`、`NO_COLOR=1` を設定 | 同上 | 出力に `zzz` が現れない。英語の表示(`DEMO_LANG` のとおり)。結果が上と同じ |
| AC-431 | UNIX 系: 何も残さない | 同上 | 同上 | 終了後、一時ディレクトリの親 `D` に何も残らない。リポジトリの作業ツリーは変わらない(`dist/` を作った場合を除く。`dist/` は `.gitignore` の対象) |
| AC-432 | Windows 版: 定義 | 定義 `demo\windows\conductor.yaml` | (検査) | 4 ジョブ(`extract`、`transform`、`load`、`notify`)と、`extract` の警告の遷移(`notify` へ)を持つ。`validate` が 0。区分: Windows |
| AC-433 | Windows 版: 場面 1〜2 | 一時ディレクトリに定義とスクリプトを写す | `pipeline nightly`(場面 1)、`DEMO_EXTRACT_EXIT=1` で `pipeline nightly`(場面 2) | 場面 1 は 0。場面 2 は `notify` へ進み 0 |
| AC-434 | Windows 版: 場面 3〜4 | 同上 | `DEMO_LOAD_EXIT=2` で `pipeline`(2)、続けて `pipeline`(`load` から再開) | 2 のあと、再実行で `load` から再開して 0 |
| AC-435 | Windows 版: 場面 5〜7 | 同上 | `run extract`、`reset`、`convert conductor.yaml --to json`、`validate`、誤りの定義の `validate` | `run`・`reset`・`convert`・`validate` は 0。誤りの定義は 64 |
| AC-436 | Windows 版: 場面 8〜9 | 同上 | run ID を使う一連(場面 8、9) | run ID ごとにステートが分かれる。`--run-id` が環境変数に勝つ。`reset --run-id` と `--all` が働く |
| AC-437 | Windows 版: 場面 10 | 同上 | `pipeline --config retry.yaml retry-demo` | 2 回失敗してから成功する |
| AC-438 | Windows 版: 利用者の run ID に影響されない | 利用者の run ID の環境変数を設定 | 同上 | 結果が変わらない |

### 4.18 リリース物の受け入れテスト

リリースの仕様は 6 章。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-439 | 成果物の名前と形 | リリース | (成果物の一覧) | `conductor-<版>.tgz`(`npm pack` の出力。パッケージ名を置き換えた場合は、その名前)と `SHA256SUMS` の 2 つがある。区分: 点検 |
| AC-440 | 展開の構造 | tarball | 展開(`tar xzf`) | 最上位に `package/` があり、その中に `package.json`、`LICENSE`、`README.md`、`dist/`(`dist/bin/conductor.js` を含む)がある。`test/`・`src/` がない。テスト用ジョブがない。区分: 点検 |
| AC-441 | チェックサム | `SHA256SUMS` | `sha256sum -c`(macOS は `shasum -a 256 -c`)の形式で検査 | tarball の行があり、検査が通る。区分: 点検 |
| AC-442 | 版の表示 | tarball を空のディレクトリに `npm install --prefix <dir> ./conductor-<版>.tgz` した結果 | `<dir>/node_modules/.bin/conductor --version`(Windows は `conductor.cmd`)と、`node <dir>/node_modules/conductor/dist/bin/conductor.js --version` | 0。OUT = `conductor <版>`。区分: 点検(3 OS) |
| AC-443 | 実行時の依存が 1 つ | `package.json` と tarball | 点検(`npm ls --omit=dev --all`、tarball の中のファイル一覧) | 実行時の依存(`dependencies`)が YAML ライブラリ 1 つだけ。tarball にネイティブのファイル(`.node`、`.so`、`.dll`、`.dylib`)がない。`engines.node` が書かれている。区分: 点検 |
| AC-444 | タグの検査 | タグ `v<版>` | リリースの流れ | タグが `main` 上にない、または版が `src/version.ts` の定数か `package.json` の `version`(と `package-lock.json` の版)と違うと失敗する(タグが `main` 上にない旨 / タグと版の不一致の旨)。区分: 点検 |
| AC-445 | 下書きとプレリリース | リリースの作成 | (作成されたリリース) | 下書きである。版に `-` があればプレリリース。注記に固定の 1 文がある。区分: 点検 |
| AC-446 | 3 OS で受け入れテストが通る | Linux・macOS・Windows の実機(または同等の実行環境) | 4 章の自動の項目(各 OS で行える項目)。`CONDUCTOR_BIN` に、AC-442 で入れた配布物の `dist/bin/conductor.js` を渡す形でも通ること | 全項目が合格する(OS の区分の項目は、その OS で)。区分: 自動 |

### 4.19 YAML と JSON に固有の項目

定義の形式(YAML・JSON)、ステートの形式(JSON)、変換(YAML ⇔ JSON)の、形式固有の落とし穴を確かめる。特に記さない限り、「64」の期待は 4.1 節と同じ(終了 64、OUT 空、ステートなし、ジョブを起動しない。ERR の最初の行は `conductor: error: <ファイル>:<行>: ` で始まる(行がわかる誤り)。種類の文は [03](03-definition-format.md)・[08](08-output-and-i18n.md) のとおり)。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-450 | JSON の定義 | `P(p; a:0)` を JSON で書いた `T/conductor.json` | `validate --config conductor.json`、`run a --config conductor.json` | 0 / 0。ステートは `T/conductor.json.state`。YAML の定義と同じ結果 |
| AC-451 | `.yml` も受理 | `T/conductor.yml`(`B0`) | `run a --config conductor.yml` | 0。ステートは `T/conductor.yml.state` |
| AC-452 | 拡張子が違うファイルは拒否 | `B0` の内容を、`conductor.xml`、`conductor.txt`、拡張子なしの `conductor`、`conductor.yaml.bak` に置く(ファイルが存在しないパス `nofile.txt` も) | `validate --config <ファイル>` | 64。ERR の最初の行は `conductor: error: <ファイル>: unsupported definition file format: 拡張子は .yaml、.yml、.json のいずれかで書きます`(行なし。[03](03-definition-format.md))。大文字の拡張子 `conductor.YAML` は受理(0)。ステートなし。ジョブは起動されない(`convert` の入力は拡張子を見ない。AC-209) |
| AC-453 | YAML のタグを拒否 | `command: !!str "TJ exit 0"`、`id: !foo a`、`name: !!binary aGk=` | `validate` | 64(それぞれ別の定義)。行は、そのタグの行 |
| AC-454 | YAML の複数の文書を拒否 | `B0` の内容 + `---⏎` + もう 1 つの文書 | `validate` | 64。行は 2 つ目の文書の始まり。先頭に `---` が 1 つだけある `B0` は受理(0) |
| AC-455 | YAML の文字列でないキーを拒否 | `1: x`、`true: x`、`? [a, b]⏎: x`(トップレベルまたは `job` の中) | `validate` | 64 |
| AC-456 | YAML の重複キー(入れ子) | `job` の要素に `id` を 2 回、`pipeline` に `name` を 2 回 | `validate` | 64。`syntax error`。行は 2 回目のキーの行 |
| AC-457 | YAML のアンカー・エイリアス・マージキーは展開して受理 | 既知のキーの中に書く。(a) 1 つ目のジョブに `warning_exit_codes: &w [3]`、2 つ目に `warning_exit_codes: *w`。(b) 遷移 `- <<: {on: success, next: "@end"}`。(c) トップレベルに `defaults: &d {…}` という未知のキー | `validate`、`run` | (a)(b) は 0。展開した値で動く(2 つ目のジョブの警告コードは `[3]`、(b) は `@end` への遷移)。(c) は 64(`unknown key`。アンカーの置き場所のための未知のキーは作れない)。整合: 10 章 |
| AC-458 | 数値・真偽値に見えるジョブID | `id: 1`、`id: true`、`id: "1"`、`id: yes`、`id: on`、`id: 0x10` | `validate`(それぞれ別の定義) | `id: 1`・`id: true`・`id: 0x10` は 64(`wrong value type`)。`id: "1"`・`id: yes`・`id: on` は 0(文字列。YAML 1.2 の既定の規則。`on`・`yes` を真偽値として読まない。YAML ライブラリの版の指定で決まる。実装時に確認) |
| AC-459 | `command` の書き方(YAML のブロック・折り返し・複数行) | `command: \|⏎  TJ --out O echo a⏎  b⏎`(ブロック)、`command: >-⏎  TJ --out O echo⏎  a b`(折りたたみ)、複数行の plain スカラー(`TJ` は、各書き方に合わせて 2 つの語を書く) | `run a` | 0。`O` の引数は、それぞれ `a`、`b` / `a`、`b` / 同様(改行・空白は区切り) |
| AC-460 | 定義の中の null の扱い | `name: ~`、`command: null`、`warning_exit_codes: ~`、`retry: null`、`remove_on_complete:`(値なし)。JSON: `"command": null` | `validate` | 64。`name`・`command`・`warning_exit_codes`・`remove_on_complete`・JSON の `command` の `null` は `wrong value type`。`retry: null` は `retry must be a map`。未知のキーの値が `null` なら `unknown key`([03](03-definition-format.md)) |
| AC-461 | JSON の重複キー | `{"format_version": 1, "format_version": 1, ...}`(`conductor.json`)、`job` の要素の `id` の重複 | `validate --config conductor.json` | 64。`syntax error`(理由 `キーが重複しています`。`JSON.parse` は重複を受理し、後の値が前の値を上書きするので、自前の読み取りが検出する。[03](03-definition-format.md)) |
| AC-462 | JSON のコメントを拒否 | 行コメント `// c`、ブロックコメント `/* c */` を含む定義 | `validate --config conductor.json` | 64。`syntax error` |
| AC-463 | JSON の末尾のカンマを拒否 | `[1,]`、`{"a": 1,}`(定義の中) | `validate --config conductor.json` | 64。`syntax error` |
| AC-464 | JSON の非標準の書き方を拒否 | 一重引用符の文字列、`NaN`、`Infinity`、先頭が 0 の数(`01`)、引用符のないキー、値のあとの余分なデータ(`{...} {...}`、`{...} x`) | `validate --config conductor.json` | 64。`syntax error`(それぞれ別の定義) |
| AC-465 | JSON の最上位がオブジェクトでない | `[]`、`"x"`、`1`、`null`、`true` | `validate --config conductor.json` | 64。`wrong value type`(理由 `最上位はマップで書きます`。JSON の `null` も同じ。[03](03-definition-format.md)) |
| AC-466 | JSON の大きさの上限 | 正しい JSON の定義の末尾に、空白を足して 1,048,576 バイト / 1,048,577 バイト | `validate --config conductor.json` | 0 / 64(`definition file is too large`) |
| AC-467 | JSON の文字列のエスケープ | `"command": "TJ --out O echo \"a b\" c A 😀"`(`TJ` は 2 つの語)、不正なエスケープ `"\x"`、閉じていない文字列、制御文字(生の LF)を含む文字列 | `run a --config conductor.json`(正しいもの)、`validate`(誤り) | 正しいもの: `O` の引数 = `a b`、`c`、`A`、`😀`(サロゲートペアのエスケープが 1 つの文字になる)。誤りは 64(`syntax error`)。孤立したサロゲートのエスケープ(`"\ud800"`)の扱いは 03 章に従う(未確認) |
| AC-468 | JSON の数値の型 | `"format_version": 1.0`、`1e0`、`"warning_exit_codes": [3.0]`、`[1e1]`、`[99999999999999999999]`、`[true]` | `validate --config conductor.json` | `format_version` は 64(`unsupported format_version`)。終了コードは 64(`wrong value type`)。整数のリテラルだけを整数と読む(`Number(...)` で丸めた値を見ない) |
| AC-469 | 変換の JSON の出力の文字 | YAML: `a: "<&>"⏎b: 夜間⏎c: " x"` | `convert in.yaml --to json` | 0。OUT に `"<&>"` がそのまま(`&` にならない)と `夜間`(非 ASCII はそのまま)がある。OUT を JSON として読んだ木が入力の木と等しい(`c` の U+2028 は、` ` の書き方でも直接でもよい) |
| AC-470 | 変換の YAML の出力: 予約語に見える文字列 | JSON: `{"a": "true", "b": "1", "c": "null", "d": "@end", "e": "", "f": "~", "g": "on", "h": "1e3", "i": "0x1F", "j": "yes", "k": "-", "l": "a: b", "m": "# c"}` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 往復の木が入力と等しい(すべて文字列のまま。YAML の出力は自前の書き出しで、必要なものを引用符で囲む) |
| AC-471 | 変換の YAML の出力: 特殊な文字列 | JSON: 複数行の文字列(`"a\nb\n"`)、先頭・末尾に空白のある文字列、タブ、`"\u0007"`(制御文字)、非 ASCII、絵文字、長い文字列(1 万文字)、空のキー `""`、`@` や `%` や `` ` `` で始まるキー | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 往復の木が入力と等しい |
| AC-472 | 変換の JSON の入力の数値 | JSON: `{"a": 1, "b": 1.0, "c": 1e2, "d": -0, "e": 9223372036854775807, "f": -9223372036854775808}` | `convert in.json --to yaml` の OUT を `convert --to json` に通す | 型が変わらない(`a`・`d`・`e`・`f` は整数、`b`・`c` は浮動小数)。往復の木が入力と等しい。`e`・`f` は桁を落とさない(number に直さない) |
| AC-473 | ステートの JSON 固有の異常 | ステートが、重複するキーを持つ JSON、値のあとに余分なデータ、最上位が配列 / `null` / 文字列、`{}` | `run a` | 65(ジョブは起動されない)。余分なデータ・最上位の型の誤りは `state file is corrupt: JSON として読めません`、キーの重複は `state file is corrupt: キーが重複しています`、`{}` は `state file is corrupt: format_version がありません` |
| AC-474 | ステートの数値の型 | `exit_code` が `3.0` / `"3"` / `1e1` / `256` / `-1`、`format_version` が `1e0` | `run a` | 65。`state file is corrupt: …`(型・範囲の理由は 06 章) |
| AC-475 | JSON の `ME` の誤りの行 | `ME` を JSON で書く(4.1 節。次のとおり) | `validate --config conductor.json` | 64。ERR は 4 行。順に `…:3: unknown key`、`…:11: wrong value type`、`…:16: unknown key`、`…:20: transition target job is not defined` |
| AC-476 | 版の表示と定数 | ソース、`package.json`、実行ファイル | 点検と `--version` | `src/version.ts` の定数 `VERSION`(`export const VERSION = "0.8.0";` の 1 行)が、ソース中の版の唯一の定義。`package.json` の `version` と一致する(テストで検査。`scripts/check-version.mjs`)。`--version` の OUT = `conductor ` + `VERSION`。ビルドの指示(置換の道具、環境変数、`define`)で版を書き換えていない。区分: 点検 |
| AC-477 | Node の版の記載の一致 | `package.json`、`README.md`、`.github/workflows/ci.yml` | `node scripts/check-node-version.mjs` | `package.json` の `engines.node` の最小の主版(`>=22` なら 22)、README の `Node.js(<主版> 以降)`、`ci.yml` の `NODE_MIN` が一致する。不一致なら失敗(終了 1)。区分: 点検 |
| AC-478 | 依存の整合 | `package.json`、`package-lock.json` | `npm ci`、`npm install --package-lock-only --ignore-scripts`、`node scripts/check-deps.mjs` | `npm ci` が成功する(ロックファイルと `package.json` が一致)。`npm install --package-lock-only` のあとで `package-lock.json` に差分がない。`dependencies` が YAML ライブラリ 1 つだけ、`devDependencies` が 8 章の許可の一覧の内。区分: 点検 |
| AC-479 | ビルドできる | ソース | `npm run build`(`tsc -p tsconfig.build.json`)を、最小の Node と最新の Node で、Linux・Windows(最新の Node は macOS も) | すべて成功する。`dist/bin/conductor.js` ができ、`.ts` が出力されない。区分: 点検 |
| AC-480 | 型・整形・静的検査 | ソース | `tsc --noEmit`、`eslint . --max-warnings 0`、`prettier --check <対象>` | すべて出力が空(エラー・警告・差分なし)。区分: 点検 |
| AC-481 | 既知の脆弱性の検査 | 依存 | `npm audit --omit=dev`、`npm audit` | 結果を記録する。脆弱性が見つかっても、マージはブロックしない(警告)。区分: 点検 |
| AC-482 | 変換した定義がそのまま使える | `CV1J` を `conductor.json` に | `convert conductor.json --to yaml > conductor.yaml`、`validate`、逆向きに `convert conductor.yaml --to json > c2.json`、`validate --config c2.json` | すべて 0。展開された木が同じ |
| AC-483 | `tsc` の厳格な設定と、型情報つきの ESLint | `tsconfig.json`、`eslint.config.js` | 点検(`tsc --showConfig` の出力と、`eslint --print-config src/cli/index.ts` の出力) | `strict`、`noUncheckedIndexedAccess`、`exactOptionalPropertyTypes`、`noImplicitOverride`、`noFallthroughCasesInSwitch`、`verbatimModuleSyntax` が有効。`module` と `moduleResolution` が `NodeNext`。ESLint の `@typescript-eslint/no-floating-promises`、`no-misused-promises`、`no-explicit-any`、`no-non-null-assertion`、`consistent-type-assertions`、`only-throw-error` が `error`。区分: 点検 |
| AC-484 | 合図とジョブの終了の順序(非同期) | `src/runner/` と `src/cli/` の単体(偽の `Deps` と、本物のシグナルの登録) | 合図を、ジョブの終了の直前・直後・待機(リトライ)の直前に立てる。同じシグナルを続けて 2 回立てる。`run` / `pipeline` が終わったあとの `process.listenerCount('SIGHUP' など)` を調べる | 最初の番号だけが残る。転送は 1 回。終了済みのジョブには送らない。`run` / `pipeline` の終了後に、登録したシグナルのリスナーが外れ、数が実行前と同じに戻る(リスナーの漏れがない)。区分: 単体 |
| AC-485 | 偽の `FakeSignals` で登録に失敗すると 70 | 偽の `FakeSignals`(登録が失敗を返す) | `run a`、`pipeline p`(定義は正しい / 誤りがある) | どちらも 70。ERR = `conductor: error: failed to set up signal handling`。定義の検査(定義が誤りでも 64 にならない)もジョブの起動もしない。区分: 注入。(AC-298 を、定義が誤りの場合まで広げたもの) |
| AC-486 | Windows の `.cmd` / `.bat` の禁止文字は起動しない | Windows。`T/echo-args.cmd`(内容 `@echo off` と `exit /b 0`)。`cmd` = そのパス + 引数 1 つ。引数に `"`、`%`、`^`、`&`、`\|`、`<`、`>`、`!`、`(`、`)`、改行のどれか 1 つを含める([07](07-process-and-signals.md)。1 文字ずつ別の定義)。配列形式でも同じ | `run a` | 127(起動失敗)。ERR = `conductor: error: job 'a': cannot start command '…'`。`T/injected.txt` ができない。含まない引数(`plain`)は起動される(AC-115)。区分: 自動(Windows の CI) |
| AC-487 | Windows の Ctrl+C のチェックリストの記録 | AC-301 の記録欄 | リリース前に、Windows の実機で AC-301 を行い、結果を `test/WINDOWS_CTRL_C_CHECKLIST.md` の記録欄に書く | 記録欄に、実施日、OS の版、Node の版、(1)〜(4)の結果が書かれている。ない場合はリリースの前に実施する。区分: 手動(Windows) |

AC-475 の JSON の `ME`:

```json
{
  "format_version": 1,
  "unknown_top": true,
  "pipeline": {
    "name": "p"
  },
  "job": [
    {
      "id": "a",
      "command": "x",
      "success_exit_codes": ["zero"]
    },
    {
      "id": "b",
      "command": "y",
      "colour": "red",
      "transitions": [
        {
          "on": "success",
          "next": "nowhere"
        }
      ]
    }
  ]
}
```

JSON の定義の誤りの行は、自前の読み取りが、各キーと値の開始の位置(行・列)を記録して求める(LF の数で数える。CRLF でも同じ行番号。AC-038 と同様。[03](03-definition-format.md))。

### 4.20 Node.js に固有の項目

Node.js の実行モデル(単一のイベントループ、標準入出力の非同期の書き込み、`process.exit()` の即時の終了、未処理の例外と Promise の拒否、シグナルのリスナー、`bigint`、オブジェクトのキーの順序、ビルドの成果物)に固有の落とし穴を確かめる。

| ID | 機能 | 前提 | 実行 | 期待 |
|----|------|------|------|------|
| AC-488 | 起動時間 | ビルドした `dist/` | `conductor --version` を 20 回(起動から終了まで) | p95 が 250 ms(Linux・macOS)、500 ms(Windows)以下(仮。目安。2.1 節)。区分: 計測 |
| AC-489 | 未捕捉の例外は 70 | `P(p; a:0{cmd="TJ sleep 1"})`。`node --import <test/support/inject-throw.mjs> dist/bin/conductor.js run a` で起動する。`inject-throw.mjs` は、`setTimeout(() => { throw new Error('SECRET-INJECT'); }, 300)` を登録する(製品に試験用の経路を持たせず、外から失敗を起こす) | `run a` | 終了 70。ERR = `conductor: error: internal error`(1 行だけ。末尾の改行 1 つ)。ERR に `SECRET-INJECT`・スタック(`at ` で始まる行)が現れない。OUT 空。`process.on('uncaughtException')` のハンドラが働く(ジョブの子プロセスが残る場合の扱いは 07 章に従う) |
| AC-490 | 拒否された Promise は 70 | 同上。`inject-reject.mjs` は、`setTimeout(() => { Promise.reject(new Error('SECRET-INJECT')); }, 300)` を登録する | `run a` | 終了 70。ERR = `conductor: error: internal error`(1 行だけ)。`SECRET-INJECT`・スタックが現れない。Node の既定(`unhandledRejection` を `throw` にする)に頼らず、`process.on('unhandledRejection')` で処理する |
| AC-491 | シグナルのコールバックの中の例外は 70 | 偽の `FakeSignals`(合図のコールバックが例外を投げる)と本物のシグナル | `run a` / `pipeline p` | 70。ERR = `conductor: error: internal error`。ジョブのあとのステートの書き込みを、例外のために飛ばさない(例外が起きる前の確定したステートは残る)。区分: 注入 |
| AC-492 | 標準出力のパイプが閉じた(`EPIPE`) | 入力 `big.json`(約 1 MiB。小さな要素の配列。`--to yaml` の出力が 64 KiB を大きく超える) | `convert big.json --to yaml` を起動し、標準出力の最初のデータを受けた時点で、読み手側(`child.stdout.destroy()`)を閉じる | 終了 67。ERR に `conductor: error: cannot write to standard output`。`EPIPE`・スタックが現れない。`'error'` イベントが未処理にならず(70 にならない)、5 秒以内に終わる |
| AC-493 | 標準出力の取りこぼしがない | (a) `convert big.json --to yaml`(出力が 64 KiB を超える)。(b) `P(p; a:0{cmd="TJ big 16"})` | (a) 標準出力をパイプにし、読み手が最初の 500 ms 読まない(`child.stdout.pause()` のあと `resume()`)。(b) `run a`(ジョブが 16 MiB を、`conductor` の標準出力に継承されたパイプへ書く) | (a) 終了 0。OUT の全体が、同じ変換を標準出力をファイルにして得た結果と、バイト単位で等しい(終了前に書き込みの完了を待つ。`process.exit()` で切らない)。(b) 終了 0。OUT のバイト数がちょうど 16 MiB(`x` のみ)。`conductor` が出力を溜め込まない |
| AC-494 | 標準出力の書き込みの失敗(`/dev/full`。Linux) | Linux。`sh -c 'exec node dist/bin/conductor.js watch > /dev/full'`(ステートあり)、同様に `convert … > /dev/full` | 実行 | `watch` は 65、`convert` は 67。ERR = `conductor: error: cannot write to standard output`。70 にならない。区分: Linux |
| AC-495 | 終了後にプロセスが残らない | 各サブコマンド(`validate`、`run`、`pipeline`、`watch`、`reset`、`convert`、`--help`、`--version`、使い方の誤り) | 標準入力を開いたまま(`stdio[0] = 'pipe'` で、閉じない)実行 | 出力の完了から 1 秒以内にプロセスが終わる(標準入力の読み取り、タイマー、シグナルのリスナー、ストリームが残ってハングしない)。区分: 自動 |
| AC-496 | Node の警告が出ない | 4.1〜4.9 節の各種の実行(特に、YAML の誤り・警告を起こしうる入力、`convert`、リトライの待機) | 実行 | ERR に `(node:` で始まる行、`Warning:` を含む行(`ExperimentalWarning`、`DeprecationWarning`、`TimeoutOverflowWarning`、`MaxListenersExceededWarning`、YAML ライブラリが `process.emitWarning` で出す警告)が現れない。最小の Node と最新の Node の両方で。区分: 自動 |
| AC-497 | `process.on` の登録の点検 | ソース | 点検(構文木の走査) | `process.on` / `process.once` / `process.addListener` の呼び出しが、`src/bin/conductor.ts`(`uncaughtException` と `unhandledRejection`。それぞれ 1 回)と `src/runner/` のシグナルの部品(`platform.ts` が返す一覧のシグナルだけ。UNIX 系は `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM`、Windows は `SIGINT`)にしかない。シグナルの登録のたびに、対応する `process.off` / `removeListener` が、終了の処理(`finally`)にある。`process.on('exit')` と `beforeExit` で、非同期の処理をしない。区分: 点検 |
| AC-498 | `process.exit` の点検と、早すぎる呼び出しの検出 | ソース | 点検(構文木の走査) | `process.exit(` の呼び出しが、`src/bin/conductor.ts` に 1 か所以下で、それより前に、標準出力・標準エラーの書き込みの完了を待つ `await` がある。ほかのファイルにない。終了コードは、`process.exitCode` に設定してから終わるか、書き込みの完了を待ってから 1 か所で `process.exit` を呼ぶ。実行時の検出: AC-493(出力が欠けない)。区分: 点検 |
| AC-499 | 64 ビット符号なし整数の飽和演算(`bigint`) | `src/flow/` の単体 | 待ち時間を求める関数に、検査を通らない極端な値を直接渡す(`prev = 2n ** 63n`、`prev = 18446744073709551615n`、`multiplier = 10n`、`cap = 18446744073709551615n`)。`max` = 100、`interval = 86400n`、`multiplier = 10n`、`max_interval = 86400n` の列。待ちの秒数をタイマーのミリ秒に直す関数に 86400n と 2147483647n 以上の値 | 結果が `bigint`(`typeof` が `'bigint'`)で、`18446744073709551615n` を超えず、丸め・桁あふれ(`0n` に戻る)がない(`18446744073709551615n` のまま)。`Number()` に直して比べない。列は 86400 のまま 100 回続く。ミリ秒に直す関数は、`Number` の安全な整数の範囲に収まる値だけを返す(AC-501)。区分: 単体 |
| AC-500 | 範囲外の整数の検査は字句から `bigint` で行う | 定義(YAML・JSON): `warning_exit_codes: [9007199254740993]`(2^53 + 1)、`[2147483648]`、`[-2147483649]`、`max: 4294967296`、`max: 9007199254740993`、桁が 10,000 の整数 `format_version: 1000…0`、ステートの `exit_code: 9007199254740993` | `validate` / `run a` | すべて「範囲外・型の誤り」として拒否する(定義: 64、ステート: 65)。number に落として丸めた結果で、受理しない(`9007199254740993` を `9007199254740992` と読んで、別の判定をしない)。桁が多い入力で、ハングしない・異常終了しない。区分: 自動と単体 |
| AC-501 | タイマーの上限 | `src/runner/` または待機の部品の単体(偽の `setTimeout` の記録) | 待機を 24 時間(86,400 秒)、`max_interval` の上限で呼ぶ。続けて、検査を通らない 30 日の待機を直接呼ぶ | `setTimeout` に渡すミリ秒が、いつも 2,147,483,647 以下(超えると Node は 1 ms に直して警告する、とされる。未確認)。30 日の待機は、上限を超えない長さに分けて待つか、拒否する(07 章に従う)。`TimeoutOverflowWarning` が出ない。待機中のタイマーは、イベントループを保つ(`unref` しない)。区分: 単体 |
| AC-502 | シグナル名と番号の対応 | `src/runner/` の単体 | 子プロセスの `exit` イベントの `signal`(`'SIGKILL'`、`'SIGTERM'`、`'SIGINT'`、`'SIGHUP'`、`'SIGQUIT'` など)を番号にする関数 | `os.constants.signals` の値と一致する(UNIX 系。Linux と macOS で番号が違うシグナル(`SIGUSR1` など)も、その OS の値)。`SIGHUP`=1、`SIGINT`=2、`SIGQUIT`=3、`SIGKILL`=9、`SIGTERM`=15。`null` の `signal` と、対応表にない名前の扱いは 07 章に従う(未確認)。Windows で `exit` イベントの `signal` が返す値は、未確認。区分: 単体 |
| AC-503 | `child.kill` の戻り値で終わりを判断しない | `SL` の `sleeper` を `TJ --out O trap 500 3` にする(UNIX 系) | `run sleeper` を起動し、`ready` のあと SIGTERM | 終了 143。`O` に `trapped`、続いて `done` が記録されており、`conductor` が、ジョブの終了(`close` イベント)を待ってから終わる(`child.kill()` が `true` を返した時点で終わらない)。`conductor` の終了は、SIGTERM の送信から 500 ms 以上あと。ステート `sleeper=error`(終了コード 3 は、中断の分類により `128 + 15` が採られる。ジョブの終了コード 3 は表示に使わない)。区分: UNIX 系 |
| AC-504 | ジョブが残した孫のプロセスを待たない | `argv=["sh", "-c", "sleep 5 & exit 3"]`(UNIX 系) | `run a`(`conductor` の `exit` イベントの時刻を測る。標準出力・標準エラーのパイプは、孫が持ち続けるため、`close` は 5 秒後になることがある) | 終了コード 3。`conductor` が、ジョブ(`sh`)の終了から 2 秒以内に終わる(孫の終了を待たない)。試験は、`conductor` の `exit` イベントで判定する。区分: UNIX 系 |
| AC-505 | 入口のシバン | ビルドした `dist/bin/conductor.js` | 点検(ファイルの先頭を読む) | 先頭の行が、ちょうど `#!/usr/bin/env node`(改行は LF だけ)。`package.json` の `bin.conductor` が `dist/bin/conductor.js` を指す。ESM(`"type": "module"`)として、`node dist/bin/conductor.js` が起動できる(拡張子つきの相対 import が解決できる)。区分: 点検 |
| AC-506 | `package.json` の形 | `package.json` | 点検 | `name` = `conductor`、`version`、`type` = `module`、`bin`、`engines.node`(最小版以上を表す範囲)、`files`(`dist`、`LICENSE`、`README.md`)、`license` = `Apache-2.0`、`scripts`(9.2 節の名前)がある。`dependencies` が 1 つ(8 章)。`main` と `exports` は書かない(ライブラリとして公開しない)。区分: 点検 |
| AC-507 | 標準入力はジョブに引き継ぐ | `P(p; a:0{cmd="TJ --out O stdin"})` | `run a`(標準入力のパイプに `hello⏎` を書いて閉じる) | 終了 0。`O` = `stdin="hello\n"`。`conductor` が標準入力を読まない(`process.stdin` を読み始めない。AC-495 と合わせて、読み始めると入力を奪うため) |
| AC-508 | Windows の `.cmd` / `.bat` を、最小の Node でも最新の Node でも起動できる | Windows。`T/ok.cmd`(`exit /b 7`) | 最小の Node と最新の Node で `run a`(`cmd` = そのパス + ` plain`) | どちらも 7(AC-115)。Node は、`.cmd` / `.bat` を `shell: false` で `spawn` すると、`EINVAL` で失敗する版がある(2024 年の脆弱性の対応。どの版からか、`shell` なしで起動できるかは未確認)。**起動の方法は 07 章で決め、この項目は、その方法が両方の Node で動くことを確かめる**。区分: 自動(Windows の CI) |
| AC-509 | オブジェクトのキーの順序(整数に見えるキー) | YAML: `"2": a⏎"1": b⏎b: c`(キーは引用符つきの文字列)。JSON: `{"2": "a", "1": "b", "b": "c"}` | `convert in.yaml --to json` / `convert in.json --to yaml` | OUT のキーの順が、入力の記述順(`2`、`1`、`b`)。JavaScript の通常のオブジェクトの「整数に見えるキーを昇順に並べる」性質に影響されない。区分: 自動 |
| AC-510 | 特別な名前のキーを、普通のキーとして扱う | JSON: `{"__proto__": 1, "constructor": 2, "toString": 3, "hasOwnProperty": 4}`(変換)。定義に `__proto__: x`、`constructor: x`(キー) | `convert in.json --to yaml`、`validate` | 変換: 4 つのキーがそのまま出る(往復で木が等しい)。`Object.prototype` が汚染されない(実行の後で `({}).polluted === undefined`、`Object.prototype` のキーが増えない)。定義: `unknown key`(64)。例外で落ちない。区分: 自動と単体 |

---

## 5. CI(GitHub Actions)と品質ゲート

ワークフローは `.github/workflows/` の 3 つ: `ci.yml`、`audit.yml`、`release.yml`。GitHub Actions の部品(`actions/checkout`、`actions/setup-node`、`actions/upload-artifact`、`actions/download-artifact`)の版は、`<版>` と書いた場所を、**実装時に、その時点の最新の安定版のメジャー版で確かめて決める**(この文書では確認していない)。
Node の版は、`NODE_MIN`(最小。`package.json` の `engines.node` の最小の主版と一致させる。5.4 節)と、`lts/*`(最新の LTS 系列。`actions/setup-node` の指定。実装時に確認)で指定する。

### 5.1 `ci.yml`(名前 `CI`)

| 項目 | 内容 |
|------|------|
| 起動 | `push`(`main`)、`pull_request`(対象 `main`)、`workflow_call`(リリースから呼ばれる) |
| 権限 | `contents: read` |
| 同時実行 | `group: ci-${{ github.workflow }}-${{ github.ref }}`、`cancel-in-progress: true` |
| 環境変数 | `NODE_MIN: "22"`(最小の Node の主版。例。`package.json` の `engines.node` と一致させる。8.3 節、5.4 節) |
| チェックアウト | `actions/checkout@<版>` |
| Node の用意 | `actions/setup-node@<版>`。「最新」は `node-version: lts/*`、「最小」は `node-version: ${{ env.NODE_MIN }}`。`cache: npm`(`package-lock.json` を鍵にした npm のキャッシュ。実装時に確認) |

| ジョブ | 目的 | OS | Node | コマンド | 失敗の条件 |
|--------|------|----|------|----------|------------|
| `lint` | 型・静的検査・整形 | `ubuntu-24.04`(10 分) | 最新 | `npm ci` → ① `npx tsc --noEmit`(`npm run typecheck`)② `npx eslint . --max-warnings 0`(`npm run lint`)③ `npx prettier --check <対象>`(`npm run format:check`) | 型のエラー、ESLint の警告・エラーが 1 つでも、整形の差分が 1 つでもある(AC-480) |
| `consistency` | 依存と版の記載の整合 | `ubuntu-24.04`(5 分) | 最新 | ① `node scripts/check-version.mjs`(`src/version.ts` と `package.json` と `package-lock.json` の版の一致)② `node scripts/check-node-version.mjs`(5.4 節)③ `node scripts/check-deps.mjs`(8 章)④ `npm ci`(ロックファイルと `package.json` の一致)⑤ `npm install --package-lock-only --ignore-scripts` のあと `git diff --exit-code package-lock.json` | 版の不一致、依存の許可の違反、ロックファイルの食い違い・整理の差分(AC-476〜478) |
| `build-check` | 最小の Node で、ビルドでき、配布物の形が正しいこと | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。15 分) | 最小 | `npm ci` → `npm run build` → `node dist/bin/conductor.js --version` → `npm run build` のあと `node scripts/check-pack.mjs`(`npm pack --dry-run --json` のファイル一覧の検査。5.4 節) | ビルドの失敗、`--version` の失敗、配布物の中身の違反(AC-422、AC-440、AC-479、AC-505) |
| `test` | 3 つの OS で、全テスト(単体と受け入れ)が通ること | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。15 分) | 最新 | `npm ci` → `npm test`(`vitest run`。`globalSetup` が `tsc` でビルドする) | テストの失敗(30 秒の上限を含む) |
| `min-node-test` | 最小の Node でも、ビルドだけでなくテストが通ること | `ubuntu-24.04`、`windows-2022`(`fail-fast: false`。15 分) | 最小 | `npm ci` → `npm test` | テストの失敗。Windows で `.cmd` を最小の Node で起動できること(AC-508)を含む |
| `coverage` | 行カバレッジ 80% の維持 | `ubuntu-24.04`(20 分) | 最新 | 5.3 節のコマンド | カバレッジが 80% 未満 |
| `docs-check` | README に必要な語が残っていること | `ubuntu-24.04`(5 分) | 最新 | `node scripts/check-readme.mjs`(5.4 節。8 つの文字列) | 1 つでも欠けると `README.md is missing: <語>` を出して失敗 |
| `startup-bench` | 起動と性能の測定(2.1 節) | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: false`。20 分) | 最新 | `npm ci` → `npm run build` → `npm run test:perf`(環境変数 `STARTUP_BENCH_ENFORCE: "false"`)。測定値は `GITHUB_STEP_SUMMARY` に出る | `continue-on-error: true` のため、ジョブが失敗しても全体は失敗にならない。基準超えは警告のみ(強制するには環境変数を `"true"` にする) |

任意のジョブ(置くかどうかは、実装時に決める。ブランチ保護の必須にはしない): `current-node-test`(`node-version: node`。最新の「現行」系列の Node で `npm test`。`continue-on-error: true`)、`build-repro`(`npm run build` を 2 回行い、`dist/` のファイルの SHA-256 の一覧が一致することを確かめる。`tsc` の出力が再現可能か、3 OS で同じ結果になるかは未確認)。`dist/` はコミットしない(`.gitignore`)ので、「再生成して差分が出ないこと」の検査は、`dist/` をコミットする場合だけ意味を持つ。

`ci.yml` の骨組み(`<版>` は実装時に確認):

```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
  workflow_call:
permissions:
  contents: read
concurrency:
  group: ci-${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
env:
  NODE_MIN: "22"
jobs:
  lint:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*", cache: npm }
      - run: npm ci
      - run: npm run typecheck
      - run: npm run lint
      - run: npm run format:check
  consistency:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*", cache: npm }
      - run: node scripts/check-version.mjs
      - run: node scripts/check-node-version.mjs
      - run: node scripts/check-deps.mjs
      - run: npm ci
      - run: npm install --package-lock-only --ignore-scripts
      - run: git diff --exit-code package-lock.json
  build-check:
    name: build-check (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "${{ env.NODE_MIN }}", cache: npm }
      - run: npm ci
      - run: npm run build
      - run: node dist/bin/conductor.js --version
      - run: node scripts/check-pack.mjs
  test:
    name: test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*", cache: npm }
      - run: npm ci
      - run: npm test
  min-node-test:
    name: min-node-test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 15
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2022]
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "${{ env.NODE_MIN }}", cache: npm }
      - run: npm ci
      - run: npm test
  coverage:
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*", cache: npm }
      - run: npm ci
      - run: npm run test:coverage
      - run: node scripts/check-coverage.mjs
  docs-check:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*" }
      - run: node scripts/check-readme.mjs
  startup-bench:
    name: startup-bench (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    continue-on-error: true
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, macos-latest, windows-2022]
    env:
      STARTUP_BENCH_ENFORCE: "false"
    steps:
      - uses: actions/checkout@<版>
      - uses: actions/setup-node@<版>
        with: { node-version: "lts/*", cache: npm }
      - run: npm ci
      - run: npm run build
      - run: npm run test:perf
```

### 5.2 `audit.yml`(名前 `Audit`)

| 項目 | 内容 |
|------|------|
| 目的 | 依存の既知の脆弱性の検査(`npm audit`)。結果は警告として出し、マージはブロックしない |
| 起動 | `pull_request`(対象 `main`)、`schedule`(cron `0 3 * * 1` = 毎週月曜 03:00 UTC) |
| 権限 | `contents: read` |
| ジョブ `audit`(名前 `npm audit`) | `ubuntu-24.04`、10 分、**`continue-on-error: true`**。`actions/checkout@<版>` → `actions/setup-node@<版>`(`node-version: lts/*`)→ `npm audit --omit=dev`(実行時の依存だけ)→ `npm audit`(開発時の依存を含む。2 つ目は、1 つ目が失敗しても行うよう、別のステップに分け、`continue-on-error: true` を付ける) |
| 失敗の条件 | `npm audit` が、脆弱性を報告すると(既定では、重大度が低以上のもの。`--audit-level` で変えられる)、非 0 で終わり、ジョブは失敗するが、`continue-on-error` のため、ワークフロー全体は失敗にならない(終了コードの規則と `--audit-level` の既定は、npm の版による。実装時に確認) |

`npm audit` は、npm のレジストリへ問い合わせる(ネットワークを使う)。これは CI の検査であり、`conductor` の実行時の挙動ではない(AC-419 の対象は製品のコード)。

### 5.3 カバレッジ

| 項目 | 内容 |
|------|------|
| 計測 | `npx vitest run --coverage`(`npm run test:coverage`)。`@vitest/coverage-v8` が、`src/` の `.ts` の行の網羅率を測る(`vitest.config.ts` の `coverage`。3.9 節)。実行ファイルを `child_process` で起動する受け入れテストの中のコードは、数えない |
| 除外 | `test/`、`src/bin/`(入口)、`*.d.ts`。`include` を `src/**/*.ts` にして、`test/support/` が対象に入らないようにする |
| 基準 | 行の網羅率が **80.0% 未満なら失敗**。`vitest.config.ts` の `thresholds.lines: 80` が失敗(終了コード 1)にし、加えて `scripts/check-coverage.mjs` が、`coverage/coverage-summary.json` の `total.lines.pct` を独立に検査する(設定の書式が版で変わるため) |
| 実行する OS | `ubuntu-24.04` のみ。したがって、Windows 専用のコード(`platform.ts` の Windows の分岐)は、計測の対象に入っても、通らない行になる。この分は 80% の内で吸収できる大きさに留める(OS 別の分岐を、薄く 1 か所に集める。2.4 節) |
| 範囲 | 単体テストと、`Deps` を差し替えた流れのテストの全体 |

```sh
npx vitest run --coverage
node scripts/check-coverage.mjs
```

### 5.4 検査スクリプトの全文

すべて、依存を持たない素の Node のスクリプト(ESM。`node:` の組み込みモジュールだけ)で、3 OS で動く。各ファイルの先頭に、Apache License 2.0 のヘッダ(AC-423)を置く(ここでは省略)。

`scripts/check-version.mjs` — 版の一致(AC-476)。引数にタグの版(`v` なし)があれば、それとも比べる。

```js
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
const lock = JSON.parse(readFileSync('package-lock.json', 'utf8'));
const src = readFileSync('src/version.ts', 'utf8');
const m = /^export const VERSION = "([^"]+)";$/m.exec(src);
if (!m) {
  console.error('src/version.ts: export const VERSION = "..."; not found');
  process.exit(1);
}
const versions = {
  'src/version.ts': m[1],
  'package.json': pkg.version,
  'package-lock.json': lock.version,
  'package-lock.json packages[""]': lock.packages?.['']?.version,
};
const expected = process.argv[2] ?? pkg.version;
let failed = false;
for (const [where, v] of Object.entries(versions)) {
  if (v !== expected) {
    console.error(`${where}: ${v} != ${expected}`);
    failed = true;
  }
}
process.exit(failed ? 1 : 0);
```

`scripts/check-node-version.mjs` — Node の最小版の記載の一致(AC-477)。

```js
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
const range = pkg.engines?.node ?? '';
const m = /^>=\s*(\d+)(?:\.\d+){0,2}$/.exec(range.trim());
if (!m) {
  console.error(`package.json engines.node must look like ">=22": ${range}`);
  process.exit(1);
}
const min = m[1];
let failed = false;
const readme = readFileSync('README.md', 'utf8');
if (!readme.includes(`Node.js(${min} 以降)`)) {
  console.error(`README.md: 'Node.js(${min} 以降)' not found`);
  failed = true;
}
const ci = readFileSync('.github/workflows/ci.yml', 'utf8');
const c = /^ {2}NODE_MIN:\s*"?(\d+)"?\s*$/m.exec(ci);
if (!c || c[1] !== min) {
  console.error(`ci.yml NODE_MIN (${c ? c[1] : 'missing'}) != engines.node (${min})`);
  failed = true;
}
process.exit(failed ? 1 : 0);
```

`scripts/check-deps.mjs` — 依存の許可の一覧(8 章。AC-478)。

```js
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync('package.json', 'utf8'));
const RUNTIME_ALLOWED = new Set(['yaml']); // 8.2 節で採った YAML ライブラリの名前に合わせる
const DEV_ALLOWED = new Set([
  'typescript',
  'eslint',
  '@eslint/js',
  'typescript-eslint',
  'prettier',
  'vitest',
  '@vitest/coverage-v8',
  '@types/node',
]);
let failed = false;
const runtime = Object.keys(pkg.dependencies ?? {});
if (runtime.length !== 1 || !RUNTIME_ALLOWED.has(runtime[0])) {
  console.error(`dependencies must be exactly one allowed package: ${runtime.join(', ')}`);
  failed = true;
}
for (const name of Object.keys(pkg.devDependencies ?? {})) {
  if (!DEV_ALLOWED.has(name)) {
    console.error(`devDependencies: ${name} is not allowed`);
    failed = true;
  }
}
for (const key of ['optionalDependencies', 'peerDependencies', 'bundledDependencies']) {
  if (pkg[key] && Object.keys(pkg[key]).length > 0) {
    console.error(`${key} must be empty`);
    failed = true;
  }
}
process.exit(failed ? 1 : 0);
```

`scripts/check-readme.mjs` — README の必須の語(`docs-check`)。

```js
import { readFileSync } from 'node:fs';

const readme = readFileSync('README.md', 'utf8');
const words = [
  'SHA-256',
  '終了コードの表',
  'ステートファイルの場所',
  'at-least-once',
  'シグナル',
  'convert',
  'Node.js(',
  'npm install',
];
const missing = words.filter((w) => !readme.includes(w));
for (const w of missing) console.error(`README.md is missing: ${w}`);
process.exit(missing.length > 0 ? 1 : 0);
```

`scripts/check-coverage.mjs` — カバレッジの独立の検査(5.3 節)。

```js
import { readFileSync } from 'node:fs';

const summary = JSON.parse(readFileSync('coverage/coverage-summary.json', 'utf8'));
const pct = summary.total.lines.pct;
console.log(`total line coverage: ${pct}%`);
if (typeof pct !== 'number' || pct < 80) {
  console.error('line coverage must be at least 80%');
  process.exit(1);
}
```

`scripts/check-pack.mjs` — `npm pack` の中身の検査(AC-422、AC-440)。`npm pack --dry-run --json` の出力の書式(配列の最初の要素の `files[].path`)は、npm の版による(実装時に確認)。

```js
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

// 開発用のスクリプト。Windows の npm は .cmd なので、shell を使う(製品の起動では使わない)。
const out = execFileSync('npm', ['pack', '--dry-run', '--json', '--ignore-scripts'], {
  encoding: 'utf8',
  shell: process.platform === 'win32',
});
const files = JSON.parse(out)[0].files.map((f) => f.path);
const allowed = (p) =>
  p === 'package.json' ||
  p === 'LICENSE' ||
  p === 'README.md' ||
  (p.startsWith('dist/') && p.endsWith('.js'));
const bad = files.filter((p) => !allowed(p));
let failed = false;
if (bad.length > 0) {
  console.error(`unexpected files in the package: ${bad.join(', ')}`);
  failed = true;
}
if (!files.includes('dist/bin/conductor.js')) {
  console.error('dist/bin/conductor.js is missing');
  failed = true;
}
const head = readFileSync('dist/bin/conductor.js', 'utf8').split('\n', 1)[0];
if (head !== '#!/usr/bin/env node') {
  console.error(`dist/bin/conductor.js: bad shebang: ${JSON.stringify(head)}`);
  failed = true;
}
process.exit(failed ? 1 : 0);
```

`scripts/clean.mjs` — `npm run clean`(引数なし: `dist/` と `coverage/`)と、`npm run build`(引数 `dist`)が使う。

```js
import { rmSync } from 'node:fs';

const dirs = process.argv.length > 2 ? process.argv.slice(2) : ['dist', 'coverage'];
for (const dir of dirs) rmSync(dir, { recursive: true, force: true });
```

最小の Node を上げるときは、`package.json` の `engines.node`、README の `Node.js(<版> 以降)`、`ci.yml` の `NODE_MIN` を、同じコミットで更新する(`@types/node` の主版も、最小の Node に合わせる。8 章)。

### 5.5 品質ゲートの再現手順(ローカル)

マージの前に、次をすべて通す。

```sh
# 依存(ロックファイルのとおり)
npm ci

# 型・静的検査・整形
npm run typecheck                              # tsc --noEmit
npm run lint                                   # eslint . --max-warnings 0
npm run format:check                           # prettier --check <対象>

# 整合
node scripts/check-version.mjs
node scripts/check-node-version.mjs
node scripts/check-deps.mjs
node scripts/check-readme.mjs
npm install --package-lock-only --ignore-scripts && git diff --exit-code package-lock.json

# ビルドと配布物の形
npm run build                                  # tsc -p tsconfig.build.json
node scripts/check-pack.mjs

# テスト
npm test                                       # vitest run(単体 + 受け入れ)

# カバレッジ(Linux)
npm run test:coverage && node scripts/check-coverage.mjs

# 既知の脆弱性(警告のみ。マージはブロックしない)
npm audit --omit=dev; npm audit
```

加えて、CI で確かめる: Windows(`windows-2022`)と macOS(`macos-latest`)での `npm test`、最小の Node でのビルドとテスト(Linux・Windows)、README の必須の語、Node の版の記載の一致、起動の性能の測定(警告のみ)。
リリース前に、Windows の実機で Ctrl+C のチェックリスト(AC-301)を確かめ、結果を記録する。

ブランチ保護(どのジョブを必須にするか)の設定は、リポジトリの外にあり、未確認。想定は、`lint`・`consistency`・`build-check`・`test`・`min-node-test`・`coverage`・`docs-check` を必須にする。

---
## 6. リリース

### 6.1 配布の形

`conductor` は、npm のパッケージ(tarball。`npm pack` の出力)として配る。利用者は、Node.js(`engines` の最小版以上)を用意し、`npm install` で入れる。実行時の依存は YAML ライブラリ 1 つだけ(8 章)。OS ごとの実行ファイルは作らない(Node があれば、3 OS で同じ成果物が動く)。

| 項目 | 内容 |
|------|------|
| 成果物 | `conductor-<版>.tgz`(`<版>` は `v` なし。例: `conductor-0.8.0.tgz`。パッケージ名を置き換えた場合は、その名前の tarball)と、`SHA256SUMS` |
| 中身 | 展開すると最上位に `package/` があり、その中に `package.json`、`LICENSE`、`README.md`、`dist/`(`dist/bin/conductor.js` を含む `.js`)だけがある。`src/`、`test/`、`scripts/`、設定ファイル、テスト用ジョブを含めない(`package.json` の `files` で絞る。AC-422、AC-440) |
| 対応 OS | Linux、macOS、Windows(Node が動く環境) |
| チェックサム | 成果物の SHA-256 を、`sha256sum` の形式で `SHA256SUMS` に書き、一緒に配る。署名はしない(転送の破損の検知まで。配布元の改ざんは防げない) |
| リリースの注記 | 自動生成に、固定の 1 文を加える(`この成果物は npm のパッケージです。npm への公開は人が手で行います。SHA256SUMS で、ダウンロードしたファイルを確かめてください。`) |
| 公開 | GitHub のリリースは、**下書き(draft)として作り**、人が確認して公開する(自動では公開しない)。版に `-` を含む(例: `0.9.0-rc.1`)ものは、プレリリースにする |
| npm への公開 | **人が手で行う**(`npm publish conductor-<版>.tgz`)。ワークフローは、npm のトークンを持たず、公開をしない。公開の手順(二要素認証、`--access`、パッケージ名の取得)の細部は、npm の版と運用による(未確認) |

### 6.2 ビルドの方法(決め)

```sh
npm ci
npm run build        # node scripts/clean.mjs dist && tsc -p tsconfig.build.json
npm pack             # prepack が npm run build を再び実行する
```

| 項目 | 決め | 理由 |
|------|------|------|
| ビルド | `tsc -p tsconfig.build.json`(`src/` だけ。`rootDir: src`、`outDir: dist`) | 束ねる道具(バンドラ)を入れず、実行時の依存を増やさないため。`dist/bin/conductor.js` が入口 |
| 入口のシバン | `src/bin/conductor.ts` の先頭の `#!/usr/bin/env node`(`tsc` は、先頭のシバンを出力にそのまま残す。実装時に確認)。LF だけ | `npm` が、`bin` のファイルを実行可能にして、シムを作る(AC-505) |
| モジュール | ESM(`"type": "module"`、`module`・`moduleResolution` は `NodeNext`)。相対 import は `.js` の拡張子を付ける | Node が、そのまま読み込めるため |
| `target` | `ES2023`(最小の Node がサポートする版に合わせる。実装時に確認) | `bigint`、`Array.prototype.toSorted` など |
| ソースマップ・型宣言 | 出力しない(`sourceMap: false`、`declaration: false`) | ライブラリとして公開しないため。スタックを利用者に見せない(2.3 節) |
| 版の埋め込み | **しない** | 版は、`src/version.ts` の定数 `VERSION` の 1 つだけ。`package.json` の `version` との一致は、ビルドの前に検査する(AC-476、6.3 節) |
| 使う Node | ビルドとリリースは、最新の LTS 系列(`lts/*`)。利用者の下限は `engines.node` | 最新の道具の修正を取り込むため |

### 6.3 リリースの手順と `release.yml`

人の操作:

1. `src/version.ts` の `VERSION`、`package.json` の `version`、`package-lock.json` の版(`npm version <版> --no-git-tag-version` が、`package.json` と `package-lock.json` を更新する。実装時に確認)を、新しい版に上げる(README の例も)。`main` に取り込む。
2. `main` のそのコミットに、タグ `v<版>`(例: `v0.8.0`)を付けて push する。タグのコミットが `main` の祖先であること。タグの `v` を除いた文字列が、`VERSION` と `package.json` の `version` と**完全に一致**すること。
3. ワークフローが、下書き(draft)のリリースを作る。人が内容を確認して、公開する(自動では公開しない)。
4. 人が、リリース前に Windows の実機で Ctrl+C のチェックリスト(AC-301、AC-487)を確かめる。
5. 人が、`npm publish conductor-<版>.tgz` で npm に公開する(任意。ワークフローは行わない)。

起動: `push` の `tags: ["v*"]`。権限は、ジョブごとに必要なものだけ(`draft-release` だけ `contents: write`)。

| ジョブ | 依存 | OS | 内容 | 失敗の条件 |
|--------|------|----|------|------------|
| `verify-tag` | なし | `ubuntu-24.04`(5 分) | `actions/checkout@<版>`(`fetch-depth: 0`)、`actions/setup-node@<版>`(`lts/*`)。bash: タグ名 `${GITHUB_REF_NAME}` から `v` を除いて `version`。① `git merge-base --is-ancestor "$GITHUB_SHA" origin/main`(タグのコミットが `main` の祖先)② `node scripts/check-version.mjs "$version"`(`src/version.ts`、`package.json`、`package-lock.json` の版が `version` と等しい)。出力: `version`、`prerelease`(`version` に `-` を含めば `true`、含まなければ `false`) | タグが `main` 上にない(`tag is not on main`)、または版の不一致(`<場所>: <版> != <タグの版>`) |
| `checks` | `verify-tag` | — | `uses: ./.github/workflows/ci.yml`(5.1 節の全ジョブを再利用。マージの関門と同じ検査) | `startup-bench` は `continue-on-error` のため影響しない。それ以外のジョブの失敗で、以降が止まる |
| `pack` | `verify-tag`、`checks` | `ubuntu-24.04`(15 分) | `actions/checkout@<版>`、`actions/setup-node@<版>`(`lts/*`、`cache: npm`)、`npm ci`、`npm run build`、`npm pack --pack-destination out`(`--pack-destination` の指定は、npm の版により異なる可能性がある。実装時に確認)、`node scripts/check-pack.mjs`、`actions/upload-artifact@<版>`(名前 `tarball`、パス `out/*.tgz`、`if-no-files-found: error`) | ビルド・パッケージ・検査・アップロードのどれかの失敗 |
| `accept-dist` | `verify-tag`、`pack` | `ubuntu-24.04`、`macos-latest`、`windows-2022`(`fail-fast: true`。20 分) | 下記の手順(配布物を入れて、版の表示と受け入れテストを行う) | `--version` の不一致、受け入れテストの失敗(AC-442、AC-446) |
| `draft-release` | `verify-tag`、`accept-dist` | `ubuntu-24.04`(10 分。`permissions: contents: write`) | `actions/download-artifact@<版>`(名前 `tarball`、`path: dist-release`)。`cd dist-release && sha256sum *.tgz > SHA256SUMS`。`gh release create "$TAG" dist-release/* --repo "$GITHUB_REPOSITORY" --title "$TAG" --draft --generate-notes [--prerelease] --notes "<注記>"`(トークンは `github.token`) | `gh` の失敗 |

`verify-tag` の検査(bash):

```sh
version="${GITHUB_REF_NAME#v}"
git merge-base --is-ancestor "$GITHUB_SHA" origin/main || { echo "tag is not on main"; exit 1; }
node scripts/check-version.mjs "$version" || exit 1
case "$version" in *-*) echo "prerelease=true" >> "$GITHUB_OUTPUT" ;; *) echo "prerelease=false" >> "$GITHUB_OUTPUT" ;; esac
echo "version=$version" >> "$GITHUB_OUTPUT"
```

`accept-dist` の手順(3 OS。bash(`shell: bash`)で書く。`${{ runner.temp }}` を使い、Windows でも Node が解釈できる形のパスにする):

```yaml
- uses: actions/checkout@<版>
- uses: actions/setup-node@<版>
  with: { node-version: "lts/*", cache: npm }
- run: npm ci
- uses: actions/download-artifact@<版>
  with: { name: tarball, path: "${{ runner.temp }}/tarball" }
- name: install the package into an empty directory
  shell: bash
  run: |
    mkdir "${{ runner.temp }}/dist-test"
    npm install --prefix "${{ runner.temp }}/dist-test" "${{ runner.temp }}/tarball/conductor-${{ needs.verify-tag.outputs.version }}.tgz"
- name: version
  shell: bash
  run: |
    out=$(node "${{ runner.temp }}/dist-test/node_modules/conductor/dist/bin/conductor.js" --version)
    [ "$out" = "conductor ${{ needs.verify-tag.outputs.version }}" ] || { echo "unexpected: $out"; exit 1; }
- name: acceptance tests against the installed package
  env:
    CONDUCTOR_BIN: ${{ runner.temp }}/dist-test/node_modules/conductor/dist/bin/conductor.js
  run: npx vitest run test/acceptance
```

(`npm install <tarball>` は、パッケージの依存(YAML ライブラリ)をレジストリから取得する。このため、この手順はネットワークを使う。)
受け入れテストの `CONDUCTOR_BIN` の扱いは、3.3 節。デモ(4.17 節)は、リポジトリの作業ツリーを使うので、この手順では行わない。

### 6.4 単一の実行ファイルの配布(任意)

Node の「単一の実行可能アプリケーション」の機能(実行ファイルに、JavaScript の入口を埋め込む。実装時点の Node の版による)などで、OS ごとの単一の実行ファイルを作ることは、**任意**とする。この仕様の受け入れ基準には含めない。確認できていない点:

- 埋め込める入口の書式(ESM の入口をそのまま埋め込めるか、CommonJS の 1 ファイルに束ねる必要があるか)。束ねる場合の道具は、開発時の依存として別に選定する(8 章の許可の一覧に加える)。
- 機能の安定度(実験的か)、埋め込みに使う道具(`postject` など)の要否、`node` 本体のコピーを元にすること。
- 実行ファイルの大きさ(Node 本体を含むので、数十 MB になる見込み。未確認)。
- macOS の署名・公証(署名の無い実行ファイルが Gatekeeper に止められる可能性)、Windows の署名。
- 埋め込んだ実行ファイルで、`process.execPath`、子プロセスの起動、シグナル、`--import` による注入(AC-489)が、通常の `node` と同じに動くか。
- 作った場合は、`CONDUCTOR_BIN` にその実行ファイルを渡して、受け入れテスト(AC-446)を通すこと。通らない項目は、結果を記録する。

---

## 7. デモ

### 7.1 構成

デモは、`conductor` の主な動きを、1 本のスクリプトで順に見せる。利用者は、リポジトリの根から実行する。スクリプトは、一時ディレクトリに定義とジョブを写して実行し、終了時にそのディレクトリを消す(リポジトリには何も残らない)。デモの定義は YAML。

```text
demo/README.md                 使い方と各場面の説明
demo/conductor.yaml            UNIX 系の定義(パイプライン nightly)
demo/retry.yaml                UNIX 系のリトライの定義(パイプライン retry-demo)。場面 10 だけが使う
demo/run-demo.sh               UNIX 系のスクリプト(シェル)
demo/scripts/{extract,transform,load,notify,flaky}.sh   UNIX 系のジョブ
demo/windows/conductor.yaml, retry.yaml, run-demo.cmd   Windows 版(コマンドプロンプトのバッチ)
demo/windows/scripts/{extract,transform,load,notify,flaky}.cmd   Windows 版のジョブ
```

### 7.2 実行の仕方(UNIX 系と Windows の差)

| 項目 | UNIX 系 | Windows |
|------|---------|---------|
| 実行 | `demo/run-demo.sh`(リポジトリの根から) | `demo\windows\run-demo.cmd` |
| 実行ファイルの選択 | 環境変数 `CONDUCTOR`(`dist/bin/conductor.js` などの `.js` / `.mjs` のパスなら `node` で、それ以外のパスは直接実行する)。なければ `dist/bin/conductor.js`。それも無ければ、`node_modules/` があれば `npm run build` で作る。`node_modules/` も無ければ、`npm ci` と `npm run build` を先に行うよう案内して、終了コード 1 で終わる(スクリプトは、依存を自動では入れない)。`dist/` は `.gitignore` に入れる | 同じ(`CONDUCTOR`、なければ `dist\bin\conductor.js`) |
| 表示の言語 | `DEMO_LANG`(既定 `ja_JP.UTF-8`。英語は `en_US.UTF-8`)を `LANG` に設定。`LC_ALL`、`LC_MESSAGES` は外す | `DEMO_LANG`(既定 `ja_JP.UTF-8`。`en_US` のようにも書ける)。`LC_ALL`、`LC_MESSAGES` は空にする。コードページを実行中だけ 65001(UTF-8)にして、終わりに戻す |
| 一時停止 | `DEMO_PAUSE=1` のときだけ、各場面のあとで Enter を待つ | 同じ |
| ジョブの待ち時間 | `DEMO_SLEEP`(既定 0.5 秒。小数可) | `DEMO_SLEEP`(整数の秒。既定 1。`0` で待たない。小数は使えない) |
| ジョブの終了コード | 環境変数 `DEMO_<ジョブ名の大文字>_EXIT`(既定 0) | 同じ |
| 作業場所 | `mktemp -d` の一時ディレクトリに定義とスクリプトを写して実行。終了時に `trap` で消す | `%TEMP%` の下の一時ディレクトリ(`conductor-demo-<乱数>`)に写して実行し、終わりに消す |
| 事前の環境変数の処理 | `DEMO_RUN_ID`、`CONDUCTOR_RUN_ID` を消す(利用者の環境の影響を除く)。`NODE_OPTIONS` も消す | 同じ(空にする) |
| 色 | 見出し・コマンド・終了コードの行に ANSI の色。`watch` も色あり(標準出力が端末のとき) | 画面に色なし |
| 各コマンドの出力 | `$ conductor <引数>` の行、実行、`→ 終了コード <N>` | 同じ |
| ステートの一覧 | `ls conductor.yaml*.state`(なければ `(なし)`) | `dir /b conductor.yaml*.state`(なければ `(なし)`) |
| 場面の見出し | `=== <番号>. <題> ===` | 同じ |

### 7.3 定義とジョブ

`demo/conductor.yaml`(Windows 版は、`command` が `scripts\<名>.cmd`、`notify` は `[cmd, /c, 'scripts\notify.cmd']`。一重引用符の中の `\` は、そのまま `\`。**`.cmd` を直接起動する方法は 07 章に従う**。AC-508):

```yaml
format_version: 1

pipeline:
  name: nightly
  description: デモ用の夜間バッチ
  run_id_env: DEMO_RUN_ID

job:
  - id: extract
    command: sh scripts/extract.sh
    warning_exit_codes: [1]
    transitions:
      - on: warning
        next: notify
  - id: transform
    command: sh scripts/transform.sh
  - id: load
    command: sh scripts/load.sh
  - id: notify
    command: [sh, scripts/notify.sh]
```

`demo/retry.yaml`:

```yaml
format_version: 1

pipeline:
  name: retry-demo
  description: リトライのデモ

job:
  - id: flaky
    command: sh scripts/flaky.sh
    retry:
      exit_codes: [75]
      max: 3
      interval: 1s
```

ジョブのスクリプトの仕様(`extract`・`transform`・`load`・`notify`): `  [<名>] 実行中...` を表示する → `DEMO_SLEEP` の秒だけ待つ → `  [<名>] 終了コード <N> で終わります` を表示する → 環境変数 `DEMO_<名の大文字>_EXIT`(既定 0)の値 N で終わる。
`flaky` の仕様: カレントディレクトリの `flaky.count` の回数を 1 増やす(なければ 0 から)→ `  [flaky] <n> 回目の実行...` を表示する → 待つ → `n` が `DEMO_FLAKY_FAILS`(既定 2)以下なら `  [flaky] 終了コード 75 で終わります(一時的な失敗)` を表示して 75 で終わる。超えたら `  [flaky] 終了コード 0 で終わります` を表示して 0 で終わる。
(Windows 版のジョブは、待ちに `ping` を使うなど、実現の仕方が違ってよい。表示の文言は同じ。デモのジョブは、シェルのスクリプトであり、Node を使わない。`conductor` 自身だけが Node で動く。)

### 7.4 場面

| 場面 | 見せること | 実行するコマンド(UNIX 系。Windows も同じ引数) |
|------|-----------|--------------------------------------------------|
| 0 | 定義の中身 | `cat conductor.yaml`(Windows は `type`) |
| 1 | 全部成功。書いた順に 1 つずつ | `pipeline -v nightly` |
| 2 | `extract` が警告(`DEMO_EXTRACT_EXIT=1`)→ `transform`・`load` をスキップして `notify` へ | `pipeline -v nightly` |
| 3 | `load` のエラー(`DEMO_LOAD_EXIT=2`)で止まり、終了コード 2。止まった位置 | `pipeline -v nightly`、続けて `watch` |
| 4 | `load` から再開(`transform` は再実行しない)。完了してもステートは残り、`watch` が完了の行を出す。`--remove-on-complete` で完了時にステートを消す | `pipeline -v nightly`、`watch`、ステートの一覧、`pipeline -v --remove-on-complete nightly`、一覧、`watch` |
| 5 | 個別実行と初期化 | `run -v extract`、`reset` |
| 6 | YAML から JSON への変換 | `convert conductor.yaml --to json` |
| 7 | 検査。正しい定義は 0(`definition is valid`)、引用符が閉じていない定義は 64(`command has an unclosed quote`)。ジョブは実行されず、ステートも作られない | `validate -v`、誤りの定義 `bad.yaml`(`command: "sh -c 'echo hi"`)を作って `validate --config bad.yaml`、一覧 |
| 8 | run ID ごとに別のステートファイル(`conductor.yaml.<run ID>.state`)。同じ run ID で再開。完了後もステートは残り、`reset --run-id` で片づける | `pipeline -v --run-id build-1 nightly`(`load` で失敗)、環境変数 `DEMO_RUN_ID=build-2` で `pipeline -v nightly`、一覧、`watch --run-id build-1`、`pipeline -v --run-id build-1 nightly`(再開)、一覧、`watch --run-id build-1`、`reset --run-id build-1`、`reset --run-id build-2`、一覧 |
| 9 | `--run-id` は環境変数に勝つ。`--run-id ""` は run ID なしに固定。`reset --run-id` は 1 つ、`reset --all` は全部。最後の `watch` は `no state` | `DEMO_RUN_ID=env-1` で `pipeline`(`load` で失敗)、`DEMO_RUN_ID=env-1` と `--run-id opt-1`、`--run-id ""`、`--run-id opt-2`、一覧、`reset --run-id opt-1`、一覧、`reset --all`、一覧、`watch` |
| 10 | `flaky` が 2 回、終了コード 75 で失敗し、`conductor` は 1 秒待つたびに標準エラーへ `conductor: retry: job 'flaky' exited with code 75; retry <n> of 3 in 1 seconds` を出して実行し直し、3 回目で成功(`job=flaky result=success`) | `rm -f flaky.count`(Windows は `del`)、`pipeline -v --config retry.yaml retry-demo` |

最後に `デモは以上です。`(Windows 版も同じ)を出す。デモの受け入れテストは 4.17 節。

---

## 8. 依存ライブラリと `package.json` / ロックファイルの運用

### 8.1 方針

- 依存は最小にする。**実行時の外部ライブラリは、YAML の読み取り 1 つだけ**を認める。ほかは、`node:` の組み込みモジュールを使う(引数の解析は自前、シグナルは `process.on`、プロセスは `node:child_process`、ファイルは `node:fs`、パスは `node:path`、JSON の読み取りは自前)。YAML の書き出し(`convert --to yaml`)も自前([09](09-converter.md))。
- 実行時の依存に、ネイティブのアドオン(コンパイルを要するもの)と、ネットワークのライブラリを持たない(AC-419、AC-443)。依存のインストールスクリプト(`install`・`postinstall`)に頼らない。
- 開発時の依存は、TypeScript、ESLint(と `typescript-eslint`)、Prettier、Vitest(と `@vitest/coverage-v8`)、`@types/node` に限る(8.3 節)。テスト用のアサーション・モック・擬似端末のライブラリは入れない(擬似端末が要る項目 AC-182・AC-183 は、10 章)。
- `node:util` の `parseArgs` は、この仕様の引数の文法に合わない見込みのため使わない([04](04-cli.md))。合わない点として、確認できている範囲では、(1) 値を取るオプションが、次の語が `-h` や `--help` でも、それを値として取る規則(AC-447)、(2) 短縮形の連結(`-vh`)での `-h` の優先、(3) 使い方の誤りを 8 種類に分けた文言を出す必要がある点、がある。`parseArgs` の挙動(特に `strict` と `allowPositionals`、`--` の扱い、`-vv` の重複の検出)は、採用する Node の版の文書で確かめていない(未確認)。

### 8.2 YAML ライブラリの選定

選定の基準(すべて満たすこと):

| 基準 | 理由 | 確かめる受け入れテスト |
|------|------|------------------------|
| (1) 誤りと、各キー・値の行番号(できれば列番号)を、入力から取り出せる | 定義の誤りを、行つきで出す([03](03-definition-format.md)) | AC-009、AC-037 |
| (2) 重複キーを検出できる(ライブラリの機能、または構文木の走査で) | 重複キーを構文の誤りとして拒否する | AC-010、AC-226、AC-456 |
| (3) タグ・複数の文書・文字列でないキーを検出して拒否できる(構文木の段階で見られる) | 定義と変換で拒否する | AC-217、AC-218、AC-453〜455 |
| (4) 保守が続いている(未解決の重大な不具合や脆弱性の対応が止まっていない) | 長く使う | `npm audit`、実装時の調査 |
| (5) アンカー・エイリアス・マージキーの展開を、自前で、上限つきで実装できる(構文木にエイリアスが残る) | 爆発する入力に上限を付ける | AC-224〜230 |
| (6) 入れ子が深い入力で、スタックを使い果たさず失敗を返せる(または、渡す前の走査で防げる) | 2.2 節 | AC-232、AC-412 |
| (7) 実行時の依存が少ない(望ましくはなし)。ネイティブのコードを含まず、ネットワークを使わない。YAML 1.2 の既定の規則(`yes`・`on` が文字列)で読める | 8.1 節。AC-458、AC-228 | AC-419、AC-443、AC-458 |
| (8) 警告を、`process.emitWarning` などで標準エラーに出さない、または止められる | AC-496 | AC-496 |
| (9) ESM から読み込め、最小の Node でも最新の Node でも動く | 6 章 | AC-479 |

候補(版と保守の状況は、この文書では確認していない。実装時に確認する):

| 候補 | 特徴(確認できていない。実装時に確認する) | 評価 |
|------|-------------------------------------------|------|
| `yaml` | 文書オブジェクト(構文木。各ノードの範囲と、行・列を求める `LineCounter` を持つ)、複数文書の読み取り、重複キーの検出の設定、エイリアスの上限の設定、YAML 1.1 / 1.2 の選択、スキーマの選択を提供し、依存を持たない、とされる | 第一候補。基準(1)〜(3)・(5)・(7) は満たせる見込み。(4)(6)(8) の現状と、各設定の名前・既定値は、**未確認** |
| `js-yaml` | 読み取りの関数が、JavaScript の値を返す。エラーに行・列(`mark`)がある。重複キーは既定で誤りになる、とされる。値ごと(キーごと)の位置は返さない見込みで、AC-037(キーごとの行)が満たせない可能性が高い。保守の状況は未確認 | 第二候補。基準(1)(5) が難しい見込み |
| 自前の YAML 読み取り | — | 採らない(YAML の文法が大きく、誤りが入りやすい) |

**採用案**: 構文木(行・列・タグ・種類・エイリアスつき)を提供するライブラリを 1 つ採る(第一候補の `yaml`)。YAML を読む **定義と変換の入口**でだけ使う(ステートは JSON)。構文木を、この仕様の「木」(順序つきの写像(`Map` か、キーの順を別に持つ構造)・列・文字列・整数(`bigint` か字句)・浮動小数点数・真偽値・`null`)に変換する層を自前で書き、その層で、重複キー・タグ・複数文書・文字列でないキー・深さ・要素数・エイリアスの展開を扱う(`null` は拒否しない)。ライブラリのエラー文と例外は、表示に出さない(2.3 節)。YAML 1.2 の既定の規則で読む設定にする(AC-228、AC-458)。ライブラリの警告は、標準エラーに出さない(AC-496)。
実装時は、まず AC-010、AC-217〜232、AC-453〜460、AC-496 の入力を、小さな試験で候補に通し、基準(1)〜(9)を確かめてから決める。採用したライブラリと版は、`package.json`、`package-lock.json`、README に記録する。

JSON の定義とステートと変換の入力の読み取りは、自前の小さな字句・構文解析で行う([03](03-definition-format.md)。`JSON.parse` は、重複キーの検出・数値の整数と浮動小数点数の区別・桁数の多い整数・深さの上限・行と列の位置を得られないため)。`JSON.parse` を使ってよいのは、テストと、開発用のスクリプト(`package.json` などの読み取り)だけ(AC-421)。

### 8.3 開発時の依存の一覧

版は、すべて「実装時に確認する」。最小の Node と最新の Node の両方で動く版を選ぶ(各ツールの `engines` を確かめる)。`package.json` には、正確な版(`save-exact`)を書く。

| パッケージ | 用途 | 選定の基準 |
|------------|------|------------|
| `typescript` | `tsc`(型の検査とビルド)、点検のテストの構文木の走査(`createSourceFile`) | `strict` と `NodeNext` を扱える版。`typescript-eslint` が対応する版の範囲内 |
| `eslint`、`@eslint/js` | 静的検査(フラットな設定 `eslint.config.js`)。`@eslint/js` は、ESLint の推奨の規則の集まり(ESLint と同じ系列の版にする) | 型情報つきの検査を使える版。最小の Node で動く |
| `typescript-eslint` | `recommendedTypeChecked` の規則(`no-floating-promises` ほか)と、パーサー | `typescript` と `eslint` の対応表の範囲内(実装時に確認) |
| `prettier` | 整形(`prettier --check`) | 設定は `.prettierrc`。版を固定して、整形の差を出さない |
| `vitest` | テストの枠組み(`pool: 'forks'`、`globalSetup`、`skipIf`) | 最小の Node で動く。設定の書式(`coverage.thresholds`)は版による |
| `@vitest/coverage-v8` | カバレッジ(`provider: 'v8'`) | `vitest` と同じ版の系列 |
| `@types/node` | Node の API の型 | **主版を、最小の Node(`engines.node`)に合わせる**(最小の Node にない API を、型の検査が通さないようにする) |

上記以外の開発時の依存(`tsx`、`ts-node`、`rimraf`、`cross-env`、`execa`、`zod` など)は入れない(`scripts/check-deps.mjs` が検査する。5.4 節)。

### 8.4 `package.json` とロックファイルの運用

| 項目 | 決め |
|------|------|
| パッケージ名 | `conductor`(置き換えてよい。置き換える場合は、README、リリースの成果物の名前、`CONDUCTOR_BIN` の例、`scripts/check-*.mjs` の名前の参照をすべて合わせる) |
| `engines` | `"node": ">=<最小の主版>"`(例: `>=22`)。実装時点の最新の LTS 系列と、その 1 つ前の LTS 系列をサポートする。**実装時に、最新の 2 つの LTS 系列を確認して決める**。5.4 節の検査が、README と `ci.yml` の `NODE_MIN` との一致を保つ |
| `type` | `module` |
| `bin` | `{ "conductor": "dist/bin/conductor.js" }` |
| `files` | `["dist", "LICENSE", "README.md"]`(これ以外を、配布物に含めない) |
| `dependencies` | 8.2 節で採った YAML ライブラリだけ。正確な版(`.npmrc` の `save-exact=true`) |
| `devDependencies` | 8.3 節の一覧だけ。正確な版 |
| `package-lock.json` | コミットする。手で編集しない。`lockfileVersion` は、採用した npm が書くとおり(実装時に確認) |
| 更新 | `npm install <名前>@<版> --save-exact` のあと、`npm ci` で確かめる。更新のコミットで、`npm test`(3 OS)、`npm audit` を通す。自動の更新ツール(Dependabot など)の導入は任意(未確認) |
| 検査 | CI の `consistency`(5.1 節): `npm ci` が通る、`npm install --package-lock-only` で `package-lock.json` に差分がない、`check-deps.mjs` が通る |
| インストール | CI とリリースは `npm ci` だけを使う(`npm install` で、ロックファイルを書き換えない)。`.npmrc` の `engine-strict` は、利用者の環境に影響するので、リポジトリには置かない(置くかどうかは任意。未確認) |
| `scripts` | `clean`、`build`、`typecheck`、`lint`、`format`、`format:check`、`test`、`test:coverage`、`test:perf`、`prepack`(9.2 節) |
| ツール(`@vitest/coverage-v8` を除く、CI の部品) | `actions/*` の版を `ci.yml` に書く(5 章)。ローカルに入れるツールは、`devDependencies` だけ |

---

## 9. リポジトリのレイアウト

### 9.1 ファイルの構成

```text
package.json                    8.4 節、9.2 節
package-lock.json               コミットする
.npmrc                          save-exact=true
tsconfig.json                   型の検査用(src と test。noEmit)(9.3 節)
tsconfig.build.json             ビルド用(src だけ。dist へ出力)(9.3 節)
eslint.config.js                ESLint の設定(9.4 節)
.prettierrc, .prettierignore    Prettier の設定(9.5 節)
vitest.config.ts                3.9 節
vitest.perf.config.ts           3.9 節
.gitignore                      node_modules/、dist/、coverage/、*.tgz など(9.6 節)
LICENSE                         Apache License 2.0(配布物に入れる)
README.md                       利用者向けの文書(docs-check が必須の語を検査する。5.1 節)
.github/workflows/ci.yml        5.1 節
.github/workflows/audit.yml     5.2 節
.github/workflows/release.yml   6.3 節
scripts/                        check-version.mjs、check-node-version.mjs、check-deps.mjs、check-readme.mjs、check-coverage.mjs、check-pack.mjs、clean.mjs(5.4 節)
src/bin/conductor.ts            入口(薄い。先頭にシバン。未捕捉の例外・Promise の拒否を捕捉して終了コード 70)
src/definition/                 定義の読み込み・検査・型・エラー
src/convert/                    YAML ⇔ JSON
src/runid/                      run ID
src/flow/                       純粋な判断
src/state/                      ステートの読み書き・照合・reset
src/runner/                     子プロセスの起動・シグナル(platform.ts に OS の分岐を集める)
src/report/                     出力・メッセージ・ロケール・エスケープ・watch の表
src/cli/                        引数・各サブコマンド・Deps・終了コード
src/version.ts                  版の定数 VERSION(6.2 節)
test/<部品>/                    各部品の単体テスト(3.2 節)
test/acceptance/*.test.ts       受け入れテスト(3.3 節)
test/inspect/*.test.ts          点検のテスト(構文木の走査)
test/perf/*.test.ts             性能の計測(2.1 節)
test/jobs/tj.mjs                テスト用ジョブ(3.4 節。配布しない)
test/support/                   harness.ts、fakes.ts、global-setup.ts、tree.ts、inject-throw.mjs、inject-reject.mjs
test/fixtures/                  入力例(YAML・JSON。壊れたものを含む。prettier の対象外)
test/WINDOWS_CTRL_C_CHECKLIST.md   Windows の Ctrl+C の手動確認の手順と記録欄(AC-301、AC-487)
demo/                           7 章
docs/spec/typescript/           この仕様書
```

パッケージ(ディレクトリ)の責務と依存の向きは、[02](02-architecture.md) のとおり。`src/version.ts` は、`src/bin/conductor.ts` と `src/cli/`(`--version` の表示)から import される。`src/` は `test/` を import しない(AC-422)。

### 9.2 `package.json`

```json
{
  "name": "conductor",
  "version": "0.8.0",
  "description": "A CLI that runs jobs in order and records where it stopped",
  "license": "Apache-2.0",
  "type": "module",
  "bin": { "conductor": "dist/bin/conductor.js" },
  "files": ["dist", "LICENSE", "README.md"],
  "engines": { "node": ">=22" },
  "scripts": {
    "clean": "node scripts/clean.mjs",
    "build": "node scripts/clean.mjs dist && tsc -p tsconfig.build.json",
    "typecheck": "tsc --noEmit",
    "lint": "eslint . --max-warnings 0",
    "format": "prettier --write src test scripts vitest.config.ts vitest.perf.config.ts eslint.config.js package.json tsconfig.json tsconfig.build.json",
    "format:check": "prettier --check src test scripts vitest.config.ts vitest.perf.config.ts eslint.config.js package.json tsconfig.json tsconfig.build.json",
    "test": "vitest run",
    "test:coverage": "vitest run --coverage",
    "test:perf": "vitest run --config vitest.perf.config.ts",
    "prepack": "npm run build"
  },
  "dependencies": {
    "yaml": "<実装時に確認>"
  },
  "devDependencies": {
    "@eslint/js": "<実装時に確認>",
    "@types/node": "<実装時に確認。主版は engines の最小に合わせる>",
    "@vitest/coverage-v8": "<実装時に確認>",
    "eslint": "<実装時に確認>",
    "prettier": "<実装時に確認>",
    "typescript": "<実装時に確認>",
    "typescript-eslint": "<実装時に確認>",
    "vitest": "<実装時に確認>"
  }
}
```

`dependencies` の `yaml` は、8.2 節で採った YAML ライブラリに置き換える。`"engines"` の `>=22` は例。

### 9.3 `tsconfig.json` と `tsconfig.build.json`

`tsconfig.json`(型の検査。出力しない):

```json
{
  "compilerOptions": {
    "target": "ES2023",
    "lib": ["ES2023"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "types": ["node"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src", "test", "vitest.config.ts", "vitest.perf.config.ts"]
}
```

`tsconfig.build.json`(ビルド。`src/` だけ):

```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noEmit": false,
    "rootDir": "src",
    "outDir": "dist",
    "sourceMap": false,
    "declaration": false,
    "removeComments": false
  },
  "include": ["src"]
}
```

`removeComments: false` は、ソースの先頭の著作権表示を `dist/` に残すため。`skipLibCheck` は、依存の型定義の検査を省く(速度のため。`@types/node` と YAML ライブラリの型の不整合を、見逃さない範囲か、実装時に確認する)。`target` と `lib` は、最小の Node がサポートする版に合わせる(実装時に確認)。

### 9.4 `eslint.config.js`

出発点。ESLint のフラットな設定の書式と、`typescript-eslint` の設定の名前は、版で異なる(**実装時に、採った版の書式で確かめる**。この文書の作成時には実行していない)。

```js
import js from '@eslint/js';
import tseslint from 'typescript-eslint';

const ban = (names, message) => names.map((name) => ({ name, message }));
const NETWORK = [
  'net', 'http', 'https', 'http2', 'tls', 'dgram', 'dns', 'cluster', 'vm', 'worker_threads',
].flatMap((m) => [m, `node:${m}`]);
const CHILD = ['child_process', 'node:child_process'];
const prop = (object, property, message) => ({ object, property, message });

const JSON_PARSE = prop('JSON', 'parse', 'use the own JSON reader (duplicate keys, positions, big integers)');
const EXIT = prop('process', 'exit', 'only src/bin/conductor.ts may exit, after flushing');
const PLATFORM = prop('process', 'platform', 'only src/runner/platform.ts may branch on the OS');
const LISTEN = [
  prop('process', 'on', 'register process listeners only in src/bin and src/runner'),
  prop('process', 'once', 'register process listeners only in src/bin and src/runner'),
  prop('process', 'addListener', 'register process listeners only in src/bin and src/runner'),
];

const typed = tseslint.configs.recommendedTypeChecked.map((c) => ({ ...c, files: ['**/*.ts'] }));

export default [
  { ignores: ['dist/**', 'coverage/**', 'node_modules/**', 'test/fixtures/**'] },
  js.configs.recommended,
  ...typed,
  {
    files: ['**/*.ts'],
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
  },
  {
    files: ['**/*.mjs', '**/*.js'],
    languageOptions: {
      sourceType: 'module',
      globals: {
        process: 'readonly', console: 'readonly', performance: 'readonly',
        setTimeout: 'readonly', setInterval: 'readonly', clearTimeout: 'readonly',
        Buffer: 'readonly', URL: 'readonly',
      },
    },
  },
  {
    files: ['src/**/*.ts'],
    rules: {
      'no-eval': 'error',
      'no-implied-eval': 'error',
      'no-new-func': 'error',
      'no-restricted-imports': ['error', { paths: [...ban(NETWORK, 'forbidden in product code'), ...ban(CHILD, 'only src/runner may start processes')] }],
      'no-restricted-properties': ['error', JSON_PARSE, EXIT, PLATFORM, ...LISTEN],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/consistent-type-assertions': ['error', { assertionStyle: 'never' }],
      '@typescript-eslint/only-throw-error': 'error',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/switch-exhaustiveness-check': 'error',
    },
  },
  {
    files: ['src/runner/**/*.ts'],
    rules: {
      'no-restricted-imports': ['error', { paths: ban(NETWORK, 'forbidden in product code') }],
      'no-restricted-properties': ['error', JSON_PARSE, EXIT, PLATFORM],
    },
  },
  {
    files: ['src/runner/platform.ts'],
    rules: { 'no-restricted-properties': ['error', JSON_PARSE, EXIT] },
  },
  {
    files: ['src/bin/conductor.ts'],
    rules: { 'no-restricted-properties': ['error', JSON_PARSE, PLATFORM] },
  },
  {
    files: ['test/**/*.ts'],
    rules: { '@typescript-eslint/unbound-method': 'off' },
  },
];
```

- 警告は、すべて失敗として扱う(`eslint . --max-warnings 0`)。
- `eslint-disable` のコメントは、製品のコード(`src/`)に書かない(AC-421)。テストでの使用は、理由を同じ行に書く。
- `no-restricted-properties` が `process.exit` のような「オブジェクトの属性」を、期待どおり検出するか(特に、`import process from 'node:process'` の形や、分割代入 `const { exit } = process` の検出)は、確かめていない(未確認)。これらの形は、AC-497・AC-498 の構文木の点検が、別に見る。
- `consistent-type-assertions` の `assertionStyle: 'never'` が、`as const` を許すかは、版の文書で確かめる(未確認。許さない場合は、`as const` を使わない書き方にするか、この規則を `objectLiteralTypeAssertions` の設定に変える)。

### 9.5 `.prettierrc` と `.prettierignore`

`.prettierrc`:

```json
{
  "singleQuote": true,
  "semi": true,
  "trailingComma": "all",
  "printWidth": 100
}
```

`.prettierignore`:

```text
dist
coverage
node_modules
package-lock.json
test/fixtures
```

整形の対象は、`format:check` の引数(9.2 節)が決める。`README.md`、`docs/`、`demo/`、`.github/` は、対象に入れない(日本語の表と、手で書いたシェルの断片を、整形の対象にしない)。この文書のコードの断片は、整形の規則に合わせて書いた見込みだが、`prettier --check` を通ることは確かめていない。実装時に、`npm run format` で整形してからコミットする。

### 9.6 `.gitignore`

少なくとも、次を除く: `node_modules/`、`dist/`、`coverage/`、`*.tgz`、各 OS のエディタの一時ファイル。`package-lock.json` と `LICENSE` は除かない。

---

## 10. 未確認の事項と、他の章との整合が要る点

### 10.1 未確認の事項

- ブランチ保護(どのジョブを必須にするか)は、リポジトリの設定であり、リポジトリ内のファイルからは確認できない。
- 現在の行の網羅率の実測値、各性能の計測値(起動時間、`run` のオーバーヘッドなど)は、この文書の作成時には測れていない(基準だけを記した。「仮」は、実測して見直す)。Node の起動時間(数十 ms の見込み)も実測していない。
- この文書の作成時に、Node.js、npm、`tsc`、ESLint、Prettier、Vitest、YAML ライブラリ、GitHub Actions を実際に動かしては確かめていない。版、設定の書式、フラグ、終了コードの規則は、**実装時に確認する**。特に: `actions/setup-node` の `lts/*` と `cache: npm` の挙動、`npm pack --pack-destination` と `npm pack --dry-run --json` の出力の形、`npm audit` の終了コード、Vitest の `coverage.thresholds` と `pool` の書式、ESLint のフラットな設定と `typescript-eslint` の設定の名前、`tsc` が先頭のシバンを残すこと。
- YAML ライブラリの挙動(行・列の取り方、重複キー・タグ・複数文書・文字列でないキー・エイリアスの扱いの設定、`process.emitWarning` による警告の有無、深い入れ子での失敗の仕方)。
- `node:util` の `parseArgs` が、この仕様の文法(AC-447 ほか)を満たさない点の確認(使わない判断は、確認できた範囲の理由に基づく)。
- Node が `process.argv` の不正な UTF-8 のバイトを U+FFFD に置き換えること。そのため、不正なバイト列を含むファイル名を、`--config` や `convert` の入力に指定できない(AC-349、AC-406)。
- `spawn` の `detached: true` が UNIX 系で新しいセッション(新しいプロセスグループ)を作ること(AC-288)。07 章の規定による。
- Windows で `.cmd` / `.bat` を `shell: false` で `spawn` すると `EINVAL` になる Node の版の範囲と、それを避けて安全に起動する方法(AC-113〜115、AC-486、AC-508)。07 章で決める。
- Windows で、子プロセスの終了コードが `-1`(0xFFFFFFFF)のときに、`exit` イベントが返す値(AC-112)。Windows の `exit` イベントの `signal` の値(AC-502)。
- `sleep` / `trap` モードの `TJ` が、SIGTERM・SIGINT・SIGHUP で「シグナルで終了した」と観測されること(自分に同じシグナルを送り直す設計。3.4 節)。SIGQUIT の既定の動作。
- `setTimeout` に 2,147,483,647 ms を超える値を渡したときの挙動(AC-501)。
- 疑似端末の作り方(AC-182、AC-183)。Node の組み込みだけでは作れない。外部のライブラリを使うか、`script` コマンドを使うかは、実装時に決める(テスト専用の依存を増やす場合は、8.1 節の方針の例外として、README に理由を書く。または、端末かどうかの判定を `Deps` にして、偽の `Deps` で検査する(AC-182 の注入の併記)で代える)。
- Windows の Ctrl+C の手動の確認(AC-301)の記録は、この文書の作成時点では存在しない。
- 大文字小文字を区別しないファイルシステムでの run ID の衝突(AC-260)は、仕様の記述に基づく。
- `macos-latest` のランナーに `shasum` / `sha256sum` があること、`ubuntu-24.04` で `ps -o pgid=` が使えること、Linux の `/proc/<pid>/stat` の書式(AC-403)。
- 単一の実行ファイルの配布(6.4 節)は、任意で、この文書では確かめていない。
- `tsc` の出力が、3 OS で同じ結果になること(`build-repro`。5.1 節の任意のジョブ)。
- `npm publish` の手順の細部(二要素認証、パッケージ名の取得、`--provenance`)。
- 起動の性能の測定を強制に切り替える予定の有無は、決まっていない。
- 標準出力への書き込みの失敗(`watch`・`convert`)を、実際のプロセスで確かめる方法のうち、`EPIPE`(AC-492)は、読み手が閉じるタイミングに依存する。確実に起こすための調整(出力の大きさ)は、実装時に確認する。
- 受け入れテストのうち、4.1 節〜4.20 節の、期待の値の一部は、定義・ステート・変換の他章([03](03-definition-format.md)、[06](06-state-file.md)、[09](09-converter.md)、[10](10-errors-and-exit-codes.md))の字句に依存する。他章と食い違う場合は、他章を正とし、この文書の期待を直す。

### 10.2 他の章との整合が要る点

| 項目 | この文書の前提 | 整合が要る章 |
|------|----------------|--------------|
| OS 固有コードの置き場所 | `process.platform` の参照は `src/runner/platform.ts` の 1 か所だけ(AC-420)。ステートのディレクトリの同期、端末の判定、受け取るシグナルの一覧は、そこが出す値・関数を使う | 02、07 |
| `Deps` の役割の一覧 | 環境変数、時刻・待ち(中断できる)、ファイルシステム、プロセス実行、シグナル、標準出力・標準エラー(3.7 節)。端末かどうかの判定も `Deps` の関数にする(AC-182 の注入) | 02、04 |
| 入口の終了の方法 | `process.exit` は `src/bin/conductor.ts` に 1 か所以下で、書き込みの完了を待ってから(AC-498)。未捕捉の例外・Promise の拒否は 70(AC-489、AC-490) | 02、10 |
| 子プロセスの扱い | `stdio: 'inherit'`、`shell: false`、`close` イベントで終わりを待つ、孫を待たない(AC-504)、中断のあとも子の終了を待つ(AC-503)、新しいプロセスグループにするか(AC-288) | 07 |
| Windows の `.cmd` / `.bat` | 禁止文字の拒否(AC-486)と、Node の `EINVAL` を避ける起動の方法(AC-508)。シェルを使わないという規則(AC-116)との関係 | 07 |
| シグナルの一覧と登録 | UNIX 系は `SIGHUP`・`SIGINT`・`SIGQUIT`・`SIGTERM`、Windows は `SIGINT`。登録は 1 回、終了で解除(AC-285、AC-484、AC-497) | 07 |
| `bigint` の飽和演算 | リトライの待ち時間(`src/flow/`)は `bigint`。タイマーのミリ秒への変換は、2,147,483,647 を超えない(AC-499、AC-501) | 05、07 |
| 整数の範囲検査 | 定義・ステートの整数は、字句から `bigint` で検査する(AC-031、AC-066、AC-068、AC-474、AC-500) | 03、06 |
| JSON の自前の読み取り | 重複キー、位置、深さ、整数の字句、`__proto__` などのキー(AC-461〜468、AC-510)。木は `Map` などで、キーの順を保つ(AC-222、AC-509) | 03、06、09 |
| 標準出力の失敗 | `EPIPE`・`ENOSPC` を、`watch` は 65、`convert` は 67 の「標準出力に書けない」として扱う(AC-186、AC-236、AC-492、AC-494) | 08、09、10 |
| 版の定数 | `src/version.ts` の `export const VERSION = "0.8.0";`(1 行。AC-476) | 02、04 |
| ファイル名の復号 | Node が `argv` を UTF-8 として復号する制約(AC-349、AC-406) | 04 |
| テスト用ジョブの置き場所 | `test/jobs/tj.mjs`。配布物に含めない | 02、12 |
| 再構築の手順 | 12 章は、この文書の品質ゲートの再現(5 章)とリリース(6 章)、依存の許可の一覧(8 章)を参照する | 12 |

定義・ステート・変換の仕様(03、06、09 章)の決定と、この文書の受け入れテストの期待は、次のとおり揃えてある(受け入れテストの期待は、該当章の字句が正本である)。

| 項目 | 決定(出どころ) |
|------|----------------|
| 定義のキー名 | 配列のキーは `job`(`jobs` は未知のキー)。遷移は `transitions`、再実行は `retry`([03](03-definition-format.md)) |
| 定義の null | `wrong value type`(`retry: null` は `retry must be a map`、未知のキーの値が null なら `unknown key`) |
| 定義の YAML の高度な機能 | アンカー・エイリアス・マージキーは、定義でも展開して受理する。タグ・複数文書・文字列でないキー・重複キーは拒否 |
| 入れ子の深さ | 根のマップを数えず、32 段まで受理し 33 段目を拒否(YAML・JSON・変換で同じ)。要素数の上限は 100,000(`a` の列の要素数 n が 99,997 まで受理) |
| 大きさの上限 | 定義・ステート・変換の入力は、それぞれ 1 MiB(1,048,576 バイト)まで |
| JSON の数 | 整数のリテラルだけを整数と読む(`1.0`、`1e0` は浮動小数点数)。`format_version: 1.0` は `unsupported format_version` |
| JSON の空の入力 | 定義・変換とも `syntax error`(YAML の空の文書は、定義では「空のマップ」で `format_version is missing`、変換では `top level is not a map`) |
| `convert` の入力形式 | `--to` の反対で、拡張子は見ない([09](09-converter.md)) |
| `convert` の拒否 | `null` は拒否しない。YAML のタグ・複数文書・文字列でないキーは読み取りの `syntax error`。範囲外の数、YAML の `.inf`・`.nan`(JSON へ)、1,024 文字超のキー(JSON から YAML へ)は `unrepresentable` |
| `convert` の出力 | キーは入力の記述順、インデントは 2 スペース、末尾は改行。JSON の出力は、`<`、`>`、`&` をエスケープせず、非 ASCII をそのまま出す。浮動小数点数は、整数と区別できる書き方で出す |
| 定義の拡張子 | `.yaml`・`.yml`・`.json`(ASCII の大文字小文字を区別しない)だけ。ほかは `unsupportedFormat` |
| 理由の文 | ステートの読み取りの失敗は `JSON として読めません`、キーの重複は `キーが重複しています`。YAML の文法の誤りは `YAML の文法に誤りがあります`、JSON は `JSON の文法に誤りがあります`。深さは `入れ子が深すぎます(上限は 32 段)` |
| ステートの JSON | キー名は論理項目と同じ。キーの順と整形は 06 章に従う |

残る未確認は、10.1 節のとおりである(YAML ライブラリの挙動、Windows の実機の挙動、Node の各 API の挙動など)。
