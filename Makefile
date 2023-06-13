run-debug:
	RUST_LOG=DEBUG cargo run -- run --verbose

run:
	cargo run -- run

build:
	cargo build --release --target-dir ./build

test:
	cargo test

install:
	cargo install --path ./gears

init:
	./gears/scripts/init.sh

tendermint-start:
	tendermint start --home ~/.gears

.PHONY: build run run-debug test install init tendermint-start
