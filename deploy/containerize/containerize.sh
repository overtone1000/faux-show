#!/bin/bash

echo "This needs to be run from the workspace root with 'bash ./deploy/containerize/containerize.sh' or it will not work."
echo "This also requres the trm-rust-libs repo to be in the parent directory of this repo."

#Move to parent directory.
cd ..

podman build --file ./Shmashmexa/deploy/containerize/Containerfile --tag shmashmexa .