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
          │         │ Env tests         ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Deploy            ┆                   ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Env tests         ┆                   ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Move to next version                  ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Repeat            ┆                   ┆
          ╰───╴<╶───╯                   ╵                   ╵
```
