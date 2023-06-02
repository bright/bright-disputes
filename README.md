# Bright Disputes
This project 

## Prerequisites
1. `cargo-contract 2.x`

## Build
To build smart contract run:
```
cargo +nightly contract build --release --manifest-path contract/Cargo.toml
```

## Run
Running smart contract can be done by running `deploy.sh` script:
```
bash scripts/deploy.sh
```
Script will run an aleph-node, build and deploy smart contract.

This script is based on the [deploy.sh](https://github.com/Cardinal-Cryptography/zk-apps/blob/main/shielder/deploy/deploy.sh).

## Docker
Smart contract can be build with the Docker:
```
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile --output artifacts .
```
This will export *bright_disputes.json*, *bright_disputes.wasm*, *bright_disputes.contract*, to the *artifacts* directory.

## Tests
To run a unit test:
```
cargo +nightly test --release --manifest-path contract/Cargo.toml
```
or with docker:
```
docker build -f docker/Dockerfile.testing --progress=plain .
```