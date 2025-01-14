FROM --platform=$BUILDPLATFORM rust:1.84.0 AS chef
# cargo-chef for improved docker caching
RUN cargo install cargo-chef
# Install all of the needed compilation tools for the ARM builds
RUN apt-get update && apt-get -y install --no-install-recommends \
    binutils \
    binutils-arm-linux-gnueabihf gcc-arm-linux-gnueabihf libc6-dev-armhf-cross gfortran-arm-linux-gnueabihf g++-arm-linux-gnueabihf \
    binutils-aarch64-linux-gnu gcc-aarch64-linux-gnu libc6-dev-arm64-cross g++-aarch64-linux-gnu gfortran-aarch64-linux-gnu \
    && rm -rf /var/lib/lists/*;
# Add the targets we might build if they don't exist already
RUN rustup target add armv7-unknown-linux-gnueabihf \
    && rustup target add aarch64-unknown-linux-gnu
# Set the needed cargo ENVs to be able to properly build and link the cross-compiled binaries
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
WORKDIR /app

# Prepare the "recipe" for our app
FROM --platform=$BUILDPLATFORM chef AS planner
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Do the actual build
FROM --platform=$BUILDPLATFORM chef AS builder
# Get the recipe
COPY --from=planner /app/recipe.json recipe.json
# Use amd64 cross-compile for all versions of the container
# It's faster than whatever the hell QEMU via buildx is doing...
ARG TARGETPLATFORM
# Which build are we doing?
RUN case "$TARGETPLATFORM" in \
    "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
    "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
    "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
    *) exit 1 ;; \
esac
# Cache our pre-compiled dependencies
RUN cargo chef cook --release --target "$(cat /rust_target.txt)"
COPY . .
# Actually build the app
RUN cargo build --release --target "$(cat /rust_target.txt)"
# Copy out the final bin somewhere we can reach it
RUN cp "target/$(cat /rust_target.txt)/release/linkpedant" .

FROM debian:12.8 AS release
# Need curl for health check
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    dumb-init \
    curl  \ 
    && rm -rf /var/lib/lists/*;
WORKDIR /app
# Copy default config into container
COPY config.example.yaml config.yaml
# Copy the binary to final container
COPY --from=builder /app/linkpedant .

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/app/linkpedant"]