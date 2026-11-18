# Reverse Engineering Timestamp

## Run Record

- Date: 2026-11-19
- Commit: db631cd088bd66b846b1f5b0fcb33ccaa5c0d2d0

## Scope of Analysis

```yaml
scope_version: 1
kind: full
intent: 261118-job-retry
fingerprint: d34268005e41cad032f757ebe3e4a46ffa9bad2b
analyzed:
  paths:
    - ./
  components:
    - core::Types
    - core::DefinitionLoader
    - core::Flow
    - core::RunId
    - core::Describe
    - runtime::JobRunner
    - runtime::CommandSplit
    - runtime::Platform
    - runtime::Types
    - runtime::StateStore
    - runtime::Recorder
    - runtime::Consistency
    - cli::ArgParser
    - cli::Orchestrator
    - cli::AuxCommands
    - cli::Deps
    - cli::Reporter
    - cli::WatchView
    - cli::Escape
    - cli::SignalWatcher
    - cli::ExitMapper
    - cli::PanicHook
    - converter::Converter
    - TestHelpers
shallow:
  paths: []
```
