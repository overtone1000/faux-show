#!/bin/bash

set -e

source "./deploy/secrets.sh"

SSH_DEST=root@$SERVER_IP

#Build backend
#nix-shell ./deploy/nix/cross-shell.nix --run "cargo build --release"
BINARY_DIRECTORY=/root/faux_show/bin
ssh $SSH_DEST "mkdir -p $BINARY_DIRECTORY"
rsync --verbose --recursive --progress --delete target/aarch64-unknown-linux-gnu/release/faux-show-backend $SSH_DEST:$BINARY_DIRECTORY

#Add env file
ssh $SSH_DEST "echo EXTERNAL_USER=$EXTERNAL_USER > $BINARY_DIRECTORY/.env"
ssh $SSH_DEST "echo EXTERNAL_PASSWORD=$EXTERNAL_PASSWORD >> $BINARY_DIRECTORY/.env"

#Build frontend
#npm run-script build --prefix ./frontend
WEB_DIRECTORY=/var/www/internal
ssh $SSH_DEST "mkdir -p $WEB_DIRECTORY"
rsync --verbose --recursive --progress --delete frontend/build/** $SSH_DEST:$WEB_DIRECTORY