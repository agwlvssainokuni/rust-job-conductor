# コードの要約(Code Summary): 動きを変えない整理のリファクタリング

計画: `code-generation-plan.md`(Step 1〜11)。設計: `construction/functional-design/{functional-spec,inventory,rules,entities}.md`。テスト: Minimal、test-after。

利用者から見える動き(標準出力と標準エラーの文、終了コード、ステートファイル、`--dry-run` の出力)は変えていない。`Cargo.toml`、`Cargo.lock` に差分はない。`unsafe`、`cfg(unix)`/`cfg(windows)`、新しい依存は足していない。converter は core に依存しない。core に入出力は持ち込んでいない。

## コミット(`main` に直接。push はしていない)

各コミットで、`cargo build`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ることを確かめた(文書だけのコミットは build を確かめた)。

### 段 1(cli。BR1.1〜BR1.4、SM1)

| コミット | 内容 |
|----------|------|
| `8c4b8de` | 1a: `Ctx::definition_run_id` と `Ctx::read_checked_state`(`StateRead::{Live, Raw}`)を足し、`prepare_checked` と `graph --state` が使う。watch は `read_context` を使う |
| `2db35c0` | 1b: `check_definition` が `CheckedDefinition`(分割したコマンドを含む)を返し、`Checked`・`Session` が `commands` を持つ。dry_run、`report_execution`、`step::run_step` が使い回す。通らない誤りの枝を消す。誤りは全件を集める |
| `3ffd48e` | 1c: `check_definition_and_env` を `ctx.rs` の `Ctx` のメソッドに移す |
| `1e4e6cf` | 1d: `DryRunTarget` を `FlowTarget` に改名。入口を `dry_run::enter_flow` に。prepare を `prepare_run`(位置を返す)・`prepare_pipeline` と部品(`start_signals`、`require_job`、`require_known`、`check_run_inputs`)に分ける。run の通らない else が消える |

### 段 2(型。BR2.1〜BR2.4、SM2)

| コミット | 内容 |
|----------|------|
| `6cb07b7` | 2a: core に `Classification::as_str`・`transition_on`、`TransitionOn::as_str`。`DisplayEvent::JobResult.result` を `Option<Classification>`、`Transition.on` を `TransitionOn` に。`report_messages.rs` の組み立ての値と import だけを直す(承認済み) |
| `86845c3` | 2b: `JobStatus`・`SkipReason` に `as_str`・`parse`。writer、reader、watch(`status_word` は薄い包み)が使う |
| `28ca9ba` | 2c: `check_consistency`・`ConsistencyResult`・`MismatchKind` を core の `consistency.rs` に。runtime は再エクスポート。runtime の `consistency.rs` は `#[cfg(test)] use` だけの薄いファイルで、`mod tests` は 1 文字も変えていない |
| `ce74418` | 2d: `read_bounded`(`BoundedError::{Unreadable, TooLarge}`)にまとめる。`read_limited` は残す |

### 段 3(core。BR3.1〜BR3.3、SM3)

| コミット | 内容 |
|----------|------|
| `c34d3f7` | 3a: `git mv walk.rs walk/mod.rs` のあと、`walk/retry.rs`、`walk/env.rs` に分ける |
| `8437bad` | 3b: `DEFAULT_SUCCESS_EXIT_CODES` |
| `2979d1c` | 3c: `exit_code_of` を `integer_of` から作り、retry の読み取りも呼ぶ |
| `85efac6` | 3d: depth の文を `MAX_NESTING_DEPTH` から作る(文は同じ)。converter は `const _: () = assert!(crate::raw::MAX_DEPTH == 32);`。表示の上限 256 を `error::MAX_FILE_DISPLAY_CHARS` に寄せる |
| `d6b31cb` | 3e: `empty_step(job)` |
| `8e25bbf` | 3f: `JobExecution::exited` を足し、simulate で使う |

### 段 1〜3 の仕様書(SM1〜SM3)

