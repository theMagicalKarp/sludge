CARGO := cargo
BUILD_DIR := target
RELEASE_DIR := $(BUILD_DIR)/release
DEBUG_DIR := $(BUILD_DIR)/debug

# Default target
all: test lint build release

# Build the project in release mode
release:
	$(CARGO) build --release

# Build the project in debug mode
build:
	$(CARGO) build

# Run tests
test:
	$(CARGO) test

# Format the code
fmt:
	$(CARGO) fmt

# Check for linting issues
lint:
	$(CARGO) clippy --all-targets --all-features

fix: fmt
	$(CARGO) clippy --fix --allow-dirty --all-targets --all-features
