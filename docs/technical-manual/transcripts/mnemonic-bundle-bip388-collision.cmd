export SEED_0='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
export SEED_1='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
$MNEMONIC_BIN bundle --network mainnet --template wsh-sortedmulti --threshold 2 --slot '@0.phrase=@env:SEED_0' --slot '@1.phrase=@env:SEED_1'
