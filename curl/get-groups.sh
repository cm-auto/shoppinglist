#!/usr/bin/env bash

. credentials || exit 1

curl localhost:3030/api/v1/groups -H "authorization: basic $basic_token" -s | jq -C
