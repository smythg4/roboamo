#!usr/bin/bash

echo "Building RoboAMO for production..."

dx build --release

wasm-opt -Oz dist/*.wasm -o dist/app_opt.wasm
mv dist/app_opt.wasm dist/*.wasm

gzip -k dist/*.wasm
gzip -k dist/*.js

echo "Build complete! Size report:"
du -sh dist/*

