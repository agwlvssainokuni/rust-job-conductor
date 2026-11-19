# 12. 再構築の手引き(TypeScript でゼロから実装する)

この文書は、`conductor` を TypeScript(Node.js 上)でゼロから実装する人のための、実装の順序、Node.js と TypeScript に特有の落とし穴、未確認事項、最初に確かめる小さな試験をまとめる。各規定の本文は、02〜11 章にある。この文書は、それらへの道案内であり、規定を足さない。食い違いがあれば、各章を正とする。

- 規定の強さの語は [00-README.md](00-README.md) 4 章。
- 受け入れテスト(AC)は [11-quality-and-build.md](11-quality-and-build.md) 4 章。ここで「AC-xxx〜yyy」と書くのは、その範囲の項目を指す。
- Node.js と各ライブラリの挙動で、作成時に実際に動かして確かめたものは、章ごとに「確認」「実測」と書いてある(Node.js v26.13.0、macOS。一部は v24.22.2)。書いていないものは、公式の文書の知識によるもので、**実装時に確認する**。

## 1. 実装の順序

依存の向き([02](02-architecture.md) 3 章)の下から上へ、かつ「純粋な部分を先に、入出力を後に」の順で作る。各段階は、完了条件を満たしてから次へ進む。段階ごとの確認は、その段階の単体テストと、11 章の受け入れテストのうち、その段階で動くものの範囲で行う。受け入れテストは、ビルドした `dist/bin/conductor.js` を外から動かすので、`cli` までそろった段階 8 から本格的に使える。それ以前は、単体テストと、`Deps` の偽の部品(02 章 4.4 節)で確かめる。

| 段階 | 作るもの | 読む章 | 完了条件 | 確認する方法 |
|------|----------|--------|----------|--------------|
| 0 | 準備: `package.json`(`"type": "module"`、`engines`、`bin`、`files`)、`tsconfig.json` と `tsconfig.build.json`(`strict`、`NodeNext`)、ESLint(型情報つき)、Prettier、Vitest、`src/version.ts`、ディレクトリ(02 章 2 章)、最初の試験(4 章) | 02 の 8 章、11 の 8・9 章 | `npm run typecheck`、`npm run lint`、`npm run format:check`、`npm test`(空でも)、`npm run build` が通る。`dist/bin/conductor.js` の先頭が `#!/usr/bin/env node`。4 章の試験 T1〜T3 の結果で、YAML ライブラリの採否が決まっている | AC-476(版の定数)、AC-483(厳格な設定)、AC-505(シバン)、AC-506(`package.json` の形) |
| 1 | 土台: 終了コードの定数と `exitCodeFor`(`src/flow/`)、エラーの型(10 章 8 章)、`src/runid/`、入口 `src/bin/conductor.ts`(`uncaughtException`・`unhandledRejection`・`try/catch` で 70、`process.exit` は 1 か所以下、出力の flush) | 10、05 の 2・6 章、04 の 5 章、08 の 1.3 節 | 終了コードの範囲合わせ(例: 259→3、256→エラーなら 1)、run ID の有効・無効の例(30 文字の日本語が `TooLong`)が、単体テストで通る。偽の失敗(`throw`)で、固定の 1 行と 70 が出る | 単体テスト(10 章 4 章の例、05 章 6.1 節の例)。AC-389〜396(終了コード全件。引数だけで出せる範囲)、AC-489、AC-490、AC-498 |
| 2 | 引数の解析(字句の段階 `lex` と検査の段階)、`--help`、`--version`、使い方の全文(`report` のうち、誤りの行と使い方だけ先に) | 04、08 の 4.2・8 章 | 04 章 4.3 の判定表の全行が、表駆動テストで通る。使い方の誤りは 68 で、誤りの行・空行・使い方の全文が出る。`parseArgs` を使っていない | AC-325〜349、AC-447〜449 |
| 3 | 定義の読み込み: 取り込み(拡張子、大きさ、UTF-8)、`readJson`(自前の JSON の読み取り)、YAML の読み取り(ライブラリ + 追加の実装)、展開と上限、版の確認、構造の検査(Walk)、ジョブ間の検査、位置の出し方、`looseRunIdEnv` | 03、10 の 7.1 節 | 03 章 16 節の受理・拒否の例が、すべて表駆動テストで通る。誤りは位置の順に、種類・行・固定の理由で返る。深い入力でも異常終了しない | AC-001〜075、AC-450〜487(`validate` を動かせる段階 8 以降でも確認)。単体テストでは 03 章 7.5 節の試験、AC-412(深さ 1,000)、AC-500 |
| 4 | `command` の分割と検査(状態機械。UTF-16 のコード単位で走査。再帰を使わない) | 07 の 2〜3 章 | 07 章 3.5・3.6 節の受理・拒否の例が通る。約 30 万引数の入力が `RangeError` にならず分割できる | 単体テスト。AC-076〜098(分割)、AC-409、AC-410 |
| 5 | `src/flow/`(純粋な判断): 分類、遷移と次の動き、起動前の判断、再開位置、リトライの判断と待ち時間(`bigint` の飽和)、記録の関数(`src/state/` の純粋な部分) | 05 | 05 章の判定表と例の表が、すべて表駆動テストで通る。`node:` のモジュールを import していない | 単体テスト(05 章 13 章の「固定する入力」)。AC-268、AC-499 |
| 6 | `src/report/`: メッセージカタログ(英語・日本語)、ロケールの遅延判定、エスケープと切り詰め(コードポイント単位)、`watch` の表、色、使い方の全文 | 08 | 08 章の表の全文が、単体テストで文字単位で一致する。ロケールは最初に必要になったとき 1 回だけ判定する | AC-350〜388(出力とロケール、エスケープ)、AC-170〜187 の表の部分 |
| 7 | `src/state/`: ステートの JSON(固定のキー順)、原子的な書き込み、読み込みの 14 段階の検査、整合性の検査、`reset`、`reset --all` | 06 | 06 章 6.6 節の単体テストの入力が、すべて期待どおり。読み込みは `readJson`(`ignoreBOM: true` のデコードを含む)で行い、`JSON.parse` を使わない | 単体テスト(一時ディレクトリの実ファイル)。AC-302〜324(ステートの異常系)、AC-188〜205(`reset`)、AC-147 |
| 8 | `src/cli/`: `Deps`(02 章 4 章)、前処理 `prepare`、`run`・`pipeline` の流れ、リトライの実行ループ、`watch`・`reset`・`validate`・`convert` の配線、終了コードの決定。プロセスとシグナルは偽の部品で確かめる | 02 の 4・5 章、05 の 5〜11 章、04 の 7 章、10 の 2・8 章 | 偽の部品の上で、`run`・`pipeline` の全分岐(スキップ、拒否、再開、リトライ、中断、ステートの書き込みの失敗)が通る。ステートの書き込みの失敗は、ジョブの終了コードより優先して 65 | AC-117〜187(`run`・`pipeline`・`watch`)、AC-239〜279(run ID、リトライ)、AC-293〜300(注入) |
| 9 | `src/runner/`: `runJob`(`spawn`、`close`、`error`、転送の 1 回)、`Interrupt`、`waitRetry`、シグナル監視、`platform.ts`(`platformFor`、`detectPlatform`) | 07 | 実プロセスのテスト(UNIX 系)と、偽の `ChildLike` の単体テストが通る。Windows の `.cmd` の組み立ては、どの OS でも単体で通る。待機は、合図から 1 秒以内に打ち切れる | AC-099〜116、AC-280〜292、AC-403、AC-502〜504、AC-511、AC-515。AC-301(Windows の Ctrl+C)は実機で手動 |
| 10 | `src/convert/`: YAML ⇔ JSON の変換、JSON の自前の書き出し、YAML の自前の書き出し、表せない値の検出 | 09、03 の 4〜7 章(読み取りを共有) | 09 章 11 節の例が通る。往復で木が等しい。整数は `bigint` のまま通る | AC-206〜238、AC-509、AC-510 |
| 11 | 受け入れテストの全件、CI(GitHub Actions)、リリース(tarball と `SHA256SUMS`)、デモ、README | 11 | 11 章 4 章の「自動」が 3 OS で通る。品質ゲート(5 章)がすべて通る。リリースの手順(6 章)が、ドラフトのリリースまで通る | AC-397〜446、AC-476〜515 |

