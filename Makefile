.PHONY: lint
lint:
	docker run --rm -i -v ./.hadolint.yaml:/.config/hadolint.yaml hadolint/hadolint < Dockerfile
all: lint