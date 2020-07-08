#!/usr/bin/env bash

THIS_DIR="$(dirname "$(readlink -f "$0")")"

if [ -f "$1" ]; then
	image="$1"
else
	image="$THIS_DIR/pic1.png"
fi

curl http://127.0.0.1:8080/upload \
	--verbose \
	--form image="@$image" \