- 段階 7(`state`)と段階 9(`runner`)は、互いに依存しない(どちらも `flow` の型を使う)ので、別の人が並行して作れる。
- 段階 8 では、プロセスとシグナルを偽の部品(`ScriptProcess`、`FakeSignals`)にして、流れだけを確かめる。実プロセスとシグナルは、段階 9 で確かめる。
- 段階 3 の `readJson` は、段階 7 の読み込みが共用する(`src/definition/index.ts` の公開名。03 章 14.2 節、06 章 6.5 節)。段階 3 を先に終える。
- 各段階の終わりに、`npm run typecheck`、`npm run lint`、`npm test` を通す。循環 import と依存の向きの検査(AC-512)は、段階 1 のうちに CI に入れておくと、後から崩れない。

## 2. Node.js・TypeScript 特有の落とし穴

02〜11 章の「注意」「未確認」から集めた。各項目の末尾に、詳細の章を示す。

### 2.1 標準出力・終了・内部の失敗

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 1 | パイプへの `process.stdout.write()` は非同期のことがある。書き込みの完了前に `process.exit()` を呼ぶと、出力が切り詰められる(5 MB を書いた直後の `process.exit(0)` で、読み手に届いたのは 131072 バイトだけ。確認) | 書き込みは、`callback` つきの `write()` を `Promise` に包んで `await` する。終了の前に、空文字の書き込みを `await` して flush する。`process.exit()` は、入口 `src/bin/conductor.ts` の 1 か所以下で、書き込みの完了を待ってから。終了コードは `process.exitCode` に設定してもよい | 08 の 1.2・1.3 節、10 の 4.1 節、AC-493、AC-498 |
| 2 | 読み手が先に閉じたパイプへの書き込みは、`EPIPE`。`process.stdout` の `'error'` のリスナーがないと、`Unhandled 'error' event` の例外になる(確認) | 起動の最初に、`process.stdout` と `process.stderr` の両方に `'error'` のリスナーを付ける。標準出力の書き込みの失敗は、`watch` が 65、`convert` が 67(`cannot write to standard output`)、`--help`・`--version` は無視して 0。70 にしない | 08 の 1.2 節、10 の 2 章、AC-492、AC-494、AC-513、AC-514 |
| 3 | 標準エラーの書き込みを待たずに子を起動すると、パイプ・ファイルの先で、`conductor` の行と子の出力の順序が入れ替わりうる | 標準エラーの書き込みも `await` する(リトライの知らせ、起動前の行) | 08 の 1.2 節 |
| 4 | 想定外の例外・Promise の拒否は、Node の既定ではスタックを出して終了コード 1(または 7)になる | `process.on('uncaughtException')`・`process.on('unhandledRejection')`・入口の `try/catch` で、固定の 1 行(`conductor: error: internal error`)を標準エラーに出して 70。スタックやメッセージを出さない | 10 の 6 章、AC-489〜491 |
| 5 | 待たない Promise(`await` も `.catch` もない)は、拒否されると `unhandledRejection` になる | ESLint の `@typescript-eslint/no-floating-promises`・`no-misused-promises`・`only-throw-error` を `error` にする。シグナルや子のイベントのハンドラは、同期の関数にして、値を立てるだけにする | 02 の 7 章、AC-421、AC-512 |
| 6 | `await process.stdin` などで標準入力を読み始めると、ジョブに渡すはずの入力を奪う。また、タイマーやリスナーが残ると、プロセスが終わらない | `conductor` は標準入力を読まない。終了前にシグナルのハンドラを解除し、タイマーは `finally` で `clearTimeout` する | 07 の 5・7 章、AC-495、AC-507 |

