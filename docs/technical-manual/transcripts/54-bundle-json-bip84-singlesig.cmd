printf '%s' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  | $MNEMONIC_BIN bundle --network mainnet --template bip84 --json --slot @0.phrase=- 2>/dev/null | jq .
