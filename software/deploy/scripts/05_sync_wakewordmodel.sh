#!/bin/bash

set -e

echo Importing secrets
source "./software/deploy/scripts/secrets.sh"

echo Setting variables
SSH_DEST=$DEV_USER@$SERVER_IP


ssh -t $SSH_DEST \
    " \
    sudo mkdir -p '$(dirname $WAKEWORD_ONNX_FILE)'"
rsync \
    --rsync-path="sudo rsync" \
    --verbose \
    --recursive \
    --progress \
    --delete \
    /home/tyler/livekit/livekit_output/prod_hey_faux_show_v1/prod_hey_faux_show_v1.onnx \
    $SSH_DEST:$WAKEWORD_ONNX_FILE