.PHONY: all
all: fmt test lint build
lint:
	docker run --rm -i -v ./.hadolint.yaml:/.config/hadolint.yaml hadolint/hadolint < Dockerfile
test:
	cargo test
fmt:
	cargo fmt
build:
	docker buildx build --platform linux/amd64,linux/arm/v7,linux/arm64 -t linkpedant:dev .
