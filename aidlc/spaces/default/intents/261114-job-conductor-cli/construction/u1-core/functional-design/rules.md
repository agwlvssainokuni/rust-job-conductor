# Rules(u1-core)

上流: requirements(FR1〜FR4、NFR4)、contract-summary(C2、E2)、entities(この Unit の entities.md)、stories。各規則は、`BR{group}.{seq}` の ID を持ち、出典(FR / NFR)を示す。

```yaml
rules:
  # グループ1: 定義(DefinitionLoader)
  - { id: BR1.1, statement: 遷移先に書いたジョブIDは、定義に存在する, category: validation, applies_to: Transition, trigger: 定義の読み込み時, logic: "IF 遷移先が @end でなく、かつ jobs に同じ id が無い THEN DefinitionError(unknownTarget)", violation: ジョブを実行せず、ファイル名・行・原因を返す, source: FR1.4 }
  - { id: BR1.2, statement: エラー結果には遷移先を書けない, category: constraint, applies_to: Transition, trigger: 定義の読み込み時, logic: "IF Transition.on が error THEN DefinitionError(errorTransition)", violation: 同上, source: FR1.3、FR1.4 }
  - { id: BR1.3, statement: 遷移先は、記述順で後ろのジョブ、または @end のみ, category: constraint, applies_to: Transition, trigger: 定義の読み込み時, logic: "IF 遷移先のジョブが、現在のジョブ自身、または記述順で前にある THEN DefinitionError(backwardOrSelfTarget)", violation: 同上, source: FR1.3、FR1.4 }
  - { id: BR1.4, statement: 警告と成功の終了コードは重ならず、0 は警告にできない, category: validation, applies_to: JobDefinition, trigger: 定義の読み込み時, logic: "IF warningExitCodes と successExitCodes に共通の値がある THEN DefinitionError(overlappingExitCodes). IF warningExitCodes に 0 が含まれる THEN DefinitionError(zeroAsWarning)", violation: 同上, source: FR1.4 }
  - { id: BR1.5, statement: 型の誤りを、ジョブを実行する前に検知する, category: validation, applies_to: PipelineDefinition、JobDefinition, trigger: 定義の読み込み時, logic: "IF 値の型が、定義された型と異なる(例: 終了コードが文字列)THEN DefinitionError(typeMismatch)", violation: 同上, source: NFR4、FR1.4 }
  - { id: BR1.6, statement: 定義にないキーは、エラーにする, category: validation, applies_to: PipelineDefinition、JobDefinition、Transition, trigger: 定義の読み込み時(版の確認のあと), logic: "IF 知らないキーが書かれている THEN DefinitionError(unknownKey) で、キーと行を含める", violation: 同上, source: contract-summary の E2、Contract Design の Q3 }
  - { id: BR1.7, statement: 書式の版は必須で、対応する版のみ受け付ける。版の確認は、他の検査より先に行う, category: validation, applies_to: PipelineDefinition, trigger: 定義の読み込み時, logic: "IF formatVersion が無い THEN DefinitionError(versionMissing). IF 対応する版(初版は 1)でない THEN DefinitionError(versionUnsupported). 版の確認が通らなければ、他の検査はしない", violation: 同上, source: contract-summary の E2、Contract Design の Q2 }
  - { id: BR1.8, statement: pipeline コマンドに渡された名前は、定義の name と一致する, category: validation, applies_to: PipelineDefinition, trigger: pipeline コマンドでの読み込み時(expectedPipelineName があるとき), logic: "IF expectedPipelineName が name と異なる THEN DefinitionError(pipelineNameMismatch)", violation: 同上, source: FR1.5 }
  - { id: BR1.9, statement: 定義ファイルが存在しない・読めないときは、エラーにする, category: validation, applies_to: PipelineDefinition, trigger: 定義の読み込み時, logic: "IF 指定したパスにファイルが無い、または読めない THEN DefinitionError(fileMissing または unreadable) で、パスを含める", violation: ジョブを実行しない, source: FR1.4、FR1.6 }
  - { id: BR1.10, statement: 遷移先を書かない成功・警告は、記述順で次のジョブへ進む, category: policy, applies_to: NextStep, trigger: ジョブの結果の判断時, logic: "IF 結果に対応する Transition が無い THEN nextJobId は、記述順で次のジョブ。次が無ければ、完了", violation: なし(既定の動き), source: FR1.3 }
  - { id: BR1.11, statement: 正しい定義は受理し、名前・説明・ジョブ・遷移を取り出す, category: policy, applies_to: PipelineDefinition, trigger: 定義の読み込み時, logic: "IF BR1.1〜BR1.9 のどれにも違反しない THEN PipelineDefinition を返す(説明は省略可、successExitCodes の省略時は [0])", violation: なし, source: FR1.1、FR1.2 }
  - { id: BR1.12, statement: ジョブの順序は、定義ファイルに書いた順とする(1ファイルに1パイプライン), category: policy, applies_to: PipelineDefinition、JobDefinition, trigger: 定義の読み込み時, logic: "jobs の並びは、書いた順を保つ。1ファイルに2つ以上のパイプラインがあれば、DefinitionError(typeMismatch)として拒否する", violation: 同上, source: FR1.5 }

  # グループ2: 分類と終了コード(Flow)
  - { id: BR2.1, statement: 終了コードを、成功・警告・エラーに分類する, category: calculation, applies_to: JobOutcome, trigger: ジョブが終了コードで終わったとき(started が true で、中断・起動失敗・シグナル終了でない), logic: "IF 終了コードが successExitCodes に含まれる THEN success. ELSE IF warningExitCodes に含まれる THEN warning. ELSE error. (successExitCodes が省略なら [0]。書いたら、0 を書かなければ 0 でも成功にならない)", violation: なし, source: FR2.2 }
  - { id: BR2.2, statement: スケジューラへ返す終了コードを決める, category: calculation, applies_to: JobOutcome, trigger: 分類の直後, logic: "IF success THEN 0. IF warning THEN 0. IF error で、終了コードがあり、それが 0 でない THEN そのまま. IF error で、終了コードが 0(成功コードに含まれない 0)THEN 1 で、conductorError を true にする", violation: なし, source: FR2.2、FR2.5 }
  - { id: BR2.3, statement: 起動失敗・シグナル終了・中断は、エラーとして扱い、終了コードを決める, category: calculation, applies_to: JobOutcome, trigger: JobExecution が起動失敗・シグナル終了・中断のとき, logic: "IF startFailure THEN error、127. IF terminatedBySignal がある THEN error、128 + その番号. IF interrupted で started が true THEN ジョブ自身の終了コードにかかわらず error、128 + signalNumber", violation: なし, source: FR2.2、FR2.9 }
  - { id: BR2.4, statement: 起動しなかった中断は、エラーにしない, category: policy, applies_to: JobOutcome、NextStep, trigger: JobExecution の started が false で interrupted のとき, logic: "分類は行わず、そのジョブを再開位置(resumeJobId)にする。スケジューラへ返す終了コードは 128 + signalNumber", violation: なし, source: FR2.9、Contract Design の引き継ぎ }

  # グループ3: 遷移・スキップ・完了(Flow)
  - { id: BR3.1, statement: 次に実行するジョブを決める, category: policy, applies_to: NextStep, trigger: ジョブの結果の判断時, logic: "IF 結果に対応する Transition があり、next がジョブID THEN nextJobId はそのID. IF next が @end THEN endReached を true にし、nextJobId は無し. ELSE BR1.10", violation: なし, source: FR2.3 }
  - { id: BR3.2, statement: 遷移先を指定したとき、現在のジョブと遷移先のあいだのジョブは、スキップ扱いにする, category: policy, applies_to: NextStep, trigger: 遷移先がジョブIDのとき, logic: "skippedJobIds は、記述順で現在のジョブの次から、遷移先の直前までの全ジョブ。skipReasonKind は byTransition", violation: なし, source: FR2.3 }
  - { id: BR3.3, statement: @end 以降のジョブは、すべてスキップ扱いにする, category: policy, applies_to: NextStep, trigger: 遷移先が @end のとき, logic: "skippedJobIds は、記述順で現在のジョブより後ろの全ジョブ。skipReasonKind は byEnd。endReached を true にする", violation: なし, source: FR2.3、FR4.2 }
  - { id: BR3.4, statement: エラーで終わったら停止し、再試行はしない。再開位置はエラーになったジョブとする, category: policy, applies_to: NextStep, trigger: classification が error のとき, logic: "nextJobId は無し(停止)。resumeJobId はそのジョブ。skippedJobIds は空。自動の再試行はしない", violation: なし, source: FR3.2、FR4.3 }
  - { id: BR3.5, statement: ジョブを起動する前に、実行・スキップ・拒否を決める, category: policy, applies_to: Decision, trigger: ジョブが呼ばれたとき, logic: "IF そのジョブが skipped(byTransition または byEnd)として記録されている、または endReached が true で、そのジョブが @end より後 THEN skip(理由 skipTarget または afterEnd). IF そのジョブが success または warning で記録済み THEN skip(理由 alreadyExecuted)[Q2: A]. IF そのジョブが error で記録済み、または resumeJobId、または記述順で最初のジョブでステートが空 THEN run. IF それ以外で、直前までに実行されているはずのジョブが、記録されていない(順序外)THEN reject(理由 outOfOrder)[Q1: A]", violation: reject のとき、ジョブを実行せず、ステートも変更しない, source: FR2.4、FR4.3、Functional Design の Q1・Q2 }
  - { id: BR3.6, statement: パイプラインの完了を判定する, category: policy, applies_to: NextStep, trigger: ジョブの結果の判断後、またはスキップの呼び出し時, logic: "IF 記述順で最後のジョブが、実行(success または warning)またはスキップで終わった THEN completed を true にする. IF 一連の実行で @end に到達した THEN completed を true にする. IF 個別実行で、@end のあとのジョブが呼ばれ、最後のジョブがスキップされた THEN completed を true にする", violation: なし, source: FR4.2 }
  - { id: BR3.7, statement: ジョブとジョブのあいだで中断されたときの、再開位置を決める, category: policy, applies_to: NextStep, trigger: 中断の通知(ジョブを起動していないとき), logic: "resumeJobId は、まだ起動していない、次に実行するジョブ。最後のジョブの直後なら、resumeJobId は無しで、completed を true にする。endReached が true なら、その値を引き継ぐ。エラーとしては記録しない", violation: なし, source: FR2.9、Contract Design の引き継ぎ }
  - { id: BR3.8, statement: 順序外の呼び出しを拒否し、ステートを変更しない, category: constraint, applies_to: Decision, trigger: BR3.5 が reject を返したとき, logic: "reject を返す。呼び出し側は、conductor 自身のエラーとして理由を表示し、終了コード 66 で終了する", violation: なし(拒否そのものが、規則の動き), source: Functional Design の Q1 }

  # グループ4: 定義の場所
  - { id: BR4.1, statement: 定義ファイルの場所を決める, category: policy, applies_to: PipelineDefinition, trigger: 定義の読み込み時, logic: "IF パスの指定がある THEN そのパス. ELSE 実行ディレクトリの conductor.toml", violation: BR1.9 を参照, source: FR1.6 }

  # グループ5: conductor 自身のエラーの終了コード
  - { id: BR5.1, statement: conductor 自身のエラーの終了コードを、種類ごとに決める, category: policy, applies_to: DefinitionError、Decision, trigger: conductor 自身のエラーで終了するとき, logic: "定義の誤り 64、ステートの誤り 65、拒否 66、変換の誤り 67", violation: なし, source: Functional Design の Q3 }
```

## 規則の要約

| グループ | 範囲 | 主な規則 |
|----------|------|----------|
| BR1 | 定義の読み込みと検証 | BR1.1〜BR1.12(誤りの種類、版、不明なキー、順序、既定の遷移) |
| BR2 | 結果の分類と、返す終了コード | BR2.1〜BR2.4 |
| BR3 | 遷移・スキップ・完了・再開・拒否 | BR3.1〜BR3.8 |
| BR4 | 定義ファイルの場所 | BR4.1 |
| BR5 | conductor 自身のエラーの終了コード | BR5.1 |

## Assumptions & Open Questions

- BR1.12 の「1ファイルに2つ以上のパイプラインがあれば拒否」の、具体的なエラーの種類(`typeMismatch` に含めるか、別の種類にするか)は、Code Generation で決める。 [assumption]
- BR3.5 の「直前までに実行されているはずのジョブが、記録されていない」の判定は、記述順で、その前のジョブがすべて、実行済み(成功・警告)またはスキップとして記録されていないこと、として実装する。遷移でスキップされた間のジョブは、スキップとして記録されるため、順序外ではない。 [assumption]
