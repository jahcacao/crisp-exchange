#!/bin/sh

set -e

./build.sh

echo ">> Deploying contract"

near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/mycelium_lab_near_amm.wasm