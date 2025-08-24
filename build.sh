#!/bin/bash

echo "Building RoboAMO for production..."

dx build --release

OUTPUT_DIR=target/dx/roboamo/release/web/public

# Optimize wasm (inside wasm/ subdir)
if command -v wasm-opt &> /dev/null; then
  for wasm in $OUTPUT_DIR/wasm/*.wasm; do
    [ -e "$wasm" ] || continue
    echo "Optimizing $wasm..."
    wasm-opt -Oz "$wasm" -o "${wasm%.wasm}_opt.wasm"
    mv "${wasm%.wasm}_opt.wasm" "$wasm"
  done
else
  echo "wasm-opt not found, skipping optimization"
fi

# Gzip wasm + js (nested in wasm/ and assets/)
for f in $OUTPUT_DIR/wasm/*.wasm $OUTPUT_DIR/assets/*.js; do
  [ -e "$f" ] || continue
  gzip -kf "$f"
done

echo "Build complete! Size report:"
du -sh $OUTPUT_DIR/*

