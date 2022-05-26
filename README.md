# Kerek ☸

Build, test, deploy your app on each commit.

## Flow

```

                    Kerek               Kerek               Production
                    binary              staging VM          environment
                                                            ╷
                                                            ┆
$ kerek run                                                 ┆
●╶──────────────────╮                                       ┆
                    │                                       ┆
                    │ Provision                             ┆
                    ●╶──────────────────╮                   ┆
                    │                   ┆                   ┆
                    │ Save snapshot     ┆                   ┆
                    ●╶──────────────────┤                   ┆
                    │                   ┆                   ┆
                    │                   ┆                   ┆
          ╭───╴>╶───╮                   ┆                   ┆
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
          │         │ Load snapshot     ┆                   ┆
          │         ●╶──────────────────┤                   ┆
          │         │                   ┆                   ┆
          │         │ Move to next version                  ┆
          │         ●                   ┆                   ┆
          │         │                   ┆                   ┆
          │         │ Repeat            ┆                   ┆
          ╰───╴<╶───╯                   ╵                   ╵
```
