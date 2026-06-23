echo 'wsh(pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9))' | \
  $MNEMONIC_BIN compare-cost --json | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin),indent=2,ensure_ascii=False))"
