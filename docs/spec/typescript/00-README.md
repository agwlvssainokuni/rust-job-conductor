# 00. README(TypeScript 版 仕様書の案内)

> この仕様書は v0.8.0 時点の挙動で凍結されている(v0.9.0 以降の環境変数の注入と `.env`、`graph`、`--dry-run`、`init` を含まない)。最新は [rust](../rust/00-README.md) と [language-agnostic](../language-agnostic/00-README.md)。

この仕様書は、コマンドラインツール `conductor` を、TypeScript(Node.js 上)でゼロから実装し直すための仕様である。`conductor` は、1 つの定義ファイルに書かれたジョブ(外部コマンド)の並びを、終了コードで分類しながら順に実行し、途中経過をステートファイルに残して、失敗したところから再開できるようにする、コマンドラインツールである。

この文書は、仕様書全体の入口である。範囲と版、目次、読む順序、規定の強さの語、用語集、この版が採った書式・実行環境の選択をまとめる。

## 1. 仕様書の範囲と版

| 項目 | 内容 |
|------|------|
| 対象 | `conductor` の利用者から見える挙動を、TypeScript(Node.js 上)で再実装するための仕様 |
| 版 | `conductor 0.8.0` と同じ利用者向けの挙動。`conductor --version` は `conductor 0.8.0` を出す(版の文字列は `src/version.ts` の定数 `VERSION` が唯一の定義。`package.json` の `version` との一致をテストで検査する) |
| 実装言語・実行環境 | TypeScript(`strict`)。ESM(`"type": "module"`、`module`/`moduleResolution` は `NodeNext`)。実行は Node.js。実装時点の最新の LTS 系列と、その 1 つ前の LTS 系列をサポートし、具体的な版は実装時に確認して `package.json` の `engines` に最小版を書く(11 章の例は `>=22`) |
| 外部ライブラリ | 実行時は YAML の読み取りの 1 つだけ(採用案は `yaml`。版を固定する。選定の基準と候補は [03](03-definition-format.md) 7 章と [11](11-quality-and-build.md) 8 章)。ほかは `node:` の標準モジュール。開発時の依存は、TypeScript、ESLint(`typescript-eslint`)、Prettier、Vitest(`@vitest/coverage-v8`)、`@types/node` |
| 対象 OS | Linux、macOS、Windows の 3 つ。配布物は npm のパッケージ(tarball)1 つで、3 OS で同じものが動く |
| 同じにするもの | サブコマンド、オプション、終了コード、実行の意味論(分類・分岐・再開・リトライ・run ID)、メッセージの文言(日本語・英語)、`watch` の表 |
| 含まないもの | 実装そのもの(コードは規定しない。骨格のコードは、型とシグネチャの例である)。各章が「実装に任せてよい」と書いた点 |

この仕様書だけで、同じ受理・拒否・出力・終了コードを返す実装を作れることを目標とする。確認できなかった点は「未確認」と明記し、[12](12-rebuild-guide.md) 3 章にまとめた。Node.js と各ライブラリの挙動で、作成時に実際に動かして確かめたものは、「確認」「実測」と書いてある(Node.js v26.13.0、macOS が中心。YAML ライブラリ `yaml` は 2.9.1)。書いていないものは、公式の文書の知識によるもので、「実装時に確認する」。最小の LTS 系列と最新の LTS 系列の Node.js、Linux、Windows での再確認は、全章に共通の未確認である。

## 2. 目次

