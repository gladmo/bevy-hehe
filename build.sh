#!/bin/bash

OUT_NAME="hehe"

cargo build --profile wasm-release --target wasm32-unknown-unknown

wasm-bindgen \
            --out-dir ./out \
            --out-name "${OUT_NAME}" \
            --target web \
            --no-typescript \
            target/wasm32-unknown-unknown/wasm-release/bevy-hehe.wasm

cp index.html ./out/
cp -r assets ./out/

WASM_SIZE=$(wc -c < "./out/${OUT_NAME}_bg.wasm" | tr -d ' ')
sed \
  -e "s/{{WASM_SIZE}}/${WASM_SIZE}/" \
  ./out/index.html > ./out/index.html.tmp
mv ./out/index.html.tmp ./out/index.html