### 2.2 子プロセス(`node:child_process`)

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 7 | `spawn` が失敗しても、同期的には例外にならず、`error` イベントが出る。そのあとで `close` が、負の終了コード(`-errno`)で出る(実測) | `error` のハンドラを `spawn` の直後(同じ同期の区間)に付ける。起動前の `error` を起動失敗として確定し、後から届く負の `close` を無視する(`settled` の印)。`spawn` 成功後の `error` でも、ハンドラは外さない | 07 の 4.3.1 節 |
| 8 | `exit` と `close` は違う。`close` は、標準入出力のストリームが閉じたあと | `stdio: 'inherit'` なので、`close` は `exit` の直後に出る。孫のプロセスが残っていても待たない(実測)。終了の観測は `close` で行い、`exit` は「転送の対象から外す」印に使う | 07 の 4.3.2 節、AC-504 |
| 9 | `child.kill(sig)` の戻り値 `true` は、「送れた」であって、「子が死んだ」ではない。送った直後の `child.exitCode` は `null` のまま(実測) | 戻り値で終了を判断しない。`close` を待つ。2 回目を送らないのは、`conductor` の責任(`forwarded` の印) | 07 の 4.3.4 節、AC-503 |
| 10 | Windows の `child.kill` は、子を強制終了する(文書の記述。実測していない)。`detached: true` は、新しいコンソールを作る(同) | Windows では `child.kill` を呼ばず、`detached` も指定しない(OS が Ctrl+C を同じコンソールの両方に配る)。UNIX 系だけ `detached: true`(新しいプロセスグループ) | 07 の 4.6・7.3 節 |
| 11 | pid を自分で保持して `process.kill(pid)` に渡すと、子の回収後の pid の再利用で、無関係なプロセスにシグナルを送りうる | 必ず `child.kill` を通す。許可集合(1・2・3・15)以外は送らない。pid が正の整数のときだけ | 07 の 5.2・7.2 節、AC-286、AC-287 |
| 12 | Windows で `.cmd` / `.bat` を `shell: false` で直接 `spawn` すると、Node の版によっては `EINVAL` で失敗する(2024 年の対応。境界の版は未確認)。`shell: true` にすると、引数がシェルに解釈される | `cmd.exe /d /s /c "…"` を `shell: false`・`windowsVerbatimArguments: true` で起動する。引数に禁止文字(`"` `%` `^` `&` `\|` `<` `>` `!` `(` `)` 改行 復帰 NUL)があれば、起動せず 127。組み立ては `platform.ts` の `prepareSpawn` の 1 か所 | 07 の 4.7 節、AC-113〜116、AC-486、AC-508、AC-511 |
| 13 | シグナルで終了した子は、UNIX 系で `code` が `null`、`signal` が名前(`'SIGTERM'`)。名前から番号への対応は OS で違う(`SIGUSR1` など) | 固定の表を書かず `os.constants.signals` を使う。両方 `null` なら起動失敗(127)。Windows は、終了コードを `code \| 0` で符号つきにする | 07 の 4.3.3 節、AC-502 |
| 14 | 引数に NUL を含む、実行ファイル名が空、などで、`spawn` が同期的に `TypeError`(`ERR_INVALID_ARG_VALUE`)を投げる(実測) | `try/catch` で受けて、起動失敗(127)にする | 07 の 4.1 節 |
| 15 | 版管理の道具(Volta など)の shim を介して Node を起動すると、Node 自身がシグナルで死んだとき、シェルから見た終了コードが 143 でなく 1 になる(実測) | シグナルの終了コードを検査するテストは、Node の実行ファイルを直接起動する | 07 の 1.2 節 |
| 16 | `AbortSignal`(`spawn` の `signal`)や `timeout` オプションは、子を強制終了する | 使わない。中断は、自前の `Interrupt`(`raise`・`number`・`whenRaised`)で伝える | 02 の 7.1 節、07 の 4.2 節 |

### 2.3 シグナルとタイマー

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 17 | `setTimeout` に 2^31 − 1 ミリ秒を超える値(`Infinity`、`NaN` を含む)を渡すと、`TimeoutOverflowWarning` が出て、待ち時間が 1 ミリ秒になる(確認)。長い待ちが一瞬で終わる | 1 回のタイマーを 2^30 ミリ秒以下に分け、経過を単調な時計(`process.hrtime.bigint()`)で再確認する。`Date.now()` を使わない | 05 の 7.3 節、07 の 7.5 節、AC-501 |
| 18 | `Promise.race` でタイマーと合図を競わせると、負けたタイマーが残って、プロセスの終了が遅れる | `finally` で必ず `clearTimeout` する。合図から 1 秒以内に打ち切る。ポーリングをしない | 07 の 7.5 節、AC-275 |
| 19 | `process.on('SIGTERM', …)` のリスナーを、すべて外すと、そのシグナルは OS の既定の動作(終了)に戻る(実測)。リスナーだけでは、プロセスの終了を妨げない(実測) | `watch`・`reset`・`validate`・`convert` では登録しない。登録は `run`・`pipeline` で、プロセスで 1 回だけ。テストでは `stop()` で外す | 07 の 7.1・7.2 節、AC-285、AC-297 |
| 20 | シグナルのハンドラが、`await` の途中に割り込んで動く | ハンドラは同期で、`raise` を呼ぶだけ(出力、ステートの書き込み、`child.kill` を呼ばない)。有無と番号は、1 つの同期関数の中で一緒に更新する | 07 の 6.2・7.1 節 |
| 21 | ハンドラの第 2 引数(番号)が、すべての OS・版で渡されるかは未確認 | 名前から固定の表(`SIGHUP`→1、`SIGINT`→2、`SIGQUIT`→3、`SIGTERM`→15)で番号を引く | 07 の 7.2 節 |
| 22 | `nohup` で起動して SIGHUP が無視されていても、`process.on('SIGHUP')` のリスナーは受け取る(実測) | 4 つのシグナルを必ず受けることを要求する。Windows は `SIGINT` だけ(`SIGBREAK`・`SIGHUP` は対象外) | 07 の 7.2・7.3 節、AC-515 |

