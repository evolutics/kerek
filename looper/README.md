# Kerek ☸

Build, test, deploy your app on each commit.

## Flow

```

                    Kerek               Kerek               Production
                    binary              staging VM          environment
                                                            ╷
                                                            ┆
$ kerek run                                                 ┆
●╶────────╭───╴>╶───╮                                       ┆
          │         │                                       ┆
          │         │ Provision                             ┆
          │         ●╶──────────────────╮                   ┆
          │         │                   ┆                   ┆
          │         │ Build             ┆                   ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Deploy            ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Staging env tests ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Deploy            ┆                   ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Production env tests                  ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Move to next version                  ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Repeat            ┆                   ┆
          ╰───╴<╶───╯                   ╵                   ╵
```
