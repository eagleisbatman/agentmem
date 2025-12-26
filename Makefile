.PHONY: build install test clean

build:
	cargo build --release

install:
	cargo install --path .

test:
	cargo test

clean:
	cargo clean

