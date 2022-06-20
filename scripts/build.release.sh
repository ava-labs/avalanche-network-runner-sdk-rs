#!/usr/bin/env bash
set -xue

if ! [[ "$0" =~ scripts/build.release.sh ]]; then
  echo "must be run from repository root"
  exit 255
fi

# git submodule add https://github.com/googleapis/googleapis
# git submodule add https://github.com/ava-labs/avalanche-network-runner
git submodule update --init

cargo build --release
