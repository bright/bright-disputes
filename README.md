# Bright Disputes
This project is a dApp for raising and solving the disputes on the Substrate-based blockchains. Process of building and running smart contract can be found in the sections bellow. Showcase scenario can be found in the [documentation](https://github.com/bright/bright-disputes/blob/main/doc/README.md).

## Prerequisites
1. `cargo-contract 3.0.1`
2. `ink-wrapper 0.5.0`
2. `rustc-1.69`

## Build (manually)
To build a smart contract locally we can run:
```
cargo contract build --release --manifest-path contract/Cargo.toml
```
This call will generate files:
```
contract/target/ink/bright_disputes.json
contract/target/ink/bright_disputes.wasm
contract/target/ink/bright_disputes.contract
```
and they can be deployed on the node.

## Build (docker)
Smart contract can be build with the Docker:
```
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile --output artifacts .
```
This will export *bright_disputes.json*, *bright_disputes.wasm*, *bright_disputes.contract*, to the *artifacts* directory.

## Build (script)
The last way to build and run Bright Disputes is to use a script. We simplify the whole process, by providing a `scripts/deploy.sh` script. It will starts aleph node, building and deploying smart contract on it. Script will also pre-fund [accounts](https://github.com/bright/bright-disputes/blob/main/doc/accounts), which can be used to play/test with the smart contract. More details can be found in the [showcase](https://github.com/bright/bright-disputes/blob/main/doc/README.md). Script is using three docker images:
* disputes-node - is an image of the aleph node, where our smart contract is going to be deployed
* disputes-cliain - is an image of `cliain` tool, which is a wrapper over `substrate-api-client` library. It simplify calls over Substrate chain extrinsic.
* disputes-ink-dev - this image contains environment for building a smart contract

To run a script just type:
```
bash scripts/deploy.sh
```

Please note, that this script is based on the [deploy.sh](https://github.com/Cardinal-Cryptography/zk-apps/blob/main/shielder/deploy/deploy.sh) created by the Cardinal-Cryptography.

## Tests
To run a unit test:
```
cargo test --release --manifest-path contract/Cargo.toml
```
or with docker:
```
docker build -f docker/Dockerfile.testing --progress=plain .
```

## E2E tests
To run E2E tests on your local machine, first run a aleph-node, build and deploy smart contract. We can do it, by running `deploy.sh` script:
```
bash scripts/deploy.sh
```

Next we need to use [ink-wrapper](https://crates.io/crates/ink-wrapper) tool to generate a type-safe code for calling smart contract from our e2e tests:
```
cd tests
ink-wrapper -m ../contract/target/ink/bright_disputes.json --wasm-path ../contract/target/ink/bright_disputes.wasm | rustfmt +nightly --edition 2021 > bright_disputes.rs
```

Finally we can run a e2e tests by calling:
```
cargo +nightly test --release
```
