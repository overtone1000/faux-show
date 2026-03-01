#!/bin/bash

IMAGE_NAME=shmashmexa
USER=tyler

#SERVER_IP=10.10.10.155 #This is hard wired but will predominantly use wifi
SERVER_IP=10.10.30.252

SSH_DEST=$USER@$SERVER_IP
REMOTE_QUADLET_DIR="/home/$USER/.config/containers/systemd/$IMAGE_NAME"
LOCAL_QUADLET_DIR="./quadlet"