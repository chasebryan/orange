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
	trap '/usr/bin/rm -rf -- "$$cargo_home"' EXIT; \
	cargo_home="$$(CDPATH= cd -- "$$cargo_home" && pwd -P)"; \
	repro_home_b="$$(/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-repro-home.XXXXXXXX")"; \
	trap '/usr/bin/rm -rf -- "$$cargo_home" "$$repro_home_b"' EXIT; \
	repro_home_b="$$(CDPATH= cd -- "$$repro_home_b" && pwd -P)"; \
	gate_uid="$$(/usr/bin/id -u)"; \
	gate_gid="$$(/usr/bin/id -g)"; \
	namespace_runner=( \
		/usr/bin/unshare \
		--user \
		--map-current-user \
		--keep-caps \
		--mount \
		--pid \
		--fork \
		--kill-child=KILL \
		--mount-proc \
		--net \
		/usr/bin/setpriv \
		--bounding-set=-all \
		--inh-caps=-all \
		--ambient-caps=-all \
		--no-new-privs \
	); \
	if ! "$${namespace_runner[@]}" /bin/true >/dev/null 2>&1; then \
		namespace_runner=( \
			/usr/bin/sudo \
			--non-interactive \
			-- \
			/usr/bin/unshare \
			--mount \
			--pid \
			--fork \
			--kill-child=KILL \
			--mount-proc \
			--net \
			/usr/bin/setpriv \
			--bounding-set=-all \
			--inh-caps=-all \
			--ambient-caps=-all \
			--reuid "$$gate_uid" \
			--regid "$$gate_gid" \
			--clear-groups \
			--no-new-privs \
		); \
		"$${namespace_runner[@]}" /bin/true; \
	fi; \
	run_cargo() { \
		( \
			exec 8<&- 9<&-; \
			cd -- /; \
			"$${namespace_runner[@]}" \
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
	capture_archive_path="$$cargo_home/repro-source.tar"; \
	capture_paths_path="$$cargo_home/repro-source.paths"; \
	repro_source_archive=/proc/self/fd/9; \
	repro_source_paths=/proc/self/fd/8; \
	repro_source_paths_after="$$cargo_home/repro-source-after.paths"; \
	copy_compiler_source() { \
		local destination="$$1"; \
		/usr/bin/mkdir -- "$$destination"; \
		/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --extract --file="$$repro_source_archive" --directory="$$destination"; \
	}; \
	/usr/bin/env -i PATH=/usr/bin:/bin GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 /usr/bin/git -C "$$repository_root" ls-files --cached -z > "$$capture_paths_path"; \
	/usr/bin/env -u TAR_OPTIONS /usr/bin/tar --create --file="$$capture_archive_path" --format=gnu --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --mode='u+rwX,go+rX,go-w,u-s,g-s,o-t' --hard-dereference --null --verbatim-files-from --no-recursion --directory="$$repository_root" --files-from="$$capture_paths_path"; \
	exec 8<"$$capture_paths_path"; \
	exec 9<"$$capture_archive_path"; \
	/usr/bin/rm -- "$$capture_paths_path" "$$capture_archive_path"; \
	repro_source_archive_identity="$$(/usr/bin/sha256sum --binary -- "$$repro_source_archive")"; \
	repro_source_paths_identity="$$(/usr/bin/sha256sum --binary -- "$$repro_source_paths")"; \
	verify_capture_identity() { \
		[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_archive")" == "$$repro_source_archive_identity" ]] || { printf '%s\n' 'captured source archive changed during checks' >&2; exit 1; }; \
		[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_paths")" == "$$repro_source_paths_identity" ]] || { printf '%s\n' 'captured source path inventory changed during checks' >&2; exit 1; }; \
	}; \
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
	/usr/bin/rm -- "$$repro_source_paths_after"; \
	verify_capture_identity; \
	manifest="$$cargo_home/check-src/compiler/Cargo.toml"; \
	run_cargo /bin/bash -p -c 'for capability_set in CapInh CapPrm CapEff CapBnd CapAmb; do [[ "$$(/usr/bin/sed -n "s/^$${capability_set}:[[:space:]]*//p" /proc/self/status)" == 0000000000000000 ]] || exit 1; done; [[ $$$$ == 1 && $$PPID == 0 && "$$(/usr/bin/id -u)" == "$$1" && "$$(/usr/bin/id -g)" == "$$2" && "$$(/usr/bin/sed -n "s/^NoNewPrivs:[[:space:]]*//p" /proc/self/status)" == 1 && ! -e /proc/self/fd/8 && ! -e /proc/self/fd/9 && -z "$$(/usr/bin/sed -n "2p" /proc/net/route)" ]]' gate-isolation "$$gate_uid" "$$gate_gid"; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 PYTHONPYCACHEPREFIX="$$cargo_home/snapshot-python-cache" /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import sys, unittest; sys.path.insert(0, sys.argv.pop(1)); unittest.main(module=None)' "$$cargo_home/check-src" discover -s "$$cargo_home/check-src/tools/tests" -p 'test_*.py'; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
	run_cargo cargo fmt --manifest-path "$$manifest" --all -- --check; \
	run_cargo cargo clippy --manifest-path "$$manifest" --workspace --all-targets --locked --offline -- -D warnings; \
	run_cargo cargo clippy --manifest-path "$$manifest" --workspace --lib --bins --locked --offline -- -D warnings -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::string_slice -D clippy::indexing_slicing -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic; \
	run_cargo cargo doc --manifest-path "$$manifest" --workspace --no-deps --locked --offline; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --all-targets --locked --offline; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --all-targets --release --locked --offline; \
	/usr/bin/mkdir -- "$$repro_home_b/deep"; \
	copy_compiler_source "$$cargo_home/repro-a"; \
	copy_compiler_source "$$repro_home_b/deep/src"; \
	run_cargo /usr/bin/env CARGO_TARGET_DIR="$$cargo_home/target-a" cargo build --manifest-path "$$cargo_home/repro-a/compiler/Cargo.toml" -p orangec --bin orangec --release --locked --offline; \
	run_cargo /usr/bin/env CARGO_HOME="$$repro_home_b/cargo" CARGO_TARGET_DIR="$$repro_home_b/deep/target" cargo build --manifest-path "$$repro_home_b/deep/src/compiler/Cargo.toml" -p orangec --bin orangec --release --locked --offline; \
	artifact_a="$$cargo_home/target-a/release/orangec"; \
	artifact_b="$$repro_home_b/deep/target/release/orangec"; \
	for artifact in "$$artifact_a" "$$artifact_b"; do \
		[[ -f "$$artifact" && ! -L "$$artifact" ]] || { printf 'optimized orangec artifact type is invalid: %s\n' "$$artifact" >&2; exit 1; }; \
	done; \
	artifact_a_mode="$$(/usr/bin/stat --format=%a -- "$$artifact_a")"; \
	artifact_b_mode="$$(/usr/bin/stat --format=%a -- "$$artifact_b")"; \
	[[ "$$artifact_a_mode" == "$$artifact_b_mode" ]] || { printf 'optimized orangec artifact modes differ: %s -> %s\n' "$$artifact_a_mode" "$$artifact_b_mode" >&2; exit 1; }; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning -c 'import filecmp, sys; raise SystemExit(0 if filecmp.cmp(sys.argv[1], sys.argv[2], shallow=False) else "optimized orangec builds differ across source roots")' "$$artifact_a" "$$artifact_b"; \
	run_cargo cargo test --manifest-path "$$manifest" --workspace --doc --locked --offline; \
	run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "$$cargo_home/check-src/tools/validate_foundation.py"; \
	verify_capture_identity; \
	copy_compiler_source "$$cargo_home/check-reference"; \
	/usr/bin/find "$$cargo_home/check-reference" -mindepth 1 ! -type d -printf '%P\0' | /usr/bin/sort --zero-terminated > "$$cargo_home/check-reference.paths"; \
	tested_roots=("$$cargo_home/check-src" "$$cargo_home/repro-a" "$$repro_home_b/deep/src"); \
	tested_root_index=0; \
	for tested_root in "$${tested_roots[@]}"; do \
		tested_root_index="$$((tested_root_index + 1))"; \
		/usr/bin/find "$$tested_root" -mindepth 1 ! -type d -printf '%P\0' | /usr/bin/sort --zero-terminated > "$$cargo_home/tested-$$tested_root_index.paths"; \
		/usr/bin/cmp --silent -- "$$cargo_home/tested-$$tested_root_index.paths" "$$cargo_home/check-reference.paths" || { printf 'tested source membership changed during checks: %s\n' "$$tested_root" >&2; exit 1; }; \
	done; \
	while IFS= read -r -d '' relative_path; do \
		[[ -f "$$cargo_home/check-reference/$$relative_path" && ! -L "$$cargo_home/check-reference/$$relative_path" ]] || { printf 'captured source type is invalid during final comparison: %s\n' "$$relative_path" >&2; exit 1; }; \
		reference_mode="$$(/usr/bin/stat --format=%a -- "$$cargo_home/check-reference/$$relative_path")"; \
		for tested_root in "$${tested_roots[@]}"; do \
			[[ -f "$$tested_root/$$relative_path" && ! -L "$$tested_root/$$relative_path" ]] || { printf 'tested source type changed during checks: %s/%s\n' "$$tested_root" "$$relative_path" >&2; exit 1; }; \
			tested_mode="$$(/usr/bin/stat --format=%a -- "$$tested_root/$$relative_path")"; \
			[[ "$$tested_mode" == "$$reference_mode" ]] || { printf 'tested source mode changed during checks: %s/%s (%s -> %s)\n' "$$tested_root" "$$relative_path" "$$reference_mode" "$$tested_mode" >&2; exit 1; }; \
			/usr/bin/cmp --silent -- "$$tested_root/$$relative_path" "$$cargo_home/check-reference/$$relative_path" || { printf 'tested source bytes changed during checks: %s/%s\n' "$$tested_root" "$$relative_path" >&2; exit 1; }; \
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
