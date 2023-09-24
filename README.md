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
