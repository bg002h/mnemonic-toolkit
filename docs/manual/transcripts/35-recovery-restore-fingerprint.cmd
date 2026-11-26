seed="abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon about"
printf '%s' "$seed" |
  $MNEMONIC_BIN restore --from phrase=- --scheme bip84 \
    --expect-fingerprint 73c5da0a