### 2.4 JSON と数値

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 23 | `JSON.parse` は、重複キーを黙って後勝ちにし、巨大な整数を倍精度に丸め(`9007199254740993` → `9007199254740992`)、`1.0` と `1` の区別を失い、入れ子の深さに上限がなく、位置を返さず、単独のサロゲートを通す。`"__proto__"` を代入でコピーするとプロトタイプが汚染される | 製品のコードで `JSON.parse` を使わない。自前の `readJson`(明示のスタック、エントリの配列、整数は字句から `bigint`)で読む。`JSON.parse` を使ってよいのは、テストと開発用のスクリプトだけ | 03 の 5.3 節、06 の 6.5 節、AC-421、AC-461〜468、AC-510 |
| 24 | `JSON.stringify` は、キーを挿入順に出すが、整数に見えるキー(`"1"`、`"2"`)は先頭に昇順で並べる(確認)。`undefined` のプロパティは出さない。U+2028・U+2029 をエスケープしない。`-0` は `0` | ステートは、キーを固定の順に 1 つずつ代入した型を `JSON.stringify(…, null, 2)` で出し、U+2028・U+2029 を ` `・` ` に置換する。変換の JSON の出力には `JSON.stringify` を使わず、自前で書く(U+007F〜U+009F などをエスケープする規定のため) | 06 の 4.3 節、09 の 7 章、AC-509 |
| 25 | `number` は 2^53 を超える整数を表せない。`Number(2n ** 63n)` は丸められる。`&` は 32 ビット符号付きに変換する | リトライの待ち時間(64 ビット符号なしの飽和演算)は `bigint`。範囲外の整数の検査は、字句から `bigint` で行い、`number` に落とさない。終了コードは `number`(`reported & 0xFF` は、32 ビット符号付きの範囲内でだけ使う) | 05 の 0.2 節、03 の 5.1 節、AC-499、AC-500 |
| 26 | オブジェクト(`{}`・`Record`)にマップを入れると、キー `__proto__` が特別に扱われ、整数に見えるキーが並べ替えられ、`in` 演算子が `toString` に真を返す | 木のマップは `entries` の配列で持つ。キーの検索は一時的な `Map`、許されたキーの判定は `Set`。`in`・`hasOwnProperty` を使わない | 03 の 6.1 節、09 の 4.4 節、AC-509、AC-510 |

### 2.5 文字列と Unicode

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 27 | `String.prototype.trim()` は、U+FEFF を取り除き、U+0085 を取り除かない(確認)。仕様の「空白」(Unicode の White_Space)と違う | 自前の関数(`/^\p{White_Space}+\|\p{White_Space}+$/gu` で置換)を使う。製品のコードで `trim()` を使わない。列挙した文字の単体テストで確かめる | 03 の 1 章、07 の 2 章 |
| 28 | `s.length` は UTF-16 のコード単位の数。文字数、バイト数、書記素の数と違う。`"あ".repeat(30)` は `.length` が 30 で、UTF-8 では 90 バイト | 「文字」は、`for…of` か `Array.from` で数えたコードポイント。バイト数は `Buffer.byteLength(s, 'utf8')`。run ID の長さの上限(64 バイト)、エスケープ後の切り詰め(コードポイント)に注意。サロゲートペアの途中で切らない | 03 の 1 章、05 の 6.1 節、08 の 11 章 |
| 29 | Node は、`process.argv` と `process.env` の不正な UTF-8 のバイトを U+FFFD に置き換えて渡す(macOS で確認)。元のバイト列は取り戻せない | 「非 UTF-8 を理由にした `invalid value`」を実装しない。U+FFFD を含む引数は、普通の文字列として扱い、エスケープ(08 章 9 章)を通して出す。元のバイト列を持つファイル名は開けない(制約として受け入れる)。Windows の不正な UTF-16 は未確認 | 04 の 4.4 節、10 の 7.4 節、AC-349、AC-406 |
| 30 | `TextDecoder('utf-8')` の既定は、先頭の BOM を黙って取り除く。`fatal: false` では、不正なバイトが U+FFFD に置換される | `new TextDecoder('utf-8', { fatal: true, ignoreBOM: true })`。BOM の扱いは、定義と変換では受理(取り除く)、ステートでは拒否 | 03 の 3 章、06 の 6.1.2 節、09 の 3 章 |
| 31 | `path.extname('.yaml')` は空文字列を返す(確認)。拡張子の大文字小文字を `toLowerCase()` で比べると、意図しない文字が一致しうる | 定義ファイルの拡張子は、03 章 2.1 節の規則で自前に判定する(ASCII の大文字小文字を区別しない) | 03 の 2.1 節 |
| 32 | 対にならないサロゲート(`"\ud800"`)は、`JSON.parse` も YAML ライブラリも受理して、単独のサロゲートを含む文字列を返す。そのままでは、出力が正しい UTF-8 にならない | 定義・変換の読み取りで `syntax` として拒否する(YAML は展開後の文字列を検査)。ステートの文字列には現れない | 03 の 5.3・7.4 節、09 の 4.1 節 |

