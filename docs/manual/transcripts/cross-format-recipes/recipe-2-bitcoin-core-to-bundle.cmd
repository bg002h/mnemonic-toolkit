cp $FIXTURES_DIR/core-mainnet-receive-change-pair.json wallet.json
# Step 1: filter to active-receive descriptor only, materialize envelope.
$MNEMONIC_BIN import-wallet --format bitcoin-core --blob wallet.json \
  --select-descriptor active-receive --json \
  > envelope.json

# Step 2: synthesize ms1/mk1/md1 cards from the envelope (NO seed).
$MNEMONIC_BIN bundle --network mainnet --import-json envelope.json

# Step 3: attach a seed for the single cosigner.
$MNEMONIC_BIN bundle --network mainnet \
  --import-json envelope.json \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
