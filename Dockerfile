FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml .
COPY Cargo.lock .

# Create dummy main and lib to allow for caching of dependencies
RUN echo "fn main() {}" >> dummy.rs; touch dummy-lib.rs; sed -i 's#src/main.rs#dummy.rs#' Cargo.toml; sed -i 's#src/lib.rs#dummy-lib.rs#' Cargo.toml;
RUN cargo build --release

# Swap back from dummy files 
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml; sed -i 's#dummy-lib.rs#src/lib.rs#' Cargo.toml;

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