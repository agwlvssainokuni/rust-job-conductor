# Bolt の計画: v0.10.0 の小さな機能のまとまり

出典: 要件(`requirements.md`)、コンポーネントの一覧(`components.md`)、ユニットの定義(`unit-of-work.md`)、ユニットの依存(`unit-of-work-dependency.md`)、契約のまとめ(`contract-summary.md`)、チームの実践(`team-practices.md`)、実装の計画の質問(`delivery-planning-questions.md`)。

Bolt は、作業を 1 周回す単位で、終わると動くものができる。今回は 1 つのユニットを 1 つの Bolt にし、B1 から B7 の順に 1 つずつ直列に作る(スコープ Q4、チームの実践)。薄い縦切り(walking skeleton。本物の機能の前に、最初から最後まで通る最小の形を作ること)は作らない(team.md の Walking Skeleton)。

## 共通の完了の条件(Definition of Done)

各 Bolt は、次をすべて満たしたら完了とする(Q2)。

1. その機能のコードとテスト(test-after。core → runtime → cli の層の順に、各層の後にテスト)がそろう。
2. ローカルの macOS で `cargo test --workspace --locked`、`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings` が通る。
3. その機能の README とデモを更新する(使い方の表は `args.rs`、`report/messages.rs`、README の 3 か所を揃える)。
4. `main` に push した後の CI(ubuntu・macos・windows のテスト、MSRV 1.100、行カバレッジ 80%、`build-check`、`docs-check`)が通る。

## Bolt の並び

| 順 | Bolt | ユニット | 依存の確認 |
|---|---|---|---|
| 1 | Bolt 1 | u1-pipeline-retry | 依存なし |
| 2 | Bolt 2 | u2-optional-env-file | 依存なし |
| 3 | Bolt 3 | u3-unset-env | u2 の後(満たす) |
| 4 | Bolt 4 | u4-graph-dot | 依存なし |
| 5 | Bolt 5 | u5-log-dir | 依存なし |
| 6 | Bolt 6 | u6-attempt-times | u5 の後(満たす) |
| 7 | Bolt 7 | u7-watch-follow | u1〜u6 の後(満たす) |

この並びは、ユニットの依存の形(DAG)の順序に反しない。

## 各 Bolt

### Bolt 1: u1-pipeline-retry

- **完了の条件**: 共通の条件に加え、`[pipeline.retry]` が `retry` の無いジョブに当てはまり、`retry = false` で打ち消せ、ジョブの表が丸ごと勝ち、成功・警告との重なりが 64 になる(FR1.1〜FR1.6)。
- **確かめること**: 既定の当てはめを定義の検証の中に置いても、リトライの判断と流れを変えずに済む(ADR-003)。
- **見せるもの**: `[pipeline.retry]` を書いた定義で、失敗するジョブが既定どおりリトライされ、`retry = false` のジョブはリトライされない。

### Bolt 2: u2-optional-env-file

- **完了の条件**: 共通の条件に加え、`{ path, required = false }` の無いファイルが黙って飛ばされ、あるが読めないファイルは 64 のまま(FR2.1〜FR2.5)。
- **確かめること**: 文字列と表を混ぜた `env_file` が、今の定義と同じに動く。
- **見せるもの**: `.env.local` が無い環境でも、同じ定義がそのまま動く。

### Bolt 3: u3-unset-env

- **完了の条件**: 共通の条件に加え、`unset_env` の名前が子の環境から消え、`env`・`.env` との重なりが 64 になる(FR3.1〜FR3.5)。
- **確かめること**: 統合の結果に「消す名前」を足しても、Bolt 2 の変更を壊さない(契約 C1)。
- **見せるもの**: 親の `HTTP_PROXY` が、`unset_env` を書いたジョブにだけ渡らない。

### Bolt 4: u4-graph-dot

- **完了の条件**: 共通の条件に加え、`graph --format dot` が `-v` と `--state` に対応し、値が escape される(FR4.1〜FR4.4)。
- **確かめること**: text・mermaid と同じ構造と重ね合わせから DOT を描ける(ADR-006)。
- **見せるもの**: `conductor graph --format dot | dot -Tsvg` で図ができる(Graphviz が入っている環境で)。

### Bolt 5: u5-log-dir

- **完了の条件**: 共通の条件に加え、`log_dir` のとき、出力が端末とファイルの両方に出て、試行ごとに別のファイルになり、ディレクトリが無ければ作られる(FR5.1〜FR5.5)。時刻の整形(CoreTime)と現在の時刻(RuntimeClock)ができる。
- **確かめること**(気がかり A): 子の出力をパイプで受けて端末とファイルへ書く仕組みが、3 つの OS の CI で同じに動く。`log_dir` を書かないときの動きが変わらない。
- **見せるもの**: `logs/build.20261125T091248Z.1.log` に、端末に出たのと同じ出力が残る。

### Bolt 6: u6-attempt-times

- **完了の条件**: 共通の条件に加え、ステートに `[[job.attempt]]` の時刻が書かれ、`watch` に所要時間が出て、時刻の無い古いステートが 65 と `reset` の案内になる(FR6.1〜FR6.9)。
- **確かめること**(気がかり B): ステートの形の変更が、今のテスト(ステートの読み書き、`nfr_content.rs`、デモのテスト)を意図どおりに変えるだけで済む。
- **見せるもの**: パイプラインを動かした後の `watch` に、各ジョブの所要時間が出る。

### Bolt 7: u7-watch-follow

- **完了の条件**: 共通の条件に加え、`watch --follow` が一定の間隔で表示し直し、Ctrl-C とパイプラインの終わりで止まる。UNIX 系の端末だけで描き直す(FR7.1〜FR7.5)。最後に版を 0.10.0 にする(FR8.2)。
- **確かめること**(気がかり C): 描き直しと止まり方が、端末・パイプ・Windows のそれぞれで要件どおりになる。
- **見せるもの**: 別の端末でパイプラインを動かしながら、`watch --follow` で様子を見続けられ、終わると自動で止まる。
