#!/usr/bin/env bash

set -eux

rm -rf ~/.tlcs
cargo run -- init test

cargo run -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 34uatom
