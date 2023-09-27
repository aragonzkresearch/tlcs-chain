#!/usr/bin/env bash

set -eux

rm -rf ~/.tlcs
cargo run --bin tlcs -- init test

cargo run --bin tlcs -- add-genesis-account cosmos1syavy2npfyt9tcncdtsdzf7kny9lh777pahuux 34uatom
cargo run --bin tlcs -- add-genesis-account cosmos1skgmlw2j4qupafzcg5qvacd76mfzfe69la0hxz 53uatom
cargo run --bin tlcs -- add-genesis-account cosmos1gc308w6mg7skucsdxdjehhewr4aetwq24zf92m 50uatom

touch ~/.tlcs/config/resend.toml
echo 'tendermint_url = "http://localhost:26657"' >>~/.tlcs/config/resend.toml
echo 'from_user = "kevin"' >>~/.tlcs/config/resend.toml
echo 'chain_id = "test-chain"' >>~/.tlcs/config/resend.toml
