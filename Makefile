SHELL := /bin/bash



TARGET=
USE_CROSS=
BINARY_NAME?=pact
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
	if [[ $(BUILDER) == "cross" ]]; then \
		command -v cross > /dev/null 2>&1 || cargo install cross@0.2.5; \
	fi
	if [[ $(TARGET) == *"musl"* ]]; then \
		$(BUILDER) build --release --target=$(TARGET) --bin $(BINARY_NAME); \
	else \
		$(BUILDER) build --release --target=$(TARGET); \
	fi
