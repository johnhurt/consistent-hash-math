#!/bin/sh

set -e

wasm-pack  build --release

cd www 

rm -rf dist

npm run build 
