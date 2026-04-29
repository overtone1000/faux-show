#!/bin/bash

set -e

#Must be run from repo root with `bash ./deploy/update.sh`
source "./deploy/secrets.sh"

SSH_DEST=tyler@$SERVER_IP

#Build backend
nix build --extra-experimental-features 'nix-command flakes' ./deploy/nix/#
NIX_STORE_DIR=$(readlink -f result)
nix store sign --extra-experimental-features nix-command  --recursive --key-file ./deploy/nix/nix-store-binary-cache-key-secret $NIX_STORE_DIR
nix copy --extra-experimental-features nix-command --to ssh://$SERVER_IP $NIX_STORE_DIR
ssh $SERVER_IP "sed -i \"s|nix_store_dir=\\\".*\\\";|nix_store_dir=\\\"$NIX_STORE_DIR\\\";|\" /etc/nixos/trm_nixos/devices/raspberry_pi_kiosk/imports/faux-show-backend.nix"

PROGRAM_DIRECTORY=/root/faux_show
ssh -t $SSH_DEST "sudo mkdir -p $BINARY_DIRECTORY/bin"
rsync --rsync-path="sudo rsync" --verbose --recursive --progress --delete target/aarch64-unknown-linux-gnu/release/faux-show-backend $SSH_DEST:$PROGRAM_DIRECTORY/bin

#Add env file
ssh -t $SSH_DEST "echo EXTERNAL_USER=$EXTERNAL_USER | sudo tee $PROGRAM_DIRECTORY/.env"
ssh -t $SSH_DEST "echo EXTERNAL_PASSWORD=$EXTERNAL_PASSWORD | sudo tee -a $PROGRAM_DIRECTORY/.env"

#Build frontend
npm run-script build --prefix ./frontend
WEB_DIRECTORY=/var/www/internal
ssh -t $SSH_DEST "sudo mkdir -p $WEB_DIRECTORY"
rsync  --rsync-path="sudo rsync" --verbose --recursive --progress --delete frontend/build/** $SSH_DEST:$WEB_DIRECTORY