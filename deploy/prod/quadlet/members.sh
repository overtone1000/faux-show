#!/bin/bash

set -e

QUADLET_MEMBERS=(
    #Pod first
    #Resources next
    shmashmexa_config-volume
    #Container builds
    #Containers in dependency order
    shmashmexa
)

echo Quadlet members are: ${QUADLET_MEMBERS[@]}