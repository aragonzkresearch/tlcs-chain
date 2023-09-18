#!/bin/sh
#
#run some basic test transactions on new chain

TEST_CHAIN_BIN='../build/release/tlcs'

echo "race draft rival universe maid cheese steel logic crowd fork comic easy truth drift tomorrow eye buddy head time cash swing swift midnight borrow" | $TEST_CHAIN_BIN keys add kevin --recover
echo "all victory hero talent forget twice quote you office vacant sleep kangaroo disorder scorpion humble gorilla coast pudding edge garlic bid dutch excuse magic" | $TEST_CHAIN_BIN keys add alice --recover
echo "quick rack fancy cruel knee early summer clock group apology excuse file voice album fold cave garbage student awake twenty stereo argue draw human" | $TEST_CHAIN_BIN keys add ahmad --recover

#touch ~/.tlcs/config/resend.toml
#echo 'tendermint_url = "http://localhost:26617"' >>~/.tlcs/config/resend.toml
#echo 'from_user = "kevin"' >>~/.tlcs/config/resend.toml
#echo 'chain_id = "test-chain"' >>~/.tlcs/config/resend.toml

TEST_LATEST_ROUND=$(curl -s https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/latest | cut -d, -f1 | cut -d: -f2)
TEST_UNIX_TIME=$(date +%s)

$TEST_CHAIN_BIN tx kevin timelock keypair $(expr $TEST_LATEST_ROUND + 600) 1 $(expr $TEST_UNIX_TIME + 120)

sleep 10

$TEST_CHAIN_BIN tx alice timelock contribute $(expr $TEST_LATEST_ROUND + 600) 1 0
$TEST_CHAIN_BIN tx ahmad timelock contribute $(expr $TEST_LATEST_ROUND + 600) 1 0

sleep 6

curl -s localhost:1317/tlcs/timelock/v1beta1/keypairs | jq
