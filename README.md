# Bright Disputes
This project is a dApp for raising and solving the disputes on the Substrate-based blockchains.

## Prerequisites
1. `cargo-contract 2.x`
2. `ink-wrapper 0.4.1`

## Build
To build smart contract run:
```
cargo +nightly-2022-11-28 contract build --release --manifest-path contract/Cargo.toml
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
cargo +nightly-2022-11-28 test --release --manifest-path contract/Cargo.toml
```
or with docker:
```
docker build -f docker/Dockerfile.testing --progress=plain .
```

## E2E tests
To run E2E tests on you local machine, first run a aleph-node. We can do it, by running `deploy.sh` from the running part. After that we need build contract on our local machine:
```
cargo +nightly-2022-11-28 contract build --release --manifest-path contract/Cargo.toml
```
and upload it to the node:
```
cargo contract upload --manifest-path contract/Cargo.toml --suri //Alice --url ws://localhost:9944 || true
```
after that we need to use [ink-wrapper](https://crates.io/crates/ink-wrapper) tool to generate a type-safe code for calling smart contract from our e2e tests:
```
cd tests
ink-wrapper -m ../contract/target/ink/bright_disputes.json --wasm-path ../contract/target/ink/bright_disputes.wasm | rustfmt --edition 2021 > bright_disputes.rs
```
Finally we can run a e2e tests by calling:
```
cargo +nightly test --release
```
