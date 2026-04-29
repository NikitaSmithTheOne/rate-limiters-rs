.PHONY: tools bench bench-one bench-quick bench-filter

# *** GENERAL ***
install:
	./scripts/install.sh

build: 
	cargo build

test: 
	cargo test

check_all:
	cargo clippy && cargo fmt --check && cargo test

# *** BENCHMARKS ***
# Run all benchmarks
bench:
	cargo bench

# Run benchmarks for a single algorithm, e.g. `make bench-one BENCH=token_bucket`
bench-one:
	cargo bench --bench $(BENCH)

# Quick smoke run with reduced sample count, e.g. `make bench-quick BENCH=token_bucket`
bench-quick:
	cargo bench --bench $(BENCH) -- --quick

# Filter benchmarks by ID regex, e.g. `make bench-filter BENCH=token_bucket FILTER=TokenBucket/refresh`
bench-filter:
	cargo bench --bench $(BENCH) -- "$(FILTER)"

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

changelog:
	git cliff -o CHANGELOG.md

# *** PUBLISH ***
# package:
# 	make check_all && cargo package

# publish:
# 	make package && cargo publish