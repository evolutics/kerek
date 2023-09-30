# Wheelsticks: Zero-downtime deployments for Docker Compose

Wheelsticks is an addition to Docker Compose that helps updating services with
zero downtime.

## Motivation

Docker Compose offers simple, declarative orchestration of containerized apps.

When updating a service container with Docker Compose using `docker compose up`,
the old container is stopped _before_ a new container is started
(**`stop-first`** case):

```
old container              stop
------------------------------|
                                                  start            new container
                                                  |-----------------------------
```

This causes a service interruption as there is a time when neither container is
available.

Imagine that we could make the container lifetimes overlap instead
(**`start-first`** case):

```
old container                                  stop
--------------------------------------------------|
                              start                                new container
                              |-------------------------------------------------
```

If a reverse proxy seamlessly switches traffic over from old to new container,
then a zero-downtime deployment is achieved.

The
[Compose specification](https://github.com/compose-spec/compose-spec/blob/master/deploy.md)
in fact defines options to distinguish above two cases: `stop-first` (default)
and `start-first`. But support for this part of the specification is optional,
and plain Docker Compose always applies `stop-first` irrespective of what's in
your Compose files.

However, Wheelsticks supports both options. Just run `wheelsticks deploy` in
place of `docker compose up`. No need to change any other Docker or Docker
Compose workflows.

## Installation

### Prerequisites

- Docker or Podman
- Docker Compose

### Build

Pre-compiled executables are coming soon, until then run

```bash
cargo install --git https://github.com/evolutics/wheelsticks.git
```

The built executable is installed into the folder `~/.cargo/bin` by default.

### Docker CLI plugin

Coming soon for integration with the familiar Docker CLI.

## Usage

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
    localhost:8080    +-------------------------+        +-----------------+
----------------------| reverse-proxy container |--------| greet container |
                  :80 | (stop-first)            |    :80 | (start-first)   |
                      +-------------------------+        +-----------------+
```

With this design the service stays available, even during updates. You can play
with it as follows:

```bash
cd example
wheelsticks deploy
curl localhost:8080 # … prints "Hi from A"

export GREET_VERSION=B
wheelsticks deploy
curl localhost:8080 # … prints "Hi from B"

docker compose down
```

To see above deployments in action, use a separate shell session to run

```bash
while true; do curl --fail --max-time 0.2 localhost:8080; sleep 0.01s; done
```

## Command-line arguments reference

### `wheelsticks -h`

```
Zero-downtime deployments for Docker Compose

Usage: wheelsticks [OPTIONS] <COMMAND>

Commands:
  deploy  Create or update services
  help    Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>
          Location of client config files
  -c, --context <CONTEXT>
          Name of the context to use to connect to the daemon (overrides
          DOCKER_HOST env var and default context set with "docker context use")
  -D, --debug
          Enable debug mode [aliases: verbose]
  -H, --host <HOST>
          Daemon socket to connect to
  -l, --log-level <LOG_LEVEL>
          Set the logging level [possible values: debug, info, warn, error,
          fatal]
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
      --container-engine <CONTAINER_ENGINE>
          Container engine to use [default: docker] [possible values: docker,
          podman]
      --compose-engine <COMPOSE_ENGINE>
          Compose engine to use; Podman Compose is not supported due to missing
          features [default: "docker compose"] [possible values: docker-compose,
          "docker compose"]
  -h, --help
          Print help
  -V, --version
          Print version
```

### `wheelsticks deploy -h`

```
Create or update services

Usage: wheelsticks deploy [OPTIONS] [SERVICE_NAMES]...

Arguments:
  [SERVICE_NAMES]...

Options:
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
  -h, --help
          Print help (see more with '--help')
```
