run-debug:
	RUST_LOG=DEBUG cargo run -- run --verbose

run:
	cargo run -- run

build:
	cargo build --release --target-dir ./build

build-linux-server:
	(export OPENSSL_INCLUDE_DIR='/usr/include/openssl'; export OPENSSL_LIB_DIR='/usr/lib/x86_64-linux-gnu'; cargo build --release --target-dir ./build)

test:
	cargo test -- --nocapture

install:
	cargo install --path ./tlcs

init:
	./tlcs/scripts/init.sh

tendermint-start:
	tendermint start --home ~/.tlcs

.PHONY: build run run-debug test install init tendermint-start
