# エンティティ(Entities): args.rs のオプションの表化

永続のデータ、公開の型(`CliCommand`、`ParsedArgs`、`UsageError`)は変えない。`args.rs` の中の非公開の型だけを足す。

```yaml
entities:
  - name: Opt
    description: オプションの識別子(args.rs の内部。新規)。Verbose、Config、To、RunId、All、RemoveOnComplete、DryRun、Force、Assume、Format、State。
  - name: OptSpec
    description: オプションの表の 1 行(args.rs の内部。新規)。
    attributes:
      - name: opt
        logical_type: Opt
      - name: long
        logical_type: 長い名前の文字列
      - name: takes_value
        logical_type: 真偽
      - name: allowed
        logical_type: Allowed(Commands(コマンドの種類の並び)か WithDryRun)
  - name: Check
    description: 検査の順の表の 1 つ(args.rs の内部。新規)。NotAllowed(Opt) か Duplicate(Opt)。
  - name: Token
    description: 字句に分けた 1 つの引数(args.rs の内部。既存)。オプションごとの変種を Opt(Opt, 値) の 1 つにまとめる。Unknown と Positional は残す。
```
