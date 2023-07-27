
# Prerequisites 

Install cargo-contract:
```
cargo install --force --locked cargo-contract --version 3.0.1
```

Install ink-wrapper
```
cargo install ink-wrapper --locked --force --version 0.5.0
```

# Build
First we need to build a smart contract:
```
cargo contract build --release --manifest-path contract/Cargo.toml
```

next we need use ink-wrapper to generate a safe code for our tests:
```
ink-wrapper -m ../contract/target/ink/bright_disputes.json | rustfmt --edition 2021 > bright_disputes.rs
```
# Run
Please note that, before running E2E test we need to have a running node.

We start from deploying our smart contract to the node:
```
cargo contract upload --manifest-path contract/Cargo.toml --suri //Alice --url ws://localhost:9944 --execute  || true
```

Now we can run e2e tests:
```
cargo +nightly test --release
```