#!/bin/bash
set -eu

echo -ne "714\n" > tmp2
echo -n $1 >> tmp2
echo -ne "\n" >> tmp2
cat > tmpproof
./src/x/tlcs/crypto/bin/verifier4blockchain tmpproof tmpverified < tmp2
cat tmpverified
