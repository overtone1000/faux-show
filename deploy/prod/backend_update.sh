#!/bin/bash

set -e

#Rebuild image
source "../containerize/containerize.sh"

#Push image
source "./push_image_to_prod.sh"

#Push and start quadlet
source "./update_quadlet.sh"