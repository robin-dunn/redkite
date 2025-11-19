.PHONY: all build run test clean check

all: build

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

check:
	cargo check
