#!/bin/bash

set -e

source "./deploy/scripts/secrets.sh"

SSH_DEST=tyler@$SERVER_IP
LINK_DIRECTORY=/root/faux_show/bin
LINK_NAME=faux-show-backend
WEB_DIRECTORY=/var/www/internal
NIX_STORE_DIR=$(readlink -f ./deploy/nix/result)

#Copy backend to device and set symlink
nix copy --extra-experimental-features nix-command --to ssh://$SERVER_IP $NIX_STORE_DIR
ssh $SERVER_IP "sudo mkdir -p $LINK_DIRECTORY"
ssh $SERVER_IP "sudo rm -f $LINK_DIRECTORY/$LINK_NAME"
ssh $SERVER_IP "sudo ln -s $NIX_STORE_DIR $LINK_DIRECTORY/$LINK_NAME"

#Copy frontend to device
ssh -t $SSH_DEST "sudo mkdir -p $WEB_DIRECTORY"
rsync  --rsync-path="sudo rsync" --verbose --recursive --progress --delete frontend/build/** $SSH_DEST:$WEB_DIRECTORY