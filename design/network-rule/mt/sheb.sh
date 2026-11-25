#!/bin/sh
exec env CHAIN=signet sh "$(dirname "$0")/fakecli.sh" "$@"
