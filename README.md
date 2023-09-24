# Wheelsticks: Zero-downtime deployments for Docker Compose

Container orchestrator for lightweight environments.

## Motivation

This is a work in progress. The plan is to target single-node environments with
support for these:

- Simple, declarative orchestration using [Compose](https://compose-spec.io)
  files.
- Zero-downtime deployments.
- Efficient resource usage.
- Deploying locally, but also remotely over SSH.
- Image distribution via SSH, alternatively via image registry.
- Building and deploying with Podman or Docker.

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
