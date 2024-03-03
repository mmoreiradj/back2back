[private]
default:
  @just --list --unsorted

install-crd: generate
  kubectl apply -f yaml/crd.yaml

generate:
  cargo run --bin crdgen > yaml/crd.yaml

run:
  cargo run
