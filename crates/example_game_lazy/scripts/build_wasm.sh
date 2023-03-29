#!/bin/bash

set -x
set -eo pipefail

BINARY_NAME=example_game_lazy

cargo build --bin $BINARY_NAME --profile wasm-release --target wasm32-unknown-unknown
cp -r wasm generated_wasm
wasm-bindgen --no-typescript --out-name bevy_game --out-dir generated_wasm --target web ../../target/wasm32-unknown-unknown/wasm-release/$BINARY_NAME.wasm
cp -r assets generated_wasm