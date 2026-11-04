#!/bin/bash

set -e

source ./experimental/voice_control/moonshine/commons.sh

echo Creating systemd directory for root.
mkdir -p $LINK_DIR

echo Cleaning any pre-existing link.
rm --force $LINK_NAME

echo Creating new link.
ln -s $QUADLET_DIR $LINK_NAME

echo Restarting systemctl daemon.
systemctl --user daemon-reload

echo Starting service
systemctl --user restart $CONTAINER_NAME

echo Watching log.
journalctl --user -fxeu $CONTAINER_NAME