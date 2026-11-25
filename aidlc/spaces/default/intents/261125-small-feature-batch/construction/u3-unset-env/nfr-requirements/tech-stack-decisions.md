# 技術の選択: u3-unset-env

出典: 技術スタック(`technology-stack.md`)、要件(`requirements.md` の NFR4、NFR7)、機能の仕様(`functional-spec.md` の 1)、ビジネスルール(`rules.md` の BR3.2)、契約(`contract-summary.md` の C1)、team.md の Code Style。

| 項目 | 選択 | 理由 |
|---|---|---|
| 定義の読み取り | 既存の `walk/env.rs` に `unset_env` を足す | 名前の規則の検査を `env` と共有できる |
| 統合 | `EffectiveEnv` に `removals` を足す | C1 の output_u3 |
| 消す仕組み | std の `Command::env_remove` を、足す前に呼ぶ | OS ごとの比べ方を std に任せる(Q1)。`cfg` が要らない |
| 差し替え口 | cli の `Runtime::run_job` に消す名前を渡す | 今の形に引数を足すだけ |

## 要件

| ID | 要件 | 合否の基準 | 出典 |
|---|---|---|---|
| NFR7.1 | `cargo fmt --all -- --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` が通る | CI の `lint` | NFR7 |
