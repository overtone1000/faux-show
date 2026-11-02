#!/bin/bash

set -e

echo Importing secrets
source "./software/deploy/scripts/secrets.sh"

echo Setting variables
SSH_DEST=$DEV_USER@$SERVER_IP

REMOTE_DIR=/var/lib/faux-show #This is a declared environment variable for the backend application.

ssh -t $SSH_DEST \
    " \
    sudo mkdir -p $REMOTE_DIR "
rsync \
    --rsync-path="sudo rsync" \
    --verbose \
    --recursive \
    --progress \
    --delete \
    ./voice_control_examples/livekit-wakeword-minimal-example/hey_faux_show_v1.onnx \
    $SSH_DEST:$REMOTE_DIR