.DEFAULT_GOAL := check
override SHELL := /bin/bash
override .SHELLFLAGS := -p -c
unexport BASH_ENV ENV

.PHONY: check check-compiler check-policy test-policy
.NOTPARALLEL: check

check: check-policy check-compiler

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
	repository_root="$${repository_manifest%/compiler/Cargo.toml}"; \
	repro_source_archive="$$cargo_home/repro-source.tar"; \
	repro_source_paths="$$cargo_home/repro-source.paths"; \
	repro_source_paths_after="$$cargo_home/repro-source-after.paths"; \
	copy_compiler_source() { \
		local destination="$$1"; \
		/usr/bin/mkdir -- "$$destination"; \
		/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --extract --file="$$repro_source_archive" --directory="$$destination"; \
	}; \
	/usr/bin/env -i PATH=/usr/bin:/bin GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 /usr/bin/git -C "$$repository_root" ls-files --cached -z > "$$repro_source_paths"; \
	/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --create --file="$$repro_source_archive" --format=gnu --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --mode='u+rwX,go+rX,go-w,u-s,g-s,o-t' --hard-dereference --null --verbatim-files-from --no-recursion --directory="$$repository_root" --files-from="$$repro_source_paths"; \
	copy_compiler_source "$$cargo_home/check-src"; \
	while IFS= read -r -d '' relative_path; do \
		[[ -f "$$repository_root/$$relative_path" && ! -L "$$repository_root/$$relative_path" && -f "$$cargo_home/check-src/$$relative_path" && ! -L "$$cargo_home/check-src/$$relative_path" ]] || { printf '%s\n' 'tracked source type changed during archive capture' >&2; exit 1; }; \
		live_mode="$$(/usr/bin/stat --format=%a -- "$$repository_root/$$relative_path")"; \
		snapshot_mode="$$(/usr/bin/stat --format=%a -- "$$cargo_home/check-src/$$relative_path")"; \
		live_executable="$$(( (8#$$live_mode & 0111) != 0 ))"; \
		snapshot_executable="$$(( (8#$$snapshot_mode & 0111) != 0 ))"; \
		[[ "$$live_executable" == "$$snapshot_executable" ]] || { printf 'tracked source executable mode changed during archive capture: %s (%s -> %s)\n' "$$relative_path" "$$live_mode" "$$snapshot_mode" >&2; exit 1; }; \
		/usr/bin/cmp --silent -- "$$repository_root/$$relative_path" "$$cargo_home/check-src/$$relative_path" || { printf '%s\n' 'tracked source changed during archive capture' >&2; exit 1; }; \
	done < "$$repro_source_paths"; \
	/usr/bin/env -i PATH=/usr/bin:/bin GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 /usr/bin/git -C "$$repository_root" ls-files --cached -z > "$$repro_source_paths_after"; \
	/usr/bin/cmp --silent -- "$$repro_source_paths" "$$repro_source_paths_after" || { printf '%s\n' 'tracked source membership changed during archive capture' >&2; exit 1; }; \
	repro_source_archive_identity="$$(/usr/bin/sha256sum --binary -- "$$repro_source_archive")"; \
	repro_source_paths_identity="$$(/usr/bin/sha256sum --binary -- "$$repro_source_paths")"; \
	verify_capture_identity() { \
		[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_archive")" == "$$repro_source_archive_identity" ]] || { printf '%s\n' 'captured source archive changed during checks' >&2; exit 1; }; \
		[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_paths")" == "$$repro_source_paths_identity" ]] || { printf '%s\n' 'captured source path inventory changed during checks' >&2; exit 1; }; \
	}; \
	manifest="$$cargo_home/check-src/compiler/Cargo.toml"; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 PYTHONPYCACHEPREFIX="$$cargo_home/snapshot-python-cache" /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import sys, unittest; sys.path.insert(0, sys.argv.pop(1)); unittest.main(module=None)' "$$cargo_home/check-src" discover -s "$$cargo_home/check-src/tools/tests" -p 'test_*.py'; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
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
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import filecmp, sys; raise SystemExit(0 if filecmp.cmp(sys.argv[1], sys.argv[2], shallow=False) else "optimized orangec builds differ across source roots")' "$$cargo_home/repro-target-a/release/orangec" "$$cargo_home/repro-target-b/release/orangec"; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --doc --locked --offline; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
	verify_capture_identity; \
	copy_compiler_source "$$cargo_home/check-reference"; \
	/usr/bin/find "$$cargo_home/check-reference" -mindepth 1 ! -type d -printf '%P\0' | /usr/bin/sort --zero-terminated > "$$cargo_home/check-reference.paths"; \
	for tested_root in check-src repro-src-a repro-src-b; do \
		/usr/bin/find "$$cargo_home/$$tested_root" -mindepth 1 ! -type d -printf '%P\0' | /usr/bin/sort --zero-terminated > "$$cargo_home/$$tested_root.paths"; \
		/usr/bin/cmp --silent -- "$$cargo_home/$$tested_root.paths" "$$cargo_home/check-reference.paths" || { printf 'tested source membership changed during checks: %s\n' "$$tested_root" >&2; exit 1; }; \
	done; \
	while IFS= read -r -d '' relative_path; do \
		[[ -f "$$cargo_home/check-reference/$$relative_path" && ! -L "$$cargo_home/check-reference/$$relative_path" ]] || { printf 'captured source type is invalid during final comparison: %s\n' "$$relative_path" >&2; exit 1; }; \
		reference_mode="$$(/usr/bin/stat --format=%a -- "$$cargo_home/check-reference/$$relative_path")"; \
		for tested_root in check-src repro-src-a repro-src-b; do \
			[[ -f "$$cargo_home/$$tested_root/$$relative_path" && ! -L "$$cargo_home/$$tested_root/$$relative_path" ]] || { printf 'tested source type changed during checks: %s/%s\n' "$$tested_root" "$$relative_path" >&2; exit 1; }; \
			tested_mode="$$(/usr/bin/stat --format=%a -- "$$cargo_home/$$tested_root/$$relative_path")"; \
			[[ "$$tested_mode" == "$$reference_mode" ]] || { printf 'tested source mode changed during checks: %s/%s (%s -> %s)\n' "$$tested_root" "$$relative_path" "$$reference_mode" "$$tested_mode" >&2; exit 1; }; \
			/usr/bin/cmp --silent -- "$$cargo_home/$$tested_root/$$relative_path" "$$cargo_home/check-reference/$$relative_path" || { printf 'tested source bytes changed during checks: %s/%s\n' "$$tested_root" "$$relative_path" >&2; exit 1; }; \
		done; \
	done < "$$repro_source_paths"; \
	verify_capture_identity

check-policy:
	/usr/bin/env -i HOME="$$HOME" LANG=C LC_ALL=C PATH="$$PATH" PYTHONHASHSEED=0 TZ=UTC /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning tools/validate_foundation.py

test-policy:
	@set -euo pipefail; \
	pycache="$$(/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-python-cache.XXXXXXXX")"; \
	pycache="$$(CDPATH= cd -- "$$pycache" && pwd -P)"; \
	trap '/usr/bin/rm -rf -- "$$pycache"' EXIT; \
	/usr/bin/env -i HOME="$$HOME" LANG=C LC_ALL=C PATH="$$PATH" PYTHONHASHSEED=0 PYTHONPYCACHEPREFIX="$$pycache" TZ=UTC /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import sys, unittest; sys.path.insert(0, "."); unittest.main(module=None)' discover -s tools/tests -p 'test_*.py'
