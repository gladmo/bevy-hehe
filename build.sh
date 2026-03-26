#!/bin/bash

cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen \
            --out-dir ./out \
            --target web \
            --no-typescript \
            target/wasm32-unknown-unknown/wasm-release/bevy-hehe.wasm
cp index.html ./out/
          cp -r assets ./out/