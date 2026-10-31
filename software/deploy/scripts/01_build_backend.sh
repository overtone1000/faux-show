#!/bin/bash

set -e

#Update trm-rust-lib packages (changes must be committed!)
cargo update -p has-mqtt
cargo update -p hyper-services
cargo update -p websocket

#Build and sign backend
cd ./software/deploy/nix
echo Building
nix build --extra-experimental-features 'nix-command flakes' .#
echo Signing
nix store sign --extra-experimental-features 'nix-command flakes'  --recursive --key-file ./nix-store-binary-cache-key-secret $NIX_STORE_DIR
echo Done