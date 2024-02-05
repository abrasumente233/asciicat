FROM ubuntu:22.04 AS builder

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends curl ca-certificates gcc libc6-dev pkg-config libssl-dev

RUN set -eux; \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable --no-modify-path -y

ENV PATH=/root/.cargo/bin:${PATH}
RUN set -eux; \
    rustup --version

WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.toml ./
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/.cargo/git \
    --mount=type=cache,target=/app/target \
    #--mount=type=cache,target=/root/.rustup \
    set -eux; \
    cargo build --release; \
    cp ./target/release/asciicat .

EXPOSE 8080
CMD ["/app/asciicat"]
