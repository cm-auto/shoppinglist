#!/usr/bin/env bash

. credentials || exit 1

entry_id=${1:-"1"}
product=${2:-"Apple"}

payload=$(jo "product=$product")

curl -s --request PATCH localhost:3030/api/v1/entries/"$entry_id" -H "authorization: basic $basic_token" -H "content-type: application/json" -d "$payload" | jq
