# Windows の Ctrl+C の経路: リリース前の手動確認のチェックリスト

CI のランナーには、対話的なコンソールがなく、`GenerateConsoleCtrlEvent` で、テストの実行ファイルだけに
Ctrl+C を送る安定した方法がない(`CREATE_NEW_PROCESS_GROUP` で起動した子プロセスは、Ctrl+C が無効になり、
コンソールを共有すると、テストの実行ファイル自身にも届く)。このため、この経路は、自動のテストにせず、
リリース前に、Windows の実機(または、対話的なコンソールのある環境)で、次を確かめて、結果を記録する。

UNIX 系の同じ経路(SIGTERM・SIGHUP の受け取り、128 + 番号、再開位置)は、`binary_signal.rs` が、
実際のプロセスで確かめている。Windows では、`ctrlc` のハンドラが、番号 2 で合図を立てる(`platform/windows.rs`)。

## 手順

1. 次の定義を `conductor.toml` として保存する(`<helper>` は、`conductor-cli-test-helper.exe` のパス)。

   ```toml
   format_version = 1

   [pipeline]
   name = "p"

   [[job]]
   id = "sleeper"
   command = '''"<helper>" sleep'''

   [[job]]
   id = "after"
   command = '''"<helper>" exit 0'''
   ```

2. コマンドプロンプトで `conductor run sleeper` を実行し、ジョブが動いているあいだに、Ctrl+C を押す。
3. 確認すること:
   - [ ] `conductor` が終了し、終了コードが 130(`echo %ERRORLEVEL%`)である
   - [ ] ジョブ(`conductor-cli-test-helper.exe`)のプロセスが残っていない(タスクマネージャー)
   - [ ] `conductor watch` で、`sleeper` が `error`、`after` が `pending` と表示される
   - [ ] `conductor run -v sleeper` の、Ctrl+C のあとの表示に `interrupted by signal 2` が含まれる
4. Ctrl+Break とコンソールを閉じたときの通知は、この版の対象外(既定の動作)。`ctrlc` が Ctrl+Break も
   受け取るかを確かめた場合は、結果を記録する。

## 記録

| 日付 | 確認した人 | Windows の版 | 結果 |
|------|------------|--------------|------|
|      |            |              |      |
