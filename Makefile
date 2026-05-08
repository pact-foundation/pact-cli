SHELL := /bin/bash



TARGET=
USE_CROSS=
BINARY_NAME?=pact
SLIM=false
BUILDER=cargo

ifeq ($(TARGET),)
	TARGET := $(shell rustup show | grep 'Default host' | awk '{print $$3}')
endif

ifeq ($(USE_CROSS),true)
	BUILDER := cross
endif


# Shows a list of available targets for cross-compilation
target_list = $(shell rustup target list)
rustup_target_list:
	@echo "$(target_list)" | sed 's/([^)]*)//g' | tr ' ' '\n' | sed '/^\s*$$/d'

is_slim:
	echo $(SLIM)

use_cross:
	echo $(BUILDER)

cargo_test:
	$(BUILDER) test --target=$(TARGET) --verbose -- --nocapture
# Build the release version of the library
# Can be used to build for a specific target by setting the TARGET environment variable
# e.g. `make cargo_build_release TARGET=x86_64-unknown-linux-gnu`
# by default will use the host target
cargo_build_release:
	echo "Building for target: $(TARGET)"
	if [[ $(SLIM) == "true" ]]; then \
		if [[ "$(shell uname -s)" == "Linux" ]]; then \
			sudo apt install libstd-rust-dev; \
			rustup toolchain install nightly; \
			rustup component add rust-src --toolchain nightly; \
		else \
			rustup component add rust-src --toolchain nightly --target=$(TARGET); \
		fi; \
		if [[ $(BUILDER) == "cross" ]]; then \
			cargo +nightly install cross@0.2.5; \
		fi; \
	fi
	if [[ $(TARGET) == "x86_64-pc-windows-gnu" ]]; then \
		echo "installing latest cross"; \
		if [[ $(SLIM) == "true" ]]; then \
			cargo +nightly install cross --git https://github.com/cross-rs/cross; \
		else \
			cargo install cross --git https://github.com/cross-rs/cross; \
		fi; \
	else \
		if [[ $(BUILDER) == "cross" ]]; then \
			cargo install cross@0.2.5; \
		fi; \
	fi
	if [[ $(SLIM) == "true" ]]; then \
		echo "building slimmest binaries"; \
		if [[ $(TARGET) == "aarch64-unknown-linux-musl" ]]; then \
			RUSTFLAGS="-Zlocation-detail=none -C link-arg=-lgcc" $(BUILDER) +nightly build -Z build-std=std,panic_abort,core,alloc,proc_macro -Z build-std-features=panic_immediate_abort --target=$(TARGET) --bin $(BINARY_NAME) --release; \
		elif [[ $(TARGET) == *"musl"* ]]; then \
			RUSTFLAGS="-Zlocation-detail=none" $(BUILDER) +nightly build -Z build-std=std,panic_abort,core,alloc,proc_macro -Z build-std-features=panic_immediate_abort --target=$(TARGET) --bin $(BINARY_NAME) --release; \
		else \
			RUSTFLAGS="-Zlocation-detail=none" $(BUILDER) +nightly build -Z build-std=std,panic_abort,core,alloc,proc_macro -Z build-std-features=panic_immediate_abort --target=$(TARGET) --release; \
		fi; \
	fi
	if [[ $(TARGET) == *"musl"* ]]; then \
		$(BUILDER) build --release --target=$(TARGET) --bin $(BINARY_NAME); \
	else \
		$(BUILDER) build --release --target=$(TARGET); \
	fi