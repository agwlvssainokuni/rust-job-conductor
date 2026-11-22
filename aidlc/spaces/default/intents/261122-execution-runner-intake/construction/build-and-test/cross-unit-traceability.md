# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1: `JobExecution` の型 | BR1.1〜BR1.5 | a5de482〜0fd3452。`ExecutionResult`(`NotStarted`、`StartFailed`、`Exited`)。N1 で分類の表を確かめた。古い項目は残っていない。ステートの書式は不変 |
| SM2 | 段 2: `run_job` の入力 | BR2.1〜BR2.4 | 10fb5a3〜c49ff0b。`run_job(job, line, env, signal)`。N2 で分け直さないことを確かめた。空のプログラム名の防御は合図の確認より前 |
| SM3 | 段 3: 読み込みの順序 | BR3.1〜BR3.5 | 33d9890〜31d6f56。形 B。N3 は直す前に落ち、直した後に通った。N4・N5 は前後とも通った。既存の取り込みのテストは無変更。converter は版の行だけ |
| SM4 | 版 | BR3.6 | ee9f409。0.9.6(`ea09b46` と同じ 22 ファイル)。タグとリリースはしていない |
| SM5 | テストの扱いと合格の条件 | BR4.1〜BR4.4 | テストの変更は T1〜T20(T19・T20 は追加の承認済み)。fmt、clippy、1,142 通過、行カバレッジ 98.23%。依存の追加なし。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
