#!/bin/bash

set -e

source "./constants.sh"

#Function to push and start quadlet
start_quadlet() {
    ssh -T $SSH_DEST "mkdir -p $REMOTE_QUADLET_DIR && rm -rd $REMOTE_QUADLET_DIR && mkdir -p $REMOTE_QUADLET_DIR"

    echo "Copying $LOCAL_QUADLET_DIR to $SSH_DEST:$REMOTE_QUADLET_DIR"
    scp -r $LOCAL_QUADLET_DIR/** $SSH_DEST:$REMOTE_QUADLET_DIR

    #Show daemon-reload results
    echo ""
    echo "Doing dry run."
    ssh -T $SSH_DEST 'echo $(/etc/systemd/system-generators/podman-system-generator --user --dryrun | grep quadlet-generator)' #Should display errors concisely.
    #ssh -T $SSH_DEST '/etc/systemd/system-generators/podman-system-generator --user --dryrun' #Should display generator output verbosely.

    echo ""
    echo "Daemon reload."
    ssh -T $SSH_DEST "systemctl --user daemon-reload"

    source $LOCAL_QUADLET_DIR/members.sh

    for MEMBER in ${QUADLET_MEMBERS[@]}
    {
        echo "Starting $MEMBER"
        ssh -T $SSH_DEST "systemctl --user restart $MEMBER"
    }
}

start_quadlet