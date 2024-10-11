#!/bin/bash

IOS_NATIVE_APP_CALLBACK_URL=com.xgensport.mobile.auth0://$AUTH0_DOMAIN/ios/com.xgensport.mobile/callback
ANDROID_NATIVE_APP_CALLBACK_URL=com.xgensport.mobile.auth0://$AUTH0_DOMAIN/android/com.xgensport.mobile/callback

export AUTH0_KEYWORD_REPLACE_MAPPINGS="{
  \"NATIVE_APP_LOGOUT_URLS\":[
    \"$IOS_NATIVE_APP_CALLBACK_URL\",
    \"$ANDROID_NATIVE_APP_CALLBACK_URL\"
  ],
  \"NATIVE_APP_CALLBACK_URLS\":[
    \"$IOS_NATIVE_APP_CALLBACK_URL\",
    \"$ANDROID_NATIVE_APP_CALLBACK_URL\"
  ],
  \"WEB_APP_LOGOUT_URLS\":[
    \"$WEB_APP_URL\"
  ],
  \"WEB_APP_CALLBACK_URLS\":[
    \"$WEB_APP_URL/api/auth/callback\"
  ],
  \"HASURA_GRAPHQL_ENDPOINT\":\"$HASURA_GRAPHQL_ENDPOINT/v1/graphql\",
  \"HASURA_GRAPHQL_ADMIN_SECRET\":\"$HASURA_GRAPHQL_ADMIN_SECRET\"
}"

echo -e "AUTH0_KEYWORD_REPLACE_MAPPINGS: $AUTH0_KEYWORD_REPLACE_MAPPINGS\n"

while [[ $# -gt 0 ]]; do
  case $1 in
    --import)
      pnpm a0deploy import --format=yaml --input_file=config/tenant.yaml
      exit 0
      ;;
    --export)
      pnpm a0deploy export --format=yaml --output_folder=config
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
  esac
done
