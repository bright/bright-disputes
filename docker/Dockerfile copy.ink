FROM ubuntu:jammy-20220531 AS build


ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    RUST_VERSION=nightly-2022-11-28

RUN /bin/sh -c set -eux \
    && apt-get update \
    && apt-get -y install wget \
    && dpkgArch="$(dpkg --print-architecture)" \
    && case "${dpkgArch##*-}" in amd64) rustArch='x86_64-unknown-linux-gnu' ;; arm64) rustArch='aarch64-unknown-linux-gnu' ;; *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; esac \
    && url="https://static.rust-lang.org/rustup/dist/${rustArch}/rustup-init" \
    && wget "$url" \
    && chmod +x rustup-init \
    && ./rustup-init -y --no-modify-path --profile minimal --component rust-src rustfmt --default-toolchain $RUST_VERSION \
    && rm rustup-init && chmod -R a+w $RUSTUP_HOME $CARGO_HOME \
    && rustup --version \
    && cargo --version \
    && rustc --version \
    && apt-get remove -y --auto-remove wget \
    && apt-get -y install gcc \
    && rm -rf /var/lib/apt/lists/* 

COPY /usr/local/bin/cargo-contract /usr/local/bin/cargo-contract 

COPY /usr/local/cargo/bin/ink-wrapper /usr/local/bin/ink-wrapper 

WORKDIR /code

CMD ["cargo" "contract" "build" "--release"]