| ファイル | 内容(1 行要約) |
|----------|-----------------|
| [00-README.md](00-README.md) | この文書。範囲、目次、読む順序、規定の強さの語、用語集、書式・実行環境の選択 |
| [01-overview.md](01-overview.md) | 目的、ユースケース、設計方針、非目標、制約、利用者像、概念の概観 |
| [02-architecture.md](02-architecture.md) | ディレクトリの構成、依存の向き、型の骨格、差し替え口 `Deps`、実行のシーケンス、非同期の方針、`package.json` と `tsconfig.json`、エラー処理の方針 |
| [03-definition-format.md](03-definition-format.md) | 定義ファイルの書式(YAML/JSON)、読み取りの段階、上限、YAML ライブラリの選定と補完、検査、エラーの種類(31 種)と位置の出し方 |
| [04-cli.md](04-cli.md) | コマンドライン(6 つのサブコマンド、オプション、`parseArgs` を使わない理由、引数の解釈の手順、非 UTF-8 の引数、run ID、環境変数) |
| [05-execution-flow.md](05-execution-flow.md) | 実行の意味論(分類、遷移、起動前の判断、リトライと `bigint` の待ち時間、再開、完了、`src/flow/` の純粋関数) |
| [06-state-file.md](06-state-file.md) | ステートファイル(論理項目、JSON の形式、原子的な書き込み、読み込みの 14 段階の検査、自前の JSON の読み取り、整合性、`reset`) |
| [07-process-and-signals.md](07-process-and-signals.md) | `command` の分割、`spawn` による子プロセスの起動と待ち、中断の合図 `Interrupt`、シグナルの受信と転送、リトライの待機、OS 別の差(`platform.ts`)、Windows の `.cmd` の起動 |
| [08-output-and-i18n.md](08-output-and-i18n.md) | 出力先、標準出力の flush と `EPIPE`、`-v`、メッセージカタログ(日本語・英語)、`watch` の表、ロケール、エスケープ、`--help` の全文 |
| [09-converter.md](09-converter.md) | `convert`(YAML と JSON の相互変換)の読み取りの差分、整数の `bigint`、書き出しの形式、エラー |
| [10-errors-and-exit-codes.md](10-errors-and-exit-codes.md) | 終了コードの全件と決定順序、エラー分類の一覧、TypeScript でのエラー型、想定外の内部失敗(70) |
| [11-quality-and-build.md](11-quality-and-build.md) | 非機能要件、テスト構成、受け入れテスト一覧(AC-001〜515)、CI(GitHub Actions)、リリース(tarball)、デモ、依存、リポジトリのレイアウト |
| [12-rebuild-guide.md](12-rebuild-guide.md) | ゼロから実装する順序、Node.js・TypeScript 特有の落とし穴、未確認事項の一覧、最初に確かめる小さな試験 |

## 3. 読む順序

| 目的 | 読む順序 |
|------|----------|
| 全体を理解する | 00 → 01 → 02 → 05 → 03 → 06 → 04 → 07 → 08 → 10 → 09 → 11 |
| 実装を始める | 00 → 01 → [12](12-rebuild-guide.md)(実装の順序と最初の試験)→ 12 が指す各章 |
| 定義の読み込みだけを作る | 03 → 09(読み取りを共有)→ 10(エラー) |
| 実行の流れだけを作る | 05 → 06 → 07 → 10 |
| 出力の文言を確かめる | 08 → 10 |
| 受け入れる | 11(各 AC は、該当章の字句が正本) |

各章の終わりに、その章の「実装に任せてよい点」と「未確認」がある。受け入れテスト(11 章)の期待は、定義・ステート・変換・終了コードの各章(03、06、09、10)の字句と食い違うときは、それらの章を正とする。章どうしの食い違いは、03(定義)、05(実行フロー)、06(ステート)、07(プロセスとシグナル)、09(変換)の定めを正とする。

## 4. 規定の強さの語

章によって、記号や語が少し違うが、意味は次の 3 段階に対応する。

| 強さ | 意味 | 章での表記 |
|------|------|------------|
| 必須 | 利用者から見える結果(受理・拒否、出力の文字列、終了コード、作られるファイルの内容)であり、同じにしなければならない。受け入れテストで確かめる | 「必須」「【規定】」「〜しなければならない」 |
| 目安 | 推奨する値や分け方。利用者から見える挙動を損なわない範囲で、別の値や分け方を採ってよい。基準を超えたときは警告として記録する | 「目安」「【目安】」「推奨」「仮」(暫定として置いた値) |
| 任意 | 利用者から見える結果が変わらない範囲で、実装が自由に選んでよい | 「任意」「【任意】」「実装に任せてよい」「裁量」 |

- 【Node の設計】(07 章)は、Node.js の都合で選んだ設計で、利用者から見える結果が同じなら変えてよい(目安と同じ扱い)。ただし、07 章が「規定」と明記したもの(`shell: false`、転送は 1 回、`process.platform` は `platform.ts` だけ、など)は必須である。
- 【実測】(07 章)は、作成時に実際に動かして確かめた挙動の印である。規定の強さではない。
- 骨格のコード(型・シグネチャ)は、名前や細部が他の章の定めと食い違うときは、03・05・06・07・09 の定めを正とし、骨格を読み替える。
- 「未確認」は、作成時に確かめられなかった点で、断定していない。実装時に確かめる。

## 5. 用語集

02〜11 章で使っている語を、次のように統一する。

