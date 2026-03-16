#!/bin/bash

source "./.env"

IMAGE_NAME=shmashmexa

if [ -z "$SERVER_USER" ]; then
  echo "SERVER_USER environment variable must be set to username in .env"
  return 1
else
  echo "SERVER_USER is $SERVER_USER"
fi

if [ -z "$SERVER_IP" ]; then
  echo "SERVER_IP environment variable must be set to the ip address of the target device in .env"
  return 2
else
  echo "SERVER_IP is $SERVER_IP"
fi

SSH_DEST=$SERVER_USER@$SERVER_IP

#Not using rootless here.
#REMOTE_QUADLET_DIR="/home/$USER/.config/containers/systemd/$IMAGE_NAME"

#Using rootful.
REMOTE_QUADLET_DIR="/etc/containers/systemd/$IMAGE_NAME"

REPO_DIR=".."