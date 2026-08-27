.PHONY: all help run build check test lint format format-check pre-commit clean dev-install

CARGO ?= cargo

all: build

help:
	@echo "Available targets:"
	@echo "  make run ARGS='...'    Run the binary and pass arguments to cargo run"
	@echo "  make build             Build the project"
	@echo "  make check             Check the project without producing a binary"
	@echo "  make test              Run the test suite"
	@echo "  make lint              Run clippy with warnings denied"
	@echo "  make format            Format the source code"
	@echo "  make format-check      Check formatting without changing files"
	@echo "  make pre-commit        Run formatting, lint, and test checks"
	@echo "  make clean             Remove build artifacts"
	@echo "  make dev-install       Install the current owly binary"

run:
	$(CARGO) run -- $(ARGS)

build:
	$(CARGO) build

check:
	$(CARGO) check --all-targets --all-features

test:
	$(CARGO) test --all-targets --all-features

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

format:
	$(CARGO) fmt --all

format-check:
	$(CARGO) fmt --all -- --check

pre-commit: format-check lint test

clean:
	$(CARGO) clean

dev-install:
	$(CARGO) install --path . --bin owly --locked --force
