#!/bin/bash

set -e

echo Importing secrets
source "./software/deploy/scripts/secrets.sh"

echo Setting variables
SSH_DEST=$DEV_USER@$SERVER_IP

LOCAL_TAB_DIR=software/dev
REMOTE_TAB_DIR=/var/www/config #This is a declared environment variable for the backend application.

ssh -t $SSH_DEST \
    " \
    sudo mkdir -p $REMOTE_TAB_DIR "
rsync \
    --rsync-path="sudo rsync" \
    --verbose \
    --recursive \
    --progress \
    --delete \
    $LOCAL_TAB_DIR/ \
    $SSH_DEST:$REMOTE_TAB_DIR/