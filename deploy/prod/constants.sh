#!/bin/bash

IMAGE_NAME=shmashmexa
#USER=tyler
USER=root

#SERVER_IP=10.10.10.155 #This is hard wired but will predominantly use wifi
SERVER_IP=10.10.30.252

SSH_DEST=$USER@$SERVER_IP

#Not using rootless here.
#REMOTE_QUADLET_DIR="/home/$USER/.config/containers/systemd/$IMAGE_NAME"

#Using rootful.
REMOTE_QUADLET_DIR="/etc/containers/systemd/$IMAGE_NAME"

REPO_DIR="../.."