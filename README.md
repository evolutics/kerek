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
          │         │ Base tests        ┆                   ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Build             ┆                   ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Deploy            ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Smoke tests       ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Acceptance tests  ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Deploy            ┆                   ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Smoke tests       ┆                   ┆
          │         ●╶──────────────────────────────────────┤
          │         │                   ┆                   ┆
          │         │ Move to next version                  ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Repeat            ┆                   ┆
          ╰───╴<╶───╯                   ╵                   ╵
```
