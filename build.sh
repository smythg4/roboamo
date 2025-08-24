#!/bin/bash

echo "Building RoboAMO for production..."

dx build --release

OUTPUT_DIR=target/dx/roboamo/release/web/public
# Copy the demo directory to the output directory
DEMO_SRC=assets/demo
DEMO_DEST=$OUTPUT_DIR/assets/demo

# Copy the demo directory contents
mkdir -p "$DEMO_DEST"
if [ -d "$DEMO_SRC" ]; then
  echo "Copying demo assets from $DEMO_SRC to $DEMO_DEST..."
  cp -r "$DEMO_SRC/"* "$DEMO_DEST/"
else
  echo "Demo directory not found: $DEMO_SRC"
fi

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

