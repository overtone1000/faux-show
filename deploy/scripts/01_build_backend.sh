#!/bin/bash

set -e

#Build and sign backend
cd ./deploy/nix
nix build --extra-experimental-features 'nix-command flakes' .#
nix store sign --extra-experimental-features 'nix-command flakes'  --recursive --key-file ./nix-store-binary-cache-key-secret $NIX_STORE_DIR