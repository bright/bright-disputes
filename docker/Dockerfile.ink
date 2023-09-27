FROM ubuntu:jammy-20220531 AS build

RUN apt update && apt install build-essential -y && apt install pkg-config -y
RUN apt install clang curl libssl-dev protobuf-compiler -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN apt -y install binaryen

ENV PATH="/root/.cargo/bin:${PATH}"

# Setup toolchain
RUN rustup toolchain install 1.69
RUN rustup target add wasm32-unknown-unknown --toolchain 1.69
RUN rustup component add rust-src clippy rustfmt --toolchain 1.69
RUN rustup default 1.69

RUN rustup toolchain install nightly-2023-04-16 --target wasm32-unknown-unknown \
		--profile minimal --component rustfmt clippy miri rust-src rustc-dev llvm-tools-preview

# Install crates
RUN cargo +nightly-2023-04-16 install ink-wrapper --locked --force --version 0.5.0
RUN cargo install --force --locked cargo-contract --version 3.0.1

WORKDIR /code

CMD ["/bin/bash"]