### 2.6 YAML ライブラリ

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 33 | ライブラリの既定の設定は、この仕様と違う(型の解決、マージ、重複キーの検査、警告の出力、アンカーの展開) | `parseAllDocuments`、`version: '1.2'`、`schema: 'failsafe'`(型は自前で決める)、`uniqueKeys: false`、`merge: false`、`lineCounter`、`logLevel: 'silent'`。`toJS()`・`toJSON()` を呼ばない。ライブラリの警告・エラー文を利用者に見せない | 03 の 7.3 節、AC-496 |
| 34 | 重複キーの検査をライブラリに任せると、10 万個のキーで約 78 秒かかる(確認)。1 つのノードにアンカーを何万個も並べた入力で、文書の組み立てが約 29 秒かかる(入力の大きさの 2 乗。確認) | 重複キーは自前(マップごとの `Set<string>`)。ライブラリで文書を作る前に、`Parser` の出力を明示のスタックで走査して、1 つのノードに 2 個以上のアンカーまたはタグがあれば `syntax`(前処理)。30 秒以内に終わることを、敵対的な入力で測る | 03 の 7.4 節 #4・#13、7.5 節、AC-224〜232 |
| 35 | 深い入れ子(約 1,700〜1,900 段以上)で、ライブラリは例外を投げず、文書の `errors` に `RESOURCE_EXHAUSTION` を入れる(確認)。未定義のエイリアスは、パースではエラーにならない | `errors` が空でなければ `syntax`。ライブラリの呼び出しを `try/catch` で囲み、`RangeError` も `syntax` にする。エイリアスの展開(未定義・前方参照・再帰・上限)は自前で、明示のスタックで行う | 03 の 6.4 節、7.4 節 #6・#11、AC-232、AC-412 |
| 36 | 複数文書は、`parseDocument` では `MULTIPLE_DOCS` のエラーになり、行が得にくい。明示のタグは、ノードの `tag` に入るが、ライブラリは拒否しない | `parseAllDocuments` で全文書を数える。タグは各ノードの `tag !== undefined` で検出する。`yes`・`no`・`on`・`off`、`0777`、`0o17` などの型は、自前の規則(03 章 4.2 節)で決める | 03 の 7.3・7.4 節 |
| 37 | ライブラリの版で、挙動・位置・オプションが変わりうる。先行版(`next`)は安定版でない | 正確な版を固定する(`^` を付けない)。採用する版で、03 章 7.5 節の試験をやり直す。更新のたびに再実施する | 03 の 7.2・7.5 節、11 の 8.4 節 |

### 2.7 ファイルとステート

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 38 | `fs.readFile` は、上限なしで全部を読む。特殊ファイル(`/dev/zero`、FIFO)で止まらない。`FileHandle.read` の位置に `0` を渡すと、FIFO で `ESPIPE` になる(確認) | `fs.open` した `FileHandle` の `stat()` で大きさを見たうえで、上限 + 1 バイトまで、位置 `null` で読む。`finally` で `close()` する | 06 の 6.1.1 節、03 の 3 章、09 の 3 章 |
| 39 | 一時ファイルの作成は、`"wx"`(既存なら `EEXIST`)で行う。`fs.writeFile` や `rename` だけでは、読み手が壊れた途中の内容を見うる | 一時ファイルに書く → 同期(`FileHandle.sync()`)→ 同じディレクトリで `rename` → UNIX 系はディレクトリを開いて同期(失敗は無視)。失敗時は、自分が作った一時ファイルだけ消す | 06 の 5 章 |
| 40 | `fs.unlink` は、ディレクトリに失敗する(macOS: `EPERM`。Linux: `EISDIR` と理解している。Windows は未確認)。`fs.rm`・`fs.rmdir` は、空のディレクトリを消しうる | `reset` は `fs.unlink` だけを使い、`ENOENT` だけを成功とする。ディレクトリを指すパスは消さず `removeFailed`(65) | 06 の 8.1 節 |
| 41 | `fs.readdir` の順序は保証されない。`withFileTypes` の `isFile()` が、シンボリックリンクを区別する | `reset --all` は、名前を `sort()` し、`fs.lstat` で通常のファイルだけを対象にする | 06 の 8.2 節 |
| 42 | Windows の `fs.rename` は、既存のファイルへの置き換えや、他のプロセスが開いているファイルで、失敗しうる(未確認) | 失敗は `writeFailed`(65)。再試行しない。ディレクトリの同期はしない | 06 の 5.1 節 |
| 43 | Linux で、書いた直後のスクリプトを起動すると `ETXTBSY` で失敗することがある(一般に知られた事象。Node での発生は未確認) | テストのスクリプトは、`test/jobs/` に事前に置くか、閉じてから起動する | 07 の 11 章 |

### 2.8 ESM・TypeScript・ビルド

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 44 | `NodeNext` の ESM は、相対 import に拡張子 `.js` が要る。`verbatimModuleSyntax` では、型だけの import に `import type` が要る。`dist/` には `.js` が出る | `"module": "NodeNext"`、`"type": "module"`、相対 import は `.js` つき、型は `import type`。`tsc` は先頭のシバンを残すこと(実装時に確認) | 02 の 8 章、11 の 9.3 節、AC-505 |
| 45 | `any`、`as`、`!`(非 null 表明)、`@ts-ignore` は、外部の入力の誤りを型の検査から隠し、実行時の例外を生む | 製品のコードで使わない(`unknown` から絞り込む)。`noUncheckedIndexedAccess` の下で、添字の結果の `undefined` を検査してから使う。`exactOptionalPropertyTypes` が煩雑なら、緩めてよい(`strict` は緩めない) | 02 の 8.2・10.2 節、AC-421、AC-483 |
| 46 | `Error` を継承したクラスの `instanceof` は、`target` が古い(ES5)と壊れる | `target: ES2022` 以上(この仕様は `ES2023`)。`catch (e)` の `e` は `unknown` なので、`instanceof` で絞り込む | 10 の 8.1 節 |
| 47 | `package.json` の `engines` と CI の Node の版が食い違う。`@types/node` の主版が、最小の Node より新しいと、最小の Node にない API を型が通してしまう | `engines` の最小版を、CI の `NODE_MIN` と README に揃える(検査スクリプト)。`@types/node` の主版を最小の Node に合わせる | 11 の 5.4・8.3 節 |
| 48 | `exports` に `package.json` だけを書くと、外からの import が拒否される。`files` を絞らないと、`src/`・`test/` が配布物に入る | ライブラリとして公開しない。`files` は `["dist", "LICENSE", "README.md"]`。`npm pack --dry-run` で確かめる | 02 の 8.1 節、11 の 6.1 節、AC-422、AC-440 |
| 49 | 循環 import は、`tsc` を通っても実行時に `undefined` を生みうる | `index.ts`(バレル)経由の import だけにする。循環の検査と `no-restricted-imports` を CI で実行する | 02 の 3 章、AC-512 |
| 50 | `process.platform`・`os.platform()` を、あちこちで読むと、OS 差の置き場所が散る | `src/runner/platform.ts` の 1 か所だけ。ほかは `PlatformKind` を引数で受け取る。OS 名の文字列(`'win32'` など)も、ほかに書かない | 07 の 8.2 節、AC-420 |

