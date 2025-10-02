# ZKP2P TLSNotary FFI Testing Makefile

# Variables
CC = g++
CFLAGS = -Wall -Wextra -std=c++11
RUST_LIB_DIR = target/release
RUST_LIB_NAME = tlsnprover
INCLUDE_DIR = generated/include
TEST_SOURCE = tests/test_ffi.cpp
TEST_BINARY = tests/test_ffi

# Detect OS for library path
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_PATH_VAR = DYLD_LIBRARY_PATH
    LIB_EXT = .dylib
    LDFLAGS = -lpthread -ldl
else
    LIB_PATH_VAR = LD_LIBRARY_PATH
    LIB_EXT = .so
    LDFLAGS = -lpthread -ldl -lm
endif

# Default target
.PHONY: all
all: test

# Help target
.PHONY: help
help:
	@echo "ZKP2P TLSNotary FFI Testing"
	@echo ""
	@echo "Targets:"
	@echo "  test                 - Build Rust library, compile C++ test, and run"
	@echo "  build-rust           - Build the Rust FFI library(generates C++ bindings via build.rs)"
	@echo "  build-cross-platform - Run cross-platform build script"
	@echo "  build-test           - Compile the C++ test binary"
	@echo "  run-test             - Run the compiled test binary"
	@echo "  test-verbose         - Run test with verbose debug output"
	@echo "  debug                - Build and run in debug mode"
	@echo "  check-deps           - Check if required dependencies are installed"
	@echo "  check-lib            - Check if the Rust FFI library exists"
	@echo "  notary-help          - Show instructions for starting local notary"
	@echo "  clean                - Clean build artifacts"
	@echo "  clean-all            - Clean all artifacts including Rust build"
	@echo "  help                 - Show this help message"
	@echo ""
	@echo "Configuration:"
	@echo "- Uses config/default.toml for defaults"
	@echo "- Override with ZKP2P_* environment variables or .env file"
	@echo "- Test credentials in .env as ZKP2P_TEST_* variables"

# Build Rust FFI library (generates C bindings via build.rs)
.PHONY: build-rust
build-rust:
	@echo "🔨 Building Rust FFI library..."
	cargo build --release
	@echo "✅ Rust library built successfully"
	@echo "✅ C++ bindings generated automatically via build.rs"

# Cross-platform compilation
.PHONY: build-cross-platform
build-cross-platform:
	@echo "🌍 Running cross-platform build..."
	@chmod +x build-cross-platform.sh
	./build-cross-platform.sh
	@echo "✅ Cross-platform build completed"

# Compile C++ test
.PHONY: build-test
build-test: build-rust
	@echo "🔧 Compiling C++ test..."
	$(CC) $(CFLAGS) -o $(TEST_BINARY) $(TEST_SOURCE) \
		-L$(RUST_LIB_DIR) -l$(RUST_LIB_NAME) -I$(INCLUDE_DIR) $(LDFLAGS)
	@echo "✅ Test binary compiled"

# Run test
.PHONY: run-test
run-test: build-test
	@echo "🧪 Running FFI test..."
	@echo "Command: export $(LIB_PATH_VAR)=./target/release:$$$(LIB_PATH_VAR) && \
	if [ -f .env ]; then set -a && source .env && set +a; fi && \
	./$(TEST_BINARY)"
	@export $(LIB_PATH_VAR)=./target/release:$$$(LIB_PATH_VAR) && \
	if [ -f .env ]; then set -a && source .env && set +a; fi && \
	./$(TEST_BINARY)

# Main test target 
.PHONY: test
test: run-test


# Show notary server instructions
.PHONY: notary-help
notary-help:
	@echo "🏛️  Local Notary Server Setup"
	@echo ""
	@echo "To start local notary server:"
	@echo "1. Clone TLSNotary repo:"
	@echo "   git clone https://github.com/tlsnotary/tlsn.git"
	@echo "2. Navigate to repo:"
	@echo "   cd tlsn"
	@echo "3. Start server:"
	@echo "   cargo run --release --bin notary-server"
	@echo ""
	@echo "Then run: make test"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "🧹 Cleaning test artifacts..."
	rm -f $(TEST_BINARY)
	@echo "✅ Test cleanup complete"

# Clean everything including Rust artifacts
.PHONY: clean-all
clean-all: clean
	@echo "🧹 Cleaning all artifacts..."
	cargo clean
	@echo "✅ Full cleanup complete"

# Check dependencies
.PHONY: check-deps
check-deps:
	@echo "🔍 Checking dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found. Install Rust: https://rustup.rs/"; exit 1; }
	@command -v bindgen >/dev/null 2>&1 || { echo "❌ bindgen-cli not found. Install with: cargo install bindgen-cli"; exit 1; }
	@echo "✅ All dependencies found"

# Check if library exists
.PHONY: check-lib
check-lib:
	@if [ -f "$(RUST_LIB_DIR)/lib$(RUST_LIB_NAME)$(LIB_EXT)" ]; then \
		echo "✅ Rust FFI library found at $(RUST_LIB_DIR)/lib$(RUST_LIB_NAME)$(LIB_EXT)"; \
	else \
		echo "❌ Rust FFI library not found. Run 'make build-rust' first"; \
		exit 1; \
	fi

# Verbose test run with debug info
.PHONY: test-verbose
test-verbose: build-test
	@echo "🐛 Running FFI test with verbose output..."
	@echo "Library path: $(RUST_LIB_DIR)"
	@echo "Test binary: $(TEST_BINARY)"
	@echo "OS detected: $(UNAME_S)"
	@echo "Library path variable: $(LIB_PATH_VAR)"
	@echo ""
	@echo "Command: export $(LIB_PATH_VAR)=$(RUST_LIB_DIR):$$$(LIB_PATH_VAR) && ./$(TEST_BINARY)"
	@export $(LIB_PATH_VAR)=$(RUST_LIB_DIR):$$$(LIB_PATH_VAR) && ./$(TEST_BINARY)

# Debug build and test
.PHONY: debug
debug:
	@echo "🔨 Building Rust library in debug mode..."
	cargo build
	@echo "🔧 Compiling C++ test with debug symbols..."
	$(CC) $(CFLAGS) -g -o $(TEST_BINARY) $(TEST_SOURCE) \
		-Ltarget/debug -l$(RUST_LIB_NAME) -I$(INCLUDE_DIR) $(LDFLAGS)
	@echo "🧪 Running debug test..."
	@export $(LIB_PATH_VAR)=target/debug:$$$(LIB_PATH_VAR) && \
	if [ -f .env ]; then set -a && source .env && set +a; fi && \
	./$(TEST_BINARY)
	rm -rf $(TEST_BINARY).dSYM