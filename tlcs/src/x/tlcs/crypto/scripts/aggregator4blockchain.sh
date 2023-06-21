#!/bin/bash
set -eu

echo -ne "714\n" >tmp2
cat > tmpproof
./src/x/tlcs/crypto/bin/aggregator4blockchain tmpproof tmpaggregatedpk $@ < tmp2
cat tmpaggregatedpk
rm -f tmpproof
rm -f tmpaggregatedpk
rm -f tmp2

