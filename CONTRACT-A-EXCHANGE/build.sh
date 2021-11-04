#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e
cd "`dirname $0`"
cargo build --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/contract_a_exchange.wasm ./res/
#wasm-opt -Oz --output ./res/contract_a_exchange.wasm ./res/contract_a_exchange.wasm