source .env

NIXOS_DIR="/etc/nixos"

if ![[ -v LOCAL_CONFIG_DIR ]]; then
    echo "LOCAL_CONFIG_DIR needs to be supplied in .env"
    exit 1
fi

if ![[ -v SERVER_IP ]]; then
    echo "SERVER_IP needs to be supplied in .env"
    exit 1
fi


echo ""
echo "Loading config from $LOCAL_CONFIG_DIR"
echo "     to nix directory $NIXOS_DIR"
echo "     at IP address $SERVER_IP"

echo ""
echo "Copying config"
echo ""
rsync --verbose --recursive --times --progress --copy-links --delete $LOCAL_CONFIG_DIR root@$SERVER_IP:$NIXOS_DIR/

echo ""
echo "Rebuilding"
echo ""
ssh -l root $SERVER_IP "sudo nixos-rebuild switch --show-trace"