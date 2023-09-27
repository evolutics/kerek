# Wheelsticks: Zero-downtime deployments for Docker Compose

Docker Compose offers simple, declarative orchestration of containerized apps.

When updating a service container with Docker Compose, the old container is
stopped _before_ a new container is started. This causes a service interruption
(unless the service is replicated).

To achieve a zero-downtime deployment instead, you can use Wheelsticks.

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

### Quick start

1. For services whose container lifetimes should overlap during an update,
   configure their update order like so:

   ```yaml
   services:
     web:
       image: …
       deploy:
         update_config:
           order: start-first # Most important line.
   ```

   While this follows the
   [Compose specification](https://github.com/compose-spec/compose-spec/blob/master/deploy.md#update_config),
   support for `start-first` is optional: plain Docker Compose ignores it,
   always using `stop-first` instead.

1. In order to seamlessly switch traffic from an old to a new container during a
   deployment, you need to use something like a reverse proxy.

1. Now each time you update your services, simply run

   ```bash
   wheelsticks deploy
   ```

### Demo

See the [`example`](example) folder for a demo. You can play with it as follows:

```
cd example
docker compose up --detach
curl localhost:8080

# Deploy an update:
HI_VERSION=B wheelsticks deploy

# Deploy another update:
HI_VERSION=C wheelsticks deploy
```

Note how the service at localhost:8080 is always available, even during
deployments.

You can clean up with `docker compose down`.

## Command-line arguments reference

### `wheelsticks --help`

```
Zero-downtime deployments for Docker Compose

Usage: wheelsticks [OPTIONS] <COMMAND>

Commands:
  deploy  Update containers
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

### `wheelsticks deploy --help`

```
Update containers

Usage: wheelsticks deploy [OPTIONS] [SERVICE_NAMES]...

Arguments:
  [SERVICE_NAMES]...

Options:
      --build
          Build images before starting containers
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
          Print help
```
