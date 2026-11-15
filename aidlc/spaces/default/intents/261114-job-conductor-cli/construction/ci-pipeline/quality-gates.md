# Quality Gates

## Sources

- チームの実践(`team.md`): 行カバレッジ 80%、rustfmt と clippy、3 OS のテスト、マージ前に CI を通す [memory:M1]
- Build and Test の Target Verification Matrix(BT-01〜BT-11) [scope]
- 質問への回答 Q1=A、Q2=B、Q3=A [Q1] [Q2] [Q3]

## Assumptions & Open Questions

None.

## マージの関門(必須のチェック)

`main` への取り込みの前に、次のチェックがすべて成功すること(リポジトリの設定の「必須のステータスチェック」に指定する。**手動の設定**)。

| 関門 | CI のジョブ | 基準 | Build and Test の対応する目標 |
|------|-------------|------|-------------------------------|
| 整形 | `lint` | `cargo fmt --all -- --check` が成功 | BT-04 |
| 静的検査 | `lint` | `cargo clippy ... -D warnings` が警告 0 | BT-04 |
| ビルド(最小の Rust) | `build-check`(Linux、Windows) | Rust 1.85 で全ターゲットがビルドできる。C のビルドを要する依存がない | BT-05、BT-06 |
| テスト(3 OS) | `test`(Linux、macOS、Windows) | 全テストが成功 | BT-03、BT-09 |
| テスト(最小の Rust) | `msrv-test` | Rust 1.85 で全テストが成功 | BT-05 |
| カバレッジ | `coverage` | 行カバレッジ 80% 以上(補助の実行ファイルを除く)。未満なら失敗 | BT-01、BT-02、BT-10(要件 NFR5) |
| 文書 | `docs-check` | README に必要な節がある | — |

## 警告のみ(ブロックしない)

| 検査 | ジョブ | 内容 |
|------|--------|------|
| 起動の性能 | `startup-bench` | `run` のオーバーヘッドが 100 ms 以下ほか。基準超えは警告。3 OS の実測を重ねたあと、`STARTUP_BENCH_ENFORCE=true` にして、必須に加える |
| 依存の脆弱性 | `audit.yml` | `cargo audit`(プルリクエストと毎週月曜)。結果は警告 |

## リリースの関門

タグ `v*` のリリースは、次をすべて満たしたときだけ、下書きのリリースを作る: タグが `main` の履歴にある、タグの版と `Cargo.toml` の版が一致する、マージの関門のすべてが、タグのコミットで成功する、4 つのターゲットのビルドとスモークテストが成功する。**公開は人が行う**。

## 人が行うこと(自動にできないもの)

1. リポジトリの設定で、`main` の必須のステータスチェック(上の表のジョブ)と、`v*` タグの保護を設定する。
2. リリースを公開する前に、Windows の実機で `crates/u3-cli/tests/WINDOWS_CTRL_C_CHECKLIST.md` を実施する(Build and Test の BT-11。受け入れ済みの手動確認)。
3. 下書きのリリースの内容(ファイル 4 つと `SHA256SUMS`、本文)を確認して公開する。

## Build and Test の未検証の項目の行き先

| 目標 | このステージでの扱い |
|------|---------------------|
| BT-09(Linux・Windows のテスト実行) | `test` ジョブが、マージ前に実行する。**初回の GitHub Actions の実行で確かめる**(ここでは、実行していない) |
| BT-10(要件 NFR5: 80% 未満で CI が失敗) | `coverage` ジョブの `--fail-under-lines 80`。**初回の実行で確かめる** |
| BT-11(Windows の Ctrl+C) | 手動の確認(上の「人が行うこと」2) |
