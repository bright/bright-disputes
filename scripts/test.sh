#!/usr/bin/env bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
CARGO_TOML="${SCRIPT_DIR}"/../contract/Cargo.toml

# Run unit tests
cd "${SCRIPT_DIR}"/../contract
cargo +nightly test --release

# Run e2e tests
cd "${SCRIPT_DIR}"/../tests
cargo +nightly-2022-11-28 contract build --release --manifest-path "${CARGO_TOML}"
cargo contract upload --manifest-path "${CARGO_TOML}" --suri //Alice --url ws://localhost:9944 || true
ink-wrapper -m "${SCRIPT_DIR}"/../contract/target/ink/bright_disputes.json --wasm-path "${SCRIPT_DIR}"/../contract/target/ink/bright_disputes.wasm | rustfmt --edition 2021 > "${SCRIPT_DIR}"/../tests/bright_disputes.rs
cargo +nightly-2022-11-28 test --release