| コミット | 内容 |
|----------|------|
| `e99b38c` | `docs/spec/rust` の 02・04・05・06・07・10・12 章の内部の名前を新しい置き場所と名前に合わせる |

### 段 4(テストと文書。BR4.1〜BR4.5、SM4)

| コミット | 内容 |
|----------|------|
| `d72501a` | 4a: graph のテストを形ごとのファイルに組み直す(64 件 → 58 件)。仕様書の参照(rust の 05・11、language-agnostic の 11 の AC-475、AC-476a〜e)も直す |
| `6ed6f3c`、`76a4fb8`、`f265fd6`、`5e00dfb`、`d374a63`、`d8eb718`、`9c12b5b`、`fb9911e`、`b0f0c86` | 4b: converter の src のテスト名を日本語に(error、intake、resolve、scalar、toml_read、toml_write、types、yaml_read、yaml_write) |
| `05dfda8`、`b4ab8a3`、`22fc5a4`、`e336b78` | 4b: converter の tests のテスト名を日本語に(convert_ac、convert_roundtrip、read_resolve、nfr_security)。language-agnostic の 11 章の参照も同じコミットで直す。`env_probe_child` の改名と `--exact` の文字列は同じコミット |
| `9475f6b` | 4c: `fake_wait` と、偽物の `run_job` の土台に `JobExecution::exited` |
| `7e58fde` | 4d: README の終了コード 68 の行と関連の行(`--remove-on-complete`、run ID の節、graph の節) |
| `6bf9b3f` | 4e: 5 つの仕様書の `00-README.md` に凍結の明記 |

## テストの件数(`cargo test --locked --workspace`、macOS)

| 時点 | 通った | 失敗 | 無視 |
|------|--------|------|------|
| 基準(Step 1、`513115b`) | 1,123 | 0 | 1 |
| 段 1 のあと(`1e4e6cf`) | 1,123 | 0 | 1 |
| 段 2 のあと(`ce74418`) | 1,123 | 0 | 1 |
| 段 3 のあと(`8e25bbf`) | 1,123 | 0 | 1 |
| 段 4 のあと(`6bf9b3f`) | 1,117 | 0 | 1 |

減った 6 件は、4a でまとめた 6 件。converter は改名だけで、件数は 117(116 通過 + 1 無視)のまま。

最後に `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --locked --workspace` が通ることを確かめた(Step 10)。行カバレッジは Build and Test が確かめる。

## 4a でまとめた 6 件(入力と期待の全文がまったく同じもの)

| # | 残したテスト | 消したテスト |
|---|--------------|--------------|
| 1 | `graph_shape_skip::c1_a_が_d_へ_b_が_e_へ飛ぶ交差は_行_a_と行_b_で描く`(旧 `graph_crossing_lanes`) | `graph_lane_polish::交差する通常の飛び越しは_レーンの図になる` |
| 2 | `graph_shape_both::交差に_両方の飛び越しが混ざる形は_レーンの図になる`(旧 `graph_mixed_shapes`。2 つのコメントを合わせた) | `graph_lane_polish::交差と同じ行き先に_両方の飛び越しが混ざる形は_レーンの図になる` |
| 3 | `graph_shape_both::d1_両方の飛び越しは_通常の飛び越しと同じ線で描き_主線の線だけ省く`(旧 `graph_lane_polish`。unify 側のコメントを足した) | `graph_layout_unify::d1_両方の飛び越しの図は_変わらない` |
| 4 | `graph_shape_both::n1_両方の飛び越しと通常の飛び越しの交差`(旧 `graph_layout_unify`。lane_polish 側のコメントを足した) | `graph_lane_polish::両方の飛び越しが交差する形は_通常の飛び越しが交差する形と同じくレーンの図になる` |
| 5 | `graph_shape_branch::b1_分岐は_2本の線を同時に開き_実行されうるジョブの下だけ主線を描く`(旧 `graph_branch_lanes`) | `graph_layout_unify::b1_分岐の図は_実行されうるジョブの下の合流の行だけ主線を描く` |
| 6 | `graph_perf::幅の広い交差の最悪の形は_1000_ジョブでも_1秒以内`(行の数も確かめる) | `graph_crossing_lanes::幅の広い交差の最悪の形は_1000_ジョブでも_1秒以内` |

