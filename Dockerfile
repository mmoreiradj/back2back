FROM rust:alpine AS builder

WORKDIR /app

ENV CARGO_HOME=/root/.cargo

RUN rustup target add x86_64-unknown-linux-musl

RUN apk add musl-dev --no-cache

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --target=x86_64-unknown-linux-musl && \
    cp target/x86_64-unknown-linux-musl/release/controller /app/controller

FROM cgr.dev/chainguard/static

WORKDIR /app

COPY --chown=nonroot:nonroot --from=builder /app/controller /app/controller

EXPOSE 8080

ENTRYPOINT [ "/app/controller" ]
