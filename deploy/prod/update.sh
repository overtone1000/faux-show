#!/bin/bash

set -e

source "./constants.sh"

LOCAL_QUADLET_DIR=$REPO_DIR/deploy/prod/quadlet

LOCAL_BACKEND_SRC_DIR=$REPO_DIR/backend
LOCAL_FRONTEND_SRC_DIR=$REPO_DIR/frontend/build
LOCAL_TRM_RUST_LIB_DIR=$REPO_DIR/../trm-rust-libs

REMOTE_BUILD_DIR=~/shmashmexa_build
REMOTE_BACKEND_SUB=/source/Shmashmexa/backend
TARGET_SUB=/target
REMOTE_BACKEND_SRC_DIR=${REMOTE_BUILD_DIR}${REMOTE_BACKEND_SUB}
REMOTE_TRM_RUST_LIB_DIR=$REMOTE_BUILD_DIR/source/trm-rust-libs
REMOTE_BACKEND_OUTPUT=${REMOTE_BUILD_DIR}${TARGET_SUB}

REMOTE_BACKEND_DIR=$REMOTE_QUADLET_DIR/build/Shmashmexa/backend/target/release
REMOTE_FRONTEND_DIR=$REMOTE_QUADLET_DIR/build/Shmashmexa/frontend/build

clear_from_remote() {
    echo "Clearing $SSH_DEST:$1"
    ssh -T $SSH_DEST "mkdir -p $1 && rm -rd $1 && mkdir -p $1"
}

sync_to_server () {
    ssh -T $SSH_DEST "mkdir -p $2"
    rsync -avP --delete $1 $SSH_DEST:$2
}

#Function to push and start quadlet

update_source() {
    sync_to_server $LOCAL_QUADLET_DIR/       $REMOTE_QUADLET_DIR
    sync_to_server $LOCAL_BACKEND_SRC_DIR/   $REMOTE_BACKEND_SRC_DIR
    sync_to_server $LOCAL_FRONTEND_SRC_DIR/  $REMOTE_FRONTEND_DIR
    sync_to_server $LOCAL_TRM_RUST_LIB_DIR/ $REMOTE_TRM_RUST_LIB_DIR
}

build_backend() {
    #Build the backend
    ssh -T $SSH_DEST "mkdir -p $REMOTE_BUILD_DIR"
    ssh -T $SSH_DEST \
    "\
        podman run \
        --mount=type=bind,source=$REMOTE_BUILD_DIR,destination=/build,ro=false \
        docker.io/library/rust:alpine \
        cargo build --release --manifest-path=/build/$REMOTE_BACKEND_SUB/Cargo.toml --target-dir=/build/$TARGET_SUB
    "
}

copy_build_result() {
    echo ""
    echo "Copying cargo build output for container image build."
    clear_from_remote $REMOTE_BACKEND_DIR
    ssh -T $SSH_DEST "cp --recursive $REMOTE_BACKEND_OUTPUT/release/** $REMOTE_BACKEND_DIR"
}

start_quadlet() {
    #Show daemon-reload results
    echo ""
    echo "Doing dry run."
    #ssh -T $SSH_DEST 'echo $(/etc/systemd/system-generators/podman-system-generator --user --dryrun | grep quadlet-generator)' #Should display errors concisely.
    ssh -T $SSH_DEST 'echo $(/etc/systemd/system-generators/podman-system-generator --dryrun | grep quadlet-generator)' #Should display errors concisely.
    #ssh -T $SSH_DEST '/etc/systemd/system-generators/podman-system-generator --user --dryrun' #Should display generator output verbosely.

    echo ""
    echo "Daemon reload."
    #ssh -T $SSH_DEST "systemctl --user daemon-reload"
    ssh -T $SSH_DEST "systemctl daemon-reload"

    #Build can take a long time. Can follow along on a separate ssh session with
    #journalctl --user -fxeu shmashmexa-build

    source $LOCAL_QUADLET_DIR/members.sh
    for MEMBER in ${QUADLET_MEMBERS[@]}
    {
        echo "Starting $MEMBER"
        #ssh -T $SSH_DEST "systemctl --user restart $MEMBER"
        ssh -T $SSH_DEST "systemctl restart $MEMBER"
    }
}

#Don't clean up. Better to leave files for next rsync.
#cleanup_deprecated() {
#    #Remove source and copied build output but be sure to preserve cargo build output for subsequent use
#    echo ""
#    echo "Cleaning up source and copied build output from remote"
#    clear_from_remote $REMOTE_FRONTEND_DIR
#    clear_from_remote $REMOTE_BACKEND_SRC_DIR
#    clear_from_remote $REMOTE_BACKEND_DIR
#    clear_from_remote $REMOTE_TRM_RUST_LIB_DIR
#}

update_source
build_backend
copy_build_result
start_quadlet