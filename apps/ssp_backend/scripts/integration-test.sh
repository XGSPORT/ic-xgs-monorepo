#!/bin/bash

set -e

BIN_DIR="$(pwd)/bin"

ID_TOKEN_ISSUER_BASE_URL=$ID_TOKEN_ISSUER_BASE_URL \
ID_TOKEN_AUDIENCE=$ID_TOKEN_AUDIENCE \
POCKET_IC_MUTE_SERVER=1 \
POCKET_IC_BIN="$BIN_DIR/pocket-ic" \
TEST_CANISTER_WASM_PATH="../../target/wasm32-unknown-unknown/release/ssp_backend.wasm" \
cargo test --package ssp_backend --test '*'
