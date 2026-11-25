#!/bin/sh
# usage: CHAIN=signet ; reads -stdin args
read a
case "$a" in
 getblockcount) echo 100;;
 getblockchaininfo) echo "{\"chain\": \"$CHAIN\", \"mediantime\": 1700000000}";;
 getindexinfo) echo '{}';;
 gettxout) echo;;
 *) exit 1;;
esac
