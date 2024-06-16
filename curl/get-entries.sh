#!/usr/bin/env bash

. credentials || exit 1

curl localhost:3030/api/v1/entries -H "authorization: basic $basic_token" -s | jq -C
