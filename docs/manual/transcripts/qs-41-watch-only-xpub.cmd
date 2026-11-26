printf '%s' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  | $MNEMONIC_BIN convert \
      --from phrase=- \
      --to xpub \
      --scheme bip84 \
      --network mainnet
