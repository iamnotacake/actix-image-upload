#!/usr/bin/env bash

THIS_DIR="$(dirname "$(readlink -f "$0")")"

curl http://127.0.0.1:8080/upload \
	--verbose \
	--form image=@$THIS_DIR/pic1.png \

