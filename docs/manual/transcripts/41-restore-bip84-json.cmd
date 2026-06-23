seed="abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon about"
printf '%s' "$seed" |
  $MNEMONIC_BIN restore --from phrase=- --template bip84 \
    --expect-fingerprint 73c5da0a --json
