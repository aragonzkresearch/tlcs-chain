#!/bin/bash
set -eu

# arguments: round signature pk [list of 0/1 values]
echo -ne "714\n" >tmp2
echo -n $1 >> tmp2
echo -ne "\n" >> tmp2
echo -n $2 >> tmp2
echo -ne "\n" >> tmp2
echo -n $3 > tmpaggregatedpk
echo -ne "\n" >> tmpaggregatedpk
cat > tmpproof
shift
shift
shift
./src/x/tlcs/crypto/bin/invert4blockchain tmpproof tmpaggregatedpk $@ < tmp2
rm -f tmpproof
rm -f tmpaggregatedpk
rm -f tmp2

