# 指標との対応(Cross-Unit Traceability)

ユニットは 1 つ(この intent 全体)。規則の詳細は `construction/functional-design/rules.md`、コードとの対応は `construction/code-generation/traceability.json`。

| ID | 指標 | 規則 | 確かめた結果 |
|----|------|------|--------------|
| SM1 | 段 1: 動きを変えない整理 | BR1.1〜BR1.6、BR1.8、BR9.1 | 古い説明、使われないもの、二重の処理、公開範囲、名前と型を直した。cli の `Runtime::run_job` から `job` を外し、テストの偽物は `CONDUCTOR_JOB_ID` から記録する。期待の文字列は無変更で通る |
| SM2 | 計算量 | BR1.7、BR2.1〜BR2.4、BR9.1 | 性能のテスト 3 件(N1〜N3)が上限の中。直す前と後の時間を記録した |
| SM3 | 段 2: 利用者から見える変更 | BR3.1〜BR3.3、BR8.1 | converter とステートの読み込みを形 B に(N4・N5 は直す前に落ちた)。空白だけのコマンド名は 64(N6・N7)。仕様書に「v0.9.7 までは…」 |
| SM4 | 版 | BR3.4 | 0.9.8(`7239aa2` と同じ範囲)。タグとリリースはしていない |
| SM5 | 既存のテストの扱いと件数 | BR1.8、BR1.9、BR8.1、BR9.2、BR9.3 | テストの変更は T1〜T25、消したテストは §4.6 の 4 件。1,157 − 4 + 8 = 1,161 |
| SM6 | 合格の条件 | BR9.4 | fmt、clippy、1,161 通過、行カバレッジ 98.13%。`Cargo.lock` の差分は版の行だけ。MSRV と他の OS は CI |

上流: `construction/code-generation/code-generation-plan.md`、`construction/code-generation/unit-test-instructions.md`。
