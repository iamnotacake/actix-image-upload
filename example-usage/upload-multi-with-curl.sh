#!/usr/bin/env bash

THIS_DIR="$(dirname "$(readlink -f "$0")")"

curl http://127.0.0.1:8080/upload \
	--verbose \
	--form image="@$THIS_DIR/pic1.png" \
	--form image="@$THIS_DIR/pic2.jpg" \
	--form image="@$THIS_DIR/pic3.bmp;type=image/bmp" \

