# things-tui — terminal UI for things-api
#
# Targets:
#   make              build a debug binary into target/debug/things-tui
#   make release      build an optimized release binary
#   make run          build + launch (asks for first-run config if none saved)
#   make install      copy the release binary to PREFIX/bin (default /usr/local/bin)
#   make uninstall    remove the installed binary
#   make universal    build a fat Mac binary (Intel + Apple Silicon) -> dist/things-tui
#   make clean        wipe build artifacts

PREFIX ?= /usr/local
BIN_DIR := $(PREFIX)/bin
DIST_DIR := dist

.PHONY: all release run install uninstall universal clean

all:
	cargo build

release:
	cargo build --release

run:
	cargo run --release

install: release
	@mkdir -p $(BIN_DIR)
	install -m 0755 target/release/things-tui $(BIN_DIR)/things-tui
	@echo "installed → $(BIN_DIR)/things-tui"

uninstall:
	rm -f $(BIN_DIR)/things-tui

# Fat binary for distribution. Requires both Rust targets to be installed:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
universal:
	@mkdir -p $(DIST_DIR)
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	lipo -create \
		target/aarch64-apple-darwin/release/things-tui \
		target/x86_64-apple-darwin/release/things-tui \
		-output $(DIST_DIR)/things-tui
	@echo "universal binary → $(DIST_DIR)/things-tui"
	@file $(DIST_DIR)/things-tui

clean:
	cargo clean
	rm -rf $(DIST_DIR)
