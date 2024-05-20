# Wheelsticks: Zero-downtime deployments for Docker Compose

Wheelsticks is an addition to Docker Compose that helps updating services with
zero downtime.

## Motivation

Docker Compose offers simple, declarative orchestration of containerized apps.

When updating a service container with Docker Compose using `docker compose up`,
the old container is stopped _before_ a new container is started
(**`stop-first`** case):

```
    Old container          Stop
┄┄┄┄───────────────────────┤
                                           Start       New container
                                           ├────────────────────────┄┄┄┄
```

This causes a service interruption as there is a time when neither container is
available.

Imagine that we could make the container lifetimes overlap instead
(**`start-first`** case):

```
    Old container                          Stop
┄┄┄┄───────────────────────────────────────┤
                           Start                       New container
                           ├────────────────────────────────────────┄┄┄┄
```

If a reverse proxy seamlessly switches traffic over from old to new container,
then a zero-downtime deployment is achieved.

The
[Compose specification](https://github.com/compose-spec/compose-spec/blob/master/deploy.md)
in fact defines options to distinguish above two cases: `stop-first` (default)
and `start-first`. But support for this part of the specification is optional,
and plain Docker Compose always applies `stop-first` irrespective of what is in
your Compose files.

However, Wheelsticks supports both options. Just run `wheelsticks deploy` in
place of `docker compose up`. No need to change any other Docker or Docker
Compose workflows.

## Installation

### Prerequisites

- Docker or Podman
- Docker Compose

### Installing

See [releases](https://github.com/evolutics/wheelsticks/releases) for
pre-compiled executables.

Alternatively, build from source with `cargo install wheelsticks`.

### Docker CLI plugin

Optionally, Wheelsticks can be set up as a Docker CLI plugin. With that, calls
to `wheelsticks deploy` can be replaced by `docker deploy`, which some people
prefer. Example [setup](https://github.com/docker/cli/issues/1534):

```bash
mkdir --parents ~/.docker/cli-plugins
ln --symbolic "$(which wheelsticks)" ~/.docker/cli-plugins/docker-deploy
```

## Usage

### Quick start

For services whose container lifetimes should overlap during an update,
configure their update order like so:

```yaml
# compose.yaml
services:
  greet:
    deploy:
      update_config:
        order: start-first # Most important line.
    # …
```

See [`example/compose.yaml`](example/compose.yaml) for a demo. It defines a
service called `greet` made available on localhost:8080 via a `reverse-proxy`:

```
    localhost:8080    ╭───────────────╮        ╭───────────────╮
──────────────────────┤ reverse-proxy ├────────┤ greet         │
                  :80 │ (stop-first)  │    :80 │ (start-first) │
                      ╰───────────────╯        ╰───────────────╯
```

With this design the service stays available, even during updates. You can play
with it as follows:

```bash
cd example
wheelsticks deploy --wait
curl localhost:8080 # … prints "Hi from A"

export GREET_VERSION=B
wheelsticks deploy --wait
curl localhost:8080 # … prints "Hi from B"

docker compose down
```

To see above deployments in action, use a separate shell session to run

```bash
while true; do curl --fail --max-time 0.2 localhost:8080; sleep 0.01s; done
```

### Conditions when services are updated

By default, a service is updated only if its service config hash changes. This
hash is calculated over all service fields in the Compose file except `build`,
`deploy.replicas`, `pull_policy`, and `scale`
(see [source](https://github.com/docker/compose/blob/main/pkg/compose/hash.go)).

Note that the service config hash does _not_ depend on the container image
contents but just the `image` field. Thus, reusing an image tag like `latest`
does not cause an update.

Using `--force-recreate` always updates services irrespective of config hash
changes.

| Command                                 | Effect                                        |
| --------------------------------------- | --------------------------------------------- |
| `wheelsticks deploy`                    | Update all services with changed config hash  |
| `wheelsticks deploy --dry-run`          | Update nothing but show what would be changed |
| `wheelsticks deploy x`                  | Update service `x` if its config hash changed |
| `wheelsticks deploy --force-recreate`   | Always update all services                    |
| `wheelsticks deploy --force-recreate x` | Always update service `x`                     |
| `docker compose config --hash '*'`      | Show service config hashes for Compose file   |

### Service update process

Services are updated in alphabetical order (more precisely, in lexicographical
order by Unicode code point).

For each service, containers are stopped then started (`stop-first`, default) or
started then stopped (`start-first`), respectively, and this is repeated for
replicas. The following visualizes the process for a service with 3 replicas.

**`stop-first` case:**

```
               1. Stop old
┄┄┄┄───────────┤

                       2. Start new
                       ├────────────────────────────────────────────┄┄┄┄

                               3. Stop old
┄┄┄┄───────────────────────────┤

                                       4. Start new
                                       ├────────────────────────────┄┄┄┄

                                               5. Stop old
┄┄┄┄───────────────────────────────────────────┤

                                                       6. Start new
                                                       ├────────────┄┄┄┄
```

**`start-first` case:**

```
               1. Start new
               ├────────────────────────────────────────────────────┄┄┄┄

                       2. Stop old
┄┄┄┄───────────────────┤

                               3. Start new
                               ├────────────────────────────────────┄┄┄┄

                                       4. Stop old
┄┄┄┄───────────────────────────────────┤

                                               5. Start new
                                               ├────────────────────┄┄┄┄

                                                       6. Stop old
┄┄┄┄───────────────────────────────────────────────────┤
```

### Podman support

Pass `--container-engine podman` to use Podman instead of Docker.

Podman Compose is not supported as it currently lacks some needed features like
the calculation of service config hashes (`docker compose config --hash '*'`).

## Alternatives

Other lightweight options for single-node environments:

- [Docker or Podman without Compose](https://github.com/evolutics/zero-downtime-deployments-with-podman)
- [`docker rollout`](https://github.com/Wowu/docker-rollout)
- [Docker Swarm mode](https://docs.docker.com/engine/swarm/)
- [K3s](https://docs.k3s.io)
- PaaS like [CapRover](https://caprover.com), [Dokku](https://dokku.com), etc.
- [`podman kube play`](https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html)

## Command-line arguments reference

### `wheelsticks -h`

```
Zero-downtime deployments for Docker Compose

Usage: wheelsticks [OPTIONS] <COMMAND>

Commands:
  deploy               Create or update services
  provision            Provisions host with container engine
  run-with-ssh-config  Runs command with wrapped `ssh` in `$PATH` that uses
                           given SSH config
  help                 Print this message or the help of the given
                           subcommand(s)

Options:
      --config <CONFIG>        Location of client config files
  -c, --context <CONTEXT>      Name of the context to use to connect to the
                               daemon (overrides DOCKER_HOST env var and default
                               context set with "docker context use")
  -D, --debug                  Enable debug mode [aliases: verbose]
  -H, --host <HOST>            Daemon socket to connect to
  -l, --log-level <LOG_LEVEL>  Set the logging level [possible values: debug,
                               info, warn, error, fatal]
      --tls                    Use TLS; implied by --tlsverify
      --tlscacert <TLSCACERT>  Trust certs signed only by this CA
      --tlscert <TLSCERT>      Path to TLS certificate file
      --tlskey <TLSKEY>        Path to TLS key file
      --tlsverify              Use TLS and verify the remote
  -h, --help                   Print help
  -V, --version                Print version
```

### `wheelsticks deploy -h`

```
Create or update services

Usage: wheelsticks deploy [OPTIONS] [SERVICE_NAMES]...

Arguments:
  [SERVICE_NAMES]...  Services to consider

Options:
      --ansi <ANSI>
          Control when to print ANSI control characters [possible values: never,
          always, auto]
      --compatibility
          Run compose in backward compatibility mode
      --dry-run
          Execute command in dry run mode
      --env-file <ENV_FILE>
          Specify an alternate environment file
  -f, --file <FILE>
          Compose configuration files
      --parallel <PARALLEL>
          Control max parallelism, -1 for unlimited
      --profile <PROFILE>
          Specify a profile to enable
      --progress <PROGRESS>
          Set type of progress output [possible values: auto, tty, plain, quiet]
      --project-directory <PROJECT_DIRECTORY>
          Specify an alternate working directory (default: the path of the,
          first specified, Compose file)
  -p, --project-name <PROJECT_NAME>
          Project name
      --build
          Build images before starting containers
  -d, --detach
          This has no effect as detached mode is always on; for migration only
      --force-recreate
          Recreate containers even if their configuration hasn't changed
      --no-build
          Don't build an image, even if it's missing
      --no-start
          Don't start the services after creating them
      --pull <PULL>
          Pull image before running [possible values: always, missing, never]
      --quiet-pull
          Pull without printing progress information
      --remove-orphans
          Remove containers for services not defined in the Compose file
  -V, --renew-anon-volumes
          Recreate anonymous volumes instead of retrieving data from the
          previous containers
  -t, --timeout <TIMEOUT>
          Use this timeout in seconds for container shutdown when containers are
          already running
      --wait
          Wait for services to be running|healthy
      --wait-timeout <WAIT_TIMEOUT>
          timeout in seconds waiting for application to be running|healthy
      --compose-engine <COMPOSE_ENGINE>
          Compose engine to use; Podman Compose is not supported due to missing
          features [default: "docker compose"] [possible values: docker-compose,
          "docker compose"]
      --container-engine <CONTAINER_ENGINE>
          Container engine to use [default: docker] [possible values: docker,
          podman]
  -h, --help
          Print help (see more with '--help')
```

### `wheelsticks provision -h`

```
Provisions host with container engine

Usage: wheelsticks provision [OPTIONS] <HOST>

Arguments:
  <HOST>  Reference like "localhost", "[ssh://]<host>", "vagrant://[<vm>]"

Options:
      --force                    Go ahead without prompting user to confirm
  -F, --ssh-config <SSH_CONFIG>  Path to SSH config file
  -h, --help                     Print help
```

### `wheelsticks run-with-ssh-config -h`

```
Runs command with wrapped `ssh` in `$PATH` that uses given SSH config

Usage: wheelsticks run-with-ssh-config <SSH_CONFIG> <COMMAND>...

Arguments:
  <SSH_CONFIG>  Path to SSH config file
  <COMMAND>...  Program with arguments to run

Options:
  -h, --help  Print help (see more with '--help')
```
