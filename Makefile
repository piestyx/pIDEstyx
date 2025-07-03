# Makefile for pIDEstyx
# Mainly placeholder

APP_NAME := pidestyx
RUST_CRATE := ide-core/crates/core-lib
EDITOR_DIR := ide-core/apps/editor
LLM_RUNNER_DIR := ide-core/apps/llm-runner
UI_DIR := ide-core/apps/ui
OUTPUT_DIR := dist

VERSION := $(shell git describe --tags --always)
DATE := $(shell date +%Y%m%d)

# Default target
.PHONY: all
all: build

## Build for the current platform
.PHONY: build
build:
	cargo build --release --manifest-path=$(RUST_CRATE)/Cargo.toml

## Run tests
.PHONY: test
test:
	cargo test --manifest-path=$(RUST_CRATE)/Cargo.toml

## Run the UI in development mode
.PHONY: dev
dev:
	cd $(UI_DIR) && npm install && npm run dev

## Clean build artifacts
.PHONY: clean
clean:
	cargo clean --manifest-path=$(RUST_CRATE)/Cargo.toml
	rm -rf $(OUTPUT_DIR)

## Build Windows binary (CPU only)
.PHONY: build-windows
build-windows:
	cross build --target x86_64-pc-windows-gnu --release --manifest-path=$(RUST_CRATE)/Cargo.toml

## Package Windows installer
.PHONY: package-windows
package-windows: build-windows
	mkdir -p $(OUTPUT_DIR)/windows
	cp target/x86_64-pc-windows-gnu/release/$(APP_NAME).exe $(OUTPUT_DIR)/windows/
	# Call your installer script here, e.g. Inno Setup or cargo-wix
	echo "TODO: Build .exe installer using Inno Setup"

## Patch langserver
.PHONY: patch-langserver
patch-langserver:
	cd langserver-patch && ./patch.sh

## Show help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build            - Build for current platform"
	@echo "  build-windows    - Build Windows binary (CPU only)"
	@echo "  package-windows  - Package Windows .exe installer"
	@echo "  dev              - Run UI in development mode"
	@echo "  test             - Run tests"
	@echo "  clean            - Clean build artifacts"
	@echo "  patch-langserver - Apply language server patch"
	@echo "  help             - Show this help message"
