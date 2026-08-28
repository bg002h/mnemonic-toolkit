printf '%s' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  | $MNEMONIC_BIN convert \
      --from phrase=- \
      --to xpub \
      --template bip84 \
      --network mainnet
