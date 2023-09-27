FROM ubuntu:jammy-20220531 AS build

RUN apt update && \
    apt install wget -y && \
    apt install build-essential -y && \
    apt install curl -y && \
    apt install ca-certificates -y && \
    apt clean && \
    rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

RUN wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /bright_dispute/

# RUN rustup update stable
RUN rustup install 1.69
RUN rustup default 1.69
RUN rustup install nightly-2023-04-19
RUN rustup component add rust-src --toolchain 1.69
RUN rustup target add wasm32-unknown-unknown --toolchain 1.69
RUN cargo +nightly-2023-04-19 install ink-wrapper --locked --force --version 0.5.0
RUN cargo install --force --locked cargo-contract --version 3.0.1

WORKDIR /bright_dispute/contract
RUN cargo contract build --release 

WORKDIR /bright_dispute/cli
RUN ink-wrapper -m ../contract/target/ink/bright_disputes.json | rustfmt --edition 2021 > src/bright_disputes_ink.rs
RUN cargo +nightly-2023-04-19 build --release 

FROM scratch AS export-stage
COPY --from=build /bright_dispute/contract/target/ink/bright_disputes.json .
COPY --from=build /bright_dispute/contract/target/ink/bright_disputes.wasm .
COPY --from=build /bright_dispute/contract/target/ink/bright_disputes.contract .
COPY --from=build /bright_dispute/cli/target/release/bright_disputes_cli .
