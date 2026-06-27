# libmwemu — build & test helpers (standalone repo).

CARGO_TARGET :=
ifeq ($(shell uname),Darwin)
	CARGO_TARGET := --target x86_64-apple-darwin
endif

# Password-protected bundle with the sample binaries (test/) and the
# proprietary Windows DLLs (maps/) that can't be redistributed in-repo.
# Publish it as a release asset on mwemuorg/libmwemu (tag: test-data).
TEST_ZIP_URL := https://github.com/mwemuorg/libmwemu/releases/download/test-data/test.zip
TEST_ZIP_PASSWORD := mwemuTestSystem

.PHONY: build release tests fmt clippy clean

build:
	cargo build $(CARGO_TARGET)

release:
	cargo build --release $(CARGO_TARGET)

# Fetch the test bundle once into ./test and ./maps, then run the suite.
tests:
	@if [ ! -d test ]; then \
		echo "fetching test bundle..."; \
		if which wget >/dev/null 2>&1; then \
			wget -q $(TEST_ZIP_URL); \
		else \
			curl -fsSL -O $(TEST_ZIP_URL); \
		fi; \
		unzip -o -P $(TEST_ZIP_PASSWORD) test.zip; \
		rm -f test.zip; \
	fi
	cargo test --verbose $(CARGO_TARGET)
	cargo test --release --verbose $(CARGO_TARGET)

fmt:
	cargo fmt --check

clippy:
	cargo clippy

clean:
	cargo clean
	rm -rf test test.zip
