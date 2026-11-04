#!/bin/bash

set -e

source ./experimental/voice_control/moonshine/commons.sh

systemctl --user stop $CONTAINER_NAME

echo Deleting link.
rm --force $LINK_NAME

echo Restarting.
systemctl --user daemon-reload