## テストのファイルの変更の一覧

### 段 1〜3

- `crates/cli/tests/report_messages.rs`(段 2a。承認済みの例外): `with_all_events` の結果の値を `Some(Classification::Success|Warning|Error)`、`None` に、遷移の `on` を `TransitionOn::Warning`・`TransitionOn::Success` に。`v_の文言` の `result: "warning"`・`"error"` を `Some(Classification::Warning)`・`Some(Classification::Error)` に、`on: "warning"` を `TransitionOn::Warning` に。`conductor_core` の import に `Classification`、`TransitionOn` を足した。**期待する出力の文字列に差分はない**(`git diff` で確かめた)。
- ほかのテストのファイルは、段 1〜3 のコミットで変えていない(`git diff --stat 513115b ce74418 -- 'crates/*/tests/*'` は `report_messages.rs` だけ、`git diff --stat ce74418 8e25bbf -- 'crates/*/tests/*'` は空)。src の中の `mod tests` も変えていない(`runtime/src/state/writer.rs` と `consistency.rs` は、`mod tests` の `use super::*` のために、本体の側に `#[cfg(test)] use` を置いた)。

### 段 4

- 4a: `crates/core/tests/graph_lane_polish.rs`、`graph_branch_lanes.rs`、`graph_crossing_lanes.rs`、`graph_mixed_shapes.rs`、`graph_layout_unify.rs` を消し(`git rm`)、`graph_shape_skip.rs`(5)、`graph_shape_same_target.rs`(6)、`graph_shape_both.rs`(14)、`graph_shape_branch.rs`(12)を作った。`graph_drawable.rs` に、辺の一覧に残る形 3 件(旧 lane_polish 2 件、旧 branch 1 件)と主線の列 0 の 2 件(旧 unify)を移した(16 → 21)。`graph_common/mod.rs` に `letter_definition`、`lanes`、`both` を足した。移したテストの入力と期待は変えていない(主線の列 0 のテストは、`n1()` の代わりに同じ遷移をその場で組み立てる)。
- 4b: converter の 13 ファイルの、テスト名(117 件)だけを変えた。`nfr_security.rs` は、`--exact` に渡す名前と `#[ignore]` の理由の文も合わせた。対応は次の表。
- 4c: `crates/cli/tests/common/mod.rs` に `fake_wait` を足し、`ScriptedRuntime`・`MemRuntime` の `wait` から呼ぶ。両方の `run_job` の土台を `JobExecution::exited` で作る(Scripted は `exit_code: None` で上書き)。`waits`、`interrupt_wait` のフィールドは残した。`flow_retry.rs` は変えていない。

### 4b の改名の対応

