FROM ubuntu:22.04 AS base

#--------------------------------------------------
FROM base AS builder

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
    objcopy --compress-debug-sections ./target/release/asciicat ./asciicat

#--------------------------------------------------
FROM base AS app

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# Install run-time dependencies, remove extra APT files afterwards.
# This must be done in the same `RUN` command, otherwise it doesn't help
# to reduce the image size.
RUN set -eux; \
		apt update; \
		apt install -y --no-install-recommends \
			ca-certificates \
			; \
		apt clean autoclean; \
		apt autoremove --yes; \
		rm -rf /var/lib/{apt,dpkg,cache,log}/

# Copy app from builder
WORKDIR /app
COPY --from=builder /app/asciicat .

EXPOSE 8080
CMD ["/app/asciicat"]