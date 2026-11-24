# エンティティ(Entities): 小さな直し

永続のデータ(ステートファイル)は扱わない。利用者に見える出力も変えない。型の形、項目、変種、メソッド、可視性は変えない。変わるのは、説明の行、仕様書の記述、テストの置き場所と数、ソースの検査の確かめだけ。詳細は `inventory.md`。D1〜D8 は、ユーザーが決めた(すべて推奨のとおり。2026-11-25)。T 表と消すテスト 3 件は、設計の承認で認めてもらう(承認待ち)。

```yaml
entities:
  # ---- 説明だけが変わる(本番の形は変えない) ----
  - name: decide_from_state(core。公開)
    description: ステートから起動前の判断を決める関数。plan/mod.rs と types.rs の NextStep の説明と、仕様書 rust 05 の 341・850 行目(D5。決定: 直す。推奨のとおり。ユーザー、2026-11-25)が、非公開の decide_before_run の代わりに、この名前を指すようになる。関数そのものは変えない。
  - name: initial_state / apply_* / check_consistency(core。公開)
    description: ステートの更新と照合の関数。runtime はこれらを再公開しない。core の lib.rs の説明の「runtime が同じ名前で再公開する」を直す(C3。D1。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。関数そのものは変えない。
  - name: decide_before_run(core の flow/decide.rs の中だけ)
    description: 判定の本体。変えない。説明から名指しされるのは、flow/decide.rs の中と、仕様書の本体の説明だけになる。
  - name: CliCommand::new(cli。crate の中)
    description: 指定のない値でコマンドを作る関数。使うのは引数の解釈だけになるので、説明の「テストが使う」を消す。関数そのものは変えない。

  # ---- 仕様書の記述が実装に揃う(実装は変えない) ----
  - name: resolve_run_id(cli。crate の中)
    description: run ID の決定。引数 option の型は Option<&RunIdArg>(検査済みの --run-id の指定)。仕様書 rust 02 の記述をこれに揃える。
  - name: consistency(core。公開モジュール)
    description: ステートと定義の照合(check_consistency、ConsistencyResult、MismatchKind。再公開もされる)。仕様書 rust 02 の再公開と公開モジュールの一覧に足す。仕様書 rust 06 の 5 行目の実装の所在も、照合と記録を core の consistency.rs・flow/record.rs と書く(D6。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。

  # ---- テスト(本番の成果物に入らない) ----
  - name: plan_simulate_perf(core の統合テスト)
    description: 計画の性能のテスト。P1・P2 に、env と env_file を持つ 1,000 ジョブの計画の 1 秒以内のテスト(元 plan_perf.rs)が加わり、3 件になる。移すテストの名前に番号を付けない(D8。決定: 変えない。推奨のとおり。ユーザー、2026-11-25)。言語に依らない仕様書の根拠の欄も plan_simulate_perf にする(D3。決定: 直す。推奨のとおり。ユーザー、2026-11-25)。
  - name: plan_perf(core の統合テスト)
    description: 無くなる。2 件のうち 1 件は P2 と同じ場面なので消え、1 件は plan_simulate_perf へ移る。
  - name: tests::init_types(cli の単体テスト)
    description: 3 件から 1 件(InitTarget::from_path の境界)になる。オプションの既定は args_grammar の init の既定のテストが確かめる。
  - name: nfr_source(cli の統合テスト)
    description: ソースの検査。テスト専用の経路を実行ファイルに持たない、の確かめが、製品のコードの #[path に加えて include!・include_str!・include_bytes! も止める。マクロの名前と ! を、行の空白を除いて探す(D4。決定: 推奨のとおり。ユーザー、2026-11-25)。数(6 件)と名前は変えない。
```

## 利用者に見える変わること

なし。`conductor` の出力、誤りの表示、終了コード、ステートファイル、`--dry-run` の出力、`conductor --version`(`conductor 0.9.8`)は変わらない。
