#!/usr/bin/env bash

. credentials || exit 1

product="Tomato"
amount="1"
unit="kg"
note="The fresher, the better :)"
group_id="2"

payload=$(jo "product=$product" "amount=$amount" "unit=$unit" "note=$note" "group_id=$group_id")

curl localhost:3030/api/v1/entries -H "authorization: basic $basic_token" -H "content-type: application/json" -d "$payload" | jq
