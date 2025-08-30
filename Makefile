.PHONY: tools

# *** GENERAL ***
tools:
	./tools/install.sh

build: 
	cargo build

test: 
	cargo test

check_all: 
	cargo clippy && cargo fmt --check && cargo test

# *** RELEASE ***
release-patch-dry:
	cargo release patch --config ./release.toml -v

release-patch:
	make check_all && cargo release patch --config ./release.toml -v --execute

release-minor-dry:
	cargo release minor --config ./release.toml -v

release-minor:
	make check_all && cargo release minor --config ./release.toml -v --execute

release-major-dry:
	cargo release major --config ./release.toml -v

release-major:
	make check_all && cargo release major --config ./release.toml -v --execute

# *** PUBLISH ***
# package:
# 	make check_all && cargo package

# publish:
# 	make package && cargo publish