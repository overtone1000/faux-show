#!/bin/bash

set -e

QUADLET_MEMBERS=(
    #Pod first
    #Resources next
    shmashmexa_config-volume
    #Container builds
    shmashmexa-build
    #Containers in dependency order
    shmashmexa
)

echo Quadlet members are: ${QUADLET_MEMBERS[@]}