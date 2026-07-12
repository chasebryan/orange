.DEFAULT_GOAL := check

.PHONY: check check-compiler check-policy test-policy

check: check-compiler test-policy check-policy

check-compiler:
	cargo fmt --manifest-path compiler/Cargo.toml --all -- --check
	cargo clippy --manifest-path compiler/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings
	RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path compiler/Cargo.toml --workspace --no-deps --locked --offline
	cargo test --manifest-path compiler/Cargo.toml --workspace --all-targets --locked --offline
	cargo test --manifest-path compiler/Cargo.toml --workspace --doc --locked --offline

check-policy:
	python3 tools/validate_foundation.py

test-policy:
	python3 -m unittest discover -s tools/tests -p 'test_*.py'
