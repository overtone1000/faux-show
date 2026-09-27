#!/bin/bash

set -e

echo Importing secrets
source "./software/deploy/scripts/secrets.sh"

echo Setting variables
SSH_DEST=$DEV_USER@$SERVER_IP
LINK_DIRECTORY=/root
LINK_NAME=faux-show-backend
ENVIRONMENT_DIRECTORY=/root/faux-show-environment
ENVIRONMENT_FILE=$ENVIRONMENT_DIRECTORY/.env
WEB_DIRECTORY=/var/www/internal
NIX_STORE_DIR=$(readlink -f ./software/deploy/nix/result)
DEVICE_NAME="Faux Show"
DEVICE_ID="faux_show"

#Copy backend to device and set symlink
echo Copying via nix copy
nix copy --extra-experimental-features nix-command --to ssh://$SERVER_IP $NIX_STORE_DIR
ssh $SERVER_IP " \
    sudo mkdir -p $LINK_DIRECTORY \
    && sudo rm -f $LINK_DIRECTORY/$LINK_NAME \
    && sudo ln -s $NIX_STORE_DIR $LINK_DIRECTORY/$LINK_NAME \
    "

#Set environment file
echo Setting remote environment file
ssh -t $SSH_DEST \
    " \
    sudo mkdir -p $ENVIRONMENT_DIRECTORY \
    && echo EXTERNAL_USER=$EXTERNAL_USER | sudo tee $ENVIRONMENT_FILE \
    && echo EXTERNAL_PASSWORD=$EXTERNAL_PASSWORD | sudo tee -a $ENVIRONMENT_FILE \
    && echo KIOSK_USER_ID=$KIOSK_USER_ID | sudo tee -a $ENVIRONMENT_FILE \
    && echo DEVICE_NAME=$DEVICE_NAME | sudo tee -a $ENVIRONMENT_FILE \
    && echo DEVICE_ID=$DEVICE_ID | sudo tee -a $ENVIRONMENT_FILE \
    && echo PHOTOPRISM_KEY=$PHOTOPRISM_KEY | sudo tee -a $ENVIRONMENT_FILE \
    && echo HAS_KEY=$HAS_KEY | sudo tee -a $ENVIRONMENT_FILE \
    "

#Copy frontend to device
echo Copying frontend
ssh -t $SSH_DEST "sudo mkdir -p $WEB_DIRECTORY"
rsync  --rsync-path="sudo rsync" --verbose --recursive --progress --delete software/frontend/build/** $SSH_DEST:$WEB_DIRECTORY

#Restart services
echo Restarting services
ssh -t $SSH_DEST "sudo systemctl restart faux-show-backend cage-tty1"