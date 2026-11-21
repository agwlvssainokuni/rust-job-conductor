// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! conductor-core: conductor の中核ライブラリ(Unit conductor-core)。
//!
//! 定義ファイル(`conductor.toml`)の読み込みと検証(DefinitionLoader)と、
//! ジョブの結果の分類・次の動き・完了・再開位置の判断(Flow)を受け持つ。
//! ジョブの起動(conductor-runtime)、ステートの読み書き(conductor-runtime)、コマンドと表示(conductor-cli)は、
//! このクレートの外にある。
//!
//! # 公開する関数(契約 C2)
//!
//! | 関数 | 役割 | 入出力 | 失敗 |
//! |------|------|--------|------|
//! | [`load_definition`] | 定義ファイルを読み、検証する(W1) | ファイル → [`PipelineDefinition`] | [`DefinitionError`] の並び(複数) |
//! | [`parse_definition`] | 文字列の定義を検証する(ファイルに触れない入口) | 文字列 → [`PipelineDefinition`] | 同上 |
//! | [`decide_before_run`] | ジョブを起動する前の判断(W2、BR3.5) | 引数 → [`Decision`] | なし(純粋) |
//! | [`classify_and_decide`] | 実行後の分類と次の動き(W3、BR2.x、BR3.x) | 引数 → ([`JobOutcome`], [`NextStep`]) | なし(純粋) |
//! | [`resume_point_on_interrupt`] | ジョブのあいだの中断の再開位置(W4、BR3.7) | 引数 → [`ResumePoint`] | なし(純粋) |
//!
//! 入出力に触れるのは、[`definition::intake`] だけである。それ以外は、引数だけから結果を決める
//! 純粋な処理で、ファイルにもプロセスにも環境変数にも触れない。
//!
//! # モジュールの構成(logical-components との対応)
//!
//! | モジュール | 部品 | 責務 |
//! |------------|------|------|
//! | [`types`] | 共有する型(Types) | データの形だけ(契約 C1) |
//! | [`error`] | 表示用の組み立て(Describe)、位置の変換(Locate) | 固定の文だけでエラーを表示する。バイトの位置から行番号を求める |
//! | [`definition::intake`] | 取り込み(Intake) | 場所の決定、大きさの確認、上限 + 1 バイトまでの読み取り |
//! | [`definition::parse`] | 読み取り(Parse) | 位置を保つ文書の木にする。文法の誤りは `syntax` |
//! | [`definition::depth`] | 深さの検査(Depth) | 入れ子 32 段の上限 |
//! | [`definition::version`] | 版の確認(VersionCheck) | `format_version` が 1 であること。通らなければ、そこで終える |
//! | [`definition::walk`] | 構造の検査(Walk) | キー、型、構造の制約、遷移、終了コード、名前。誤りはすべて集める |
//! | [`flow`] | 判断(Flow) | 起動前の判断、分類、次の動き、完了、再開位置 |
//!
//! 定義の読み込みの流れは Intake → Parse → Depth → VersionCheck → Walk の順である。
//!
//! # 呼び出しの順序(Cli から見て)
//!
//! ジョブごとに、[`decide_before_run`] → (`Run` のとき)ジョブの起動 → [`classify_and_decide`] →
//! ステートの書き込み。`Skip` のときは、ジョブを起動せず、必要ならスキップをステートに記録して、
//! 終了コード 0 で終わる。スキップの記録のあとの完了の判定には、[`resume_point_on_interrupt`] を使う
//! (最後のジョブがスキップなら `completed` が `true`)。`Reject` のときは、ジョブを実行せず、
//! ステートも変えず、conductor 自身のエラー(終了コード 66。BR5.1)として扱う。
//!
//! # 守る性質
//!
//! - 外部の入力(定義ファイル)に対して、パニックしない。失敗は、戻り値のエラーで返す(NFR4.7)。
//! - エラーの表示に、入力の値、ライブラリのエラー文、ソースの抜粋を含めない。キー名は、
//!   制御文字をエスケープし、64 文字までに切る(NFR4.6)。
//! - 定義に書かれたコマンド([`Command`]。文字列または文字列の配列)は、データとして扱い、
//!   解釈も起動もしない(NFR4.5)。

pub mod definition;
pub mod envfile;
pub mod error;
pub mod flow;
pub mod graph;
pub mod plan;
pub mod run_id;
pub mod types;

pub use definition::{load_definition, parse_definition};
pub use envfile::{
    EffectiveEntry, EffectiveEnv, EnvContext, EnvEntry, EnvFileContent, EnvFileError,
    EnvFileErrorKind, EnvOrigin, EnvWarning, EnvWarningKind, LoadedEnvFiles, env_file_refs,
    is_reserved_env_name, merge_env, parse_env_file, summarize_errors,
};
pub use error::*;
pub use flow::{
    RetryDecision, TransitionResolution, classify_and_decide, decide_before_run,
    resolve_transition, resume_point_on_interrupt, retry_decision,
};
pub use plan::{
    Assumption, EnvNameEntry, ExecutionPlan, NextStepNote, PlanDisposition, PlanEnd, PlanEntry,
    PlanInput, PlanMode, PlanRejection, ResumeInfo, ResumeKind, SimulatedResult, plan_pipeline,
    plan_run,
};
pub use run_id::{
    DEFAULT_RUN_ID_ENV, MAX_RUN_ID_LEN, RunId, RunIdError, is_valid_env_name, peek_run_id_env,
};
pub use types::*;
