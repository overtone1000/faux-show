#!/bin/bash

CONTAINER_NAME=sh_dev
IMAGE_NAME=shmashmexa

podman build --tag IMAGE_NAME .
podman stop $CONTAINER_NAME
podman run --publish 30125:30125 --publish 30126:30126 --publish 4430:4430 --name $CONTAINER_NAME --replace --env DEVELOPMENT_MODE=false $IMAGE_NAME