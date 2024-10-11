#!/bin/bash

set -e

./scripts/download-pocket-ic.sh

./scripts/build-canister.sh --ignore-env-file --issuer $ID_TOKEN_ISSUER_BASE_URL --audience $ID_TOKEN_AUDIENCE
