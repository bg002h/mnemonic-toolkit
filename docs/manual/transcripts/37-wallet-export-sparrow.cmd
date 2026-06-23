$MNEMONIC_BIN export-wallet \
  --format sparrow \
  --template wsh-sortedmulti \
  --threshold 2 \
  --multisig-path-family bip48 \
  --network mainnet \
  --wallet-name "VaultColdStorage" \
  --slot @0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX \
  --slot @0.fingerprint=b8688df1 \
  --slot @0.path=m/48\'/0\'/0\'/2\' \
  --slot @1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6 \
  --slot @1.fingerprint=28645006 \
  --slot @1.path=m/48\'/0\'/0\'/2\' \
  --slot @2.xpub=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx \
  --slot @2.fingerprint=5436d724 \
  --slot @2.path=m/48\'/0\'/0\'/2\' \
  --output sparrow-wallet.json
cat sparrow-wallet.json
