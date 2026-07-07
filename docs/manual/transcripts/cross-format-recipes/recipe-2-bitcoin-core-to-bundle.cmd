cp $FIXTURES_DIR/core-mainnet-receive-change-pair.json wallet.json
# Step 1: import Core's split receive (/0/*, internal:false) + change
# (/1/*, internal:true) export directly. The toolkit auto-recombines the
# same-key receive/change pair into ONE BIP-389 multipath /<0;1>/* bundle
# (index 0 = receive, index 1 = change — same key); no hand-combine needed.
# Materialize the canonical envelope.
$MNEMONIC_BIN import-wallet --format bitcoin-core --blob wallet.json --json \
  > envelope.json

# Step 2: synthesize ms1/mk1/md1 cards from the envelope (NO seed).
$MNEMONIC_BIN bundle --network mainnet --import-json envelope.json

# Step 3: attach a seed for the single cosigner.
$MNEMONIC_BIN bundle --network mainnet \
  --import-json envelope.json \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