### 2.9 テストと CI

| # | 落とし穴 | 対処 | 章 |
|---|----------|------|----|
| 51 | テストが `dist/` を使うのに、ビルドが古い | Vitest の `globalSetup` で `tsc` を実行する。受け入れテストは、`dist/bin/conductor.js` を `node` で起動する | 11 の 3.3・3.9 節 |
| 52 | 起動したテストが他のテストとファイル・プロセスを共有する | `pool: 'forks'`。一時ディレクトリ(`mkdtemp`)で定義とステートを作り、終了後に削除する | 11 の 3.8・3.9 節 |
| 53 | UNIX 系だけ(シグナル、権限、シンボリックリンク、プロセスグループ)、Windows だけ(`.cmd`)、Linux だけ(`/dev/full`、`/proc`)のテストが他の OS で失敗する | `describe.skipIf(process.platform === 'win32')` などで囲む(テストでは `process.platform` を読んでよい) | 11 の 3.10 節 |
| 54 | Windows の Ctrl+C は、CI のランナーに対話的なコンソールがなく、自動化できない | 実機で手動(チェックリスト)。結果を記録する(AC-301) | 07 の 7.3 節、11 の 4.10 節 |
| 55 | 疑似端末は、Node の組み込みだけでは作れない | 端末の判定を `Deps` の関数にして、偽の `Deps` で検査する(AC-182 の注入の併記)か、`script` コマンドを使う | 11 の 10.1 節 |
| 56 | 起動時間・CPU 時間の基準は、環境で揺れる | 起動の計測は目安(警告のみ)。待機中の CPU 時間(AC-403)は強制(Linux。`/proc` を読めなければスキップ) | 11 の 2.1 節、AC-403、AC-488 |

## 3. 未確認事項の一覧

作成時に確かめられなかった点を、章ごとにまとめた。いずれも、実装時に、採用する Node.js の版・ライブラリの版・実機で確かめる。確かめた結果は、該当章の規定を直すか、実装の判断として記録する。

「確認」「実測」とある挙動は、Node.js v26.13.0(macOS)で確かめた範囲である。**最小の LTS 系列の Node、最新の LTS 系列の Node、Linux、Windows での再確認**は、全章に共通の未確認である(以下の表には、章ごとに追加のものだけを書く)。

| 章 | 未確認の点 |
|----|------------|
| 02 | サポートする Node の版(最新の LTS 系列と 1 つ前の LTS 系列の具体的な番号)。循環の検査と import の制限の道具(`madge`、`eslint-plugin-import`、`no-restricted-imports`)の選定と設定。`tsc` が先頭のシバンを残すこと、`bin` の実行権限、Windows の `bin` のラッパー。端末の判定(`process.stdout.isTTY`)が UNIX 系と Windows で期待どおりか。`exports` に `package.json` だけを出したときの `npm pack`・`npm install -g`・`npx` の動作。`exactOptionalPropertyTypes` と `noUncheckedIndexedAccess` の併用の複雑さ。`satisfies` による偽実装の検査の書き方 |
| 03 | YAML ライブラリ(`yaml` 2.9.1 で確認したもの)の、採用する版・Windows・LTS 系列での挙動(03 章 7.5 節の試験を再実施)。`js-yaml` の位置・木の API。前処理が正しい入力を誤って拒否しないこと(列挙した入力に限る)。引用符なしのスカラー中の制御文字、ディレクティブ、CR だけの改行の細部。2 つ目の文書の `contents` が `null` になる入力。複数行の引用符なしのスカラーの、文字の大きさの境界。YAML の文法の誤りの行の厳密な規則。Windows など他の OS でのエラーの `code`、権限・シンボリックリンクの `unreadable` の判定。`\p{White_Space}` が他の Node の版で同じ集合か |
| 04 | Windows の `process.argv` の不正な UTF-16。Linux・Windows での U+FFFD への置換。`parseArgs` の挙動(v26.13.0 だけで確認)。`=` を含む連結の短縮形の細部。元のバイト列のファイル名を `Buffer` のパスで開く方法。Windows の `process.env` の大文字小文字。`reset --all` の対象の細部 |
| 05 | Windows での、実行中ジョブへのシグナル転送の細部と、`exitCode` が負になる場合の実機の値。`process.env` の大文字小文字と、空の値の環境変数の見え方。タイマーが目標の時刻よりどれだけ早く発火しうるか(経過の再確認で、この量に依存しないようにしてある)。ほぼ同時の複数のシグナルの順序 |
| 06 | Windows の `fs.rename`(既存ファイルへの置き換え、他のプロセスが開いているとき)、`path.win32`、ディレクトリへの `fs.unlink`、ディレクトリを開いたときの `unreadable`、ディレクトリの `sync`、ACL。Linux の、ディレクトリへの `fs.unlink` の `EISDIR` と、ディレクトリを開いて読んだときの `unreadable`。`FileHandle.writeFile` が部分的な書き込みを書き切ること。`readdir` の `withFileTypes` が種別を返さないファイルシステム。`JSON.parse` の reviver がソースの字句を渡す機能の有無(採否に影響しない)。クロックが戻ったときの一時ファイル名(衝突の再試行 16 回に任せている) |
| 07 | Windows の実機: Ctrl+C の終了コード 130、`.cmd` の起動(`cmd.exe /d /s /c` の形、`ComSpec`)、`close` の終了コードの符号、Ctrl+Break とコンソールを閉じたときの通知、プログラムの探索、`child.kill` が子を強制終了すること、`detached: true` が新しいコンソールを作ること。`.cmd` を直接 `spawn` したときに `EINVAL` になる Node の版の境界。Linux の `EACCES` の `close` の値、`detached: true` による `PGID`、シグナル名と番号の対応。`detached: true` による制御端末の喪失の影響。`child.kill` と子の回収・`exit` の配送の間に割り込めるか。`process.on` が例外を投げる条件。ほぼ同時の複数のシグナルの順序。`spawn` に約 30 万個の引数を渡したときの失敗。標準出力がパイプのとき書き込みが同期か。子での `SIGPIPE` の扱い。Linux の `ETXTBSY` |
| 08 | Linux・Windows での 1.2 節の確認(5 MB のパイプ書き込みと `process.exit()`、`EPIPE`、空の書き込みの完了待ち)。Windows のパイプ・端末への書き込みが同期か。子の `SIGPIPE` の既定への復帰(Linux・Windows)。端末(疑似端末)で `isTTY` が真になること。`String.prototype.toWellFormed()` が最小の LTS 系列にあること |
| 09 | YAML の受理する文法の細部(03 章と同じ)。YAML の文法の誤りの行番号の厳密な規則。`---` だけの文書の `rootNotMap` の行。マージで取り込まれる値の中の表せない値の場所(この仕様の決定) |
| 10 | YAML の文法の誤りの行番号。Windows で子の終了コードが Node から渡るときの符号、`process.exitCode` に範囲外の整数を渡したときの Windows での切り詰め。`uncaughtException` のハンドラが戻った場合の続行と、ハンドラ自身が例外を投げたときの終了コード 7(文書の記述のみ)。`spawn` 失敗時の `error` と `close` の順序(Linux・Windows)。メモリ不足や V8 の致命的な異常終了(対象外)。ESLint の規則名と設定 |
| 11 | ブランチ保護の設定。網羅率と各性能の実測値。npm、`tsc`、ESLint、Prettier、Vitest、GitHub Actions を実際に動かしての確認全般(`setup-node` の `lts/*` と `cache: npm`、`npm pack` の出力の形、`npm audit` の終了コード、Vitest の `coverage.thresholds` と `pool`、ESLint のフラットな設定)。`tsc` の出力が 3 OS で同じ結果になること。Windows の実機での `.cmd` の起動(AC-113〜115、AC-486、AC-508)と Ctrl+C の手動確認(AC-301。記録は存在しない)。Linux での `detached` と `ps -o pgid=`、`/proc/<pid>/stat`(AC-288、AC-403)。`TJ` が SIGTERM・SIGINT・SIGHUP で「シグナルで終了」と観測されること、SIGQUIT の既定の動作。疑似端末の作り方(AC-182、AC-183)。`EPIPE` を確実に起こす出力の大きさ(AC-492、AC-514)。単一の実行ファイルの配布(任意)。`npm publish` の細部 |

