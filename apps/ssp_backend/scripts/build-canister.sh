#!/bin/bash

set -e

# parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --issuer)
      ID_TOKEN_ISSUER_BASE_URL="$2"
      shift # past argument
      shift # past value
      ;;
    --audience)
      ID_TOKEN_AUDIENCE="$2"
      shift # past argument
      shift # past value
      ;;
    --env-file)
      ENV_FILE_PATH="$2"
      shift # past argument
      shift # past value
      ;;
    --ignore-env-file)
      ENV_FILE_PATH=""
      shift # past argument
      ;;
  esac
done

# load environment variables from .env
if [[ ! -z "$ENV_FILE_PATH" ]]; then
  echo -e "\nLoading environment variables from $ENV_FILE_PATH file...\n"
  source $ENV_FILE_PATH
fi

if [[ -z "$ID_TOKEN_ISSUER_BASE_URL" ]]; then
  echo -e "\nError: ID_TOKEN_ISSUER_BASE_URL is not set\n"
  exit 1
fi

if [[ -z "$ID_TOKEN_AUDIENCE" ]]; then
  echo -e "\nError: ID_TOKEN_AUDIENCE is not set\n"
  exit 1
fi

# build canister
echo -e "\nBuilding canister..."
echo -e "JWT Issuer: $ID_TOKEN_ISSUER_BASE_URL\nJWT Audience: $ID_TOKEN_AUDIENCE\n"

ID_TOKEN_ISSUER_BASE_URL=$ID_TOKEN_ISSUER_BASE_URL \
ID_TOKEN_AUDIENCE=$ID_TOKEN_AUDIENCE \
cargo build --target wasm32-unknown-unknown --release -p ssp_backend --locked

echo -e "\nDone!\n"
