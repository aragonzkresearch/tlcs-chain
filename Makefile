run-debug:
	RUST_LOG=DEBUG cargo run -- run --verbose

run:
	cargo run -- run

build:
	cargo build --release --target-dir ./build

test:
	cargo test

install:
	cargo install --path ./tlcs

init:
	./tlcs/scripts/init.sh

tendermint-start:
	tendermint start --home ~/.tlcs

.PHONY: build run run-debug test install init tendermint-start
