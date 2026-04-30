#!/bin/bash

set -e

source "./deploy/scripts/secrets.sh"

SSH_DEST=tyler@$SERVER_IP
LINK_DIRECTORY=/root
LINK_NAME=faux-show-backend
ENVIRONMENT_DIRECTORY=/root/faux-show-environment
ENVIRONMENT_FILE=$ENVIRONMENT_DIRECTORY/.env
WEB_DIRECTORY=/var/www/internal
NIX_STORE_DIR=$(readlink -f ./deploy/nix/result)

#Copy backend to device and set symlink
nix copy --extra-experimental-features nix-command --to ssh://$SERVER_IP $NIX_STORE_DIR
ssh $SERVER_IP "sudo mkdir -p $LINK_DIRECTORY"
ssh $SERVER_IP "sudo rm -f $LINK_DIRECTORY/$LINK_NAME"
ssh $SERVER_IP "sudo ln -s $NIX_STORE_DIR $LINK_DIRECTORY/$LINK_NAME"

#Set environment file
ssh -t $SSH_DEST "sudo mkdir -p $ENVIRONMENT_DIRECTORY"
ssh -t $SSH_DEST "echo EXTERNAL_USER=$EXTERNAL_USER | sudo tee $ENVIRONMENT_FILE"
ssh -t $SSH_DEST "echo EXTERNAL_PASSWORD=$EXTERNAL_PASSWORD | sudo tee -a $ENVIRONMENT_FILE"

#Copy frontend to device
ssh -t $SSH_DEST "sudo mkdir -p $WEB_DIRECTORY"
rsync  --rsync-path="sudo rsync" --verbose --recursive --progress --delete frontend/build/** $SSH_DEST:$WEB_DIRECTORY

#Restart services
ssh -t $SSH_DEST "sudo systemctl restart faux-show-backend cage-tty1"