| 語 | 意味 |
|----|------|
| 定義ファイル | パイプラインを書いたファイル。YAML(`.yaml`、`.yml`)か JSON(`.json`)。既定の名前は `conductor.yaml` |
| 定義 | 定義ファイルを検証して読み込んだ結果(`Definition`。`format_version`、`pipeline`、`job`) |
| パイプライン | 名前をもち、ジョブの並びからなる 1 つの流れ。1 つの定義ファイルに 1 つ |
| ジョブ | 実行する外部コマンド 1 つと、その終了コードの扱い(成功・警告のコード、遷移、リトライ)。ID をもつ |
| ジョブID | ジョブの識別子(`id`)。定義の中で一意。空でなく、`@` で始まらない |
| `command` | ジョブが実行するコマンド。文字列(分割して実行する)か、文字列のリスト(そのまま渡す)。シェルは介さない |
| 成功コード・警告コード | ジョブの終了コードのうち、成功(`success_exit_codes`。省略時は `[0]`)・警告(`warning_exit_codes`)として扱う値 |
| 分類 | ジョブの結果を、成功・警告・エラー・分類なし(起動前の中断)に分けること |
| 遷移(`transitions`) | 成功・警告のとき、次に実行するジョブを指定すること。`@end` は、残りをスキップして終える |
| スキップ | 遷移などで、実行されないジョブ。理由は「遷移による」(`byTransition`)「`@end` による」(`byEnd`) |
| 拒否 | 順序外などで、`run` が実行を断ること(終了コード 66) |
| リトライ(`retry`) | 決めた終了コードで終わったジョブを、待ってから実行し直すこと。最大回数・間隔・倍率・上限を指定できる。待ち時間は `bigint` の秒 |
| run ID | 実行単位を区別する文字列(`A-Za-z0-9._-`、UTF-8 で 1〜64 バイト、先頭は `.` でない)。ステートファイルを分ける |
| ステートファイル | 実行の途中経過を JSON で保存するファイル。名前は `<定義ファイル名>.state`(run ID があれば `<定義ファイル名>.<run ID>.state`) |
| 再開位置(`resume_job_id`) | 次に実行する、または再開するジョブのID |
| 完了 | 全ジョブが終わった状態(`completed = true`)。完了のステートは、読み込みでは「空」とみなす |
| 空のステート | ステートが無いのと同じ扱い(ファイルがない、長さ 0、完了)。読み込みの戻り値は `undefined` |
| 耐久性(durable / fast) | ステートの書き込みで、内容の同期まで行うか(durable)、省くか(fast) |
| 実行方式 | `run`(1 ジョブ。`individual`)と `pipeline`(全ジョブ。`pipeline`) |
| at-least-once | ジョブは、少なくとも 1 回は実行される。結果の記録の前に落ちると、再実行されうる |
| 中断の合図(`Interrupt`) | シグナルを受けたことと、最初の番号を、シグナルのハンドラと待つ側(`runJob`、`waitRetry`、`cli` の流れ)のあいだで共有する値(`raise`・`number`・`whenRaised`) |
| 起動失敗 | 子プロセスを作れなかったこと(終了コード 127)。終了の状態を取得できない場合も、安全側でこれに含める |
| プラットフォーム層 | OS 差を吸収する唯一の場所(`src/runner/platform.ts`。`platformFor`、`detectPlatform`)。`process.platform` を読むのはここだけ |
| 木(`TreeNode`) | YAML・JSON の読み取り結果の共通の木。`null`、`bool`、`int`(`bigint`)、`hugeInt`(範囲外の整数)、`float`、`string`、`seq`、`map` の 8 種。マップは `entries` の配列で持つ |
| マップ・リスト | キーと値の対応・値の並び(YAML のブロック/フロー、JSON のオブジェクト/配列) |
| 段階(Intake、Parse、Resolve、VersionCheck、Walk) | 定義の読み込みの処理の段階。前の段階で失敗したら、そこで打ち切る(03 章 2.2 節) |
| 種類(`kind`)・理由(`reason`) | エラーの識別子(`syntax`、`typeMismatch` など)と、固定の日本語の原因の文 |
| 案内 | ステートの誤りに付ける、`conductor reset` で回復できるという文 |
| `Deps` | `src/cli/` の差し替え口。環境、時刻、定義、ステート、変換、プロセス、シグナル、標準出力・標準エラーの interface の束 |
| バレル(`index.ts`) | 各ディレクトリの公開窓口。他のディレクトリからの import は、必ず相手の `index.ts` を通す |
| flush | 標準出力・標準エラーの書き込みの完了を待つこと。終了の前に行い、`process.exit()` を書き込みの完了前に呼ばない(08 章 1.3 節) |
| TJ(テストジョブ) | 受け入れテスト用のスクリプト `test/jobs/tj.mjs`。配布しない |
| AC | 受け入れテストの項目(`AC-xxx`)。11 章 4 章 |

