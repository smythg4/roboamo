#!/bin/bash

echo "Building RoboAMO for production..."

dx build --release

OUTPUT_DIR=target/dx/roboamo/release/web/public

wasm-opt -Oz $OUTPUT_DIR/*.wasm -o $OUTPUT_DIR/app_opt.wasm
mv $OUTPUT_DIR/app_opt.wasm $OUTPUT_DIR/*.wasm

gzip -k $OUTPUT_DIR/*.wasm
gzip -k $OUTPUT_DIR/*.js

echo "Build complete! Size report:"
du -sh $OUTPUT_DIR/*

