# Kerek: Zero-downtime deployments for Docker Compose

Kerek is an addition to Docker Compose that helps updating services with zero
downtime.

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

However, Kerek supports both options. Just run `kerek deploy` in place of
`docker compose up`. No need to change any other Docker or Docker Compose
workflows.

## Installation

### Prerequisites

- Docker or Podman
- Docker Compose

### Installing

```
cargo install --git https://github.com/evolutics/kerek
```

### Docker CLI plugin

Optionally, Kerek can be set up as a Docker CLI plugin. With that, calls to
`kerek deploy` can be replaced by `docker deploy`, which some people prefer.
Example [setup](https://github.com/docker/cli/issues/1534):

```bash
mkdir --parents ~/.docker/cli-plugins
ln --symbolic "$(which kerek)" ~/.docker/cli-plugins/docker-deploy
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

Se
[`examples/zero_downtime_deployment/compose.yaml`](examples/zero_downtime_deployment/compose.yaml)
for a demo. It defines a service called `greet` made available on localhost:8080
via a `reverse-proxy`:

```
    localhost:8080    ╭───────────────╮        ╭───────────────╮
──────────────────────┤ reverse-proxy ├────────┤ greet         │
                  :80 │ (stop-first)  │    :80 │ (start-first) │
                      ╰───────────────╯        ╰───────────────╯
```

With this design the service stays available, even during updates. You can play
with it as follows:

```bash
cd examples/zero_downtime_deployment
kerek deploy --wait
curl localhost:8080 # … prints "Hi from A"

export GREET_VERSION=B
kerek deploy --wait
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

| Command                            | Effect                                        |
| ---------------------------------- | --------------------------------------------- |
| `kerek deploy`                     | Update all services with changed config hash  |
| `kerek deploy --dry-run`           | Update nothing but show what would be changed |
| `kerek deploy x`                   | Update service `x` if its config hash changed |
| `kerek deploy --force-recreate`    | Always update all services                    |
| `kerek deploy --force-recreate x`  | Always update service `x`                     |
| `docker compose config --hash '*'` | Show service config hashes for Compose file   |

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

### Support for Podman and other container engines

Pass `--container-engine podman` or set the environment variable
`CONTAINER_ENGINE=podman` to use Podman instead of Docker. Kerek is
engine-agnostic so you may use any other container engine with a compatible CLI.

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

### `kerek --help`

```
Zero-downtime deployments for Docker Compose

Usage: kerek [OPTIONS] <COMMAND>

Commands:
  deploy           Create or update services
  provision        Provisions host with container engine, making system-wide
                       changes
  transfer-images  Copies images from default to specified Docker host
  tunnel-ssh       Forwards local Unix domain socket to remote Docker host
                       over SSH
  help             Print this message or the help of the given subcommand(s)

Options:
      --container-engine <CONTAINER_ENGINE>
          Container engine program to use

          [env: CONTAINER_ENGINE=]
          [default: docker]

      --dry-run
          Do not apply changes, only show what would be done

      --config <CONFIG>
          Location of client config files

  -c, --context <CONTEXT>
          Name of the context to use to connect to the daemon (overrides
          DOCKER_HOST env var and default context set with `docker context use`)

  -D, --debug
          Enable debug mode

  -H, --host <HOST>
          Daemon socket to connect to

  -l, --log-level <LOG_LEVEL>
          Set the logging level

          [possible values: debug, info, warn, error, fatal]

      --tls
          Use TLS; implied by --tlsverify

      --tlscacert <TLSCACERT>
          Trust certs signed only by this CA

      --tlscert <TLSCERT>
          Path to TLS certificate file

      --tlskey <TLSKEY>
          Path to TLS key file

      --tlsverify
          Use TLS and verify the remote

  -h, --help
          Print help

  -V, --version
          Print version
```

### `kerek deploy --help`

```
Create or update services

Builds, (re)creates, and starts containers for a service.

If service names are given as command-line operands, this command does not
automatically start any of their linked services.

The containers are always started in the background and left running (detached
mode).

If there are existing containers for a service, and the service's configuration
or image was changed after the container's creation, the changes are picked up
by recreating the containers (preserving mounted volumes). Whether the old
containers are stopped before or after the new containers are started is
controlled via `services.*.deploy.update_config.order` in a Compose file.

If you want to force recreating all containers, use the `--force-recreate` flag.

Usage: kerek deploy [OPTIONS] [SERVICE_NAMES]...

Arguments:
  [SERVICE_NAMES]...
          Services to consider

Options:
      --all-resources
          Include all resources, even those not used by services

      --ansi <ANSI>
          Control when to print ANSI control characters

          [possible values: never, always, auto]

      --compatibility
          Run compose in backward compatibility mode

      --env-file <ENV_FILE>
          Specify an alternate environment file

  -f, --file <FILE>
          Compose configuration files

      --parallel <PARALLEL>
          Control max parallelism, -1 for unlimited

      --profile <PROFILE>
          Specify a profile to enable

      --progress <PROGRESS>
          Set type of progress output

          [possible values: auto, tty, plain, json, quiet]

      --project-directory <PROJECT_DIRECTORY>
          Specify an alternate working directory (default: the path of the,
          first specified, Compose file)

  -p, --project-name <PROJECT_NAME>
          Project name

      --build
          Build images before starting containers

      --force-recreate
          Recreate containers even if their configuration and image haven't
          changed

      --no-build
          Don't build an image, even if it's policy

      --no-start
          Don't start the services after creating them

      --pull <PULL>
          Pull image before running

          [possible values: always, missing, never]

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
          Maximum duration to wait for the project to be running|healthy

  -h, --help
          Print help (see a summary with '-h')
```

### `kerek provision --help`

```
Provisions host with container engine, making system-wide changes

This targets a host via SSH, unless host `localhost` and no SSH config file are
passed as arguments, in which case the current machine is targeted.

Usage: kerek provision [OPTIONS] <HOST>

Arguments:
  <HOST>
          Reference like `localhost` or `[ssh://][<user>@]<hostname>[:<port>]`

Options:
      --force
          Go ahead without prompting user to confirm

  -F, --ssh-config <SSH_CONFIG>
          Path to SSH config file

  -h, --help
          Print help (see a summary with '-h')
```

### `kerek transfer-images --help`

```
Copies images from default to specified Docker host

By default, only images absent on the destination host are transferred. An image
is considered present if the name matches one of these forms:

- `<namespace>:<tag>`
- `<namespace>@<digest>`
- `<namespace>:<tag>@<digest>`

Examples:

    kerek --host ssh://192.0.2.1 transfer-images my-img
    DOCKER_HOST=ssh://from kerek --host ssh://to transfer-images my-img
    DOCKER_CONTEXT=from kerek --context to transfer-images my-img
    docker compose config --images | kerek --host … transfer-images -

    kerek --host … transfer-images --compress zstd my-img
    kerek --host … transfer-images --compress 'xz -9' my-img

Usage: kerek transfer-images [OPTIONS] [IMAGES]...

Arguments:
  [IMAGES]...
          Images to copy; use `-` to pass image names as stdin lines

Options:
      --compress <COMPRESS>
          Compression command to use (`bzip2`, `gzip`, `xz`, `zstd`, etc.)

      --force
          Copy images without checking if the destination already has such
          images; useful for replacing images with `latest` tag

  -h, --help
          Print help (see a summary with '-h')
```

### `kerek tunnel-ssh --help`

```
Forwards local Unix domain socket to remote Docker host over SSH

This runs an SSH tunnel in the background. Meanwhile, you can connect to the
remote Docker host using `DOCKER_HOST=unix:///path/to/kerek.sock` locally. Note
that a custom SSH config file can be specified, unlike with vanilla Docker.

Example:

    kerek tunnel-ssh my-ssh-host
    CONTAINER_HOST="unix://${PWD}/kerek.sock" podman ps
    fuser --kill -TERM kerek.sock

Usage: kerek tunnel-ssh [OPTIONS] <SSH_HOST>

Arguments:
  <SSH_HOST>
          Reference like `[ssh://][<user>@]<hostname>[:<port>]`

Options:
      --local-socket <LOCAL_SOCKET>
          Path to Unix domain socket on localhost to be forwarded

          [default: kerek.sock]

      --remote-socket <REMOTE_SOCKET>
          Path to Unix domain socket of Docker host on remote

  -F, --ssh-config <SSH_CONFIG>
          Path to SSH config file

  -h, --help
          Print help (see a summary with '-h')
```
