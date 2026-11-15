# 要件: Windows 向けデモ

## 意図の分析

依頼(`[desc]` Initial description: 「Windows用のデモをつくってほしい。」)は、UNIX 系の `demo/run-demo.sh` が見せている `conductor` の主な動きを、Windows でも同じように見せられるようにすることである。現状のデモは `sh` 前提で、`demo/README.md` にも「Windows では動かない」と書かれている。`conductor` はシェルを通さずにジョブを起動するため、Windows では、ジョブのスクリプト・定義ファイル・実行スクリプトを、Windows のものとして用意する必要がある。

`conductor` 本体のコードは変えない。追加するのは、デモ用のファイルと、その案内だけである。

## 機能要件

### FR1. Windows 用デモの構成

- FR1.1: Windows 用のデモ一式を、`demo/windows/` に置く。UNIX 向けのデモ(`demo/run-demo.sh`、`demo/conductor.toml`、`demo/scripts/*.sh`)は、変えない。`[Q2]`
- FR1.2: ジョブのスクリプトと実行スクリプトは、cmd(`.cmd`)で書く。PowerShell や、追加のインストールを前提にしない。`[Q1]`
- FR1.3: `demo/windows/conductor.toml` は、UNIX 版と同じ内容のパイプライン(`nightly`:`extract`、`transform`、`load`、`notify`)を、Windows のコマンドで書く。`extract` の終了コード 1 は警告で、`notify` へ進む。`notify` の `command` は、文字列の配列で書く。`[scope]`

### FR2. 場面の再現

- FR2.1: `demo/windows/run-demo.cmd` を実行すると、UNIX 版と同じ 0〜6 の場面を、この順に見せる。`[scope]`
  - 0: 定義ファイルの表示
  - 1: 全部が成功する
  - 2: 警告で分岐する
  - 3: エラーで止まる(終了コードがそのまま返り、`watch` で止まった位置が見える)
  - 4: 同じコマンドをもう一度 → 失敗した `load` から再開
  - 5: ジョブを 1 つだけ実行する(`run`)と、`reset`
  - 6: 定義ファイルを YAML に変換する(`convert`)
- FR2.2: 各ジョブの終了コードは、環境変数 `DEMO_<ジョブ名>_EXIT`(例 `DEMO_LOAD_EXIT=2`)で決める。既定は 0 である。`[scope]`
- FR2.3: 各ジョブの待ち時間は、環境変数 `DEMO_SLEEP` で変えられる。`[scope]`
- FR2.4: `DEMO_PAUSE=1` のときは、各場面のあとで Enter を待つ。`[scope]`
- FR2.5: `CONDUCTOR` 環境変数で、使う実行ファイルを指定できる。未指定で `target\release\conductor.exe` が無いときは、`cargo build --release --locked --bin conductor` でビルドする。`[scope]`
- FR2.6: 一時ディレクトリで動き、リポジトリには何も残さない。`[scope]`
- FR2.7: 表示の言語は、既定は日本語で、`DEMO_LANG` で英語に切り替えられる。切り替えの仕組みは、Windows の `conductor` の言語判定に合わせる。`[scope]`

### FR3. 案内

- FR3.1: `demo/README.md` を更新し、UNIX 版と Windows 版の使い方を並べて書く。「Windows では、`sh` のスクリプトは動きません」の一文は、Windows 版の案内に書き換える。`[Q3]`
- FR3.2: ルートの `README.md` は変えない。`[Q3]`

## 非機能要件

- NFR1(互換): Windows 10 / 11(x86_64)に標準で入っている `cmd.exe` だけで動く。PowerShell、WSL、Git Bash を前提にしない。`[Q1]`
- NFR2(保守): 新しいファイルは、Apache のライセンスヘッダ(`REM` のコメント)を付け、コメントは日本語で書く。`[memory:M1]`
- NFR3(検証): Windows の実機(または CI の windows-2022)で、場面 0〜6 が最後まで動き、各場面の終了コードが README の説明と合うことを、実行して確かめる。`[desc]`
- NFR4(品質): 既存の `cargo test`、`cargo fmt`、`cargo clippy`、README の文書チェックは、変更後も通る。`[memory:M1]`

## 制約

- C1: `conductor` はシェルを通さずにジョブを起動する。`.cmd` を直接 `command` に書くか、`cmd /c` を明示する必要がある。`[Q1]`
- C2: ルートの `README.md` には変更を加えない(CI の文書チェックが見る語句を、動かさないため)。`[Q3]`
- C3: `conductor` 本体(`crates/`)は変えない。`[scope]`

## 前提

- A1: `[assumption]` cmd には小数の待ち時間がないため、`DEMO_SLEEP` は整数の秒とし、既定は 1 秒とする(UNIX 版の既定は 0.5 秒)。
- A2: `[assumption]` Windows 版の `watch` は、色が付かない(Windows では色の表示が未対応)。デモの説明には、その旨を書く。
- A3: `[assumption]` 色付きの見出し(ANSI)は使わない。素の文字だけで見やすくする。

## 対象外

- PowerShell 版、PowerShell 7 版のデモ
- ルートの `README.md` への案内の追加
- CI への Windows デモの自動実行の追加
- `conductor` 本体の変更

## 未解決の問い

- なし(前提 A1〜A3 は、後続の段階で、実機で確かめて調整する)。

## Sources

- `[desc]` Initial description: 「Windows用のデモをつくってほしい。」
- `[scope]` Workflow-selected scope: express
- `[Q1]`〜`[Q3]`: `requirements-analysis-questions.md` のユーザー回答(Q1: B、Q2: A、Q3: A)
- `[memory:M1]` `aidlc/spaces/default/memory/team.md`(Code Style、Testing Posture)
- コード調査: `aidlc/spaces/default/codekb/rust-job-conductor/`

## Assumptions & Open Questions

- A1〜A3 は、上の「前提」のとおり。後続の段階で確認する。
