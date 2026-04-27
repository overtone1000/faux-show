#!/bin/bash

set -e

QUADLET_MEMBERS=(
    #Pod first
    #Resources next
    faux_show_config-volume
    #Container builds
    faux_show-build
    #Containers in dependency order
    faux_show
)

echo Quadlet members are: ${QUADLET_MEMBERS[@]}