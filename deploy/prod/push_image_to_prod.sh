#!/bin/bash

set -e

source "./constants.sh"

#Function to push an image
push_image_to_prod ( ) {
    
    echo "Creating archive"
    podman push $IMAGE_NAME oci-archive:/tmp/$IMAGE_NAME

    echo "Transferring archive"
    scp -r /tmp/$IMAGE_NAME $USER@$SERVER_IP:/tmp/$IMAGE_NAME

    echo "Pulling image from archive on remote"
    podman --remote pull $IMAGE_NAME oci-archive:/tmp/$IMAGE_NAME

    echo "Cleaning up"
    ssh -l root $SERVER_IP "rm /tmp/$IMAGE_NAME"
}

#Push image
push_image_to_prod