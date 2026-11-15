# Observability Design(u3-cli)

上流: observability-requirements(NFR6.1〜NFR6.4)、security-design、functional-spec、rules(BR6.1〜BR6.6、BR7.1)、Refined Mockups。`conductor` は、メトリクス、構造化ログ、分散トレース、アラート、ダッシュボード、SLI/SLO、相関 ID、利用状況の収集を持たない。観測は、**標準出力・標準エラーの内容、終了コード、`watch` の表示**だけで、これらを、既存のスケジューラのログが拾う。ここでは、表示の設計(Reporter)を示す。

## 表示の部品(Reporter)

| 部品 | 責務 |
|------|------|
| DisplayEvent | 表示する内容(種類とデータ)。流れ(Cli)が作る |
| Locale(遅延) | (インスタンスごとの遅延のセル。プロセスで共有する静的な `OnceLock` は使わない。`Env` を差し替える同じプロセスのテストが、互いに影響しないように) `LC_ALL`、`LC_MESSAGES`、`LANG` の順で、`ja` で始まれば日本語、それ以外は英語。最初の表示のときだけ判定し、結果を保つ(NFR1.4、NFR6.3) |
| Messages | 英語と日本語の対の、定数の表。種類ごとに、固定の文(入力の値は、埋め込み用の穴に、エスケープして入れる) |
| Escape | 入力に由来する文字列の、エスケープと切り詰め(security-design の 2。NFR4.2) |
| Color(遅延) | 端末(`IsTerminal`)で、UNIX 系で、`NO_COLOR` が未設定または空のときだけ、状態の列に色をつける(NFR6.1)。Windows は、色なし |
| Sink | 標準出力・標準エラーへの書き込みの出口(trait。テストで差し替える。Q2: A)。書き込みの失敗は、値で返す |

## 表示の種類と出力先(NFR6.2、NFR6.4)

| 種類 | level | 出力先 | 条件 |
|------|-------|--------|------|
| definitionErrors、commandErrors、usageError、stateError、consistencyMismatch、rejected、startFailure、successCodeNotListed、conversionError | error | 標準エラー | 常に |
| jobResult、jobSkipped、transition、interrupted | verbose | 標準エラー | `-v`(run・pipeline)のとき |
| watchTable、watchEmpty | output | 標準出力 | watch |
| (convert の変換結果) | output | 標準出力 | convert の成功のとき。表示ではなく、結果の出力 |

- **形式(イメージ。Code Generation で確定し、テストで固定する):**
  - エラー: `conductor: error: <場所>: <種類の文>: <原因>`(1つの誤りにつき1行。定義の誤りは、すべて、1行ずつ)。
  - `-v`: `[conductor] job=<id> result=<success|warning|error> (exit code <n>)`、`[conductor] job=<id> skipped (<理由>)`、`[conductor] transition: <結果> -> <遷移先> (skipped: <id>, ...)`、`[conductor] interrupted by signal <n>`。
  - watch: 1行目にパイプライン名、続いて、`JOB  STATUS  NOTE` の3列(列の幅は、最も長い値に合わせる)。状態は、`success`・`warning`・`skipped`・`error`・`pending` の文字。補足は、終了コード、スキップの理由(英語・日本語)。
- 日本語の文言は、英語の文言と対にして、Messages に持つ。ジョブID・パスなどは、言語に関わらず、そのまま(エスケープのみ)。

## 設計の要点

1. **既定では、静か:** 成功・警告・スキップで、`-v` なしなら、`conductor` 自身の出力は、標準出力・標準エラーとも、空(NFR6.2)。Locale と Messages は、準備されない(NFR1.4)。
2. **エラーは必ず出る(NFR6.4):** `conductor` 自身のエラーの全種類(64〜68、70)で、標準エラーに、原因が、場所とともに出る。定義の誤りは、すべて、まとめて出す。ライブラリのエラー文、ソースの抜粋、入力の値は、含めない。
3. **色に頼らない(NFR6.1):** 状態は、常に、文字で書く。色は、装飾。ファイル・パイプには、制御文字を含めない。
4. **表示の失敗は、本筋を止めない:** 標準エラーへの書き込みの失敗は、無視する。標準出力の失敗は、`watch` で 65、`convert` で 67。
5. **診断のしやすさのテスト(NFR6.4):** 全種類のエラーで、表示が、空でなく、場所を含み、入力の値の生の制御文字を含まないことを、種類ごとのテストで、確かめる。

## 設計の判断(代替案との比較)

### D1 表示の文言を、定数の表(英語と日本語の対)で持つ

- **Context:** 英語と日本語を切り替える。ライブラリ(gettext など)を使うか。
- **Decision:** Rust の定数の表(種類 × 言語)で持つ。
- **Consequences:** 良い点は、依存がなく、起動が速く、文言が、コードの近くにある。悪い点は、言語を足すとき、表を増やす(2言語の範囲では、問題ない)。
- **Alternatives Rejected:** gettext 系のライブラリ: 言語の追加が容易だが、依存と、実行時の読み込みの費用が、起動の速さを損なう。

### D2 `-v` は1段階

- **Context:** 表示の粒度。要件は、オプションの名称と粒度を、設計で定めるとしている。
- **Decision:** `-v` / `--verbose` の1段階で、ジョブの結果、スキップ、遷移、中断を出す(Refined Mockups の Q1)。
- **Consequences:** 良い点は、単純で、スケジューラのログを汚さない。悪い点は、細かい制御(`-vv`)ができない。
- **Alternatives Rejected:** 複数段階(`-v`、`-vv`): 柔軟だが、この版の要件(1段階)を超え、文言の数が増える。

## この Unit に該当しない観測の項目

- メトリクス、構造化ログ、分散トレース、相関 ID、アラート、ダッシュボード、SLI/SLO、利用状況の収集: 持たない。

## Assumptions & Open Questions

- 表示の文言は、Code Generation で確定し、テストで固定する。日本語の文言の品質は、人のレビューに委ねる。 [assumption]
