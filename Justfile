[private]
default:
  @just --list --unsorted

install-crd: generate
  kubectl apply -f yaml/crd.yaml

generate:
  cargo run --bin crdgen > yaml/crd.yaml

run:
  cargo run

compile:
  #!/usr/bin/env bash
  docker run --rm \
    -v cargo-cache:/root/.cargo \
    -v $PWD:/volume \
    -w /volume \
    -t clux/muslrust \
    cargo build --release --bin controller
  cp target/x86_64-unknown-linux-musl/release/controller .

docker-build IMAGE = "ghcr.io/mmoreiradj/back2back:latest": compile
  docker build -t {{IMAGE}} .

k3d-start:
  k3d registry create registry.localhost --port 5000
  k3d cluster create local \
    -p "80:80@loadbalancer" \
    -p "443:443@loadbalancer" \
    --registry-use registry.localhost

docker-build-backup IMAGE = "ghcr.io/mmoreiradj/back2back-postgres-backup:latest":
  docker build -t {{IMAGE}} -f pgbackup.Dockerfile .