## 4. 最初に確かめる小さな試験

大きな実装に入る前に、次の小さな試験を、使い捨てのプログラムで行う。結果によって、設計の判断(ライブラリの採否、前処理の要否、実装の方式)が変わるものを先に並べた。結果は、README か実装のコメントに記録する。

| ID | 確かめること | 方法 | 合格の基準・結果の使い道 |
|----|--------------|------|----------------------------|
| T1 | YAML ライブラリの採否(最重要) | 採用案の `yaml`(最新の安定版。先行版は使わない)で、03 章 7.5 節の 10 項目と 16 節の例を、使い捨てのテストにする: (1) 4.1 節の「受理」の構文がすべて読める。(2) スカラー・マップ・リスト・キー・根に付けたタグを検出できる。(3) `parseAllDocuments` で複数の文書を数えられる。(4) エイリアスが参照のまま残る。(5) 各ノードの行・列が取れ、位置が 13.2 節の表と合う。(6) 文法の誤りの行が 13.4 節の表と合う。(7) 1 MiB の `[` の連続と `- ` の連続が、プロセスを落とさず `errors` で返る。(8) 重複キー・文字列でないキーを拒否しない(自前で検出する前提)。(9) 敵対的な入力(10 万個のキー、1 MiB の `&a ` の連続など)が、前処理と `uniqueKeys: false` で 30 秒以内(目標 5 秒)に終わる。(10) 警告を標準エラーに出さない。(11) 版・保守の状況・`npm audit` | 基準を満たせばその版を固定する(`^` なし)。満たさなければ、前処理・後処理の補完(03 章 7.4 節)か、別のライブラリに替える。位置が合わなければ、補正の実装(03 章 13.2 節)を決める |
| T2 | 最小の Node と最新の LTS の Node での、【実測】の再確認 | 03・05・06・07・08 章の「確認」「実測」のうち、設計を左右するもの(`setTimeout` の上限、`child.kill` の戻り値、`close` と `exit`、`error` のあとの負の `close`、`stdout` の `EPIPE`、`TextDecoder`、`JSON.stringify` のキー順、`trim()` の差)を、最小版と最新版の両方で動かす | 一致すれば、そのまま。違えば、該当章の規定を直すか、版ごとの対処を決める。`engines` の最小版の根拠になる |
| T3 | 標準出力の flush と `EPIPE` | UNIX 系で、5 MB を `process.stdout.write` した直後の `process.exit()`(切り詰め)と、書き込みの `callback` を待ったあとの `process.exit()`、空文字の書き込みでの flush、読み手が閉じたパイプへの書き込み(`callback` の `EPIPE`、`'error'` のリスナーの有無)。Linux・Windows でも | 08 章 1.2・1.3 節の方式(callback を待つ、リスナーを最初に付ける)を確定する。切り詰めが起こらない OS でも、方式は同じにする |
| T4 | `child_process` の基本 | シグナルで終わるジョブの `close` の `(code, signal)`、存在しないプログラムの `error` と負の `close`、実行権限がない場合、`child.kill` の戻り値と `exitCode`、`os.constants.signals` の番号、`detached: true` の `PGID`(Linux でも)、NUL を含む引数の `TypeError`、孫を残した `close` の時刻 | `runJob` の設計(07 章 4・5 章)を確定する。Linux の値の差を記録する |
| T5 | タイマー | `setTimeout` に 2^31 を渡して `TimeoutOverflowWarning` と 1 ミリ秒を確認。2^30 ミリ秒ずつの分割と、`Promise.race`・`clearTimeout` で、合図から 1 秒以内に打ち切れ、タイマーが残らないか | `waitRetry`(07 章 7.5 節)の設計を確定する。AC-501、AC-275 の期待を確かめる |
| T6 | シグナルのリスナーと、無視されて起動された SIGHUP | `process.on` のあと `process.off` で SIGTERM を送り、終了コード 143 になるか。`nohup` 経由、またはバックグラウンドで起動したプロセスで、4 つのシグナルを受けられるか。ハンドラの第 2 引数 | 受けられなければ、入口で明示的に再設定する方法を決める(07 章 7 章)。リスナーの解除の規則を確定する |
| T7 | `readJson` と `JSON.parse` の違い | 自前の `readJson` に、重複キー、巨大な整数、`1.0`、深さ 100,000 の入れ子、単独のサロゲートのエスケープ、`__proto__` のキー、BOM、空白だけの入力を渡す。`JSON.parse` との比較 | 03 章 5.3 節、06 章 6.6 節の期待が通る。深い入力でも異常終了しない(明示のスタック) |
| T8 | 文字列の基本 | `String.prototype.trim()` と `\p{White_Space}` が、03 章 1 章の空白の集合(U+0009〜U+000D、U+0020、U+0085、U+00A0、U+1680、U+2000〜U+200A、U+2028、U+2029、U+202F、U+205F、U+3000)と一致するか(`trim()` は一致しないはず)。`Buffer.byteLength`、`for…of`、`String.prototype.isWellFormed`・`toWellFormed`、`Number('1e-400')`、`path.extname('.yaml')` | 自前の空白の関数を確定する。極小値の扱いを記録する(03 章 4.2 節) |
| T9 | ファイル操作 | `fs.open` の `"wx"` と `EEXIST`、`FileHandle.read` の位置 `null`(FIFO と `/dev/zero`)、`sync`、ディレクトリを開いて `sync`、既存ファイルへの `fs.rename`(Windows の CI。他のプロセスが開いているとき)、ディレクトリへの `fs.unlink`(Linux・Windows)、`readdir` の `withFileTypes` | 06 章 5 章・8 章の方針(失敗は `writeFailed`・`removeFailed`、再試行なし)を確認する |
| T10 | Windows の `.cmd`(CI の Windows ランナー) | `cmd.exe /d /s /c "…"` を `shell: false`・`windowsVerbatimArguments: true` で起動。禁止文字を含む引数が拒否されるか、含まない引数で終了コード 7 が取れるか。最小の Node と最新の Node で。`.cmd` を直接 `spawn` したときに `EINVAL` になる版 | 07 章 4.7 節の方式と禁止文字の集合を確定する。AC-113〜115、AC-486、AC-508 の期待を確かめる |
| T11 | 引数と環境変数の非 UTF-8 | Linux でも、`process.argv` と `process.env` の不正なバイトが U+FFFD になるか(04 章 4.4 節の 3 件)。Windows の不正な UTF-16 | 04 章 4.4 節の規定(`invalid value` を作らない)が、すべての OS で成り立つか確かめる |
| T12 | 内部の失敗の捕捉 | `uncaughtException`・`unhandledRejection` のハンドラで 70 が出るか。ハンドラが戻ったあと、プロセスが続行しないか(`process.exit(70)` を呼ぶ)。ハンドラ内の例外 | 入口の設計(10 章 6 章)を確定する。AC-489〜491 の外から起こす方法(`--import` の試験用スクリプト)を確かめる |
| T13 | 深い入力に対する自前の部品 | 自前の JSON の読み取り、木の走査、分割器、書き出しに、深さ 100,000 の入力、30 万引数の `command` を渡す(スタックを使い果たさないか) | 異常終了せず、定められた誤りで返る(03 章 6.4 節、AC-232、AC-410、AC-412) |
| T14 | CI の道具 | `actions/setup-node` の `lts/*`、`cache: npm`。Vitest の `coverage.thresholds`・`pool: 'forks'`・`globalSetup`。ESLint のフラットな設定と `recommendedTypeChecked`。循環の検査の道具。`npm pack --dry-run --json`。`npm audit` の終了コード | 11 章 5・9 章の記述を、実際の版に合わせて直す |
| T15 | 端末の判定と色 | `process.stdout.isTTY` が、端末・パイプ・ファイル・`/dev/null`(UNIX 系、疑似端末つき)、Windows のコンソールで期待どおりか。疑似端末の作り方 | 08 章 6.3 節の方式を確定する。AC-182、AC-183 の方法を決める(11 章 10.1 節) |
| T16 | 標準出力の書き込みの失敗の実機 | Linux の `/dev/full` に標準出力をつないで `watch`・`convert` を動かす。`| head -c 1` で `--help`、`watch`、`convert` | 65、67、0 になり、70 にならない。偽の部品での検査(AC-186、AC-236)と、実機の結果が一致することを記録する |

T1〜T3 は、段階 0 のうちに行う。T4〜T9 は、該当する段階(3、7、9)の最初に行う。T10〜T16 は、段階 9〜11 で、CI を組むときに行う。

## 5. 完了の判断

- 11 章 4 章の受け入れテストのうち、「自動」の区分が、3 OS で通る(OS の区分の項目は、その OS で)。
- 「手動」(AC-301)は、Windows の実機で行い、記録を残す。「点検」(AC-116、AC-419〜423、AC-443〜445、AC-476〜481、AC-483、AC-512)は、CI で通す。
- 「計測」(AC-488 ほか)は、目安として記録する(強制の項目は、その基準を満たす)。
- 品質ゲート(11 章 5 章)、最小の Node と最新の LTS の Node でのテスト、`engines` と CI の Node の版の一致の検査が通る。
- 3 章の未確認事項のうち、確かめられたものは、該当章の記述(または実装の判断の記録)を直し、確かめられないものは、「未確認」のまま残す。
