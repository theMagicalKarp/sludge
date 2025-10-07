CARGO := cargo
BUILD_DIR := target
RELEASE_DIR := $(BUILD_DIR)/release
DEBUG_DIR := $(BUILD_DIR)/debug

# Default sludge source file (override with FILE=path)
DEFAULT_FILE := examples/main.sludge
FILE ?= $(DEFAULT_FILE)

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

# -------------------------------------------------------
# Sludge CLI Commands (defaults to $(DEFAULT_FILE))
# -------------------------------------------------------

# Run a sludge program
run:
	@echo "🛢️ Running sludge program: $(FILE)"
	$(CARGO) run -- run $(FILE)

# Print AST as JSON
ast:
	@echo "🧠 Printing AST for: $(FILE)"
	$(CARGO) run -- ast $(FILE)

# Launch interactive REPL
repl:
	@echo "💬 Launching sludge REPL..."
	$(CARGO) run -- repl

# Clean up build artifacts
clean:
	$(CARGO) clean

.PHONY: all build release test fmt lint fix run repl ast clean
