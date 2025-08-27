build: 
	cargo build

test: 
	cargo test

check_all: 
	cargo clippy && cargo fmt --check && cargo test

package:
	make check_all && cargo package

publish:
	make package && cargo publish