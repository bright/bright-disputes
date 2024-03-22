# Bright Disputes
This project is a dApp for raising and solving the disputes on the Substrate-based blockchains. Process of building and running smart contract can be found in the sections bellow. Showcase scenario can be found in the [documentation](https://github.com/bright/bright-disputes/blob/main/doc/README.md).

## Prerequisites
1. `cargo-contract 3.0.1`
2. `ink-wrapper 0.5.0`
3. `rustc-1.69`
4. `jq`

Before running anything, please update submodules.
```
git submodule update --init --recursive
```
## Build (manually)
Bright Disputes is actually a smart contract and a CLI, which allows to execute commands on the Substrate node. In this section we will present how to manually build them.

### Building smart contract
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

### Building CLI
Follow the instructions from the [README](https://github.com/bright/bright-disputes/blob/main/cli/README.md). file.

## Build (docker)
Smart contract can be build with the Docker:
```
DOCKER_BUILDKIT=1 docker build -f docker/Dockerfile --output artifacts . 
```
This will export *bright_disputes.json*, *bright_disputes.wasm*, *bright_disputes.contract*, to the *artifacts* directory.

## Build (script)
The last way to build and run Bright Disputes is to use a script. We simplify the whole process, by providing a `scripts/deploy.sh` script. It will starts aleph node, building and deploying smart contract on it. Script will also pre-fund [accounts](https://github.com/bright/bright-disputes/blob/main/doc/accounts), which can be used to play/test with the smart contract. More details can be found in the [showcase](https://github.com/bright/bright-disputes/blob/main/doc/README.md). Script is using three docker images:
* disputes-node - is an image of the aleph node, where our smart contract is going to be deployed
* disputes-cliain - is a image of `cliain` tool, which is a wrapper over `substrate-api-client` library. It simplify calls over Substrate chain extrinsic.
* disputes-ink-dev - this image contains environment for building a smart contract

To run a script just type:
```
bash scripts/deploy.sh
```

Please note, that this script is based on the [deploy.sh](https://github.com/Cardinal-Cryptography/zk-apps/blob/main/shielder/deploy/deploy.sh) created by the Cardinal-Cryptography.

## Running UI
To run a UI, please follow the instructions from the [README](./ui/README.md) file in ui folder.

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
Currently, we have four major tests for testing different endings of the dispute:
* No majority of votes.
* Verdict against the owner of the dispute.
* Verdict against the defendant of the dispute.
* Testing dispute rounds.

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
cargo +nightly-2023-04-19 test --release
```

The output of the e2e test is:
```
test bright_disputes_test::test_dispute_verdict_none ... ok
test bright_disputes_test::test_dispute_verdict_positive ... ok
test bright_disputes_test::test_dispute_verdict_negative ... ok
test bright_disputes_test::test_dispute_rounds ... ok
```