## Review

**Verdict:** READY
**Reviewer:** aidlc-architecture-reviewer-agent
**Date:** 2026-11-14T23:09:02Z
**Iteration:** 1
**Verdict Dictated:** no

### Findings

| ID | Severity | Location | Finding | Required action | Status |
|---|---|---|---|---|---|
| R-01 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/cicd-pipeline.md > 段階と確認の対応(段階 4〜8)、起動の条件 | 9つの段階が、どのジョブ(名前、OS のマトリクス、Linux だけのジョブ)で動くかが書かれていない。「必須のチェック」(段階 5 など)は、ジョブ名(マトリクスなら OS ごとの名前)で指定するため、ジョブの構成がないと、どれを必須にするかが決まらない。段階 5 は、段階 4 の `cargo test --workspace` にすでに含まれるテスト(cfg で OS ごとに有効になる)と同じ実行なのか、別の実行なのかも読み取れない。段階 7(起動時間)の実行の場所とビルドの種類(デバッグかリリースか)も未定で、デバッグビルドでは 20ms の基準の意味が変わる。15 分の上限も、1ジョブに全段階を入れるのか分けるのかで意味が違う | ジョブの構成を表で足す(ジョブ名、OS、含む段階、`timeout-minutes`)。マージを止める必須のチェックとして登録するジョブ名を列挙し、段階 5 は、段階 4 の中の OS 固有のテストを指すのか別の実行かを明記する。段階 7 のビルドの種類(`--release` かどうか)を決める | New |
| R-02 | Major | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/cicd-pipeline.md > 段階 3(`cargo tree` の確認) | 「`-sys` で終わるクレートが依存に含まれないこと」の確認は、対象と辺の種類が未指定である。`cargo tree` は既定で開発用の依存(この Unit は `tempfile`)も含み、`tempfile` は、Linux では `linux-raw-sys`、Windows では `windows-sys` を引き込む。ワークスペース全体を対象にすると、他の Unit の依存も入る。そのまま実装すると、常に失敗するか、検査が意味をなさなくなる。また、`-sys` 以外の C のビルドを要する依存(`cc`、`cmake` の利用など)の判定方法も「C のビルドスクリプトを持つもの」としか書かれていない | 対象を `--package <この Unit のパッケージ> --edges normal`(開発用の依存を除く)に絞ると明記する。許可する例外(`libc`)と、拒否する条件(`-sys` で終わる名前、`cc` クレートを依存に持つこと)を、機械的に確かめられる形で書く | New |
| R-03 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/cicd-pipeline.md > 段階 6(時間の上限) | 時間の上限は、別のスレッドでジョブを動かし、`recv_timeout` を超えたらテストを失敗にする、という作りである。しかし、そのとき子プロセス(眠り続ける補助の実行ファイル)は、回収も終了もされず、動き続ける。シグナル転送のテストは、転送が壊れているときが、まさにこのハングの場合である。Windows では、動き続ける補助の実行ファイルが、ビルドの成果物のファイルを開いたままにし、後続のビルドやキャッシュの保存を妨げうる | 時間切れのとき、テストが子プロセスを強制終了して回収する(または、テスト用の補助の関数が、終了のガードを持つ)ことを、設計に書く。補助の実行ファイル自身にも、上限(例: 数十秒後に自分で終了)を持たせる案を検討する | New |
| R-04 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/cicd-pipeline.md > 段階 3(`unsafe`・`cfg` の検索)、段階 5 | ソースの検索の対象範囲が未指定である。段階 5 の OS 固有のテストは、`#[cfg(unix)]` と `#[cfg(windows)]` を使うため、`platform` の外に `cfg(unix)`・`cfg(windows)` がないことの検索を、テストと補助の実行ファイルまで含めて行うと失敗する。root のときに権限のテストを自分でスキップする記述も、スキップが「成功」として数えられ、実行されたのかが見えない | 検索の対象を `crates/u2-runtime/src/` のうち、テストと補助の実行ファイルを除いた範囲に限る、と明記する。スキップしたテストは、標準エラーに理由を出し、可能なら `#[ignore]` ではなく、実行時の通知で見えるようにする | New |
| R-05 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/cicd-pipeline.md > 段階 3(最小の Rust のバージョン 1.85) | `cargo +1.85 build --locked` は、最新の安定版で作られた `Cargo.lock` を使う。ロックされた依存の一つが、1.85 より新しい `rust-version` を要求していると、このビルドは失敗する(この失敗は、MSRV の検査が正しく働いた結果だが、対処の方針がない)。また、同じ検査は、補助の実行ファイルも含めてビルドする | 失敗したときの対処(ロックを更新して依存を下げる、または最小のバージョンを引き上げて tech-stack-decisions を更新する)を、短く書く | New |
| R-06 | Minor | aidlc/spaces/default/intents/261114-job-conductor-cli/construction/u2-runtime/infrastructure-design/traceability.json > NFR4.1 | NFR4.1 の確認方法には、`sh`・`cmd` を、この Unit のソースが呼ばないことの「ソースの文字列の検索」が含まれるが、cicd-pipeline の段階 3 の検索(`unsafe`、`cfg`)には、この検索がない。状態が `OK` なのに、上流の確認方法の一部がパイプラインにない。NFR4.2・NFR4.3・NFR4.5 の対象も、段階 4 の「テストを含む」という記述に束ねられており、個別の確認の目標が薄い | 段階 3 に、`sh`・`cmd` の呼び出しがないことの検索を足し、traceability の NFR4.1 の対象に含める | New |

### Validation Tool Results

| Tool | Result | Interpretation |
|---|---|---|
| traceability.json の照合(上流の NFR ID の網羅) | 13 件(NFR1.1、NFR2.1、NFR3.1、NFR4.1〜NFR4.9、NFR5.1)が、upstream_ids と coverage の両方に存在し、nfr-requirements・nfr-design に現れる NFR ID と一致 | 網羅は取れている。NFR4.1 の検索の一部が、パイプラインに無い点だけが差分(Minor) |

### Summary

library の CI 設計として、方針(ワークスペースで1つのワークフロー、秘密情報なし、ロールバックの扱い、仮の値の明示)は、team.md と nfr-design に整合している。ただし、ジョブの構成とマージを止めるチェックの名前が未定で、`cargo tree` の確認が書かれたまま実行できない点は、Major として、CI Pipeline のステージの前に補う必要がある。Major は2件なので、READY とするが、これらは承認の前に直す価値が高い。
