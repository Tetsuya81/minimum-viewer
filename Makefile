.PHONY: test build run install

test:
	nix develop -c cargo test

build:
	nix develop -c cargo build

run:
	nix develop -c cargo run

install:
	nix develop -c cargo install --locked --path . --root $(HOME)/.local