| ファイル | 旧 | 新 |
|----------|----|----|
| src/error.rs | display_has_no_input_values_for_every_kind | 表示は_どの種類でも入力の値を含まない |
| src/error.rs | display_without_line_and_with_key_path | 行がなく_キーの場所がある表示 |
| src/error.rs | sanitize_escapes_control_and_format_chars | sanitize_は制御文字と書式文字をエスケープする |
| src/error.rs | sanitize_truncates_to_64_chars | sanitize_は64文字で切り詰める |
| src/error.rs | key_path_format | キーの場所の書式 |
| src/intake.rs | missing_file_is_file_missing | 存在しないファイルは_file_missing |
| src/intake.rs | exactly_one_mib_is_accepted_and_one_more_is_too_large | ちょうど1_mibは受理し_1バイト多いと大きすぎる |
| src/intake.rs | endless_reader_stops_at_limit_plus_one | 終わりのない入力は_上限より1バイト多いところで止まる |
| src/intake.rs | invalid_utf8_is_unreadable | utf8_でない入力は読めない |
| src/intake.rs | directory_is_unreadable | ディレクトリは読めない |
| src/intake.rs | too_large_wins_over_invalid_utf8 | 大きすぎるは_utf8_でないより優先する |
| src/intake.rs | mode_000_file_is_unreadable | 権限_000_のファイルは読めない |
| src/intake.rs | mode_000_file_over_1_mib_is_unreadable_not_too_large | 権限_000_で1_mibを超えるファイルは_大きすぎるでなく読めない |
| src/intake.rs | io_errors_map_to_fixed_reasons | 入出力の誤りは固定の理由に写す |
| src/resolve.rs | budget_exactly_at_limit_is_accepted_and_one_byte_over_is_too_large | 上限ちょうどの大きさは受理し_1バイト超は大きすぎる |
| src/resolve.rs | every_alias_visit_is_charged | エイリアスを訪れるたびに数える |
| src/resolve.rs | aliased_merge_keys_and_discarded_items_are_charged | エイリアスのマージキーと_捨てた要素も数える |
| src/resolve.rs | default_budget_is_16_mib | 既定の上限は16_mib |
| src/scalar.rs | core_schema_bool_and_null | コアスキーマの真偽値と_null |
| src/scalar.rs | core_schema_integers | コアスキーマの整数 |
| src/scalar.rs | core_schema_floats | コアスキーマの浮動小数 |
| src/scalar.rs | quoted_scalars_are_always_strings | 引用符つきのスカラーは常に文字列 |
| src/scalar.rs | tags | タグ |
| src/scalar.rs | scalar_value_maps_unrepresentable | スカラーの値は表せないものを写す |
| src/toml_read.rs | syntax_error_has_line | 文法の誤りは行を持つ |
| src/toml_read.rs | empty_toml_is_empty_map | 空の_toml_は空のマップ |
| src/toml_read.rs | datetime_is_marked | 日時は印をつける |
| src/toml_read.rs | depth_32_ok_and_33_syntax | 深さ32は受理し_33は文法の誤り |
| src/toml_read.rs | nested_tables_depth | 入れ子のテーブルの深さ |
| src/toml_read.rs | very_deep_inputs_fail_without_abort | とても深い入力は_異常終了せずに失敗する |
| src/toml_read.rs | i64_range_is_handled_by_the_library | i64_の範囲はライブラリが扱う |
| src/toml_write.rs | strings_are_quoted_and_survive | 文字列は引用符で囲み_そのまま残る |
| src/toml_write.rs | floats_and_integer_edges | 浮動小数と整数の端 |
| src/toml_write.rs | tables_arrays_of_tables_and_empty_values | テーブルと_テーブルの配列と_空の値 |
| src/toml_write.rs | empty_root_is_empty_text | 空の最上位は空の文字列 |
| src/toml_write.rs | non_map_root_is_an_error | マップでない最上位は誤り |
| src/types.rs | nan_equals_nan_and_signed_zero_differs | nan_は_nan_と等しく_符号の違う0は異なる |
| src/types.rs | types_never_equal_across_kinds | 種類が違えば等しくない |
| src/types.rs | map_equality_ignores_key_order_but_array_order_matters | マップの等しさはキーの順を見ず_配列は順を見る |
| src/yaml_read.rs | syntax_error_has_line | 文法の誤りは行を持つ |
| src/yaml_read.rs | duplicate_key_is_syntax_with_line | 重複するキーは行つきの文法の誤り |
| src/yaml_read.rs | recursive_anchor_is_syntax | 再帰するアンカーは文法の誤り |
| src/yaml_read.rs | unknown_alias_is_syntax | 知らないエイリアスは文法の誤り |
| src/yaml_read.rs | depth_32_ok_and_33_syntax | 深さ32は受理し_33は文法の誤り |
| src/yaml_read.rs | very_deep_inputs_fail_without_abort | とても深い入力は_異常終了せずに失敗する |
| src/yaml_read.rs | counts_documents_and_keeps_first_tree | 文書の数を数え_最初の木を残す |
| src/yaml_read.rs | scalars_keep_text_style_and_tags | スカラーは文字と書き方とタグを保つ |
| src/yaml_read.rs | anchors_and_aliases_are_kept_unexpanded | アンカーとエイリアスは展開せずに残す |
| src/yaml_write.rs | strings_that_look_like_other_types_are_quoted | ほかの型に見える文字列は引用符で囲む |
| src/yaml_write.rs | double_quoted_escapes | 二重引用符のエスケープ |
| src/yaml_write.rs | floats_keep_form | 浮動小数は形を保つ |
| src/yaml_write.rs | nested_structure_layout | 入れ子の構造の配置 |
| src/yaml_write.rs | empty_root_map | 空の最上位のマップ |
| tests/convert_ac.rs | ac_7_1_1_toml_to_yaml | ac_7_1_1_toml_から_yaml |
| tests/convert_ac.rs | ac_7_1_2_yaml_with_anchors_to_toml | ac_7_1_2_アンカーつきの_yaml_から_toml |
| tests/convert_ac.rs | ac_7_1_3_broken_input_returns_kind_and_line_and_no_output | ac_7_1_3_壊れた入力は種類と行を返し_出力しない |
| tests/convert_ac.rs | input_format_is_the_opposite_of_target_and_extension_is_ignored | 入力の形式は変換後の反対で_拡張子は見ない |
| tests/convert_ac.rs | convert_reports_file_level_failures_in_order | 変換はファイルの失敗を決まった順で報告する |
| tests/convert_ac.rs | exactly_one_mib_of_valid_input_converts | ちょうど1_mibの正しい入力は変換できる |
| tests/convert_ac.rs | convert_text_does_not_touch_files_and_uses_the_label | convert_text_はファイルに触れず_ラベルを使う |
| tests/convert_ac.rs | input_with_comments_converts_and_comments_are_dropped | コメントつきの入力は変換でき_コメントは捨てる |
| tests/convert_roundtrip.rs | definition_like_toml_roundtrips_through_yaml | 定義の形の_toml_は_yaml_を通って往復する |
| tests/convert_roundtrip.rs | definition_like_yaml_roundtrips_through_toml | 定義の形の_yaml_は_toml_を通って往復する |
| tests/convert_roundtrip.rs | array_command_roundtrips_toml_yaml_toml | 配列形式の_command_は往復する_toml_yaml_toml |
| tests/convert_roundtrip.rs | array_command_roundtrips_yaml_toml_yaml | 配列形式の_command_は往復する_yaml_toml_yaml |
| tests/convert_roundtrip.rs | merged_values_are_expanded_with_priority | マージした値は優先順位どおりに展開する |
| tests/convert_roundtrip.rs | scalar_boundaries_roundtrip_from_yaml | スカラーの境界は往復する_yaml_から |
| tests/convert_roundtrip.rs | scalar_boundaries_roundtrip_from_toml | スカラーの境界は往復する_toml_から |
| tests/convert_roundtrip.rs | floats_roundtrip_both_ways | 浮動小数は両方向に往復する |
| tests/convert_roundtrip.rs | i64_edges_roundtrip | i64_の端は往復する |
| tests/convert_roundtrip.rs | empty_and_nested_collections_roundtrip | 空と入れ子の集まりは往復する |
| tests/convert_roundtrip.rs | retry_table_roundtrips_through_yaml_and_back_to_toml | retry_の表は往復する_yaml_を通って_toml_に戻る |
| tests/convert_roundtrip.rs | retry_table_roundtrips_from_yaml | retry_の表は往復する_yaml_から |
| tests/convert_roundtrip.rs | deepest_allowed_structure_roundtrips | 許される最も深い構造は往復する |
| tests/convert_roundtrip.rs | output_is_deterministic | 出力は毎回同じ |
| tests/nfr_security.rs | source_has_no_process_net_or_env_access | ソースはプロセスとネットワークと環境変数に触れない |
| tests/nfr_security.rs | source_has_no_static_initialization | ソースは静的な初期化を持たない |
| tests/nfr_security.rs | source_does_not_use_unwrap_or_panic_on_input_paths | ソースは入力の経路で_unwrap_や_panic_を使わない |
| tests/nfr_security.rs | convert_creates_no_files_outside_input_and_ignores_cwd_changes | 変換はファイルを作らず_カレントディレクトリの変更に左右されない |
| tests/nfr_security.rs | env_probe_child | 環境を調べる子プロセス |
| tests/nfr_security.rs | env_independence | 環境変数に左右されない |
| tests/nfr_security.rs | tags_do_not_construct_values | タグは値を組み立てない |
| tests/nfr_security.rs | display_never_contains_input_values_for_every_kind | 表示は_どの種類でも入力の値を含まない |
| tests/nfr_security.rs | library_error_text_never_appears | ライブラリの誤りの文は現れない |
| tests/nfr_security.rs | key_names_are_escaped_and_truncated_in_key_path | キーの場所のキー名は_エスケープし切り詰める |
| tests/nfr_security.rs | out_of_range_empty_and_broken_inputs_do_not_panic | 範囲外と空と壊れた入力で_パニックしない |
| tests/nfr_security.rs | random_and_truncated_inputs_never_panic | ランダムな入力と切り詰めた入力で_パニックしない |
| tests/nfr_security.rs | successful_random_structures_roundtrip | 変換できたランダムな構造は往復する |
| tests/read_resolve.rs | multiple_documents_are_unrepresentable | 複数の文書は表せない |
| tests/read_resolve.rs | root_must_be_a_map | 最上位はマップでなければならない |
| tests/read_resolve.rs | anchors_and_aliases_expand | アンカーとエイリアスを展開する |
| tests/read_resolve.rs | alias_used_as_a_key | キーに使ったエイリアス |
| tests/read_resolve.rs | merge_key_single_list_and_priority | マージキーの単独と並びと優先順位 |
| tests/read_resolve.rs | merge_key_with_inline_map_and_own_key_before_merge | インラインのマップのマージキーと_マージより前の自身のキー |
| tests/read_resolve.rs | quoted_merge_key_is_a_normal_key | 引用符つきのマージキーは普通のキー |
| tests/read_resolve.rs | merge_key_with_non_map_value_is_unrepresentable | 値がマップでないマージキーは表せない |
| tests/read_resolve.rs | duplicate_key_through_alias_is_syntax | エイリアスを通した重複するキーは文法の誤り |
| tests/read_resolve.rs | node_limit_exactly_100000_is_accepted_and_one_more_is_too_large | 要素数ちょうど100000は受理し_1つ多いと大きすぎる |
| tests/read_resolve.rs | toml_input_has_the_same_node_limit | toml_の入力も要素数の上限は同じ |
| tests/read_resolve.rs | alias_bomb_is_too_large_without_exhausting_memory | エイリアスの爆発は_メモリを使い果たさずに大きすぎる |
| tests/read_resolve.rs | alias_bomb_of_large_scalars_is_too_large_by_size | 大きなスカラーのエイリアスの爆発は_大きさで大きすぎる |
| tests/read_resolve.rs | alias_bomb_through_merge_keys_and_values_is_too_large_by_size | マージキーと値を通したエイリアスの爆発は_大きさで大きすぎる |
| tests/read_resolve.rs | expanded_size_budget_boundary_with_the_default_16_mib | 既定の16_mibでの展開後の大きさの境界 |
| tests/read_resolve.rs | merge_of_a_big_map_many_times_counts_discarded_items | 大きなマップを何度もマージすると_捨てた要素も数える |
| tests/read_resolve.rs | expanded_depth_over_32_is_syntax_even_when_read_depth_is_small | 読んだ深さが小さくても_展開後の深さが32を超えれば文法の誤り |
| tests/read_resolve.rs | read_depth_limit_32_33 | 読み取りの深さの上限_32_と_33 |
| tests/read_resolve.rs | extremely_deep_inputs_fail_with_syntax_not_abort | 極端に深い入力は_異常終了せずに文法の誤りで失敗する |
| tests/read_resolve.rs | first_unrepresentable_value_in_input_order_with_key_path | 入力の順で最初の表せない値を_キーの場所つきで報告する |
| tests/read_resolve.rs | null_forms_and_other_unrepresentable_kinds | null_の書き方と_ほかの表せない種類 |
| tests/read_resolve.rs | toml_datetime_is_unrepresentable | toml_の日時は表せない |
| tests/read_resolve.rs | tag_str_and_matching_core_tags_are_accepted | str_のタグと_一致するコアのタグは受理する |
| tests/read_resolve.rs | core_schema_scalars | コアスキーマのスカラー |
| tests/read_resolve.rs | failure_order_syntax_before_everything_after_it | 失敗の順_文法の誤りはその後のすべてより先 |
| tests/read_resolve.rs | failure_order_multi_document_before_root_not_map_before_values | 失敗の順_複数の文書_最上位がマップでない_値の順 |
| tests/read_resolve.rs | failure_order_expansion_limits_before_unrepresentable_values | 失敗の順_展開の上限は表せない値より先 |
| tests/read_resolve.rs | failure_order_first_in_input_order_within_the_same_step | 失敗の順_同じ段の中では入力の順で最初のもの |
| tests/read_resolve.rs | error_messages_never_contain_input_values | 誤りの文は入力の値を含まない |

