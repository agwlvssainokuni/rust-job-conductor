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

//! クレートの内部(引数の解析、表示、run ID、終了コード)を直接確かめる単体テスト。
//!
//! `lib.rs` の最後の宣言から読む。テストのときだけコンパイルし、本番の成果物には入らない。
//! 公開の入口(`run_cli_with` と差し替え口)だけで確かめるテストは、`tests/` の統合テストにある。

mod common;
mod docs_support;

mod args_grammar;
mod docs_dry_run;
mod docs_graph;
mod docs_init;
mod escape_table;
mod exit_codes;
mod graph_long_id;
mod graph_report;
mod init_template;
mod plan_report;
mod report_messages;
mod run_id_arg;
