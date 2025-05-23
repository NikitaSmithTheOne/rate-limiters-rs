build: 
	cargo build

test: 
	cargo test

check_all: 
	cargo clippy && cargo fmt --check && cargo test