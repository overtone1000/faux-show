#!/bin/bash

set -e

source ./experimental/voice_control/containers/commons.sh

systemctl --user stop linux-voice-assistant

echo Cleaning any pre-existing link.
rm --force $LINK_NAME

echo Creating new link.
ln -s ./experimental/voice_control/containers/linux-voice-assistant $LINK_NAME

echo Restarting.
systemctl --user daemon-reload