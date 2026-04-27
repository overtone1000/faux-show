#!/bin/bash

set -e

#Must be run from repo root with `bash ./deploy.update`
source "./deploy/secrets.sh"

SSH_DEST=tyler@$SERVER_IP

#Build backend
#nix-shell ./deploy/nix/cross-shell.nix --run "cargo build --release"
PROGRAM_DIRECTORY=/root/faux_show
ssh -t $SSH_DEST "sudo mkdir -p $BINARY_DIRECTORY/bin"
rsync --rsync-path="sudo rsync" --verbose --recursive --progress --delete target/aarch64-unknown-linux-gnu/release/faux-show-backend $SSH_DEST:$PROGRAM_DIRECTORY/bin

#Add env file
ssh -t $SSH_DEST "echo EXTERNAL_USER=$EXTERNAL_USER | sudo tee $PROGRAM_DIRECTORY/.env"
ssh -t $SSH_DEST "echo EXTERNAL_PASSWORD=$EXTERNAL_PASSWORD | sudo tee -a $PROGRAM_DIRECTORY/.env"

#Build frontend
#npm run-script build --prefix ./frontend
WEB_DIRECTORY=/var/www/internal
ssh -t $SSH_DEST "sudo mkdir -p $WEB_DIRECTORY"
rsync  --rsync-path="sudo rsync" --verbose --recursive --progress --delete frontend/build/** $SSH_DEST:$WEB_DIRECTORY