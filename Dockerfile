FROM --platform=$BUILDPLATFORM rust:1.83.0 AS chef
RUN cargo install cargo-chef
RUN apt-get update && apt-get -y install --no-install-recommends \
    binutils \
    binutils-arm-linux-gnueabihf gcc-arm-linux-gnueabihf libc6-dev-armhf-cross gfortran-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
    binutils-aarch64-linux-gnu gcc-aarch64-linux-gnu libc6-dev-arm64-cross g++-aarch64-linux-gnu gfortran-aarch64-linux-gnu \
    && rm -rf /var/lib/lists/*;
WORKDIR /app
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
    "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
    "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
    *) exit 1 ;; \
esac
RUN rustup target add "$(cat /rust_target.txt)"

FROM --platform=$BUILDPLATFORM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM chef AS builder
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target "$(cat /rust_target.txt)"
COPY . .
RUN cargo build --release --target "$(cat /rust_target.txt)"
RUN cp "target/$(cat /rust_target.txt)/release/linkpedant" .

FROM debian:12.8 AS release
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    dumb-init \
    && rm -rf /var/lib/lists/*;
WORKDIR /app
COPY --from=builder /app/linkpedant .
COPY config.example.yaml config.yaml

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/linkpedant"]