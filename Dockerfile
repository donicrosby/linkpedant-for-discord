FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
COPY locales locales
COPY src src
RUN cargo build --release

FROM debian:latest AS release
RUN apt update \
    && apt install -y \
    dumb-init \
    && rm -rf /var/lib/lists/*;

WORKDIR /app
COPY --from=builder /app/target/release/linkpedant .
COPY config.example.yaml config.yaml

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/linkpedant"]