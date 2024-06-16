#!/usr/bin/env bash

. credentials || exit 1

entry_id=${1:-"1"}

curl -s localhost:3030/api/v1/entries/"$entry_id" -H "authorization: basic $basic_token" | jq
