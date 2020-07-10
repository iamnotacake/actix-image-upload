#!/usr/bin/env bash

THIS_DIR="$(dirname "$(readlink -f "$0")")"

JSON_FILE="$THIS_DIR/base64.json"

curl http://127.0.0.1:8080/upload \
	--verbose \
	--header "Content-Type: application/json" \
	--request "POST" \
	--data "@$JSON_FILE" \

