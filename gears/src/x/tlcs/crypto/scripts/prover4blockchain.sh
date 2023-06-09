#!/bin/bash
set -eu
  
echo -ne "714\n" >tmp2
echo -ne $1 >>tmp2
echo -ne "\n" >> tmp2
rm -f tmpproof
./src/x/tlcs/crypto/bin/prover4blockchain tmpproof < tmp2
cat tmpproof 
