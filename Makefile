.PHONY: test build run

test:
	nix develop -c cargo test

build:
	nix develop -c cargo build

run:
	nix develop -c cargo run