## 変えた文書

- `docs/spec/rust/02・04・05・06・07・10・12`(段 1〜3 の内部の名前)、`docs/spec/rust/05`(804 行の図の例の置き場所)、`docs/spec/rust/11`(graph のテストのファイルと件数)。
- `docs/spec/language-agnostic/11-quality-and-build.md`: AC-475、AC-476a〜e の graph のテストのファイル名、converter のテスト名の参照(AC-206〜AC-234、AC-416)。`convert_ac::ac_7_1_3` は、新しい名前も `ac_7_1_3_` で始まるので、そのまま。
- `README.md`: 終了コード 68 の行、`--remove-on-complete` の行、run ID の節の値の規則、graph の節の終了コード。必須の語句(`終了コードの表`、`| 69 |` ほか)は残した。
- `docs/spec/{go,python,typescript,csharp,powershell}/00-README.md`: 見出しの直後に凍結の明記。中身は変えていない(中の converter のテスト名の参照も、凍結版としてそのまま)。

## 計画からのずれ

- 3a: `git mv` の前に、移す先のディレクトリ `crates/core/src/definition/walk/` を `mkdir -p` で作った(`git mv` は、無いディレクトリに移せない)。
- 4a: 消した 5 つのファイルは `git rm` で消した(移す先が 4 つの新しいファイルと既存の `graph_drawable.rs` に分かれるため、`git mv` で 1 対 1 にできない)。
- 2b: `runtime/src/state/writer.rs` の単体テストが `use super::*` で `JobStatus` を使うため、本体の import から外した `JobStatus` を `#[cfg(test)] use` で残した(2c と同じ形)。
- 1a〜1d の内部の名前(`start_signals`、`require_known`、`check_run_inputs`、`enter_flow`)は、設計にない名前を、設計の部品の説明に合わせて付けた。

## 見送ったこと(確認の Minor の指摘)

- src の `mod tests` の補助(next_step、classify、record、ctx)に、`JobExecution::exited` と同じ組み立てが残る。段 1〜3 は src のテストを変えない方針(Q3: A)のため、寄せていない。後の intent で寄せられる。
- `enter_flow` の「位置引数なし」の分岐は到達しない。動きは同じため、そのまま残した。後の args.rs の intent で引数の読み取りを整理するときに消せる。
