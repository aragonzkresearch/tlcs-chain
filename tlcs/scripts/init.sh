#!/usr/bin/env bash

set -eux

rm -rf ~/.tlcs
cargo run -- init test
