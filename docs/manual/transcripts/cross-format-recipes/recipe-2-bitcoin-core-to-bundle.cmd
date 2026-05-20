# Step 1: filter to active-receive descriptor only, materialize envelope.
mnemonic import-wallet --format bitcoin-core --blob wallet.json \
  --select-descriptor active-receive --json \
  > envelope.json

# Step 2: synthesize ms1/mk1/md1 cards from the envelope (NO seed).
mnemonic bundle --network mainnet --import-json envelope.json

# Step 3: attach a seed for the single cosigner.
mnemonic bundle --network mainnet \
  --import-json envelope.json \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
