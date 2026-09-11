RUSTUP ?= rustup
ifeq ($(PROCESSOR_ARCHITECTURE),x86)
WINDOWS_MINGW_TOOLCHAIN ?= stable-i686-pc-windows-gnu
WINDOWS_MSVC_TOOLCHAIN ?= stable-i686-pc-windows-msvc
else
WINDOWS_MINGW_TOOLCHAIN ?= stable-x86_64-pc-windows-gnu
WINDOWS_MSVC_TOOLCHAIN ?= stable-x86_64-pc-windows-msvc
endif
ifeq ($(OS),Windows_NT)
TOOLCHAIN ?= $(WINDOWS_MINGW_TOOLCHAIN)
else
TOOLCHAIN ?= stable
endif
TARGET ?=

TARGET_FLAG := $(if $(TARGET),--target $(TARGET),)
RUSTUP_CARGO := $(RUSTUP) run $(TOOLCHAIN) cargo

.PHONY: all setup build release debug check test fmt lint
.PHONY: setup-windows setup-windows-mingw setup-windows-msvc
.PHONY: setup-linux setup-macos
.PHONY: linux-x86_64 linux-aarch64 windows-x86 windows-x86_64
.PHONY: windows-x86-mingw windows-x86_64-mingw
.PHONY: windows-x86-msvc windows-x86_64-msvc
.PHONY: macos-x86_64 macos-aarch64 macos-universal

all: release

setup:
	$(RUSTUP) toolchain install $(TOOLCHAIN) --profile minimal
	$(RUSTUP) component add --toolchain $(TOOLCHAIN) rustfmt clippy

setup-windows: setup-windows-mingw

setup-windows-mingw:
	$(RUSTUP) toolchain install $(WINDOWS_MINGW_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MINGW_TOOLCHAIN) i686-pc-windows-gnu x86_64-pc-windows-gnu

setup-windows-msvc:
	$(RUSTUP) toolchain install $(WINDOWS_MSVC_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MSVC_TOOLCHAIN) i686-pc-windows-msvc x86_64-pc-windows-msvc

setup-linux: setup
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) x86_64-unknown-linux-musl aarch64-unknown-linux-musl

setup-macos: setup
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) x86_64-apple-darwin aarch64-apple-darwin

build: release

release:
	$(RUSTUP_CARGO) build --locked --release --bin mary $(TARGET_FLAG)

debug:
	$(RUSTUP_CARGO) build --locked --bin mary $(TARGET_FLAG)

check:
	$(RUSTUP_CARGO) check --locked --all-targets

test:
	$(RUSTUP_CARGO) test --locked --all-targets

fmt:
	$(RUSTUP_CARGO) fmt --all -- --check

lint:
	$(RUSTUP_CARGO) clippy --locked --all-targets -- -D warnings

linux-x86_64:
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) x86_64-unknown-linux-musl
	$(RUSTUP_CARGO) build --locked --release --bin mary --target x86_64-unknown-linux-musl

linux-aarch64:
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) aarch64-unknown-linux-musl
	$(RUSTUP_CARGO) build --locked --release --bin mary --target aarch64-unknown-linux-musl

windows-x86: windows-x86-mingw

windows-x86_64: windows-x86_64-mingw

windows-x86-mingw:
	$(RUSTUP) toolchain install $(WINDOWS_MINGW_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MINGW_TOOLCHAIN) i686-pc-windows-gnu
	$(RUSTUP) run $(WINDOWS_MINGW_TOOLCHAIN) cargo build --locked --release --bin mary --target i686-pc-windows-gnu

windows-x86_64-mingw:
	$(RUSTUP) toolchain install $(WINDOWS_MINGW_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MINGW_TOOLCHAIN) x86_64-pc-windows-gnu
	$(RUSTUP) run $(WINDOWS_MINGW_TOOLCHAIN) cargo build --locked --release --bin mary --target x86_64-pc-windows-gnu

windows-x86-msvc:
	$(RUSTUP) toolchain install $(WINDOWS_MSVC_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MSVC_TOOLCHAIN) i686-pc-windows-msvc
	$(RUSTUP) run $(WINDOWS_MSVC_TOOLCHAIN) cargo build --locked --release --bin mary --target i686-pc-windows-msvc

windows-x86_64-msvc:
	$(RUSTUP) toolchain install $(WINDOWS_MSVC_TOOLCHAIN) --profile minimal
	$(RUSTUP) target add --toolchain $(WINDOWS_MSVC_TOOLCHAIN) x86_64-pc-windows-msvc
	$(RUSTUP) run $(WINDOWS_MSVC_TOOLCHAIN) cargo build --locked --release --bin mary --target x86_64-pc-windows-msvc

macos-x86_64:
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) x86_64-apple-darwin
	$(RUSTUP_CARGO) build --locked --release --bin mary --target x86_64-apple-darwin

macos-aarch64:
	$(RUSTUP) target add --toolchain $(TOOLCHAIN) aarch64-apple-darwin
	$(RUSTUP_CARGO) build --locked --release --bin mary --target aarch64-apple-darwin

macos-universal: macos-x86_64 macos-aarch64
	mkdir -p target/universal-apple-darwin/release
	lipo -create \
		target/x86_64-apple-darwin/release/mary \
		target/aarch64-apple-darwin/release/mary \
		-output target/universal-apple-darwin/release/mary