## 6. この版が採った書式・実行環境の選択

この版が採った、書式・名前・文言・実行環境の選択を、一覧にする。

| 項目 | 内容 | 章 |
|------|------|----|
| 定義ファイルの書式 | YAML(YAML 1.2 のコアスキーマ)と JSON(RFC 8259)。書式は拡張子で決める(`.yaml`・`.yml` は YAML、`.json` は JSON。ASCII の大文字小文字を区別しない。判定は自前で、`path.extname` を使わない)。それ以外は `unsupportedFormat`(64) | 03 |
| 既定のファイル名 | `conductor.yaml`(`--config` で指す) | 03、04 |
| 定義のキー名 | `format_version`、`pipeline`(`name`、`description`、`run_id_env`、`remove_on_complete`)、`job`(`id`、`command`、`warning_exit_codes`、`success_exit_codes`、`transitions`、`retry`) | 03 |
| YAML の部分集合 | アンカー・エイリアス・マージキーは展開して受理。タグ、複数の文書、文字列でないキー、重複したキー、タブのインデントは拒否。`yes`・`no`・`on`・`off` は文字列。スカラーの型は自前で決める(`schema: 'failsafe'`) | 03 |
| YAML ライブラリ | `yaml`(2.9.1 で確認。版は実装時に確認して固定。先行版は使わない)。`parseAllDocuments`、`uniqueKeys: false`、`merge: false`、`logLevel: 'silent'`。重複キーの検査・アンカーの展開・位置の補正・敵対的な入力の前処理は自前 | 03、11 |
| JSON の読み取り | 自前の字句・構文解析 `readJson`(`src/definition/read-json.ts`。定義・変換・ステートで共用)。`JSON.parse` は製品のコードで使わない(重複キー、巨大な整数、位置、深さを扱えないため)。整数は字句から `bigint` | 03、06 |
| 読み込みの上限 | ファイル 1 MiB、要素数 100,000、文字の大きさ 16 MiB、入れ子の深さ 32 段(根のマップを数えず、33 段目を拒否) | 03 |
| 空白の判定 | Unicode の White_Space(U+0009〜U+000D、U+0020、U+0085、U+00A0、U+1680、U+2000〜U+200A、U+2028、U+2029、U+202F、U+205F、U+3000)。`String.prototype.trim()` は使わない(U+FEFF を取り除き、U+0085 を取り除かないため) | 03、07 |
| 定義のエラーの種類 | 31 種(`unsupportedFormat` を含む)。位置は行(と順序づけのための列)で出し、誤りは位置の順に 1 行ずつ。理由は常に日本語の固定文で、入力の値を含めない | 03、08、10 |
| ステートファイル | JSON(UTF-8、BOM なし。`TextDecoder` は `fatal: true`・`ignoreBOM: true`)。名前は `<定義ファイル名>.state`。キーは固定の順序で書き、インデントは空白 2 つ、末尾は LF 1 つ。U+2028・U+2029 は ` `・` ` に置換する。読み込みは 14 段階の検査で、キーの重複・BOM・最上位がオブジェクトでない値を拒否する | 06 |
| ステートの理由文 | 全 23 件。`JSON として読めません`、`キーが重複しています`、`内容を JSON にできません` を含む | 06、10 |
| `convert` | YAML と JSON の相互変換(`--to yaml\|json`)。入力の形式は `--to` の反対とみなし、拡張子は見ない。出力のキーの順は入力の記述順、JSON はインデント 2 スペース・末尾 LF。`null` は表せる。表せない値は、範囲外の数、YAML の NaN・無限大(JSON へ)、1,024 文字を超えるキー(JSON から YAML へ)の 3 つ。書き出しは自前(`JSON.stringify` を使わない) | 09 |
| 変換のエラー | 6 種(`fileMissing`、`unreadable`、`tooLarge`、`syntax`、`unrepresentable`、`rootNotMap`)。終了コード 67 | 09、10 |
| `retryNotTable` の文言 | `retry must be a map` / `retry はマップで書きます` | 08 |
| 使い方(`--help`) | `convert <file> --to <yaml\|json>`、`--config` の既定は `conductor.yaml` | 08 |
| 引数の解析 | `node:util` の `parseArgs` を使わず、自前の字句・検査の 2 段階(最初の誤りで例外になり、`-h`/`-V` の優先と誤りの優先順位が実装できない、値を取るオプションの規則が合わない、などの理由。v26.13.0 で確認した範囲) | 04 |
| 非 UTF-8 の引数 | Node.js は不正なバイトを U+FFFD に置き換えて渡す(macOS で確認)。非 UTF-8 を理由にした `invalid value` は無い | 04、10 |
| 整数 | 64 ビット符号なしの飽和演算(リトライの待ち時間)は `bigint`。範囲外の整数の検査は、字句から `bigint` で行い、`number` に落とさない。終了コードは `number` | 03、05 |
| 終了コード | 0、1〜255(ジョブの値)、64(定義)、65(ステート)、66(拒否)、67(変換)、68(使い方)、70(内部)、127(起動失敗)、128+番号(シグナル) | 10 |
| 想定外の内部失敗 | `process.on('uncaughtException')`・`process.on('unhandledRejection')` と入口の `try/catch` で、`conductor: error: internal error` を 1 行出して 70 | 10 |
| 標準出力・終了 | 書き込みは `callback` を待つ(`Promise` に包む)。`process.stdout`・`process.stderr` の `'error'` のリスナーを最初に付ける。標準出力に書けない失敗は、`watch` が 65、`convert` が 67、`--help`・`--version` は無視して 0。`process.exit()` は入口の 1 か所以下で、書き込みの完了を待ってから | 08、10 |
| 非同期の方針 | `async`/`await`。ジョブ・出力・ステートの書き込みは直列。`Promise.all`・`Promise.race` で仕事を並行させない(例外は `waitRetry` の内側)。`worker_threads` を使わない。待たない Promise を作らない(ESLint) | 02 |
| シグナル | UNIX 系は SIGHUP・SIGINT・SIGQUIT・SIGTERM を `process.on` で受け、子プロセスの pid にだけ `child.kill` で 1 回転送(`detached: true` の新しいプロセスグループ)。Windows は Ctrl+C(SIGINT)だけを受け、転送しない(`child.kill` を呼ばない)。中断の合図は自前の `Interrupt`(`AbortSignal` を使わない)。終了コードは 128+番号 | 07 |
| 子プロセス | `spawn`(`shell: false`、`stdio: 'inherit'`)。終了の観測は `close`。Windows の `.cmd` / `.bat` は、禁止文字を拒否したうえで、`cmd.exe /d /s /c` を `shell: false`・`windowsVerbatimArguments: true` で起動 | 07 |
| タイマー | 1 回の `setTimeout` は 2^30 ミリ秒以下に分割し、経過は `process.hrtime.bigint()` で再確認する。合図から 1 秒以内に打ち切る | 05、07 |
| ディレクトリ構成 | `src/{bin,definition,convert,runid,flow,state,runner,report,cli}`、`src/version.ts`、`test/`(`test/support/`、`test/jobs/`)。各ディレクトリの `index.ts` が公開窓口 | 02 |
| ESM・TypeScript | `"type": "module"`、`NodeNext`、`strict`、`noUncheckedIndexedAccess`、`exactOptionalPropertyTypes`、`verbatimModuleSyntax`。相対 import に `.js` を付ける。`any`・`as`・`!` を製品のコードで使わない。`tsc` で `dist/` に出力し、`bin` は先頭が `#!/usr/bin/env node` | 02、11 |
| 品質ゲート | `tsc --noEmit`、ESLint(型情報つき)、`prettier --check`、依存の向きと循環の検査、`vitest run`(3 OS)、カバレッジ 80% 以上(Linux)、`npm audit`(定期。警告のみ)、起動時間の計測(目安)、最小の Node と最新の Node でのテスト、`engines` と CI の Node の版の一致の検査 | 11 |
| リリース | `v*` のタグ(`main` 上で、`src/version.ts` および `package.json` の版と一致)で、`npm pack` の tarball を作り、`SHA256SUMS` を付けて、下書きのリリースに添付する。npm への公開は人が手で行う(ワークフローは行わない)。単一の実行ファイルの配布は任意 | 11 |

## 7. 表記のきまり

- 章への参照は、`03 章 6.3 節` のように書く。ほかの章へのリンクは、このディレクトリのファイルだけである。
- コードブロックの TypeScript のコードは、型とシグネチャの骨格(実装ではない)である。
- メッセージの文言は、`LANG=C`(英語)と日本語の両方を、各章の表に原文のまま書いてある。理由の文(定義・ステート・変換)は、ロケールに関わらず日本語の固定文である。
- 確認できなかった点は「未確認」と書き、[12](12-rebuild-guide.md) 3 章にまとめる。
