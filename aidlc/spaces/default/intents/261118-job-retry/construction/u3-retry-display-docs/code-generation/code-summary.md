# Code Summary — u3-retry-display-docs(表示の文言の確定と文書)

根拠: `code-generation-plan.md`(承認済み)、`unit-test-instructions.md`、`requirements.md`(FR6.2、FR8.1)。README に書いた動きは、u1、u2 の設計(`rules.md`)と、実装・テストで確かめた事実だけである。

## 作成・変更したファイル

コミットは 2 件(`main` 上。push はしていない)。

| コミット | 内容 |
|---|---|
| 35851cb | `README.md` にリトライの説明を追加(58 行追加、3 行削除) |
| de4e3ad | `crates/cli/tests/docs_retry.rs`(新規、4 件) |

### README.md(もともとあったファイルへの変更。計画に名前を挙げたもの)

- 目次: 新しい小節への項目を追加
- 「定義ファイルの書き方」のキーの表: `[job.retry]` の行を追加
- 新しい小節「リトライの設定(`[job.retry]`)(v0.8.0 以降)」: 例の定義、5 つのキー(`exit_codes`、`max`、`interval`、`multiplier`、`max_interval`)の意味と範囲の表、省略・重なり・誤り(64、`validate` も検出)の規則、「リトライの動き」(リトライする条件、使い切ったとき、リトライの前の知らせ(英語と日本語の例)、待機中のシグナル、ステートファイル、同じ処理が複数回動くこと、`[job.retry]` を書かないジョブは従来どおり)
- 「ステートファイルの場所」: 「自動の再試行はしません」を、「`[job.retry]` を書かなければ、自動の再試行はしません」に直し、回数は記録せず、再開は数え直すことを追記
- 「終了コードの表」: 64 に `[job.retry]` の誤り、128 + 番号にリトライの待機中の中断を追記
- 「保証の範囲(at-least-once)」: リトライするジョブは複数回動くこと、再開で回数が数え直されることを追記
- 「シグナルの扱い」: 待機中のシグナルの動き(すぐ待機をやめ、再開位置はそのジョブ、128 + 番号、エラーとしては記録しない、1 秒以内)を追記

README の例の定義(新しい小節の最初の TOML ブロック):

```toml
format_version = 1

[pipeline]
name = "nightly"

[[job]]
id = "extract"
command = "python scripts/extract.py"

[job.retry]
exit_codes = [75, 111]
max = 5
interval = "10s"
multiplier = 2
max_interval = "60s"
```

### テスト(追加 4 件)

`crates/cli/tests/docs_retry.rs`(新規): README に `[job.retry]` と 5 つのキーと版の表記があること、古い一文がそのままの形では残っていないこと、README の例の定義を `load_definition` に通すと誤りなく読めて、リトライの各値(75・111、5 回、10 秒、倍率 2、上限 60 秒)が入っていること、CI の `docs-check` の 7 つの必須の語が README にあること。改行コード(CRLF)は正規化して比べる。

## 文言の確認の結果(FR6.2)

u2 が書いた最初の文言は、既存の文言と README の用語に合っているため、**変更しなかった**(`messages.rs` と `report_messages.rs` は未変更)。README には、その英語と日本語の文を、そのまま引用した。

- 英語: `conductor: retry: job 'a' exited with code 75; retry 2 of 3 in 40 seconds`
- 日本語: `conductor: retry: ジョブ 'a' が終了コード 75 で終わりました。40 秒後にリトライします(2/3 回目)`
- 表示のレベルは、設計(Q3: A)のとおり、`-v` なしで常に標準エラー

## 計画からのずれ・気づいた点

- 計画からのずれはない
- 英語の知らせは、待ち時間が 1 のとき `in 1 seconds` と複数形になる(例: `retry 1 of 3 in 1 seconds`)。既存の文言に単数・複数の使い分けがなく、この Unit の範囲外のため、直していない。直す場合は、u2 が足した `report_messages.rs` のテストの更新が要る(小さな修正)
- 版の表記は、計画の前提どおり `v0.8.0 以降`。リリースの版は、あなたが決める。違っていれば、タグを付ける前に README を直す

## 確認の結果(ローカル、macOS、rustc 1.100.0)

- `cargo test --workspace --locked`: 803 件が成功、失敗 0 件、無視 1 件(既存)。変更前は 799 件(今回の追加 4 件を足すと 803 件)
- `cargo fmt --all -- --check`: 問題なし
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: 警告 0 件
- カバレッジ(`cargo llvm-cov ... --fail-under-lines 80`、開発担当の実測): 行 97.80%。下限 80% を満たす
- CI の `docs-check` と同じ 7 語(`SHA-256`、`Gatekeeper`、`終了コードの表`、`ステートファイルの場所`、`at-least-once`、`シグナル`、`convert`): すべて README にある
- `msrv_consistency`: 3 件成功(MSRV の記載は変えていない)
- 新しい外部クレート: なし。`Cargo.toml` と `Cargo.lock` は変更なし。既存のアサーションの変更: なし

## 実行していない(書いたが実行していない)テストと検査

- Linux と Windows でのテスト: ローカルは macOS のため。CI で実行する
- CI での実行: この時点では、プッシュしていないため、CI は実行されていない
- `demo/` は、計画どおり変更していない
