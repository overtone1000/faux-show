#!/bin/bash

set -e

#Build and sign backend
nix build --extra-experimental-features 'nix-command flakes' ./deploy/nix/#
nix store sign --extra-experimental-features nix-command  --recursive --key-file ./deploy/nix/nix-store-binary-cache-key-secret $NIX_STORE_DIR