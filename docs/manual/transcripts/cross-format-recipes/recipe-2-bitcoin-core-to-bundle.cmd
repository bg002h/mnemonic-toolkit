cp $FIXTURES_DIR/core-mainnet-receive-change-pair.json wallet.json
# Step 1: combine Core's split receive (/0/*) + change (/1/*) entries into
# ONE BIP-389 multipath descriptor. Core never emits the combined form on
# export; md1's use-site must be the multipath /<0;1>/* (index 0 = receive,
# index 1 = change — same key). Automatic recombination is a planned follow-up.
recv=$(jq -r '.descriptors[] | select(.internal == false) | .desc' wallet.json)
printf '%s\n' "$recv" | sed -E 's,#[a-z0-9]+$,,; s,/0/\*\),/<0;1>/*),' > combined.desc

# Step 2: import the combined descriptor through the --format descriptor door,
# materialize the canonical envelope.
$MNEMONIC_BIN import-wallet --format descriptor --blob combined.desc --json \
  > envelope.json

# Step 3: synthesize ms1/mk1/md1 cards from the envelope (NO seed).
$MNEMONIC_BIN bundle --network mainnet --import-json envelope.json

# Step 4: attach a seed for the single cosigner.
$MNEMONIC_BIN bundle --network mainnet \
  --import-json envelope.json \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
