.DEFAULT_GOAL := check
override SHELL := /bin/bash
override .SHELLFLAGS := -p -c
unexport BASH_ENV ENV

.PHONY: check check-compiler check-policy test-policy
.NOTPARALLEL: check

check: check-policy test-policy check-compiler

check-compiler:
	@set -euo pipefail; \
	umask 077; \
	cargo_home="$$(/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-cargo-home.XXXXXXXX")"; \
	cargo_home="$$(CDPATH= cd -- "$$cargo_home" && pwd -P)"; \
	trap '/usr/bin/rm -rf -- "$$cargo_home"' EXIT; \
	run_cargo() { \
		( \
			cd -- /; \
			/usr/bin/env -i \
				CARGO_HOME="$$cargo_home" \
				CARGO_NET_OFFLINE=true \
				CARGO_TARGET_DIR="$$cargo_home/target" \
				CARGO_TERM_COLOR=never \
				HOME="$$HOME" \
				LANG=C \
				LC_ALL=C \
				PATH="$$PATH" \
				RUSTDOCFLAGS="-D warnings" \
				RUSTUP_TOOLCHAIN=1.96.1 \
				SOURCE_DATE_EPOCH=0 \
				TZ=UTC \
				"$$@" \
		); \
	}; \
	repository_manifest="$(abspath $(dir $(lastword $(MAKEFILE_LIST))))/compiler/Cargo.toml"; \
	repro_source_archive="$$cargo_home/repro-source.tar"; \
	copy_compiler_source() { \
		local destination="$$1"; \
		/usr/bin/mkdir -- "$$destination"; \
		/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --extract --file="$$repro_source_archive" --directory="$$destination"; \
	}; \
	/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --create --file="$$repro_source_archive" --exclude=./compiler/target --directory="$${repository_manifest%/compiler/Cargo.toml}" -- .; \
	copy_compiler_source "$$cargo_home/check-src"; \
	manifest="$$cargo_home/check-src/compiler/Cargo.toml"; \
	run_cargo cargo fmt --manifest-path "$$manifest" --all -- --check; \
	run_cargo cargo clippy --manifest-path "$$manifest" --workspace --all-targets --locked --offline -- -D warnings; \
	run_cargo cargo clippy --manifest-path "$$manifest" --workspace --lib --bins --locked --offline -- -D warnings -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::string_slice -D clippy::indexing_slicing -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic; \
	run_cargo cargo doc --manifest-path "$$manifest" --workspace --no-deps --locked --offline; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --all-targets --locked --offline; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --all-targets --release --locked --offline; \
	copy_compiler_source "$$cargo_home/repro-src-a"; \
	copy_compiler_source "$$cargo_home/repro-src-b"; \
	run_cargo /usr/bin/env CARGO_TARGET_DIR="$$cargo_home/repro-target-a" cargo build --manifest-path "$$cargo_home/repro-src-a/compiler/Cargo.toml" -p orangec --bin orangec --release --locked --offline; \
	run_cargo /usr/bin/env CARGO_TARGET_DIR="$$cargo_home/repro-target-b" cargo build --manifest-path "$$cargo_home/repro-src-b/compiler/Cargo.toml" -p orangec --bin orangec --release --locked --offline; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import filecmp, sys; raise SystemExit(0 if filecmp.cmp(sys.argv[1], sys.argv[2], shallow=False) else "optimized orangec builds differ across source roots")' "$$cargo_home/repro-target-a/release/orangec" "$$cargo_home/repro-target-b/release/orangec"; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --doc --locked --offline

check-policy:
	/usr/bin/env -i HOME="$$HOME" LANG=C LC_ALL=C PATH="$$PATH" PYTHONHASHSEED=0 TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning tools/validate_foundation.py

test-policy:
	@set -euo pipefail; \
	pycache="$$(/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-python-cache.XXXXXXXX")"; \
	pycache="$$(CDPATH= cd -- "$$pycache" && pwd -P)"; \
	trap '/usr/bin/rm -rf -- "$$pycache"' EXIT; \
	/usr/bin/env -i HOME="$$HOME" LANG=C LC_ALL=C PATH="$$PATH" PYTHONHASHSEED=0 PYTHONPYCACHEPREFIX="$$pycache" TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import sys, unittest; sys.path.insert(0, "."); unittest.main(module=None)' discover -s tools/tests -p 'test_*.py'
